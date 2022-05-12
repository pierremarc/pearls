use core::time;
use std::{cmp::Ordering, collections::HashSet, fmt::Display, time::SystemTime};

use chrono::{DateTime, Datelike, Duration, TimeZone, Weekday};

use crate::{
    store::{Avail, Intent, ProjectRecord, TaskRecord},
    util::{date_time_from_st, st_from_date_time},
};

// fn intents_for_user(project_name: &str, intents: &Vec<Intent>) {
//     let ret = intents
//         .iter()
//         .filter(|&i| &i.project == project_name)
//         .collect();
// }

fn cmp_by_deadline(a: &ProjectRecord, b: &ProjectRecord) -> Ordering {
    match (a.end_time, b.end_time) {
        (None, None) => a.start_time.cmp(&b.start_time),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (Some(a), Some(b)) => a.cmp(&b),
    }
}

// fn find_start_of_week(start: LocalTime) -> LocalTime {
//     let starting_week = start.iso_week();
//     for i in 0..8 {
//         let attempt = start - Duration::days(i);
//         if attempt.iso_week() != starting_week {
//             return attempt;
//         }
//     }
//     start
// }

// fn find_end_of_week(start: LocalTime) -> LocalTime {
//     let starting_week = start.iso_week();
//     for i in 0..8 {
//         let attempt = start + Duration::days(i);
//         if attempt.iso_week() != starting_week {
//             return attempt - Duration::days(2);
//         }
//     }
//     start
// }

// struct Week {
//     mon: u64,
//     tue: u64,
//     wed: u64,
//     thu: u64,
//     fri: u64,
// }

fn work_day(dt: &chrono::DateTime<chrono::Local>) -> (SystemTime, SystemTime) {
    let start = dt.date().and_hms(10, 0, 0);
    let end = dt.date().and_hms(18, 0, 0);

    (st_from_date_time(&start), st_from_date_time(&end))
}

fn daily_avail(dt: &chrono::DateTime<chrono::Local>, avails: &Vec<&Avail>) -> u64 {
    let (start, end) = work_day(dt);
    avails
        .iter()
        .filter_map(|a| {
            if a.start_time < end && a.end_time > start {
                Some(a.weekly.as_secs() / 5)
            } else {
                None
            }
        })
        .min()
        .unwrap_or(0)
}

fn working_days(dt: &DateTime<chrono::Local>) -> u64 {
    match dt.weekday() {
        Weekday::Mon => 5,
        Weekday::Tue => 4,
        Weekday::Wed => 3,
        Weekday::Thu => 2,
        Weekday::Fri => 1,
        Weekday::Sat => 0,
        Weekday::Sun => 0,
    }
}

fn weekly_avail(start_time: &SystemTime, avails: &Vec<&Avail>) -> u64 {
    let start = date_time_from_st(start_time);

    (0u64..working_days(&start))
        .map(|i| daily_avail(&(start + (Duration::days(i as i64))), avails))
        .sum()
}

pub fn next_monday(dt: &DateTime<chrono::Local>) -> DateTime<chrono::Local> {
    match dt.weekday() {
        Weekday::Mon => *dt + Duration::days(7),
        Weekday::Tue => *dt + Duration::days(6),
        Weekday::Wed => *dt + Duration::days(5),
        Weekday::Thu => *dt + Duration::days(4),
        Weekday::Fri => *dt + Duration::days(3),
        Weekday::Sat => *dt + Duration::days(2),
        Weekday::Sun => *dt + Duration::days(1),
    }
}

fn sum_done(username: &str, project_name: &str, dones: &Vec<TaskRecord>) -> u64 {
    dones
        .iter()
        .filter(|d| &d.project == project_name && &d.username == username)
        .fold(0, |acc, rec| {
            acc + rec
                .end_time
                .duration_since(rec.start_time)
                .unwrap_or_else(|_| time::Duration::from_secs(0))
                .as_secs()
        })
}

fn find_intent<'a>(username: &str, project: &str, intents: &'a Vec<Intent>) -> Option<&'a Intent> {
    intents
        .iter()
        .find(|i| &i.username == username && &i.project == project)
}

#[derive(Debug)]
pub struct WorkLoad {
    start: chrono::DateTime<chrono::Local>,
    user: String,
    project: String,
    load: Duration,
}

impl WorkLoad {
    // fn new(user: &str, project: &str, load: &Duration) -> WorkLoad {
    //     WorkLoad {
    //         user: String::from(user),
    //         project: String::from(project),
    //         load: load.clone(),
    //     }
    // }

    fn partial(user: &str, project: &str) -> WorkLoad {
        WorkLoad {
            start: chrono::Local.timestamp(0, 0),
            user: String::from(user),
            project: String::from(project),
            load: Duration::zero(),
        }
    }

    fn and_load(&self, start: chrono::DateTime<chrono::Local>, load: Duration) -> WorkLoad {
        WorkLoad {
            user: self.user.clone(),
            project: self.project.clone(),
            start,
            load,
        }
    }

    pub fn start(&self) -> &chrono::DateTime<chrono::Local> {
        &self.start
    }
    pub fn user(&self) -> &str {
        &self.user
    }

    pub fn project(&self) -> &str {
        &self.project
    }

    pub fn load(&self) -> &Duration {
        &self.load
    }
}

