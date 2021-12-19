use crate::bot;
use html::{table, Element};
use shell::util::{human_duration, make_table_row};
use std::time;

pub fn since(
    handler: &mut bot::Context,
    user: String,
    since: time::SystemTime,
) -> Option<(String, String)> {
    match handler.store.select_user(user, since) {
        Ok(results) => {
            let left: Vec<String> = results
                .iter()
                .map(|rec| {
                    format!(
                        "{}\t{}\t{}",
                        rec.project,
                        rec.task,
                        human_duration(rec.duration)
                    )
                })
                .collect();
            let rows: Vec<Element> = results
                .iter()
                .map(|rec| {
                    make_table_row(vec![
                        rec.project.clone(),
                        rec.task.clone(),
                        human_duration(rec.duration),
                    ])
                })
                .collect();

            Some((left.join("\n"), table(rows).as_string()))
        }
        Err(_) => None,
    }
}
