use crate::bot;
use std::time;

pub fn start(
    handler: &mut bot::CommandHandler,
    user: String,
    duration: time::Duration,
    project: String,
    task: String,
) -> Option<(String, String)> {
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
                "You are already doing {}, you should stop it first with !stop or use !switch",
                task
            ),
            String::new(),
        )),
        None => {
            let start = time::SystemTime::now();
            match handler
                .store
                .insert_do(user, start, start + duration, project, task)
            {
                Ok(_) => Some(("doing OK".into(), String::new())),
                Err(err) => Some((format!("Error: {}", err), String::new())),
            }
        }
    }
}
