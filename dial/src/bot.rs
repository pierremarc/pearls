use crate::notif::{start_notifications, Notification, NotificationHandler};
use chrono;
use chrono_humanize;
use crossbeam_channel::{unbounded, Receiver, Sender};
use matrix_bot_api::handlers::{HandleResult, Message, MessageHandler};
use matrix_bot_api::{ActiveBot, MatrixBot, MessageType};
use shell::expr::{parse_command, Command};
use shell::store::{ts, Record, Store};
use std::convert::TryInto;
use std::path::Path;
use std::thread;
use std::time;

// fn st_from_ts(ts: i64) -> time::SystemTime {
//     time::SystemTime::UNIX_EPOCH + time::Duration::from_millis(ts.try_into().unwrap())
// }

fn human_ts(millis: i64) -> String {
    let d = chrono::Duration::from_std(time::Duration::from_millis(millis.try_into().unwrap_or(0)))
        .unwrap();
    chrono_humanize::HumanTime::from(d).to_text_en(
        chrono_humanize::Accuracy::Precise,
        chrono_humanize::Tense::Present,
    )
}

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
                    let now = ts(&time::SystemTime::now());
                    let d = end - now;
                    println!("now={}, end={}, d={}", now, end, d / 1000);
                    let msg = format!(
                        "{}: your current task will end in {}
                    You can !more <duration> to continue",
                        user,
                        human_ts(d)
                    );
                    self.bot
                        .send_message(&msg, &self.room_id, MessageType::TextMessage);
                })
                .unwrap(),
        };
    }
}

struct CommandHandler {
    chan: Sender<String>,
    store: Store,
    room_id: String,
    last_message_id: String,
}

fn make_table_row(cells: Vec<String>) -> String {
    let inner: String = cells
        .iter()
        .map(|s| format!("<td>{}</td>", s))
        .collect::<Vec<String>>()
        .join("");
    format!("<tr>{}</tr>", inner)
}

// fn join(a: Vec<String>, b: Vec<String>) -> (String, String) {
//     (a.join(""), b.join(""))
// }

fn split(a: Vec<(String, String)>) -> (Vec<String>, Vec<String>) {
    let output0 = a.iter().map(|(s, _)| s.clone()).collect();
    let output1 = a.iter().map(|(_, s)| s.clone()).collect();
    (output0, output1)
}

impl CommandHandler {
    fn make_list(&mut self) -> Option<(String, String)> {
        let now = time::SystemTime::now();

        match self.store.select_current_task(|row| {
            // let id: i64 = row.get(0)?;
            let username: String = row.get(1)?;
            // let start: i64 = row.get(2)?;
            let end: i64 = row.get(3)?;
            let project: String = row.get(4)?;
            let task: String = row.get(5)?;
            let remaining = end - ts(&now);
            Ok(format!(
                "{} is {}ing on {}, they will be done in {}",
                username,
                task,
                project,
                human_ts(remaining)
            ))
        }) {
            Ok(ref strings) if strings.len() > 0 => Some((strings.join("\n"), String::new())),
            Ok(_) => Some(("Time to !do something.".into(), String::new())),
            Err(_) => None,
        }
    }

    fn make_do(&mut self, user: String, com: Command) -> Option<(String, String)> {
        let now = time::SystemTime::now();
        let pendings = self
            .store
            .select_current_task(|row| {
                let username: String = row.get(1)?;
                let task: String = row.get(5)?;
                Ok((username, task))
            })
            .unwrap_or(Vec::new());
        match pendings.iter().find(|&(u, _)| u == &user) {
            Some((_, task)) => Some((
                format!(
                    "You are already doing {}, you should stop it with !stop",
                    task
                ),
                String::new(),
            )),
            None => {
                self.store
                    .log(&Record::new(now, user.clone(), com))
                    .unwrap();
                Some(("doing OK".into(), String::new()))
            }
        }
    }

    fn make_stop(&mut self, user: String) -> Option<(String, String)> {
        let empty: Vec<i64> = Vec::new();
        let pendings = self
            .store
            .select_current_task_for(user.clone(), |row| {
                let id: i64 = row.get(0)?;
                Ok(id)
            })
            .unwrap_or(empty);
        let pending = pendings.first();
        match pending {
            Some(id) => match self.store.update_task_end(*id, time::SystemTime::now()) {
                Err(_) => None,
                Ok(_) => Some((
                    "Done, you can !do a new one".into(),
                    "Done, you can <strong>!do</strong> a new one".into(),
                )),
            },
            None => Some((
                String::from("Ther's nothing to !stop for you"),
                String::new(),
            )),
        }
    }

    fn make_more(&mut self, user: String, duration: time::Duration) -> Option<(String, String)> {
        let empty: Vec<i64> = Vec::new();
        let pendings = self
            .store
            .select_current_task_for(user.clone(), |row| {
                let id: i64 = row.get(0)?;
                Ok(id)
            })
            .unwrap_or(empty);
        let pending = pendings.first();
        match pending {
            Some(id) => match self
                .store
                .update_task_end(*id, time::SystemTime::now() + duration)
            {
                Err(err) => Some((format!("Error: {}", err), String::new())),
                Ok(_) => Some((format!("Keep up the good work!"), String::new())),
            },
            None => Some((
                String::from("There's nothing to !more for you, sorry."),
                String::new(),
            )),
        }
    }

