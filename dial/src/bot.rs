use crate::make;
use crate::notif::end_of_task;
use crossbeam_channel::{unbounded, Receiver, Sender};
use matrix_bot_api::handlers::{HandleResult, Message, MessageHandler};
use matrix_bot_api::{ActiveBot, MatrixBot, MessageType, Room};
use shell::expr::{parse_command, Command};
use shell::store::{ConnectedStore, Store};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct CommandHandler {
    chan: Sender<String>,
    arc_store: Arc<Mutex<Store>>,
    last_message_id: String,
    base_url: String,
}

pub struct Context<'a> {
    pub store: &'a mut ConnectedStore,
    pub room_id: String,
    pub base_url: String,
}

fn exec_command(context: &mut Context, user: String, body: String) -> Option<(String, String)> {
    match parse_command(&body) {
        Ok(com) => {
            let u = user;
            match com {
                Command::Ping => Some(("pong".into(), String::new())),
                Command::List => make::list(context),
                Command::Add(project) => make::new(context, u, project),
                Command::Do(project, task, duration) => {
                    make::start(context, u, duration, project, task)
                }
                Command::Done(project, task, duration) => {
                    make::done(context, u, duration, project, task)
                }
                Command::Stop => make::stop(context, u),
                Command::More(d) => make::more(context, u, d),
                Command::Digest(project) => make::digest(context, project),
                Command::Since(since) => make::since(context, u, since),
                Command::Switch(project, task) => make::switch(context, u, project, task),
                Command::Deadline(project, end) => make::deadline(context, project, end),
                Command::Provision(project, d) => make::provision(context, project, d),
                Command::Complete(project, end) => make::complete(context, project, end),
                Command::Note(project, content) => make::note(context, u, project, content),
                Command::Meta(project) => make::meta(context, u, project),
                Command::Parent(child, parent) => make::parent(context, u, child, parent),
                Command::Help => make::help(context),
            }
        }
        Err(err) => make::parse_error(&err),
    }
}

impl MessageHandler for CommandHandler {
    fn init_handler(&mut self, _bot: &ActiveBot) {
        // println!("CommandHandler::init_handler");
        // bot.join_room(&self.room_id);
        // let path = self.arc_store.get_path();
        // Store::new(path)
        //     .map(|store| {
        //         start_notifications(
        //             path,
        //             Notifier {
        //                 store: store,
        //                 bot: bot.clone(),
        //             },
        //         )
        //     })
        //     .expect("Failed at creating a store for notification handler");
    }

    fn handle_join(&mut self, bot: &ActiveBot, room: &Room) -> HandleResult {
        let success = "
        Hello there! I'm ready to take commands, 
        type `!help` for help
        ";
        let error = "
        Sorry there! Something went wrong when
        creating your database. You might want 
        to kick me out and invite me again 
        in order to fix this.
        ";
        if let Ok(mut store) = self.arc_store.lock() {
            if store.connect(&room.id).is_ok() {
                bot.send_message(success, &room.id, MessageType::RoomNotice)
            } else {
                bot.send_message(error, &room.id, MessageType::RoomNotice)
            }
        } else {
            bot.send_message(error, &room.id, MessageType::RoomNotice)
        }

        HandleResult::StopHandling
    }

    fn handle_message(&mut self, bot: &ActiveBot, message: &Message) -> HandleResult {
        println!(">>>>>> \nhandle_message {}\n", &message.body);
        let user = message.sender.clone();
        let body = message.body.clone();
        let room = message.room.clone();
        let base_url = self.base_url.clone();
        if message.id == self.last_message_id {
            return HandleResult::StopHandling;
        }
        self.last_message_id = message.id.clone();
        if body.chars().next().unwrap_or('_') != '!' {
            self.chan
                .try_send(format!("[{}] {}> {}", room, user, body))
                .unwrap_or(());
            return HandleResult::ContinueHandling;
        }
        if let Ok(mut store) = self.arc_store.lock() {
            if let Ok(connected) = store.connected(&room) {
                let mut context = Context {
                    store: connected,
                    room_id: room.clone(),
                    base_url,
                };

                match exec_command(&mut context, user, body) {
                    Some((ref msg, ref html)) if html.is_empty() => {
                        bot.send_message(msg, &room, MessageType::RoomNotice)
                    }
                    Some((ref msg, ref html)) => {
                        bot.send_html_message(msg, html, &room, MessageType::RoomNotice)
                    }
                    None => {}
                };
            } else {
                println!("Ouch, could not get a connection for: {}", &room);
            }
        } else {
            println!("Ouch, could not lock the store");
        }

        HandleResult::ContinueHandling
    }
}

pub fn start_bot(
    path: &Path,
    homeserver: &str,
    user: &str,
    password: &str,
    base_url: &str,
) -> Receiver<String> {
    let (s, r) = unbounded::<String>();
    let h = String::from(homeserver);
    let u = String::from(user);
    let p = String::from(password);
    let base_url = String::from(base_url);
    let store = Store::new(String::from(path.to_string_lossy()));
    let arc_store = Arc::new(Mutex::new(store));

    thread::spawn(move || {
        let mut bot = MatrixBot::new(CommandHandler {
            base_url,
            chan: s,
            last_message_id: String::new(),
            arc_store: arc_store.clone(),
        });

        end_of_task(bot.get_activebot_clone(), arc_store.clone());
        bot.set_verbose(false);
        bot.run(&u, &p, &h);
    });

    r
}

#[cfg(test)]
mod tests {}
