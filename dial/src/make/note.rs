use super::common::select_project;
use crate::bot;

pub fn note(
    handler: &mut bot::Context,
    username: String,
    project: String,
    content: String,
) -> Option<(String, String)> {
    match select_project(handler, &project) {
        Err(candidates) => Some((candidates.as_text(""), candidates.as_html(""))),
        Ok(_) => match handler.store.insert_note(project, username, content) {
            Err(_err) => Some((
                "Sorry, Err'd while saving to DB".into(),
                "Sorry, Err'd while saving to DB".into(),
            )),
            Ok(_) => Some(("Noted.".into(), String::new())),
        },
    }
}
