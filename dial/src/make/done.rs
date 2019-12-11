use crate::bot;
use shell::util::{human_duration, st_from_ts};
use std::time;

pub fn done(
    handler: &mut bot::CommandHandler,
    user: String,
    duration: time::Duration,
    project: String,
    task: String,
) -> Option<(String, String)> {
    let now = time::SystemTime::now();
    let pendings = handler
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
                "You are already doing {}, you're covered, or tricky :)",
                task
            ),
            String::new(),
        )),
        None => {
            let given_start = now - duration;
            let start = handler
                .store
                .select_latest_task_for(user.clone(), |row| {
                    let end: i64 = row.get(3)?;
                    let end_time = st_from_ts(end);
                    Ok(end_time)
                })
                .map(|res| {
                    let i = res.get(0).unwrap_or(&given_start).clone();
                    i
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
