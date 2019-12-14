use crate::bot;
use std::time;

pub fn switch(
    handler: &mut bot::CommandHandler,
    user: String,
    project: String,
    task: String,
) -> Option<(String, String)> {
    let now = time::SystemTime::now();
    let pendings = handler
        .store
        .select_current_task_for(user.clone())
        .unwrap_or(Vec::new());

    match pendings.first() {
        Some(rec) => match handler
            .store
            .update_task_end(rec.id, now.clone())
            .and_then(|_| {
                handler.store.insert_do(user, now, rec.end_time, project, task.clone())
            }) {
            Err(err) => Some((format!("Error: {}", err), String::new())),
            Ok(_) => Some((format!("Good {}ing!", task.clone()), String::new())),
        },
        None => Some((
            String::from("There's nothing to !switch from, you might want to !do."),
            String::from("There's nothing to <strong>!switch</strong> from, you might want to <strong>!do<strong>."),
        )),
    }
}
