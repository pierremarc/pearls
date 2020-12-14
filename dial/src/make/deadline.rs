use crate::bot;
use std::time;

use super::common::select_project;

pub fn deadline(
    handler: &mut bot::CommandHandler,
    project: String,
    d: time::SystemTime,
) -> Option<(String, String)> {
    match select_project(handler, &project) {
        Err(candidates) => Some((
            candidates.as_text("Or if it's a new project, you must !new it first."),
            candidates.as_html("Or if it's a new project, you must !new it first."),
        )),
        Ok(_) => match handler.store.update_deadline(project, d) {
            Err(_err) => Some((
                "Sorry, Err'd while saving to DB".into(),
                "Sorry, Err'd while saving to DB".into(),
            )),
            Ok(_) => Some((
                format!("Updated deadline to {}", shell::util::st_to_datestring(&d)),
                String::new(),
            )),
        },
    }
}
