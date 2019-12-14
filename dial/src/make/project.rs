use crate::bot;
use chrono::Datelike;
use html::{body, code, div, head, html, table, Element};
use shell::cal::{day, month, week};
use shell::store::AggregatedTaskRecord;
use shell::util::{date_time_from_st, dur, human_duration, make_table_row, st_from_date_time};
use std::time;

fn find_items(
    recs: &Vec<AggregatedTaskRecord>,
    start: time::SystemTime,
    end: time::SystemTime,
) -> Vec<&AggregatedTaskRecord> {
    recs.iter()
        .filter(|rec| {
            (start < rec.start_time && rec.start_time < end)
                || (start < rec.end_time && rec.end_time < end)
                || (start < rec.start_time && rec.end_time < end)
        })
        .collect()
}

fn cal_project(recs: &Vec<AggregatedTaskRecord>) -> Element {
    let (start_time, end_time) = recs.iter().fold(
        (time::SystemTime::now(), time::UNIX_EPOCH),
        |(s, e), rec| {
            let new_start = {
                if rec.start_time < s {
                    rec.start_time.clone()
                } else {
                    s
                }
            };
            let new_end = {
                if rec.end_time > e {
                    rec.end_time.clone()
                } else {
                    e
                }
            };
            (new_start, new_end)
        },
    );
    let end = date_time_from_st(&end_time);
    let mut start = date_time_from_st(&start_time);
    let mut guard_date = start.clone();
    let mut all_days: Vec<Element> = Vec::new();

    while guard_date < end {
        let days: Vec<Element> = week(start.year(), start.month(), start.day())
            .iter()
            .map(|(s, e)| {
                start = s;
                guard_date = e;
                let items: Vec<Element> =
                    find_items(recs, st_from_date_time(&s), st_from_date_time(&e))
                        .iter()
                        .map(|rec| {
                            div(vec![
                                div(format!("{}", rec.username)),
                                div(format!("{}", rec.project)),
                                div(format!("{}", rec.task)),
                            ])
                            .set("class", "task")
                        })
                        .collect();
                div(vec![div(format!("{}", s.date())), div(items)]).set("class", "day")
            })
            .collect();
        all_days.push(div(days).set("class", "week"));
    }

    html(body(div(all_days)))
}

pub fn project(handler: &mut bot::CommandHandler, project: String) -> Option<(String, String)> {
    let available = handler
        .store
        .select_project_info(project.clone())
        .unwrap_or(Vec::new());

    match handler.store.select_project(project.clone()) {
        Ok(ref recs) => {
            println!("{}", cal_project(recs).as_string());

            let left: Vec<String> = recs
                .into_iter()
                .map(|rec| {
                    format!(
                        "{}\t{}\t{}",
                        rec.username,
                        rec.task,
                        human_duration(rec.duration)
                    )
                })
                .collect();

            let right: Vec<Element> = recs
                .into_iter()
                .map(|rec| {
                    make_table_row(vec![
                        rec.username.clone(),
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
            });
            let (h0, h1) = available
                .first()
                .map(|rec| {
                    (
                        format!(
                            "{} hours available, {} hours done\n",
                            dur(&rec.duration) / (1000 * 60 * 60),
                            done / (1000 * 60 * 60)
                        ),
                        div(vec![
                            div(code(format!(
                                "available: {} hours",
                                dur(&rec.duration) / (1000 * 60 * 60)
                            ))),
                            div(code(format!("done: {} hours", done / (1000 * 60 * 60)))),
                        ]),
                    )
                })
                .unwrap_or((format!("{} done", done), code(format!("done: {}", done))));
            Some((
                h0 + &left.join("\n"),
                div(vec![h1, table(right)]).as_string(),
            ))
        }
        Err(_) => None,
    }
}
