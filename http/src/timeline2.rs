use serde::{Deserialize, Serialize};
use serde_json::json;
use shell::store::{AggregatedTaskRecord, ConnectedStore, NoteRecord, ProjectRecord};
use std::{cmp::Ordering, convert::Infallible, str::FromStr, time::Duration};
use warp::Filter;

use crate::context::{with_context, ArcContext};

fn round_div(a: u64, b: u64) -> u64 {
    let p = a / b;
    let m = a % b;
    if m <= b / 2 {
        p
    } else {
        p + 1
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RelAbs {
    pub value: u64,
    pub percent: u64,
}

fn rel_abs(value: u64, percent: u64) -> RelAbs {
    RelAbs { value, percent }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Gauge {
    pub overtime: RelAbs,
    pub available: RelAbs,
    pub done: RelAbs,
}

fn gauge(record: &ProjectRecord, done: &Duration) -> Gauge {
    let d = done.as_secs() + 1;
    let provision = record.provision.map_or(0, |p| p.as_secs());
    let p = record.provision.map_or(d, |p| p.as_secs()) + 1;
    let ((done, done_percent), (avail, avail_percent), (over, over_percent)) = if d > p {
        (
            (d, round_div(p * 100, d)),
            (provision, 0),
            (d - p, round_div((d - p) * 100, d)),
        )
    } else {
        (
            (d, round_div(d * 100, p)),
            (provision, round_div((p - d) * 100, p)),
            (0, 0),
        )
    };

    Gauge {
        overtime: rel_abs(over / 3600, over_percent),
        available: rel_abs(avail / 3600, avail_percent),
        done: rel_abs(done / 3600, done_percent),
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TimelineProject {
    pub record: ProjectRecord,
    pub done: Duration,
    pub notes: Vec<NoteRecord>,
    pub gauge: Gauge,
    pub size: String,
}

const LOW: u64 = 50;
const MEDIUM: u64 = 250;
const HIGH: u64 = 1000;

fn get_size(n: u64) -> String {
    if n < LOW {
        String::from("small")
    } else if n < MEDIUM {
        String::from("medium")
    } else if n < HIGH {
        String::from("large")
    } else {
        String::from("huge")
    }
}

impl TimelineProject {
    fn new(record: ProjectRecord, done: Duration, notes: Vec<NoteRecord>) -> TimelineProject {
        let gauge = gauge(&record, &done);
        let size = get_size(record.provision.map(|d| d.as_secs() / 3600).unwrap_or(0));
        TimelineProject {
            record,
            done,
            notes,
            gauge,
            size,
        }
    }
}

fn cmp_by_deadline(a: &TimelineProject, b: &TimelineProject) -> Ordering {
    match (a.record.end_time, b.record.end_time) {
        (None, None) => Ordering::Equal,
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (Some(a), Some(b)) => a.cmp(&b),
    }
}

fn cmp_by_completion(a: &TimelineProject, b: &TimelineProject) -> Ordering {
    match (a.record.completed, b.record.completed) {
        (None, None) => Ordering::Equal,
        (Some(_), None) => Ordering::Greater,
        (None, Some(_)) => Ordering::Less,
        (Some(a), Some(b)) => b.cmp(&a),
    }
}

fn get_done(tasks: Vec<AggregatedTaskRecord>) -> std::time::Duration {
    tasks
        .iter()
        .fold(std::time::Duration::from_secs(0), |acc, task| {
            acc + task.duration
        })
}

fn get_projects(store: &mut ConnectedStore) -> Vec<TimelineProject> {
    store
        .select_all_project_info()
        .map(|rows| {
            let mut active_projects: Vec<TimelineProject> = rows
                .iter()
                .filter_map(|record| {
                    if record.completed.is_some() {
                        None
                    } else {
                        let tasks = store.select_project(record.name.clone());
                        let done = tasks.map(get_done);
                        let notes = store.select_notes(record.name.clone()).unwrap_or_default();
                        Some(TimelineProject::new(
                            record.clone(),
                            done.unwrap_or_else(|_| Duration::from_secs(0)),
                            notes,
                        ))
                    }
                })
                .collect();
            // let mut completed_projects: Vec<TimelineProject> = rows
            //     .iter()
            //     .filter_map(|record| {
            //         record.completed.map(|_| {
            //             let tasks = store.select_project(record.name.clone());
            //             let done = tasks.map(get_done);
            //             let notes = store.select_notes(record.name.clone()).unwrap_or_default();
            //             TimelineProject {
            //                 record: record.clone(),
            //                 done: done.unwrap_or_else(|_| Duration::from_secs(0)),
            //                 notes,
            //             }
            //         })
            //     })
            //     .collect();
            active_projects.sort_by(cmp_by_deadline);
            // completed_projects.sort_by(cmp_by_completion);
            // active_projects.extend(completed_projects);
            active_projects
        })
        .unwrap_or(Vec::new())
}

async fn timeline_handler(
    token: String,
    ctx: ArcContext<'_>,
) -> Result<impl warp::Reply, Infallible> {
    let base_path = format!("/{}/", token);
    if let Ok(mut ctx) = ctx.lock() {
        match ctx.render_with("timeline", &token, |c| {
            let projects = get_projects(c);
            json!({
                "projects": projects,
                "base": base_path,
            })
        }) {
            Ok(html) => Ok(warp::reply::html(html)),
            Err(err) => Ok(warp::reply::html(format!("Error rendering: {}", err))),
        }
    } else {
        Ok(warp::reply::html(String::from("raté ctx")))
    }
}

pub fn timeline(
    ctx: ArcContext,
) -> impl Filter<Extract = impl warp::Reply + '_, Error = warp::Rejection> + Clone + '_ {
    warp::path!(String / "timeline2")
        .and(warp::get())
        .and(with_context(ctx))
        .and_then(timeline_handler)
}
