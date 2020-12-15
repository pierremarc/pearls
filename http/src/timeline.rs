use html::{
    anchor, body, details, div, em, h1, h2, head, html, no_display, paragraph, span, style,
    summary, with_doctype, Element, Empty,
};
use shell::{
    store::{AggregatedTaskRecord, NoteRecord, ProjectRecord, StoreError},
    util::date_time_from_st,
};
use std::{
    cmp::Ordering,
    convert::Infallible,
    time::{Duration, SystemTime},
};
use warp::Filter;

use crate::{with_base_path, with_store, ArcStore};

type TimelineProject = (ProjectRecord, std::time::Duration, Vec<NoteRecord>);

fn cmp_by_deadline(a: &TimelineProject, b: &TimelineProject) -> Ordering {
    match (a.0.end_time, b.0.end_time) {
        (None, None) => Ordering::Equal,
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (Some(a), Some(b)) => a.cmp(&b),
    }
}

fn cmp_by_completion(a: &TimelineProject, b: &TimelineProject) -> Ordering {
    match (a.0.completed, b.0.completed) {
        (None, None) => Ordering::Equal,
        (Some(_), None) => Ordering::Greater,
        (None, Some(_)) => Ordering::Less,
        (Some(a), Some(b)) => b.cmp(&a),
    }
}

fn format_date(t: &SystemTime) -> String {
    let dt = date_time_from_st(t);
    dt.format("%e %B %Y").to_string()
}

fn kv<K, V>(k: K, v: V) -> Element
where
    K: Into<html::Children>,
    V: Into<html::Children>,
{
    div([div(k).class("label"), div(v).class("value")]).class("info-wrapper")
}

fn remaining(provision: Duration, done: Duration) -> Element {
    let p = provision.as_secs();
    let d = done.as_secs();
    if d > p {
        kv("Overtime: ", span(format_hour(done - provision))).add_class("overtime")
    } else {
        kv("Remaining:", span(format_hour(provision - done))).add_class("remaining")
    }
}
fn deadline(end: &SystemTime) -> Element {
    let now = SystemTime::now();
    if *end < now {
        kv("Deadline:", &format_date(end)).add_class("not-in-time")
    } else {
        kv("Deadline:", &format_date(end))
    }
}

fn to_hour(d: Duration) -> u128 {
    let m = d.as_millis() / 3600;
    let m2 = m + 500;
    m2 / 1000
}

fn format_hour(d: Duration) -> String {
    format!("{}h", to_hour(d))
}

fn round_div(a: u64, b: u64) -> u64 {
    let p = a / b;
    let m = a % b;
    if m <= b / 2 {
        p
    } else {
        p + 1
    }
}

const LOW: u64 = 50;
const MEDIUM: u64 = 250;
const HIGH: u64 = 1000;

enum ProjectSize {
    Small,
    Medium,
    Large,
    Huge,
}

fn get_size(n: u64) -> ProjectSize {
    if n < LOW {
        return ProjectSize::Small;
    } else if n < MEDIUM {
        return ProjectSize::Medium;
    } else if n < HIGH {
        return ProjectSize::Large;
    } else {
        return ProjectSize::Huge;
    }
}

fn make_gauge(provision: &Duration, done: &Duration) -> Element {
    let p = provision.as_secs() + 1;
    let d = done.as_secs() + 1;
    let (done_percent, avail_percent, over_percent) = if d > p {
        (round_div(p * 100, d), 0, round_div((d - p) * 100, d))
    } else {
        (round_div(d * 100, p), round_div((p - d) * 100, p), 0)
    };

    let class_name = match get_size(p / 3600) {
        ProjectSize::Small => "project-gauge small",
        ProjectSize::Medium => "project-gauge medium",
        ProjectSize::Large => "project-gauge large",
        ProjectSize::Huge => "project-gauge huge",
    };

    div([
        div(Empty)
            .class("time-over")
            .set("style", format!("height:{}%;", over_percent)),
        div(Empty)
            .class("time-available")
            .set("style", format!("height:{}%;", avail_percent)),
        div(Empty)
            .class("time-done")
            .set("style", format!("height:{}%;", done_percent)),
    ])
    .class(class_name)
}

fn wrapper_class(opt_completed: Option<SystemTime>) -> String {
    match opt_completed {
        None => "project-wrapper".into(),
        Some(_) => "project-wrapper completed".into(),
    }
}

fn project_title(name: &str, base_path: String, opt_completed: Option<SystemTime>) -> Element {
    match opt_completed {
        None => h2(anchor(name).set("href", format!("{}calendar/{}", base_path, name))),
        Some(t) => h2([
            em(shell::util::st_to_datestring(&t)),
            span(" "),
            anchor(name).set("href", format!("{}calendar/{}", base_path, name)),
        ]),
    }
}

fn make_notes(notes: &Vec<NoteRecord>) -> Element {
    match notes.len() {
        0 => no_display(),
        _ => details([
            summary("notes"),
            div(notes
                .iter()
                .map(|note| {
                    div([
                        div(shell::util::st_to_datestring(&note.created_at)).class("note-date"),
                        div(shell::util::display_username(note.username.clone()))
                            .class("note-user"),
                        paragraph(note.content.clone()).class("note-content"),
                    ])
                    .class("note")
                })
                .collect::<Vec<_>>()),
        ])
        .class("notes"),
    }
}

