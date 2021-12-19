use crate::bot;
use shell::store::TaskRecord;
use std::time;

pub fn more(
    handler: &mut bot::Context,
    user: String,
    duration: time::Duration,
) -> Option<(String, String)> {
    let now = time::SystemTime::now();
    let empty: Vec<TaskRecord> = Vec::new();
    let pendings = handler
        .store
        .select_current_task_for(user.clone())
        .unwrap_or(empty);

    match pendings.first() {
        Some(rec) => match handler.store.update_task_end(rec.id, now).and_then(|_| {
            let end = now + duration;
            handler
                .store
                .insert_do(user, now, end, rec.project.clone(), rec.task.clone())
        }) {
            Err(err) => Some((format!("Error: {}", err), String::new())),
            Ok(_) => Some(("Keep up the good work!".to_string(), String::new())),
        },
        None => Some((
            String::from("There's nothing to !more for you, sorry."),
            String::new(),
        )),
    }
}
