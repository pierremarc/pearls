use crate::bot;
use std::time;

use super::common::select_project;

pub fn complete(
    handler: &mut bot::CommandHandler,
    project: String,
    d: time::SystemTime,
) -> Option<(String, String)> {
    match select_project(handler, &project) {
        Err(candidates) => Some((candidates.as_text("---"), candidates.as_html("---"))),
        Ok(_) => match handler.store.update_completed(project, d) {
            Err(_err) => Some((
                "Sorry, Err'd while saving to DB".into(),
                "Sorry, Err'd while saving to DB".into(),
            )),
            Ok(_) => Some((
                format!(
                    "Updated completion date to {}",
                    shell::util::st_to_datestring(&d)
                ),
                String::new(),
            )),
        },
    }
}
