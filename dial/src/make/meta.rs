use super::common::select_project;
use crate::bot;

pub fn meta(
    handler: &mut bot::CommandHandler,
    username: String,
    project_name: String,
) -> Option<(String, String)> {
    match select_project(handler, &project_name) {
        Err(candidates) => Some((candidates.as_text(""), candidates.as_html(""))),
        Ok(project) => {
            if project.is_meta {
                Some((
                    "This project is already a meta project.".into(),
                    String::new(),
                ))
            } else if project.username != username {
                Some((
                    format!(
                        "This project has been created by {}, only they can edit this bit.",
                        project.username
                    ),
                    String::new(),
                ))
            } else {
                match handler.store.update_meta(project_name, true) {
                    Err(_) => Some(("failed to save this into DB.".into(), String::new())),
                    Ok(_) => Some(("Done".into(), String::new())),
                }
            }
        }
    }
}
