use html::{body, div, h1, head, html, style, with_doctype, Element};
use shell::{
    store::ProjectRecord,
    util::{human_duration, human_st},
};
use std::convert::Infallible;
use warp::Filter;

use crate::{with_store, ArcStore};

type TimelineProject = (ProjectRecord, Option<std::time::Duration>);

async fn timeline_handler(s: ArcStore) -> Result<impl warp::Reply, Infallible> {
    let store = s.lock().expect("hmmm");
    let projects = store.select_all_project_info().map(|rows| {
        let projects: Vec<TimelineProject> = rows
            .iter()
            .map(|record| {
                let tasks = store.select_project(record.name.clone());
                let done = tasks.map(|tasks| {
                    tasks
                        .iter()
                        .fold(std::time::Duration::from_secs(0), |acc, task| {
                            acc + task.duration
                        })
                });
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
                    (Some(end), Some(provision), Some(done)) => div(vec![
                        h1(&p.name),
                        div(format!("started: {}", human_st(&p.start_time))),
                        div(format!("due date: {}", human_st(&end))),
                        div(format!("provision: {}", human_duration(provision))),
                        div(format!("done: {}", human_duration(done))),
                    ]),
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
                body(elements),
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
