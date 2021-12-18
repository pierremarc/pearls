use std::{cmp::Ordering, convert::Infallible, time};

use html::{anchor, body, div, h2, h3, head, html, span, style, title, with_doctype, Empty};
use shell::{
    store::{ConnectedStore, ProjectRecord, StoreError},
    util::{display_username, human_duration},
};
use warp::Filter;

use crate::common::with_store;

fn document() -> String {
    let css = style(String::from(include_str!("landing.css"))).set("type", "text/css");
    let title = title(Empty).append_text("pearls");
    let content = div([
        h2("pearls"),
        span("A bot, counting time on"),
        anchor("matrix").set("href", "https://matrix.org"),
        span("network for"),
        anchor("atelier cartographique").set("href", "https://www.atelier-cartographique.be"),
        span("and friends."),
    ])
    .class("content");

    with_doctype(html([head([title, css]), body(content)]))
}

pub fn landing() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get().map(|| warp::reply::html(document()))
}

fn cmp_by_deadline(a: &ProjectRecord, b: &ProjectRecord) -> Ordering {
    match (a.end_time, b.end_time) {
        (None, None) => Ordering::Equal,
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (Some(a), Some(b)) => a.cmp(&b),
    }
}

fn get_projects(store: &mut ConnectedStore) -> Result<Vec<ProjectRecord>, StoreError> {
    let projects = store.select_all_project_info().map(|rows| {
        let mut active_projects: Vec<ProjectRecord> = rows
            .iter()
            .filter_map(|record| {
                if record.completed.is_some() {
                    None
                } else {
                    Some(record.clone())
                }
            })
            .collect();
        active_projects.sort_by(cmp_by_deadline);
        active_projects
    });

    projects
}

async fn room(
    token: String,
    arc_store: crate::common::ArcStore,
) -> Result<impl warp::Reply, Infallible> {
    let now = time::SystemTime::now();

    let mut content = div([
        h2(&token),
        div([
            span("got to"),
            anchor("timeline").set("href", format!("/{}/timeline", &token)),
            h3("active users"),
        ]),
    ])
    .class("content");
    if let Ok(mut store) = arc_store.lock() {
        if let Ok(connected) = store.connect(&token) {
            match connected.select_current_task() {
                Ok(recs) if recs.len() > 0 => {
                    for rec in recs {
                        if let Ok(duration) = rec.end_time.duration_since(now) {
                            content = content.append(
                                div(format!(
                                    "{} is performing {} on {}, they will be done in {}",
                                    display_username(&rec.username),
                                    rec.task,
                                    rec.project,
                                    human_duration(duration)
                                ))
                                .class("record"),
                            )
                        }
                    }
                }
                _ => {
                    content = content.append_text("Nothing is happening right now.");
                }
            }

            if let Ok(projects) = get_projects(connected) {
                content = content.append(h3("active projects"));
                for project in projects {
                    content = content.append(div([
                        span("→"),
                        anchor(&project.name)
                            .set("href", format!("/{}/calendar/{}", &token, &project.name)),
                    ]));
                }
            }
        }
    }
    let css = style(String::from(include_str!("landing.css"))).set("type", "text/css");
    let title = title(Empty).append_text(&token);
    let body = body(content);
    Ok(warp::reply::html(with_doctype(html([
        head([title, css]),
        body,
    ]))))
}

pub fn room_landing(
    s: crate::common::ArcStore,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!(String)
        .and(warp::get())
        .and(with_store(s))
        .and_then(room)
}
