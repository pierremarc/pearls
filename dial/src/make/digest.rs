use crate::bot;
use html::{anchor, code, details, div, h2, no_display, paragraph, table, Element};
use shell::{
    store::NoteRecord,
    util::{display_username, dur, human_duration, make_table_row},
};

use super::common::select_project;

fn make_notes_string(notes: &Vec<NoteRecord>) -> String {
    match notes.len() {
        0 => String::new(),
        _ => notes
            .iter()
            .map(|note| {
                format!(
                    "
                    {} ({})
                    {}
                    ",
                    shell::util::display_username(note.username.clone()),
                    shell::util::st_to_datestring(&note.created_at),
                    note.content.clone(),
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n"),
    }
}

fn make_notes_html(notes: &Vec<NoteRecord>) -> Element {
    match notes.len() {
        0 => no_display(),
        _ => details(div(notes
            .iter()
            .map(|note| {
                div([
                    code(format!(
                        "↪ {} ({})",
                        shell::util::display_username(note.username.clone()),
                        shell::util::st_to_datestring(&note.created_at)
                    )),
                    paragraph(note.content.clone()),
                ])
            })
            .collect::<Vec<_>>())),
    }
}

pub fn digest(handler: &mut bot::Context, project: String) -> Option<(String, String)> {
    match select_project(handler, &project) {
        Err(candidates) => Some((candidates.as_text(""), candidates.as_html(""))),
        Ok(_) => match handler.store.select_project(project.clone()) {
            Ok(ref recs) => {
                let available = handler.store.select_project_info(project.clone());
                let note_records = handler
                    .store
                    .select_notes(project.clone()).unwrap_or_default();

                let text_notes = make_notes_string(&note_records);
                let html_notes = make_notes_html(&note_records);

                let left: Vec<String> = recs
                    .iter()
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
                    .iter()
                    .map(|rec| {
                        make_table_row(vec![
                            display_username(rec.username.clone()),
                            rec.task.clone(),
                            human_duration(rec.duration),
                        ])
                    })
                    .collect();

                let done = recs
                    .iter()
                    .fold(std::time::Duration::from_secs(0), |acc, task| {
                        println!("{} {:?}", task.project, task.duration);
                        acc + task.duration
                    })
                    .as_secs()
                    / 3600;

                let (h0, h1) = available
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

                let cal_url = format!(
                    "{}/{}/calendar/{}",
                    handler.base_url, handler.room_id, project
                );
                let (cal_string, cal_html) = (
                    format!("calendar: {}", cal_url),
                    div(anchor("Calendar Link ").set("href", cal_url)),
                );
                Some((
                    format!(
                        "{}\n{}\n{}\n{}\n{}",
                        &project,
                        h0,
                        text_notes,
                        left.join("\n"),
                        cal_string
                    ),
                    div(vec![h2(&project), h1, html_notes, table(right), cal_html]).as_string(),
                ))
            }
            Err(_) => None,
        },
    }
}
