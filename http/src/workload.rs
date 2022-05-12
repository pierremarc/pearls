use std::time::SystemTime;

use chrono::{Datelike, Duration};
use html::{div, Element, Empty};
use shell::{
    cal::month_name,
    plan::{find_loads, next_monday, plan_all, WorkLoad},
    store::ConnectedStore,
    util::{date_time_from_st, display_username},
};

fn make_load(work_load: &WorkLoad) -> Element {
    let user = display_username(work_load.user());
    let project = work_load.project();
    let start = work_load.start().format("%F");
    let hours = work_load.load().num_hours();
    let title = format!("{start} --- {hours}");
    let load: Vec<Element> = (0..work_load.load().num_hours())
        .map(|_| div(Empty).class("load-hour"))
        .collect();
    div([
        div(format!("{user} {project}")).class("username"),
        div(load).class("load-value"),
    ])
    .class("load")
    .set("title", &title)
}

fn make_date(start: &chrono::DateTime<chrono::Local>) -> Element {
    div(start.format("%F").to_string()).class("date")
}

fn make_week(start: &chrono::DateTime<chrono::Local>, loads: Vec<&WorkLoad>) -> Element {
    div([
        make_date(start),
        div(loads.into_iter().map(make_load).collect::<Vec<_>>()).class("load-list"),
    ])
    .class("week-load")
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
    let mut start = next_monday(&date_time_from_st(&SystemTime::now()));
    let mut months: Vec<Element> = Vec::new();
    let mut weeks: Vec<Element> = Vec::new();
    while start < max {
        let end = next_monday(&start);
        let loads = find_loads(&plan, &start, &end);
        weeks.push(make_week(&start, loads));
        if start.month() < end.month() || end >= max {
            months.push(
                div([div(month_name(&start)).class("month-name"), div(weeks)]).class("month-load"),
            );
            weeks = Vec::new();
        }
        start = end;
    }

    div(months).class("workload-block")
}
