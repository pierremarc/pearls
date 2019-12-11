use crate::bot;
use shell::expr::Command;
use shell::store::Record;
use std::time;

pub fn start(
    handler: &mut bot::CommandHandler,
    user: String,
    com: Command,
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
                "You are already doing {}, you should stop it with !stop",
                task
            ),
            String::new(),
        )),
        None => {
            handler
                .store
                .log(&Record::new(now, user.clone(), com))
                .unwrap();
            Some(("doing OK".into(), String::new()))
        }
    }
}
