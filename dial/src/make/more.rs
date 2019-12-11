use crate::bot;
use shell::expr::Command;
use shell::store::Record;
use std::time;

pub fn more(
    handler: &mut bot::CommandHandler,
    user: String,
    duration: time::Duration,
) -> Option<(String, String)> {
    let now = time::SystemTime::now();
    let empty: Vec<(i64, String, String)> = Vec::new();
    let pendings = handler
        .store
        .select_current_task_for(user.clone(), |row| {
            let id: i64 = row.get(0)?;
            let project: String = row.get(4)?;
            let task: String = row.get(4)?;
            Ok((id, task, project))
        })
        .unwrap_or(empty);

    match pendings.first() {
        Some((id, task, project)) => match handler.store.update_task_end(*id, now).and_then(|_| {
            handler.store.log(&Record::new(
                now,
                user.clone(),
                Command::Do(project.clone(), task.clone(), duration),
            ))
        }) {
            Err(err) => Some((format!("Error: {}", err), String::new())),
            Ok(_) => Some((format!("Keep up the good work!"), String::new())),
        },
        None => Some((
            String::from("There's nothing to !more for you, sorry."),
            String::new(),
        )),
    }
}
