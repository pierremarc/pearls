use html::{body, div, h1, h2, head, html, span, style, with_doctype, Element, Empty};
use shell::{
    store::{AggregatedTaskRecord, ProjectRecord},
    util::{date_time_from_st, human_duration, human_st},
};
use std::{
    convert::Infallible,
    time::{Duration, SystemTime},
};
use warp::Filter;

use crate::{with_store, ArcStore};

type TimelineProject = (ProjectRecord, Option<std::time::Duration>);

fn format_date(t: &SystemTime) -> String {
    let dt = date_time_from_st(t);
    dt.format("%F").to_string()
}

fn kv<K, V>(k: K, v: V) -> Element
where
    K: Into<html::Children>,
    V: Into<html::Children>,
{
    div(vec![div(k).class("label"), div(v).class("value")]).class("info-wrapper")
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

fn to_hour(d: Duration) -> u64 {
    d.as_secs() / 3600
}
fn format_hour(d: Duration) -> String {
    format!("{}h", to_hour(d))
}

fn make_gauge(provision: Duration, done: Duration) -> Element {
    let p = provision.as_secs();
    let d = done.as_secs();
    let (done_percent, avail_percent, over_percent) = if d > p {
        (p * 100 / d, 0, (d - p) * 100 / d)
    } else {
        (d * 100 / p, (p - d) * 100 / p, 0)
    };

    div(vec![
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

fn make_full(
    name: &str,
    _start_time: &SystemTime,
    end: &SystemTime,
    provision: Duration,
    done: Duration,
) -> Element {
    div(vec![
        make_gauge(provision, done),
        div(vec![
            h2(name),
            kv("Deadline:", &format_date(end)),
            remaining(provision, done),
            kv("Provisioned:", &format_hour(provision)),
            kv("Done:", &format_hour(done)),
        ]),
    ])
    .class("project-wrapper")
}

fn get_done(tasks: Vec<AggregatedTaskRecord>) -> std::time::Duration {
    println!("get_done {}", tasks.len());
    tasks
        .iter()
        .fold(std::time::Duration::from_secs(0), |acc, task| {
            println!("{} {:?}", task.project, task.duration);
            acc + task.duration
        })
}

async fn timeline_handler(s: ArcStore) -> Result<impl warp::Reply, Infallible> {
    let store = s.lock().expect("hmmm");
    let projects = store.select_all_project_info().map(|rows| {
        let projects: Vec<TimelineProject> = rows
            .iter()
            .map(|record| {
                let tasks = store.select_project(record.name.clone());
                let done = tasks.map(get_done);
                println!(">> {}: {:?}", record.name.clone(), done);
                (record.clone(), done.ok())
            })
            .collect();
        projects
    });
    let css = style(String::from(include_str!("timeline.css"))).set("type", "text/css");

    match projects {
        Err(_) => Ok(warp::reply::html(with_doctype(html(vec![
            head(css),
            body(vec![div("error: no projects found")]),
        ])))),
        Ok(projects) => {
            let elements: Vec<Element> = projects
                .iter()
                .map(|(p, d)| match (p.end_time, p.provision, *d) {
                    (Some(end), Some(provision), Some(done)) => {
                        make_full(&p.name, &p.start_time, &end, provision, done)
                    }
                    (None, Some(provision), Some(done)) => div(vec![
                        h1(&p.name),
                        div(format!("started: {}", human_st(&p.start_time))),
                        div(format!("provision: {}", human_duration(provision))),
                        div(format!("done: {}", human_duration(done))),
                    ]),
                    (None, None, Some(done)) => div(vec![
                        h1(&p.name),
                        div(format!("started: {}", human_st(&p.start_time))),
                        div(format!("done: {}", human_duration(done))),
                    ]),
                    (None, None, None) => div(vec![
                        h1(&p.name),
                        div(format!("started: {}", human_st(&p.start_time))),
                    ]),
                    (Some(end), None, None) => div(vec![
                        h1(&p.name),
                        div(format!("started: {}", human_st(&p.start_time))),
                        div(format!("due date: {}", human_st(&end))),
                    ]),
                    (Some(end), Some(provision), None) => div(vec![
                        h1(&p.name),
                        div(format!("started: {}", human_st(&p.start_time))),
                        div(format!("due date: {}", human_st(&end))),
                        div(format!("provision: {}", human_duration(provision))),
                    ]),
                    (Some(end), None, Some(done)) => div(vec![
                        h1(&p.name),
                        div(format!("started: {}", human_st(&p.start_time))),
                        div(format!("due date: {}", human_st(&end))),
                        div(format!("done: {}", human_duration(done))),
                    ]),
                    (None, Some(provision), None) => div(vec![
                        h1(&p.name),
                        div(format!("started: {}", human_st(&p.start_time))),
                        div(format!("provision: {}", human_duration(provision))),
                    ]),
                })
                .collect();

            Ok(warp::reply::html(with_doctype(html(vec![
                head(css),
                body(vec![h1("Timeline"), div(elements).class("projects")]),
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
