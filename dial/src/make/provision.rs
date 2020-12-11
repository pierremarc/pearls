use crate::bot;
use std::time;

pub fn provision(
    handler: &mut bot::CommandHandler,
    project: String,
    d: time::Duration,
) -> Option<(String, String)> {
    match handler.store.update_provision(project, d) {
        Err(_err) => Some((
            "Sorry, Err'd while saving to DB".into(),
            "Sorry, Err'd while saving to DB".into(),
        )),
        Ok(_) => Some(("Updated provision".into(), String::new())),
    }
}
