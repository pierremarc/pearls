use crate::bot;
use std::time;

pub fn avail(
    handler: &mut bot::Context,
    user: String,
    start: time::SystemTime,
    end: time::SystemTime,
    weekly: time::Duration,
) -> Option<(String, String)> {
    match handler.store.insert_avail(user, start, end, weekly) {
        Err(_err) => Some((
            "Sorry, Err'd while saving to DB".into(),
            "Sorry, Err'd while saving to DB".into(),
        )),
        Ok(_) => Some((format!("Registered"), String::new())),
    }
}
