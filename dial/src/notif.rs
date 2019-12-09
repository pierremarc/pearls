use crossbeam_channel::tick;
use shell::store::Store;
use std::path::Path;
use std::thread;
use std::time;

pub enum Notification {
    EndOfTask(i64, i64, String),
}

pub trait NotificationHandler {
    fn notify(&mut self, n: Notification);
}

pub fn start_notifications<N>(path: &Path, handler: N)
where
    N: NotificationHandler + 'static + Send,
{
    match Store::new(path.clone()) {
        Ok(store) => {
            thread::spawn(move || {
                let mut h: Box<dyn NotificationHandler + Send> = Box::new(handler);
                for _ in tick(time::Duration::from_millis(10_000)).iter() {
                    match store.select_ending_tasks(|row| {
                        let task_id: i64 = row.get(0)?;
                        let username: String = row.get(1)?;
                        let end_time: i64 = row.get(2)?;
                        Ok(Notification::EndOfTask(task_id, end_time, username))
                    }) {
                        Ok(notifs) => {
                            for n in notifs.into_iter() {
                                h.notify(n);
                            }
                        }
                        Err(_) => println!("notifications Error"),
                    };
                }
            });
        }
        Err(err) => {
            println!("Could not start the notification service:\n\t{}", err);
        }
    };
}
