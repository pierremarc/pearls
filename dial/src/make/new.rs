use crate::bot;
use std::time;

pub fn new(
    handler: &mut bot::CommandHandler,
    username: String,
    project: String,
    d: time::Duration,
) -> Option<(String, String)> {
    match handler
        .store
        .insert_project(username, project, time::SystemTime::now(), d)
    {
        Err(_err) => Some((
            "Sorry, Err'd while saving to DB".into(),
            "Sorry, Err'd while saving to DB".into(),
        )),
        Ok(_) => Some(("Yeah! New Project!".into(), String::new())),
    }
}
