use crate::timeline2::TimelineProject;
use handlebars::{handlebars_helper, Handlebars};
use serde_json::{json, Value};
use shell::{store::ProjectRecord, util::date_time_from_st};
use std::time::{Duration, SystemTime};

fn duration_to_hour(d: Duration) -> u64 {
    // let m = d.as_secs() / 3600;
    // let m2 = m + 500;
    // m2
    (d.as_secs() + 1) / 3600
}

fn format_date(t: &SystemTime) -> String {
    let dt = date_time_from_st(t);
    dt.format("%e %B %Y").to_string()
}

handlebars_helper!(to_hour: |d: Duration|  duration_to_hour(d));
handlebars_helper!(some_date: |o: Option<SystemTime>|
    o.map(|d| format_date(&d)).unwrap_or(String::new())
);

handlebars_helper!(has_parent: |r: ProjectRecord| r.parent.is_some());
handlebars_helper!(has_provision: |r: ProjectRecord| r.provision.is_some());
handlebars_helper!(has_endtime: |r: ProjectRecord| r.end_time.is_some());
handlebars_helper!(has_completed: |r: ProjectRecord| r.completed.is_some());

handlebars_helper!(get_parent: |r: ProjectRecord| r.parent.unwrap_or_default());
handlebars_helper!(get_provision: |r: ProjectRecord| r.provision.map(duration_to_hour).unwrap_or(0));
// handlebars_helper!(get_endtime: |r: ProjectRecord| r.end_time.map(duration_to_hour).unwrap_or(0));
// handlebars_helper!(get_completed: |r: ProjectRecord| r.completed.map(duration_to_hour).unwrap_or(0));

handlebars_helper!(child_projects: |id: i64, projects:Vec<TimelineProject>| {
    let data = projects
    .iter()
    .filter(|tp| tp.record.parent.map_or(false, |parent_id| parent_id == id))
    .map(|tp| tp.clone()).collect::<Vec<_>>();
    json!(data)
});

pub fn register_helpers(hs: &mut Handlebars) {
    hs.register_helper("to-hour", Box::new(to_hour));
    hs.register_helper("some-date", Box::new(some_date));

    hs.register_helper("child-projects", Box::new(child_projects));

    hs.register_helper("has-parent", Box::new(has_parent));
    hs.register_helper("has-provision", Box::new(has_provision));
    hs.register_helper("has-endtime", Box::new(has_endtime));
    hs.register_helper("has-completed", Box::new(has_completed));

    hs.register_helper("get-parent", Box::new(get_parent));
    hs.register_helper("get-provision", Box::new(get_provision));
    // hs.register_helper("get-endtime", Box::new(get_endtime));
    // hs.register_helper("get-completed", Box::new(get_completed));
}
