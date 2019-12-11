use chrono;
use chrono_humanize;
use std::convert::{TryFrom, TryInto};
use std::time;

pub fn st_from_ts(ts: i64) -> time::SystemTime {
    time::SystemTime::UNIX_EPOCH + time::Duration::from_millis(ts.try_into().unwrap())
}

pub fn human_ts(millis: i64) -> String {
    let d = chrono::Duration::from_std(time::Duration::from_millis(millis.try_into().unwrap_or(0)))
        .unwrap();
    chrono_humanize::HumanTime::from(d).to_text_en(
        chrono_humanize::Accuracy::Precise,
        chrono_humanize::Tense::Present,
    )
}
pub fn human_duration(std_d: time::Duration) -> String {
    let d = chrono::Duration::from_std(std_d).unwrap();
    chrono_humanize::HumanTime::from(d).to_text_en(
        chrono_humanize::Accuracy::Precise,
        chrono_humanize::Tense::Present,
    )
}

pub fn dur(d: &time::Duration) -> i64 {
    let millis = d.as_millis();
    i64::try_from(millis).unwrap_or(i64::max_value())
}

pub fn ts(t: &time::SystemTime) -> i64 {
    dur(&t
        .duration_since(time::UNIX_EPOCH)
        .unwrap_or(time::Duration::from_millis(0)))
}

pub fn make_table_row(cells: Vec<String>) -> String {
    let inner: String = cells
        .iter()
        .map(|s| format!("<td>{}</td>", s))
        .collect::<Vec<String>>()
        .join("");
    format!("<tr>{}</tr>", inner)
}

// fn join(a: Vec<String>, b: Vec<String>) -> (String, String) {
//     (a.join(""), b.join(""))
// }

pub fn split(a: Vec<(String, String)>) -> (Vec<String>, Vec<String>) {
    let output0 = a.iter().map(|(s, _)| s.clone()).collect();
    let output1 = a.iter().map(|(_, s)| s.clone()).collect();
    (output0, output1)
}
