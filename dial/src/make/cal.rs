use crate::bot;
use chrono::Datelike;
use html::{anchor, body, div, h1, h2, head, html, span, style, with_doctype, Element};
use shell::cal::{day_of_week, month, month_name, week};
use shell::store::TaskRecord;
use shell::util::{date_time_from_st, display_username, human_duration, st_from_date_time};
use std::time;

fn find_items(
    recs: &Vec<TaskRecord>,
    start: time::SystemTime,
    end: time::SystemTime,
) -> Vec<&TaskRecord> {
    recs.iter()
        .filter(|rec| {
            (start < rec.start_time && rec.start_time < end)
                || (start < rec.end_time && rec.end_time < end)
                || (start < rec.start_time && rec.end_time < end)
        })
        .collect()
}

fn cal_project(recs: &Vec<TaskRecord>) -> Element {
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
    let start = date_time_from_st(&start_time);
    let mut current_week = week(start.year(), start.month(), start.day());
    let mut current_month = month(start.year(), start.month());
    let mut all_weeks: Vec<Element> = Vec::new();

    let (mut start_month, mut end_month) = current_month.interval();
    while start_month < end {
        println!("Month {}", start_month.month());
        all_weeks.push(
            h2(format!(
                "{} {}",
                month_name(&start_month),
                start_month.year()
            ))
            .set("class", "month-date"),
        );

        let weeks: Vec<Element> = current_month
            .iter()
            .map(|(_, _)| {
                let (mut start_week, _) = current_week.interval();
                let mut all_days: Vec<Element> = Vec::new();

                while start_week < end_month {
                    println!("Week {}", start_week.iso_week().week());
                    let days: Vec<Element> = current_week
                        .iter()
                        .map(|(s, e)| {
                            let items: Vec<Element> =
                                find_items(recs, st_from_date_time(&s), st_from_date_time(&e))
                                    .iter()
                                    .map(|rec| {
                                        div(vec![
                                            div(format!("{}", display_username(&rec.username))),
                                            div(format!("{}({})", rec.project, rec.task)),
                                            div(format!(
                                                "{}",
                                                human_duration(
                                                    rec.end_time
                                                        .duration_since(rec.start_time)
                                                        .unwrap_or(time::Duration::from_secs(0))
                                                )
                                            )),
                                        ])
                                        .set("class", "task")
                                    })
                                    .collect();
                            div(vec![
                                div(format!("{} {}", String::from(day_of_week(&s)), s.day()))
                                    .set("class", "weekday"),
                                div(items).set("class", "task-list"),
                            ])
                            .set("class", "day")
                        })
                        .collect();
                    all_days.push(div(days).set("class", "week"));

                    current_week = current_week.next();
                    let (new_start_week, _) = current_week.interval();
                    start_week = new_start_week;
                }

                if all_days.len() > 0 {
                    div(all_days).set("class", "month")
                } else {
                    div(String::new())
                        .set("style", "display:none;")
                        .set("class", "empty")
                }
            })
            .collect();

        all_weeks.extend(weeks.into_iter());
        current_month = current_month.next();
        let (new_start_month, new_end_month) = current_month.interval();
        start_month = new_start_month;
        end_month = new_end_month;
    }

    div(all_weeks)
}

pub fn cal(handler: &mut bot::CommandHandler, project: String) -> Option<(String, String)> {
    match handler.store.select_project_detail(project.clone()) {
        Ok(ref recs) => {
            let cal_element = cal_project(recs);
            let title = h1(project);
            let css = style(String::from(include_str!("cal.css"))).set("type", "text/css");
            let html_string = with_doctype(html(vec![head(css), body(vec![title, cal_element])]));
            match handler.store.insert_cal(html_string) {
                Ok(uuid) => Some((
                    format!("can be seen at http://{}/cal/{}", handler.host, uuid),
                    span(
                        anchor(format!("follow me"))
                            .set("href", &format!("http://{}/cal/{}", handler.host, uuid)),
                    )
                    .as_string(),
                )),
                Err(err) => Some((format!("{}", err), String::new())),
            }
        }
        Err(err) => Some((format!("{}", err), String::new())),
    }
}
