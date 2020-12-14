use html::{body, div, em, h1, h2, head, html, span, style, with_doctype, Element, Empty};
use shell::{
    store::{AggregatedTaskRecord, ProjectRecord, StoreError},
    util::date_time_from_st,
};
use std::{
    cmp::Ordering,
    convert::Infallible,
    time::{Duration, SystemTime},
};
use warp::Filter;

use crate::{with_store, ArcStore};

type TimelineProject = (ProjectRecord, std::time::Duration);

fn cmp_by_deadline(a: &TimelineProject, b: &TimelineProject) -> Ordering {
    if a.0.end_time.is_some() & b.0.end_time.is_none() {
        return Ordering::Less;
    }
    if a.0.end_time.is_none() & b.0.end_time.is_some() {
        return Ordering::Greater;
    }
    let now = std::time::SystemTime::now();
    let a_remaining =
        a.0.end_time
            .and_then(|t| now.duration_since(t).ok())
            .unwrap_or(Duration::new(u64::MAX, 1_000_000_000 - 1));
    let b_remaining =
        b.0.end_time
            .and_then(|t| now.duration_since(t).ok())
            .unwrap_or(Duration::new(u64::MAX, 1_000_000_000 - 1));

    if a_remaining == b_remaining {
        a.0.id.cmp(&b.0.id)
    } else {
        a_remaining.cmp(&b_remaining)
    }
}

fn cmp_by_completion(a: &TimelineProject, b: &TimelineProject) -> Ordering {
    if a.0.completed.is_some() & b.0.completed.is_none() {
        return Ordering::Less;
    }
    if a.0.completed.is_none() & b.0.completed.is_some() {
        return Ordering::Greater;
    }

    let a_elapsed =
        a.0.completed
            .and_then(|t| t.elapsed().ok())
            .unwrap_or(Duration::new(u64::MAX, 1_000_000_000 - 1));
    let b_elapsed =
        b.0.completed
            .and_then(|t| t.elapsed().ok())
            .unwrap_or(Duration::new(u64::MAX, 1_000_000_000 - 1));

    if a_elapsed == b_elapsed {
        a.0.id.cmp(&b.0.id)
    } else {
        a_elapsed.cmp(&b_elapsed)
    }
}

fn format_date(t: &SystemTime) -> String {
    let dt = date_time_from_st(t);
    dt.format("%F").to_string()
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

fn make_gauge(provision: &Duration, done: &Duration) -> Element {
    let p = provision.as_secs() + 1;
    let d = done.as_secs() + 1;
    let (done_percent, avail_percent, over_percent) = if d > p {
        (round_div(p * 100, d), 0, round_div((d - p) * 100, d))
    } else {
        (round_div(d * 100, p), round_div((p - d) * 100, p), 0)
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
    .class("project-gauge")
}

fn wrapper_class(opt_completed: Option<SystemTime>) -> String {
    match opt_completed {
        None => "project-wrapper".into(),
        Some(_) => "project-wrapper completed".into(),
    }
}

fn project_title(name: &str, opt_completed: Option<SystemTime>) -> Element {
    match opt_completed {
        None => h2(name),
        Some(t) => h2([em(shell::util::st_to_datestring(&t)), span(" "), span(name)]),
    }
}

fn make_full(
    name: &str,
    _start_time: &SystemTime,
    end: &SystemTime,
    provision: &Duration,
    done: &Duration,
    opt_completed: Option<SystemTime>,
) -> Element {
    div([
        make_gauge(provision, done),
        div([
            project_title(name, opt_completed),
            kv("Deadline:", &format_date(end)),
            remaining(*provision, *done),
            kv("Provisioned:", &format_hour(*provision)),
            kv("Done:", &format_hour(*done)),
        ]),
    ])
    .class(wrapper_class(opt_completed))
}

fn make_with_provision(
    name: &str,
    _start_time: &SystemTime,
    provision: &Duration,
    done: &Duration,
    opt_completed: Option<SystemTime>,
) -> Element {
    div([
        make_gauge(provision, done),
        div([
            project_title(name, opt_completed),
            remaining(*provision, *done),
            kv("Provisioned:", &format_hour(*provision)),
            kv("Done:", &format_hour(*done)),
        ]),
    ])
    .class(wrapper_class(opt_completed))
}

fn make_with_end(
    name: &str,
    _start_time: &SystemTime,
    end: &SystemTime,
    done: &Duration,
    opt_completed: Option<SystemTime>,
) -> Element {
    div([
        make_gauge(done, done),
        div([
            project_title(name, opt_completed),
            kv("Deadline:", &format_date(end)),
            kv("Done:", &format_hour(*done)),
        ]),
    ])
    .class(wrapper_class(opt_completed))
}

fn make_bare(
    name: &str,
    _start_time: &SystemTime,
    done: &Duration,
    opt_completed: Option<SystemTime>,
) -> Element {
    div([
        make_gauge(done, done),
        div([
            project_title(name, opt_completed),
            kv("Done:", &format_hour(*done)),
        ]),
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
                    Some((record.clone(), done.unwrap_or(Duration::from_secs(0))))
                }
            })
            .collect();
        let mut completed_projects: Vec<TimelineProject> = rows
            .iter()
            .filter_map(|record| {
                record.completed.map(|_| {
                    let tasks = store.select_project(record.name.clone());
                    let done = tasks.map(get_done);
                    (record.clone(), done.unwrap_or(Duration::from_secs(0)))
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

async fn timeline_handler(s: ArcStore) -> Result<impl warp::Reply, Infallible> {
    let css = style(String::from(include_str!("timeline.css"))).set("type", "text/css");

    match get_projects(s) {
        Err(_) => Ok(warp::reply::html(with_doctype(html([
            head(css),
            body(div("error: no projects found")),
        ])))),
        Ok(projects) => {
            let elements: Vec<Element> = projects
                .iter()
                .map(|(p, done)| match (p.end_time, p.provision) {
                    (Some(end), Some(provision)) => {
                        make_full(&p.name, &p.start_time, &end, &provision, done, p.completed)
                    }
                    (None, Some(provision)) => {
                        make_with_provision(&p.name, &p.start_time, &provision, done, p.completed)
                    }
                    (Some(end), None) => {
                        make_with_end(&p.name, &p.start_time, &end, done, p.completed)
                    }
                    (None, None) => make_bare(&p.name, &p.start_time, done, p.completed),
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
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("timeline")
        .and(warp::get())
        .and(with_store(s))
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
