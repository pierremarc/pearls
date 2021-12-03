use crate::common::{with_store, ArcStore};
use chrono::Datelike;
use html::{
    anchor, body, div, h1, head, html, no_display, span, style, with_doctype, Element, Empty,
};
use shell::cal::{day_of_week, month_name, Calendar, CalendarEvent, CalendarItem, LocalTime};
use shell::store::TaskRecord;
use shell::util::{
    after_once, date_time_from_st, display_username, dur, human_duration, string, ts,
};
use std::collections::HashSet;
use std::time::{self, SystemTime};
use warp::Filter;

fn format_tasklist(tasks: impl Iterator<Item = TaskRecord>) -> Vec<Element> {
    tasks
        .map(|rec| {
            div(vec![
                div(format!("{}({})", display_username(&rec.username), rec.task)),
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
        .collect()
}

fn make_day(day: &LocalTime, tasks: impl Iterator<Item = TaskRecord>, class: &str) -> Element {
    div(vec![
        div(format!("{} {}", day_of_week(&day), day.day())).set("class", "weekday"),
        div(format_tasklist(tasks)).set("class", "task-list"),
    ])
    .set("class", &format!("day {}", class))
}

fn month_and_year(d: &chrono::DateTime<chrono::Local>) -> String {
    format!("{} {}", month_name(d), d.year())
}

trait Interval {
    fn start(&self) -> SystemTime;
    fn end(&self) -> SystemTime;
}

impl Interval for TaskRecord {
    fn start(&self) -> SystemTime {
        self.start_time
    }
    fn end(&self) -> SystemTime {
        self.end_time
    }
}
impl Interval for CalendarEvent<TaskRecord> {
    fn start(&self) -> SystemTime {
        self.data.start_time
    }
    fn end(&self) -> SystemTime {
        self.data.end_time
    }
}

fn max<'a, I>(a: &'a I, b: &'a I) -> &'a I
where
    I: Interval,
{
    if a.end() >= b.end() {
        a
    } else {
        b
    }
}

fn min<'a, I>(a: &'a I, b: &'a I) -> &'a I
where
    I: Interval,
{
    if a.start() <= b.start() {
        a
    } else {
        b
    }
}

fn make_csv_link<I>(base_url_tabular: &str, events: &Vec<I>) -> Element
where
    I: Interval,
{
    match (events.first(), events.last()) {
        (Some(a), Some(b)) => {
            let (first, last) = events
                .iter()
                .fold((a, b), |(a, b), e| (min(a, e), max(b, e)));
            anchor("csv").set(
                "href",
                format!(
                    "/{}/{}/{}",
                    base_url_tabular,
                    ts(&first.start()) - 1,
                    ts(&last.end())
                ),
            )
        }
        _ => no_display(),
    }
}

fn cal_project(recs: &Vec<TaskRecord>, base_url_tabular: &str) -> Element {
    let mut cal: Calendar<TaskRecord> = Calendar::new();
    for t in recs.into_iter() {
        cal.push(
            date_time_from_st(&t.start_time),
            date_time_from_st(&t.end_time),
            t.clone(),
        );
    }

    let main = div(Empty).set("class", "calendar");
    let cur_month = div(Empty).set("class", "month initial");
    let cur_week = div(Empty).set("class", "week initial");

    let mut fm = after_once();
    let mut fw = after_once();

    let (res, b, w) = cal
        .iter()
        .fold((main, cur_month, cur_week), |(b, m, w), item| match item {
            CalendarItem::Month(d, events) => fm.map(
                (b, m, w),
                |(b, _, w)| {
                    (
                        b,
                        div([
                            h1(month_and_year(&d)),
                            make_csv_link(base_url_tabular, &events),
                        ])
                        .set("class", "month"),
                        w,
                    )
                },
                |(b, m, w)| {
                    (
                        b + (m + w),
                        div([
                            h1(month_and_year(&d)),
                            make_csv_link(base_url_tabular, &events),
                        ])
                        .set("class", "month"),
                        div(Empty).set("class", "week empty"),
                    )
                },
            ),
            CalendarItem::Week(_d, _events) => fw.map(
                (b, m, w),
                |acc| acc,
                |(b, m, w)| {
                    (
                        b,
                        m + w,
                        div(Empty).set("class", format!("week start-day-{}", _d.day())),
                    )
                },
            ),
            CalendarItem::EmptyDay(d, events) => (
                b,
                m,
                w + make_day(&d, events.iter().map(|e| e.data.clone()), "out-month"),
            ),
            CalendarItem::Day(d, events) => (
                b,
                m,
                w + make_day(&d, events.iter().map(|e| e.data.clone()), "in-month"),
            ),
            _ => (b, m, w),
        });

    res + (b + w)
}

fn cal(token: String, store: ArcStore, project: String) -> Option<String> {
    if let Ok(mut store) = store.lock() {
        if let Ok(connected) = store.connect(&token) {
            let available = connected
                .select_project_info(project.clone())
                .map(|rec| rec.provision.map_or(0, |d| dur(&d)) / (1000 * 60 * 60))
                .unwrap_or(0);
            let base_url_tabular = format!("{}/tabular/{}", &token, &project);
            match connected.select_project_detail(project.clone()) {
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

                    let done = recs.iter().fold(0, |acc, rec| {
                        acc + dur(&rec
                            .end_time
                            .duration_since(rec.start_time)
                            .unwrap_or(time::Duration::from_secs(0)))
                    }) / (1000 * 60 * 60);

                    let cal_element = cal_project(recs, &base_url_tabular);
                    let title = h1(names);
                    let subtitle = div(vec![
                        div(vec![
                            span(string("Done: ")),
                            span(format!("{} hours", done)),
                        ]),
                        div(vec![
                            span(string("Avail: ")),
                            span(format!("{} hours", available)),
                        ]),
                        make_csv_link(&base_url_tabular, recs),
                    ])
                    .set("class", "summary");
                    let css = style(String::from(include_str!("cal.css"))).set("type", "text/css");
                    let html_string = with_doctype(html(vec![
                        head(css),
                        body(vec![title, subtitle, cal_element]),
                    ]));

                    Some(html_string)
                }
                Err(err) => Some(format!("Store Error: {}", err)),
            }
        } else {
            Some("Failed to connect to DB".into())
        }
    } else {
        Some("Could Not Acquire A Lock On Store".into())
    }
}

pub fn calendar(
    s: ArcStore,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!(String / "calendar" / String / String)
        .and(warp::get())
        .and(with_store(s))
        .and_then(
            |token: String, client: String, name: String, store: ArcStore| async move {
                match cal(token, store, format!("{}/{}", client, name)) {
                    Some(body) => Ok(warp::reply::html(body)),
                    None => Err(warp::reject()),
                }
            },
        )
}
