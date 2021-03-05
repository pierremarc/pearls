use super::common::select_project;
use crate::bot;

pub fn parent(
    handler: &mut bot::CommandHandler,
    username: String,
    child_name: String,
    parent_name: String,
) -> Option<(String, String)> {
    match (
        select_project(handler, &child_name),
        select_project(handler, &parent_name),
    ) {
        (Err(candidates), Ok(_)) => Some((
            candidates.as_text("Child project not found"),
            candidates.as_html("Child project not found"),
        )),
        (Ok(_), Err(candidates)) => Some((
            candidates.as_text("Parent project not found"),
            candidates.as_html("Parent project not found"),
        )),
        (Err(child_candidates), Err(parent_candidates)) => Some((
            format!(
                "Child canditates:\n{}\n\nParent candidates:\n{}",
                child_candidates.as_text(""),
                parent_candidates.as_text("")
            ),
            String::new(),
        )),
        (Ok(child_project), Ok(parent_project)) => {
            if !parent_project.is_meta {
                Some((
                    format!(
                        "{} is **not** a meta project, it can't have child projects.",
                        parent_name
                    ),
                    String::new(),
                ))
            } else if child_project.is_meta {
                Some((
                    format!(
                        "{} **is** a meta project, it can't have a parent project.",
                        child_name
                    ),
                    String::new(),
                ))
            } else if child_project.username != username {
                Some((
                    format!(
                        "{} has been created by {}, only they can edit this bit.",
                        child_name, child_project.username
                    ),
                    String::new(),
                ))
            } else {
                match handler.store.update_parent(child_name, parent_project.id) {
                    Err(_) => Some(("failed to save this into DB.".into(), String::new())),
                    Ok(_) => Some(("Done".into(), String::new())),
                }
            }
        }
    }
}
