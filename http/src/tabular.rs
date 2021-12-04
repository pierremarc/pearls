use bytes::{BufMut, Bytes, BytesMut};
use csv::Writer;
use shell::{
    store::TaskRecord,
    util::{st_from_ts, st_to_datestring},
};
use std::error::Error;
use warp::{http, Filter};

use crate::common::{with_store, ArcStore};

fn format_duration_time(millis: i64) -> String {
    let minutes = millis / 1000 / 60;
    let hours = minutes / 60;
    let remaining_minutes = if hours > 0 {
        minutes % (hours * 60)
    } else {
        minutes
    };
    format!("{:02}:{:02}:00", hours, remaining_minutes)
}

fn format_duration_decimal(millis: i64) -> String {
    let hours = (millis / 1000) as f32 / (60.0 * 60.0);
    format!("{:0<2.2}", hours)
}

struct BytesWrapper(Bytes);

impl warp::Reply for BytesWrapper {
    fn into_response(self) -> warp::reply::Response {
        let content: Vec<u8> = self.0.into_iter().collect();
        let mut response = warp::reply::Response::new(content.into());
        response.headers_mut().insert(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("text/csv"),
        );
        response.headers_mut().insert(
            http::header::CONTENT_DISPOSITION,
            http::HeaderValue::from_static("attachment"),
        );
        response
    }
}

pub fn make_table(records: &Vec<TaskRecord>) -> Vec<Vec<String>> {
    records
        .iter()
        .map(|record| {
            let duration = record
                .end_time
                .duration_since(record.start_time)
                .map(|d| shell::util::dur(&d))
                .unwrap_or(0i64);
            vec![
                record.username.clone(),
                record.project.clone(),
                record.task.clone(),
                st_to_datestring(&record.start_time),
                format_duration_time(duration),
                format_duration_decimal(duration),
            ]
        })
        .collect()
}

fn to_csv(records: Vec<TaskRecord>) -> Result<BytesWrapper, Box<dyn Error>> {
    let buf = BytesMut::with_capacity(516 * records.len());
    let mut bytes_writer = buf.writer();
    {
        let mut writer = Writer::from_writer(&mut bytes_writer);
        writer.write_record(&[
            "username",
            "project",
            "task",
            "date",
            "duration (time)",
            "duration (decimal)",
        ])?;
        for record in make_table(&records) {
            writer.write_record(record)?;
        }
    }
    let bytes_back = bytes_writer.into_inner();
    Ok(BytesWrapper(bytes_back.freeze()))
}

fn collect_records(
    client: String,
    name: String,
    start: i64,
    end: i64,
    token: String,
    store: ArcStore,
) -> Vec<TaskRecord> {
    let project_name = format!("{}/{}", client, name);
    let start_time = st_from_ts(start);
    let end_time = st_from_ts(end);
    println!(
        "TABULAR {}  -->  {}",
        st_to_datestring(&start_time),
        st_to_datestring(&end_time)
    );
    if let Ok(mut store) = store.lock() {
        if let Ok(connected) = store.connect(&token) {
            return match connected.select_project_detail(project_name) {
                Err(_) => Vec::new(),
                Ok(tasks) => tasks
                    .into_iter()
                    .filter(|task| task.start_time > start_time && task.end_time <= end_time)
                    .collect(),
            };
        }
    }
    Vec::new()
}

pub fn tabular(
    s: ArcStore,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!(String / "tabular" / String / String / i64 / i64)
        .and(warp::get())
        .and(with_store(s))
        .and_then(
            |token: String, client: String, name: String, start: i64, end: i64, store: ArcStore| async move {
                match to_csv(collect_records(client, name, start, end, token , store)) {
                    Ok(body) => Ok(body),
                    Err(_) => Err(warp::reject()),
                }
            },
        )
}
