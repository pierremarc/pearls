use crate::bot;
use shell::util::{human_ts, ts};
use std::time;

pub fn list(handler: &mut bot::CommandHandler) -> Option<(String, String)> {
    let now = time::SystemTime::now();

    match handler.store.select_current_task(|row| {
        // let id: i64 = row.get(0)?;
        let username: String = row.get(1)?;
        // let start: i64 = row.get(2)?;
        let end: i64 = row.get(3)?;
        let project: String = row.get(4)?;
        let task: String = row.get(5)?;
        let remaining = end - ts(&now);
        Ok(format!(
            "{} is {}ing on {}, they will be done in {}",
            username,
            task,
            project,
            human_ts(remaining)
        ))
    }) {
        Ok(ref strings) if strings.len() > 0 => Some((strings.join("\n"), String::new())),
        Ok(_) => Some(("Time to !do something.".into(), String::new())),
        Err(_) => None,
    }
}
