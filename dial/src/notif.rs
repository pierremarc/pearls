use crossbeam_channel::tick;
use shell::store::Store;
use std::path::Path;
use std::thread;
use std::time;

pub enum Notification {
    EndOfTask(i64, time::SystemTime, String),
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
                for _ in tick(time::Duration::from_millis(2_000)).iter() {
                    match store.select_ending_tasks() {
                        Ok(recs) => {
                            for rec in recs.into_iter() {
                                h.notify(Notification::EndOfTask(
                                    rec.id,
                                    rec.end_time,
                                    rec.username,
                                ));
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
