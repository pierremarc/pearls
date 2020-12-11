use crate::bot;
use html::{code, div, h2, table, Element};
use shell::util::{display_username, dur, human_duration, make_table_row};
use std::collections::HashSet;
use std::time;

use super::common::select_project;

pub fn digest(handler: &mut bot::CommandHandler, project: String) -> Option<(String, String)> {
    match select_project(handler, &project) {
        Err(candidates) => Some((candidates.as_text(""), candidates.as_html(""))),
        Ok(_) => {
            let available = handler
                .store
                .select_project_info(project.clone())
                .unwrap_or(Vec::new());

            match handler.store.select_project(project.clone()) {
                Ok(ref recs) => {
                    let names = recs
                        .iter()
                        .fold(HashSet::<String>::new(), |mut acc, rec| {
                            acc.insert(rec.project.clone());
                            acc
                        })
                        .into_iter()
                        .collect::<Vec<String>>()
                        .join(", ");

                    let left: Vec<String> = recs
                        .into_iter()
                        .map(|rec| {
                            format!(
                                "{}\t{}\t{}",
                                display_username(rec.username.clone()),
                                rec.task,
                                human_duration(rec.duration)
                            )
                        })
                        .collect();

                    let right: Vec<Element> = recs
                        .into_iter()
                        .map(|rec| {
                            make_table_row(vec![
                                display_username(rec.username.clone()),
                                rec.task.clone(),
                                format!("{}", human_duration(rec.duration)),
                            ])
                        })
                        .collect();

                    let done = recs.iter().fold(0, |acc, rec| {
                        acc + dur(&rec
                            .end_time
                            .duration_since(rec.start_time)
                            .unwrap_or(time::Duration::from_secs(0)))
                    }) / (1000 * 60 * 60);

                    let (h0, h1) = available
                        .first()
                        .map(|rec| {
                            (
                                format!(
                                    "{} hours available, {} hours done\n",
                                    rec.provision.map_or(0, |d| dur(&d)) / (1000 * 60 * 60),
                                    done
                                ),
                                div(vec![
                                    div(code(format!(
                                        "available: {} hours",
                                        rec.provision.map_or(0, |d| dur(&d)) / (1000 * 60 * 60)
                                    ))),
                                    div(code(format!("done: {} hours", done))),
                                ]),
                            )
                        })
                        .unwrap_or((format!("{} done", done), code(format!("done: {}", done))));
                    Some((
                        format!("{}\n{}\n{}", names, h0, left.join("\n")),
                        div(vec![h2(names), h1, table(right)]).as_string(),
                    ))
                }
                Err(_) => None,
            }
        }
    }
}
