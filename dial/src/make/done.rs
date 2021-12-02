use crate::bot;
use shell::util::human_duration;
use std::time;

use super::common::{check_meta, select_project};

pub fn done(
    handler: &mut bot::Context,
    user: String,
    duration: time::Duration,
    project_name: String,
    task: String,
) -> Option<(String, String)> {
    let now = time::SystemTime::now();
    let pendings = handler.store.select_current_task().unwrap_or(Vec::new());

    match pendings.iter().find(|rec| rec.username == user) {
        Some(rec) => Some((
            format!(
                "You are already doing {}, you're covered, or tricky :)",
                rec.task
            ),
            String::new(),
        )),
        None => match select_project(handler, &project_name) {
            Err(candidates) => Some((candidates.as_text(""), candidates.as_html(""))),
            Ok(project) => match check_meta(handler, &project) {
                Some(r) => Some(r),
                None => {
                    let given_start = now - duration;
                    let start = handler
                        .store
                        .select_latest_task_for(user.clone())
                        .map(|res| {
                            let i = res
                                .first()
                                .map(|rec| rec.end_time)
                                .unwrap_or(given_start)
                                .clone();
                            match i < given_start {
                                true => given_start,
                                false => i,
                            }
                        })
                        .unwrap_or(given_start);

                    let message = match start > given_start {
                        true => format!(
                        "Recorded, but adjusted to the end of your last task. Resulting in just {}",
                        human_duration(start.elapsed().unwrap_or(time::Duration::from_millis(0)))
                    ),
                        false => "Well recorded.".into(),
                    };

                    match handler
                        .store
                        .insert_do(user, start.clone(), now, project_name, task)
                    {
                        Ok(_) => Some((message, String::new())),
                        Err(err) => Some((format!("Error: {}", err), String::new())),
                    }
                }
            },
        },
    }
}
