use serde::{Deserialize, Serialize};
use serde_json::json;
use shell::store::{AggregatedTaskRecord, ConnectedStore, NoteRecord, ProjectRecord};
use std::{cmp::Ordering, convert::Infallible, time::Duration};
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
pub struct Gauge {
    pub overtime: u64,
    pub available: u64,
    pub done: u64,
}

fn gauge(record: &ProjectRecord, done: &Duration) -> Gauge {
    let d = done.as_secs() + 1;
    let p = record.provision.map_or(d, |d| d.as_secs()) + 1;
    let (done_percent, avail_percent, over_percent) = if d > p {
        (round_div(p * 100, d), 0, round_div((d - p) * 100, d))
    } else {
        (round_div(d * 100, p), round_div((p - d) * 100, p), 0)
    };

    Gauge {
        overtime: over_percent,
        available: avail_percent,
        done: done_percent,
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TimelineProject {
    pub record: ProjectRecord,
    pub done: Duration,
    pub notes: Vec<NoteRecord>,
    pub gauge: Gauge,
}

impl TimelineProject {
    fn new(record: ProjectRecord, done: Duration, notes: Vec<NoteRecord>) -> TimelineProject {
        let gauge = gauge(&record, &done);
        TimelineProject {
            record,
            done,
            notes,
            gauge,
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
                "base": base_path
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
