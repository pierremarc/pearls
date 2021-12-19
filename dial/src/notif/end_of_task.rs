use crossbeam_channel::tick;
use matrix_bot_api::{ActiveBot, MessageType};
use shell::store::{ConnectedStore, Store};
use shell::util::human_duration;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

fn notify(
    connected: &mut ConnectedStore,
    bot: &ActiveBot,
    task_id: i64,
    end: time::SystemTime,
    user: &str,
) {
    let now = time::SystemTime::now();
    let d = end
        .duration_since(now)
        .unwrap_or_else(|_| time::Duration::from_secs(0));
    let success_message = format!(
        "{}: Your current task will end in {}
        You can !more <duration> to continue",
        user,
        human_duration(d)
    );
    let error_message = format!(
        "{}: Your current task will end in {}
        You can !more <duration> to continue.
        Besides, note that we failed to record this notification, 
        it might come back again, sorry for the inconvenience",
        user,
        human_duration(d)
    );

    match connected.insert_notification(task_id, end) {
        Ok(_) => {
            bot.send_message(
                &success_message,
                &connected.room_id(),
                MessageType::TextMessage,
            );
        }
        Err(_) => {
            bot.send_message(
                &error_message,
                &connected.room_id(),
                MessageType::TextMessage,
            );
        }
    }
}

pub fn end_of_task(bot: ActiveBot, store: Arc<Mutex<Store>>) {
    thread::spawn(move || {
        for _ in tick(time::Duration::from_millis(2_600)).iter() {
            if let Ok(mut store) = store.lock() {
                store
                    .iter_mut()
                    .map(|connected| match connected.select_ending_tasks() {
                        Ok(recs) => {
                            for rec in recs.into_iter() {
                                notify(connected, &bot, rec.id, rec.end_time, &rec.username);
                            }
                        }
                        Err(_) => println!("notifications Error"),
                    })
                    .for_each(drop);
            }
        }
    });
}
