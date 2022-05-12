use crate::bot;
use std::time;

use super::common::{check_meta, select_project};

pub fn intent(
    handler: &mut bot::Context,
    user: String,
    project_name: String,
    amount: time::Duration,
) -> Option<(String, String)> {
    match select_project(handler, &project_name) {
        Err(candidates) => Some((
            candidates.as_text("Or if it's a new project, you can !new it first."),
            candidates.as_html("Or if it's a new project, you can !new it first."),
        )),
        Ok(project) => match check_meta(handler, &project) {
            Some(r) => Some(r),
            None => match handler.store.insert_intent(user, project_name, amount) {
                Ok(_) => Some(("You won't regret it".into(), String::new())),
                Err(err) => Some((format!("Error: {}", err), String::new())),
            },
        },
    }
}
