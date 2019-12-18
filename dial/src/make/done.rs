use crate::bot;
use shell::util::human_duration;
use std::time;

pub fn done(
    handler: &mut bot::CommandHandler,
    user: String,
    duration: time::Duration,
    project: String,
    task: String,
) -> Option<(String, String)> {
    let now = time::SystemTime::now();
    let pendings = handler.store.select_current_task().unwrap_or(Vec::new());

    match pendings.iter().find(|rec| rec.username == user) {
        Some(rec) => Some((
            format!(
                "You are already doing {}, you're covered, or tricky :)",
                rec.task
            ),
            String::new(),
        )),
        None => {
            let given_start = now - duration;
            println!("given_start {:?} ", given_start);
            let start = handler
                .store
                .select_latest_task_for(user.clone())
                .map(|res| {
                    let i = res
                        .first()
                        .map(|rec| rec.end_time)
                        .unwrap_or(given_start)
                        .clone();
                    match i < given_start {
                        true => given_start,
                        false => i,
                    }
                })
                .unwrap_or(given_start);

            let message = match start > given_start {
                true => format!(
                    "Recorded, but adjusted to the end of your last task. Resulting in just {}",
                    human_duration(start.elapsed().unwrap_or(time::Duration::from_millis(0)))
                ),
                false => "Well recorded.".into(),
            };

            match handler
                .store
                .insert_do(user, start.clone(), now, project, task)
            {
                Ok(_) => Some((message, String::new())),
                Err(err) => Some((format!("Error: {}", err), String::new())),
            }
        }
    }
}
