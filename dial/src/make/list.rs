use crate::bot;
use shell::util::{display_username, human_duration};
use std::time;

pub fn list(handler: &mut bot::Context) -> Option<(String, String)> {
    let now = time::SystemTime::now();

    // let timeline_url = format!("{}/{}/timeline", handler.host, handler.room_id);

    match handler.store.select_current_task() {
        Ok(recs) if recs.len() > 0 => Some((
            recs.iter()
                .map(|rec| match rec.end_time.duration_since(now) {
                    Ok(duration) => format!(
                        "{} is {}ing on {}, they will be done in {}",
                        display_username(&rec.username),
                        rec.task,
                        rec.project,
                        human_duration(duration)
                    ),
                    Err(err) => format!(
                        "{} is {}ing on {}, they will be done in {}",
                        display_username(&rec.username),
                        rec.task,
                        rec.project,
                        err
                    ),
                })
                .collect::<Vec<String>>()
                .join("\n"),
            String::new(),
        )),
        Ok(_) => Some(("Time to !do something.".into(), String::new())),
        Err(_) => None,
    }
}