    fn make_project(&mut self, project: String) -> Option<(String, String)> {
        let available = match self.store.select_project_info(project.clone(), |row| {
            let _username: String = row.get(2)?;
            let _start: i64 = row.get(3)?;
            let dur: i64 = row.get(4)?;
            Ok(dur / (1000 * 60 * 60))
        }) {
            Err(_) => Vec::new(),
            Ok(ref d) => {
                let v = d;
                v.clone()
            }
        };

        match self.store.select_project(project.clone(), |row| {
            let username: String = row.get(1)?;
            let task: String = row.get(2)?;
            let spent_millis: i64 = row.get(3)?;
            let spent = human_ts(spent_millis);
            Ok((
                format!("{}\t{}\t{}", username, task, spent),
                make_table_row(vec![username, task, format!("{}", spent)]),
                spent_millis / (1000 * 60 * 60),
            ))
        }) {
            Ok(ref results) => {
                let ((left, right), spent) = (
                    split(
                        results
                            .into_iter()
                            .map(|(l, r, _)| (l.clone(), r.clone()))
                            .collect(),
                    ),
                    results.iter().map(|(_, _, s)| *s).collect::<Vec<i64>>(),
                );
                let done: i64 = spent.iter().fold(0, |acc, x| acc + x);
                let (h0, h1) = available
                    .first()
                    .map(|n| {
                        (
                            format!("{} hours available, {} hours done\n", n, done),
                            format!(
                                "
                            <div><code>available: {} hours </code> </div>
                            <div><code>done: {} hours </code></div>",
                                n, done
                            ),
                        )
                    })
                    .unwrap_or((
                        format!("{} done", done),
                        format!("</code><code>done: {} </code>", done),
                    ));
                Some((
                    h0 + &left.join("\n"),
                    h1 + &format!("<table>{}</table>", right.join("\n")),
                ))
            }
            Err(_) => None,
        }
    }
    // fn make_project_info(&mut self, project: String) -> Option<(String, String)>{
    //     match self.store.select_project_info(project.clone(), |row| {
    //         let _username: String = row.get(2)?;
    //         let _start: i64 = row.get(3)?;
    //         let dur: i64 = row.get(4)?;
    //         Ok(dur)
    //     }) {
    //         Err(_) => None,
    //         Ok()
    //     }
    // }

    fn make_since(&mut self, user: String, since: time::SystemTime) -> Option<(String, String)> {
        match self.store.select_user(user.clone(), since.clone(), |row| {
            let project: String = row.get(0)?;
            let task: String = row.get(1)?;
            let sum: i64 = row.get(2)?;
            Ok((
                format!("{}\t{}\t{}", project, task, human_ts(sum)),
                make_table_row(vec![project, task, format!("{}", human_ts(sum))]),
            ))
        }) {
            Ok(results) => {
                let (left, right) = split(results);
                Some((
                    left.join("\n"),
                    format!("<table>{}</table>", right.join("\n")),
                ))
            }
            Err(_) => None,
        }
    }

    fn make_add(
        &mut self,
        username: String,
        project: String,
        d: time::Duration,
    ) -> Option<(String, String)> {
        match self
            .store
            .insert_project(username, project, time::SystemTime::now(), d)
        {
            Err(_err) => Some((
                "Sorry, Err'd while saving to DB".into(),
                "Sorry, Err'd while saving to DB".into(),
            )),
            Ok(_) => Some(("Yeah! New Project!".into(), String::new())),
        }
    }

    fn make_help(&self) -> Option<(String, String)> {
        Some((
            "
        !ping
            check if the bot's still alive
        !new <project-name> <hours>
            register a new project
        !do <project-name> <task-name> <duration>
            start a new task
        !stop
            stop your current task
        !more <duration>
            add some time to your current task. the new end will NOW + <duration>
        !ls
            list current tasks
        !project <project-name>
            give stat for a given project
        !since <date or duration>
            a summary of your tasks since date
        "
            .into(),
            "
        <h4>!ping</h4>
            check if the bot's still alive
        <h4>!new <em>project-name</em> <em>hours</em></h4>
            register a new project
        <h4>!do <em>project-name</em> <em>task-name</em> <em>duration</em></h4>
            <p>start a new task</p>
            <p>you'll be notified of its ending</p>
        <h4>!stop</h4>
            stop your current task
        <h4>!more <em>duration</em></h4>
            add some time to your current task. the new end will <em>now</em> + <em>duration</em>
        <h4>!ls</h4>
            list current tasks
        <h4>!project <em>project-name</em></h4>
            give stat for a given project
        <h4>!since <em>date-or-duration</em></h4>
            a summary of your tasks since date
        "
            .into(),
        ))
    }

    fn parse_command(&mut self, user: String, body: String) -> Option<(String, String)> {
        match parse_command(&body) {
            Ok(com) => {
                let cc = com.clone();
                let u = user.clone();
                match com {
                    Command::Ping => Some(("pong".into(), String::new())),
                    Command::List => self.make_list(),
                    Command::Add(project, d) => self.make_add(u, project.clone(), d.clone()),
                    Command::Do(_, _, _) => self.make_do(u, cc),
                    Command::Stop => self.make_stop(u),
                    Command::More(d) => self.make_more(u, d.clone()),
                    Command::Project(project) => self.make_project(project),
                    Command::Since(since) => self.make_since(u, since),
                }
            }
            Err(_) => self.make_help(),
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
            println!(
                "Got a message({}) from {} in room {}:\n\t'{}'",
                message.id, user, message.room, body
            );
            return HandleResult::StopHandling;
        }
        if message.id == self.last_message_id {
            println!(
                "Got again message({}) from {} in room {}:\n\t'{}'",
                message.id, user, message.room, body
            );
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
mod tests {
    use crate::bot::*;
    use shell::store::ts;
    #[test]
    fn process_timestamp() {
        let input = 1575826006507i64;
        let st = st_from_ts(input);
        let expected = ts(&st);
        assert_eq!(input, expected);
    }
}
