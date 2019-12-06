use chrono;
use chrono_humanize;
use crossbeam_channel::{unbounded, Receiver, Sender};
use matrix_bot_api::handlers::{HandleResult, Message, MessageHandler};
use matrix_bot_api::{ActiveBot, MatrixBot, MessageType};
use shell::expr::{parse_command, Command};
use shell::store::{Record, SharedStore};
use std::thread;
use std::time;

struct CommandHandler {
    chan: Sender<Record>,
    store: SharedStore,
}

impl CommandHandler {
    fn make_result(&mut self) -> String {
        match self.store.try_read() {
            Ok(store) => {
                let now = time::SystemTime::now();
                store
                    .iter()
                    .map(|rec| match &rec.command {
                        Command::Start(name, duration) => match rec
                            .time
                            .checked_add(duration.clone())
                        {
                            Some(end) if end > now => format!(
                                "{}: {} {}",
                                rec.username,
                                name,
                                chrono_humanize::HumanTime::from(
                                    chrono::Duration::from_std(end.duration_since(now).unwrap())
                                        .unwrap()
                                )
                            ),
                            _ => String::new(),
                        },
                        _ => String::new(),
                    })
                    .collect()
            }
            Err(_) => String::from("Error while reading log"),
        }
    }
}

impl MessageHandler for CommandHandler {
    fn handle_message(&mut self, bot: &ActiveBot, message: &Message) -> HandleResult {
        let user = message.sender.clone();
        let body = message.body.clone();
        if body.chars().nth(0).unwrap_or('_') != '!' {
            return HandleResult::StopHandling;
        }
        match parse_command(&body) {
            Ok(com) => {
                // self.chan.try_send((user, com));
                match com {
                    Command::List => {
                        let result = self.make_result();
                        bot.send_message(&result, &message.room, MessageType::RoomNotice)
                    }
                    _ => {
                        self.chan
                            .try_send(Record::new(time::SystemTime::now(), user.clone(), com));
                    }
                };
            }
            Err(_) => bot.send_message(
                "Miserably failed...",
                &message.room,
                MessageType::RoomNotice,
            ),
        };
        HandleResult::StopHandling
    }
}

pub fn start_bot(
    store: SharedStore,
    homeserver: &str,
    user: &str,
    password: &str,
) -> Receiver<Record> {
    let (s, r) = unbounded::<Record>();
    let h = String::from(homeserver);
    let u = String::from(user);
    let p = String::from(password);
    thread::spawn(move || {
        let bot = MatrixBot::new(CommandHandler { chan: s, store });
        bot.run(&u, &p, &h);
    });

    r
}
