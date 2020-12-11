use crate::bot;
use std::time;

use super::common::select_project;

pub fn start(
    handler: &mut bot::CommandHandler,
    user: String,
    duration: time::Duration,
    project: String,
    task: String,
) -> Option<(String, String)> {
    let pendings = handler.store.select_current_task().unwrap_or(Vec::new());
    match pendings.iter().find(|rec| rec.username == user) {
        Some(rec) => Some((
            format!(
                "You are already doing {}, you should stop it first with !stop or use !switch",
                rec.task
            ),
            String::new(),
        )),
        None => match select_project(handler, &project) {
            Err(candidates) => Some((
                candidates.as_text("Or if it's a new project, you can !new it first."),
                candidates.as_html("Or if it's a new project, you can !new it first."),
            )),
            Ok(_) => {
                let start = time::SystemTime::now();
                match handler
                    .store
                    .insert_do(user, start, start + duration, project, task)
                {
                    Ok(_) => Some(("doing OK".into(), String::new())),
                    Err(err) => Some((format!("Error: {}", err), String::new())),
                }
            }
        },
    }
}
