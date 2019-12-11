use crate::bot;
use shell::util::{human_ts, make_table_row, split};
use std::time;

pub fn since(
    handler: &mut bot::CommandHandler,
    user: String,
    since: time::SystemTime,
) -> Option<(String, String)> {
    match handler
        .store
        .select_user(user.clone(), since.clone(), |row| {
            let project: String = row.get(0)?;
            let task: String = row.get(1)?;
            let sum: i64 = row.get(2)?;
            Ok((
                format!("{}\t{}\t{}", project, task, human_ts(sum)),
                make_table_row(vec![project, task, format!("{}", human_ts(sum))]),
            ))
        }) {
        Ok(results) => {
            let (left, right) = split(results);
            Some((
                left.join("\n"),
                format!("<table>{}</table>", right.join("\n")),
            ))
        }
        Err(_) => None,
    }
}
