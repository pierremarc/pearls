use crate::make;
use crate::notif::{start_notifications, Notification, NotificationHandler};
use crossbeam_channel::{unbounded, Receiver, Sender};
use matrix_bot_api::handlers::{HandleResult, Message, MessageHandler};
use matrix_bot_api::{ActiveBot, MatrixBot, MessageType};
use shell::expr::{parse_command, Command};
use shell::store::Store;
use shell::util::{dur, human_duration, ts};
use std::path::Path;
use std::thread;
use std::time;

struct Notifier {
    bot: ActiveBot,
    store: Store,
    room_id: String,
}

impl Notifier {}

impl NotificationHandler for Notifier {
    fn notify(&mut self, notif: Notification) {
        match notif {
            Notification::EndOfTask(tid, end, user) => self
                .store
                .insert_notification(tid, end)
                .map(|_| {
                    let now = time::SystemTime::now();
                    let d = end.duration_since(now).unwrap_or(time::Duration::from_secs(0));
                    println!("now={}, end={}, d={}", ts(&now), ts(&end), dur(&d) / 1000);
                    let msg = format!(
                        "{}: your current task will end in {}\nYou can !more <duration> to continue",
                        user,
                        human_duration(d)
                    );
                    self.bot
                        .send_message(&msg, &self.room_id, MessageType::TextMessage);
                })
                .unwrap(),
        };
    }
}

pub struct CommandHandler {
    chan: Sender<String>,
    pub store: Store,
    pub room_id: String,
    last_message_id: String,
}

impl CommandHandler {
    fn parse_command(&mut self, user: String, body: String) -> Option<(String, String)> {
        match parse_command(&body) {
            Ok(com) => {
                let u = user.clone();
                match com {
                    Command::Ping => Some(("pong".into(), String::new())),
                    Command::List => make::list(self),
                    Command::Add(project, d) => make::new(self, u, project.clone(), d.clone()),
                    Command::Do(project, task, duration) => {
                        make::start(self, u, duration, project, task)
                    }
                    Command::Done(project, task, duration) => {
                        make::done(self, u, duration, project, task)
                    }
                    Command::Stop => make::stop(self, u),
                    Command::More(d) => make::more(self, u, d.clone()),
                    Command::Project(project) => make::project(self, project),
                    Command::Since(since) => make::since(self, u, since),
                    Command::Switch(project, task) => make::switch(self, u, project, task),
                }
            }
            Err(err) => {
                println!("ParseError: {}", err);
                make::help()
            }
        }
    }
}

impl MessageHandler for CommandHandler {
    fn init_handler(&mut self, bot: &ActiveBot) {
        println!("init_handler: joining room{}", &self.room_id);
        bot.join_room(&self.room_id);
        let path = self.store.get_path();
        Store::new(path)
            .map(|store| {
                start_notifications(
                    path,
                    Notifier {
                        store: store,
                        bot: bot.clone(),
                        room_id: self.room_id.clone(),
                    },
                )
            })
            .expect("Failed at creating a store for notification handler");
    }

    fn handle_message(&mut self, bot: &ActiveBot, message: &Message) -> HandleResult {
        let user = message.sender.clone();
        let body = message.body.clone();
        if message.room != self.room_id {
            // println!(
            //     "Got a message({}) from {} in room {}:\n\t'{}'",
            //     message.id, user, message.room, body
            // );
            return HandleResult::StopHandling;
        }
        if message.id == self.last_message_id {
            // println!(
            //     "Got again message({}) from {} in room {}:\n\t'{}'",
            //     message.id, user, message.room, body
            // );
            return HandleResult::StopHandling;
        }
        self.last_message_id = message.id.clone();
        if body.chars().nth(0).unwrap_or('_') != '!' {
            self.chan.try_send(format!("{}> {}", user, body)).unwrap();
            return HandleResult::ContinueHandling;
        }
        match self.parse_command(user, body) {
            Some((ref msg, ref html)) if html.len() == 0 => {
                bot.send_message(msg, &message.room, MessageType::RoomNotice)
            }
            Some((ref msg, ref html)) => {
                bot.send_html_message(msg, html, &message.room, MessageType::RoomNotice)
            }
            None => {}
        };
        HandleResult::StopHandling
    }
}

pub fn start_bot(
    path: &Path,
    homeserver: &str,
    room_id: &str,
    user: &str,
    password: &str,
) -> Receiver<String> {
    let (s, r) = unbounded::<String>();
    let h = String::from(homeserver);
    let u = String::from(user);
    let p = String::from(password);
    let rid = String::from(room_id);
    match Store::new(path.clone()) {
        Ok(store) => {
            thread::spawn(move || {
                let bot = MatrixBot::new(CommandHandler {
                    chan: s,
                    room_id: rid,
                    last_message_id: String::new(),
                    store,
                });
                bot.run(&u, &p, &h);
            });
        }
        Err(err) => {
            println!("Could not start the bot:\n\t{}", err);
        }
    };

    r
}

#[cfg(test)]
mod tests {}
