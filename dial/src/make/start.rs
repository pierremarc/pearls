use crate::bot;
use std::time;
use strsim::levenshtein;

type ScoredName = (String, usize);

pub fn get_candidates(handler: &mut bot::CommandHandler, project: &str) -> Vec<String> {
    match handler.store.select_all_project_info() {
        Err(_) => Vec::new(),
        Ok(rows) => {
            let mut names: Vec<ScoredName> = rows
                .iter()
                .map(|record| (record.name.clone(), levenshtein(project, &record.name)))
                .collect();

            names.sort_by_key(|(_, n)| *n);
            names.iter().take(5).map(|(name, _)| name.clone()).collect()
        }
    }
}

pub fn start(
    handler: &mut bot::CommandHandler,
    user: String,
    duration: time::Duration,
    project: String,
    task: String,
) -> Option<(String, String)> {
    let pendings = handler.store.select_current_task().unwrap_or(Vec::new());
    match pendings.iter().find(|rec| rec.username == user) {
        Some(rec) => Some((
            format!(
                "You are already doing {}, you should stop it first with !stop or use !switch",
                rec.task
            ),
            String::new(),
        )),
        None => match handler.store.select_project_info(project.clone()) {
            Err(_) => Some(("DB Error".into(), String::new())),
            Ok(rows) => {
                if rows.len() == 0 {
                    let candidates = get_candidates(handler, &project);
                    let text: String = candidates.iter().map(|c| format!("\n  - {}", c)).collect();
                    let html: String = candidates
                        .iter()
                        .map(|c| format!("\n<li>{}</li>", c))
                        .collect();
                    Some((
                        format!(
                            "Project \"{}\" does not exists, similar project names are: {}",
                            &project,text
                        ),
                        format!(
                            "<h4>Project <em>{}</em> does not exists, similar project names are: </h4>
                        <ul>{}</ul>
                        ",
                            &project, html
                        ),
                    ))
                } else {
                    let start = time::SystemTime::now();
                    match handler
                        .store
                        .insert_do(user, start, start + duration, project, task)
                    {
                        Ok(_) => Some(("doing OK".into(), String::new())),
                        Err(err) => Some((format!("Error: {}", err), String::new())),
                    }
                }
            }
        },
    }
}
