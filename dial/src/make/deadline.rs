use crate::bot;
use std::time;

pub fn deadline(
    handler: &mut bot::CommandHandler,
    project: String,
    d: time::SystemTime,
) -> Option<(String, String)> {
    match handler.store.update_deadline(project, d) {
        Err(_err) => Some((
            "Sorry, Err'd while saving to DB".into(),
            "Sorry, Err'd while saving to DB".into(),
        )),
        Ok(_) => Some(("Updated deadline".into(), String::new())),
    }
}
