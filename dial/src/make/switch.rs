use crate::bot;
use shell::util::st_from_ts;
use std::time;

pub fn switch(
    handler: &mut bot::CommandHandler,
    user: String,
    project: String,
    task: String,
) -> Option<(String, String)> {
    let now = time::SystemTime::now();
    let empty: Vec<(i64, i64)> = Vec::new();
    let pendings = handler
        .store
        .select_current_task_for(user.clone(), |row| {
            let id: i64 = row.get(0)?;
            let end: i64 = row.get(3)?;
            Ok((id, end))
        })
        .unwrap_or(empty);

    match pendings.first() {
        Some((id, end)) => match handler
            .store
            .update_task_end(*id, now.clone())
            .and_then(|_| {
                let end_time = st_from_ts(*end);
                handler.store.insert_do(user, now, end_time, project, task.clone())
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