fn make_full(
    name: &str,
    base_path: String,
    end: &SystemTime,
    provision: &Duration,
    done: &Duration,
    opt_completed: Option<SystemTime>,
    notes: &Vec<NoteRecord>,
) -> Element {
    div([
        make_gauge(provision, done),
        div([
            project_title(name, base_path, opt_completed),
            deadline(end),
            remaining(*provision, *done),
            kv("Provisioned:", &format_hour(*provision)),
            kv("Done:", &format_hour(*done)),
            make_notes(notes),
        ])
        .class("project-info"),
    ])
    .class(wrapper_class(opt_completed))
}

fn make_with_provision(
    name: &str,
    base_path: String,
    provision: &Duration,
    done: &Duration,
    opt_completed: Option<SystemTime>,
    notes: &Vec<NoteRecord>,
) -> Element {
    div([
        make_gauge(provision, done),
        div([
            project_title(name, base_path, opt_completed),
            remaining(*provision, *done),
            kv("Provisioned:", &format_hour(*provision)),
            kv("Done:", &format_hour(*done)),
            make_notes(notes),
        ])
        .class("project-info"),
    ])
    .class(wrapper_class(opt_completed))
}

fn make_with_end(
    name: &str,
    base_path: String,
    end: &SystemTime,
    done: &Duration,
    opt_completed: Option<SystemTime>,
    notes: &Vec<NoteRecord>,
) -> Element {
    div([
        make_gauge(done, done),
        div([
            project_title(name, base_path, opt_completed),
            deadline(end),
            kv("Done:", &format_hour(*done)),
            make_notes(notes),
        ])
        .class("project-info"),
    ])
    .class(wrapper_class(opt_completed))
}

fn make_bare(
    name: &str,
    base_path: String,
    done: &Duration,
    opt_completed: Option<SystemTime>,
    notes: &Vec<NoteRecord>,
) -> Element {
    div([
        make_gauge(done, done),
        div([
            project_title(name, base_path, opt_completed),
            kv("Done:", &format_hour(*done)),
            make_notes(notes),
        ])
        .class("project-info"),
    ])
    .class(wrapper_class(opt_completed))
}

fn get_done(tasks: Vec<AggregatedTaskRecord>) -> std::time::Duration {
    tasks
        .iter()
        .fold(std::time::Duration::from_secs(0), |acc, task| {
            acc + task.duration
        })
}

fn get_projects(s: ArcStore) -> Result<Vec<TimelineProject>, StoreError> {
    let store = s.lock().expect("hmmm");
    let projects = store.select_all_project_info().map(|rows| {
        let mut active_projects: Vec<TimelineProject> = rows
            .iter()
            .filter_map(|record| {
                if record.completed.is_some() {
                    None
                } else {
                    let tasks = store.select_project(record.name.clone());
                    let done = tasks.map(get_done);
                    let notes = store
                        .select_notes(record.name.clone())
                        .unwrap_or(Vec::new());
                    Some((
                        record.clone(),
                        done.unwrap_or(Duration::from_secs(0)),
                        notes,
                    ))
                }
            })
            .collect();
        let mut completed_projects: Vec<TimelineProject> = rows
            .iter()
            .filter_map(|record| {
                record.completed.map(|_| {
                    let tasks = store.select_project(record.name.clone());
                    let done = tasks.map(get_done);
                    let notes = store
                        .select_notes(record.name.clone())
                        .unwrap_or(Vec::new());
                    (
                        record.clone(),
                        done.unwrap_or(Duration::from_secs(0)),
                        notes,
                    )
                })
            })
            .collect();
        active_projects.sort_by(cmp_by_deadline);
        completed_projects.sort_by(cmp_by_completion);
        active_projects.sort_by(cmp_by_deadline);
        active_projects.extend(completed_projects);
        active_projects
    });

    projects
}

async fn timeline_handler(s: ArcStore, base_path: String) -> Result<impl warp::Reply, Infallible> {
    let css = style(String::from(include_str!("timeline.css"))).set("type", "text/css");

    match get_projects(s) {
        Err(_) => Ok(warp::reply::html(with_doctype(html([
            head(css),
            body(div("error: no projects found")),
        ])))),
        Ok(projects) => {
            let elements: Vec<Element> = projects
                .iter()
                .map(|(p, done, notes)| match (p.end_time, p.provision) {
                    (Some(end), Some(provision)) => make_full(
                        &p.name,
                        base_path.clone(),
                        &end,
                        &provision,
                        done,
                        p.completed,
                        notes,
                    ),
                    (None, Some(provision)) => make_with_provision(
                        &p.name,
                        base_path.clone(),
                        &provision,
                        done,
                        p.completed,
                        notes,
                    ),
                    (Some(end), None) => {
                        make_with_end(&p.name, base_path.clone(), &end, done, p.completed, notes)
                    }
                    (None, None) => make_bare(&p.name, base_path.clone(), done, p.completed, notes),
                })
                .collect();

            Ok(warp::reply::html(with_doctype(html([
                head(css),
                body([h1("Timeline"), div(elements).class("projects")]),
            ]))))
        }
    }
}

pub fn timeline(
    s: ArcStore,
    token: String,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("timeline")
        .and(warp::get())
        .and(with_store(s))
        .and(with_base_path(token))
        .and_then(timeline_handler)
}

#[cfg(test)]
mod tests {
    use super::round_div;

    #[test]
    fn test_round() {
        assert_eq!(2, round_div(5, 2));
        assert_eq!(2, round_div(10, 6));
    }
}