impl Display for WorkLoad {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{} {} {}",
            self.project(),
            self.user(),
            self.start().format("%F"),
            self.load().num_hours()
        )
    }
}

pub type WorkPlan = Vec<(String, Vec<(String, Vec<WorkLoad>)>)>;

pub fn find_loads<'a>(
    plan: &'a WorkPlan,
    start: &chrono::DateTime<chrono::Local>,
    end: &chrono::DateTime<chrono::Local>,
) -> Vec<&'a WorkLoad> {
    // println!("find_loads [{}; {}]", start.format("%F"), end.format("%F"));
    plan.iter()
        .flat_map(|(_, user_loads)| {
            user_loads.iter().flat_map(|(_, loads)| {
                loads.iter().filter(|&load| {
                    // println!(
                    //     "{} > {} = {} && {} < {} = {}",
                    //     load.start.format("%F"),
                    //     start.format("%F"),
                    //     &load.start > start,
                    //     load.start.format("%F"),
                    //     end.format("%F"),
                    //     &load.start < end
                    // );
                    &load.start >= start && &load.start < end
                })
            })
        })
        .collect()
}

pub fn plan_all(
    projects: &Vec<ProjectRecord>,
    intents: &Vec<Intent>,
    avails: &Vec<Avail>,
    dones: &Vec<TaskRecord>,
    start_time: SystemTime,
) -> WorkPlan {
    let mut open_projects = projects
        .iter()
        .filter(|p| p.completed.is_none())
        .map(|p| p.clone())
        .collect::<Vec<_>>();
    open_projects.sort_by(cmp_by_deadline);

    let user_loads = intents
        .iter()
        .filter(|i| i.end_time.map_or(true, |t| t > start_time))
        .fold(HashSet::new(), |mut acc, i| {
            acc.insert(i.username.clone());
            acc
        })
        .iter()
        .map(|username| {
            let mut dt = date_time_from_st(&start_time);
            let user_avails = avails
                .iter()
                .filter(|a| &a.username == username)
                .collect::<Vec<_>>();
            let loads =
                open_projects
                    .iter()
                    .filter_map(|p| {
                        find_intent(&username, &p.name, intents).map(|intent| {
                            let partial = WorkLoad::partial(&username, &p.name);
                            let mut loads: Vec<WorkLoad> = Vec::new();
                            let done = sum_done(&username, &p.name, dones);
                            if done < intent.amount.as_secs() {
                                let mut remaining = intent.amount.as_secs() - done;
                                // sparing an hour
                                while remaining > 3600 {
                                    let week_avail =
                                        weekly_avail(&st_from_date_time(&dt), &user_avails);
                                    if week_avail > remaining {
                                        loads.push(partial.and_load(
                                            dt.clone(),
                                            Duration::seconds(remaining as i64),
                                        ));

                                        let consumed = {
                                            let days = remaining * working_days(&dt) / week_avail;
                                            Duration::days(days as i64)
                                        };

                                        dt = dt + consumed;
                                        break;
                                    } else {
                                        loads.push(partial.and_load(
                                            dt.clone(),
                                            Duration::seconds(week_avail as i64),
                                        ));
                                        dt = next_monday(&dt);
                                        remaining = remaining - week_avail;
                                    };
                                }
                            }
                            (p.name.clone(), loads)
                        })
                    })
                    .collect::<Vec<_>>();

            (username.clone(), loads)
        })
        .collect::<Vec<_>>();

    user_loads
}

#[cfg(test)]
mod tests {

    use crate::{plan::*, store::Store};

    #[test]
    fn all_of_a_plan_is_possible() {
        let mut store = Store::new("/home/pierre/System/src/pearls".into());
        let con = store.connect_or_create("test.db").unwrap();
        let projects = con.select_all_project_info().unwrap();
        let intents = con.select_intent_all().unwrap();
        let avails = con.select_avail_all().unwrap();
        let dones = con.select_current_task().unwrap();
        let plan = plan_all(&projects, &intents, &avails, &dones, SystemTime::now());
        for (name, user_loads) in plan {
            println!("{}", name);
            for (project, loads) in user_loads {
                println!("  {}", project);
                for load in loads {
                    println!("    {}", load);
                }
            }
        }
    }
    #[test]
    fn find_next_year_loads() {
        let mut store = Store::new("/home/pierre/System/src/pearls".into());
        let con = store.connect_or_create("test.db").unwrap();
        let projects = con.select_all_project_info().unwrap();
        let intents = con.select_intent_all().unwrap();
        let avails = con.select_avail_all().unwrap();
        let dones = con.select_current_task().unwrap();
        let plan = plan_all(&projects, &intents, &avails, &dones, SystemTime::now());
        for (name, user_loads) in plan.iter() {
            println!("{}", name);
            for (project, loads) in user_loads {
                println!("  {}", project);
                for load in loads {
                    println!("    {}", load);
                }
            }
        }
        let max_avail = date_time_from_st(
            &avails
                .iter()
                .fold(SystemTime::now(), |acc, a| acc.max(a.end_time)),
        );
        let year = date_time_from_st(&SystemTime::now()) + Duration::days(361);
        let max = max_avail.max(year);
        let mut start = next_monday(&date_time_from_st(&SystemTime::now()));

        while start < max {
            let end = next_monday(&start);
            let loads = find_loads(&plan, &start, &end);
            for load in loads {
                println!("== {}", load);
            }
            start = end;
        }
    }
}
