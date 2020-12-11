use crate::bot;
use std::time;

use super::common::select_project;

pub fn provision(
    handler: &mut bot::CommandHandler,
    project: String,
    d: time::Duration,
) -> Option<(String, String)> {
    match select_project(handler, &project) {
        Err(candidates) => Some((
            candidates.as_text("Or if it's a new project, you must !new it first."),
            candidates.as_html("Or if it's a new project, you must !new it first."),
        )),
        Ok(_) => match handler.store.update_provision(project, d) {
            Err(_err) => Some((
                "Sorry, Err'd while saving to DB".into(),
                "Sorry, Err'd while saving to DB".into(),
            )),
            Ok(_) => Some(("Updated provision".into(), String::new())),
        },
    }
}
