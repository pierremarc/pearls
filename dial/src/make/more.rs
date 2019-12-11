use crate::bot;
use std::time;

pub fn more(
    handler: &mut bot::CommandHandler,
    user: String,
    duration: time::Duration,
) -> Option<(String, String)> {
    let empty: Vec<i64> = Vec::new();
    let pendings = handler
        .store
        .select_current_task_for(user.clone(), |row| {
            let id: i64 = row.get(0)?;
            Ok(id)
        })
        .unwrap_or(empty);
    let pending = pendings.first();
    match pending {
        Some(id) => match handler
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
    }}
