use html::{body, div, h1, head, html, style, with_doctype, Element};
use shell::{
    store::{ProjectRecord, Store},
    util::{human_duration, human_st},
};
use std::{
    convert::Infallible,
    sync::{Arc, Mutex},
};
use std::{net::SocketAddr, path::Path};
use warp::Filter;

type ArcStore = Arc<Mutex<Store>>;

fn with_store(
    s: ArcStore,
) -> impl warp::Filter<Extract = (ArcStore,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || s.clone())
}

fn calendar(
    s: ArcStore,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("cal" / String)
        .and(warp::get())
        .and(with_store(s))
        .and_then(|uuid: String, s: ArcStore| async move {
            let store = s.lock().expect("This mutex should not hold");
            match store.select_cal(uuid) {
                Ok(cal) => Ok(warp::reply::html(cal.content)),
                Err(_) => Err(warp::reject()),
            }
        })
}

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

fn timeline(
    s: ArcStore,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("timeline")
        .and(warp::get())
        .and(with_store(s))
        .and_then(timeline_handler)
}

pub fn start_http(path: &Path, host: &str) {
    let addr: SocketAddr = host.parse().expect("Invalid address");
    let store = Store::new(path.clone()).expect("Failed to get a store for HTTP server");
    let arc_store = Arc::new(Mutex::new(store));

    std::thread::spawn(move || {
        let routes = calendar(arc_store.clone()).or(timeline(arc_store.clone()));
        // let runtime = tokio::runtime::Runtime::new().expect("Failed to start tokio runtime");
        let mut runtime = tokio::runtime::Builder::new()
            .basic_scheduler()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(warp::serve(routes).run(addr));
    });
}
