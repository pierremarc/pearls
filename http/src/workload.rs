use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
    time::SystemTime,
};

use chrono::{DateTime, Datelike, Duration, Local};
use html::{body, div, h2, h3, head, html, style, with_doctype, Element, Empty};
use shell::{
    cal::month_name,
    plan::{find_loads, next_monday, plan_all, WorkLoad, WorkPlan},
    store::{ConnectedStore, ProjectRecord},
    util::{date_time_from_st, display_username},
};
use warp::Filter;

use crate::common::{with_store, ArcStore};

struct UserMap {
    umap: HashMap<String, String>,
}

impl UserMap {
    fn new() -> Self {
        UserMap {
            umap: HashMap::new(),
        }
    }

    fn alias(&mut self, username: &str) -> String {
        if self.umap.contains_key(username) {
            return {
                match self.umap.get(username) {
                    Some(alias) => alias.clone(),
                    None => String::from(username),
                }
            };
        }
        let alias = format!("user-{}", self.umap.len() + 1);
        self.umap.insert(String::from(username), alias.clone());
        alias
    }
}

fn make_load(work_load: &WorkLoad, umap: &mut UserMap) -> Element {
    // let user = display_username(work_load.user());
    let project = work_load.project();
    let start = work_load.start().format("%F");
    let hours = work_load.load().num_hours();
    let title = format!("{start} --- {hours}");
    let load: Vec<Element> = (0..work_load.load().num_hours())
        .map(|_| div(Empty).class("load-hour"))
        .collect();
    div([
        div(project).class("project-name"),
        div(load).class("load-value"),
    ])
    .class(format!("load-wrapper {}", umap.alias(work_load.user())))
    .set("title", &title)
}

fn make_date(start: &chrono::DateTime<chrono::Local>) -> Element {
    h3(start.format("%F").to_string()).class("date")
}

fn make_week(
    start: &chrono::DateTime<chrono::Local>,
    loads: Vec<&WorkLoad>,
    umap: &mut UserMap,
    deadlines: Element,
) -> Element {
    div([
        make_date(start),
        deadlines,
        div(loads
            .into_iter()
            .map(|load| make_load(load, umap))
            .collect::<Vec<_>>())
        .class("load-list"),
    ])
    .class("week-load")
}

fn make_users(plan: &WorkPlan, umap: &mut UserMap) -> Element {
    let user_list = plan
        .iter()
        .map(|(username, _)| username)
        .collect::<HashSet<_>>()
        .into_iter()
        .map(|username| div(display_username(username)).class(umap.alias(username)))
        .collect::<Vec<_>>();

    div(user_list).class("user-list")
}

fn in_window(t: &SystemTime, start: &DateTime<Local>, end: &DateTime<Local>) -> bool {
    let dt = &date_time_from_st(t);
    dt >= start && dt < end
}

fn make_deadlines(
    projects: &Vec<ProjectRecord>,
    start: &DateTime<Local>,
    end: &DateTime<Local>,
) -> Element {
    let deadlines: Vec<&ProjectRecord> = projects
        .iter()
        .filter(|p| {
            p.end_time
                .map(|t| in_window(&t, start, end))
                .unwrap_or(false)
        })
        .collect();
    if deadlines.len() > 0 {
        div(deadlines
            .iter()
            .map(|p| {
                div(&p.name).class("project-name").set(
                    "title",
                    format!("{}", date_time_from_st(&p.end_time.unwrap()).format("%F")),
                )
            })
            .collect::<Vec<_>>())
        .class("dealines")
    } else {
        div(Empty).class("deadlines empty")
    }
}

pub fn render_workload(conn: &ConnectedStore) -> Element {
    let projects = conn.select_all_project_info().unwrap();
    let intents = conn.select_intent_all().unwrap();
    let avails = conn.select_avail_all().unwrap();
    let dones = conn.select_current_task().unwrap();
    let plan = plan_all(&projects, &intents, &avails, &dones, SystemTime::now());

    let max_avail = date_time_from_st(
        &avails
            .iter()
            .fold(SystemTime::now(), |acc, a| acc.max(a.end_time)),
    );
    let year = date_time_from_st(&SystemTime::now()) + Duration::days(361);
    let max = max_avail.max(year);
    let now = date_time_from_st(&SystemTime::now()).date();
    let mut start = now.and_hms(0, 0, 0);
    // let mut start = next_monday(&dt);
    let mut months: Vec<Element> = Vec::new();
    let mut weeks: Vec<Element> = Vec::new();
    let mut umap = UserMap::new();

    while start < max {
        let end = next_monday(&start);
        // let sunday = dbg!(end - chrono::Duration::hours(12));
        let loads = find_loads(&plan, &start, &end);
        let deadlines = make_deadlines(&projects, &start, &end);
        weeks.push(make_week(&start, loads, &mut umap, deadlines));
        if start.month() != end.month() || end >= max {
            months.push(
                div([
                    h2(month_name(&start)).class("month-name"),
                    div(weeks).class("month-load"),
                ])
                .class("month-load-wrapper"),
            );
            weeks = Vec::new();
        }
        start = end;
    }

    div([make_users(&plan, &mut umap), div(months).class("months")]).class("workload-block")
}

async fn workload_handler(
    token: String,
    arc_store: crate::common::ArcStore,
) -> Result<impl warp::Reply, Infallible> {
    let css = style(String::from(include_str!("workload.css"))).set("type", "text/css");
    // let base_path = format!("/{}/", token);
    if let Ok(mut store) = arc_store.lock() {
        if let Ok(connected) = store.connect_existing(&token) {
            return Ok(warp::reply::html(with_doctype(html([
                head(css),
                body(render_workload(connected)),
            ]))));
        }
    }

    Ok(warp::reply::html(with_doctype(html([
        head(css),
        body(div("Error")),
    ]))))
}

pub fn workload(
    s: ArcStore,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!(String / "load")
        .and(warp::get())
        .and(with_store(s))
        .and_then(workload_handler)
}
