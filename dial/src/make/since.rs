use crate::bot;
use shell::util::{human_duration, make_table_row, split};
use std::time;

pub fn since(
    handler: &mut bot::CommandHandler,
    user: String,
    since: time::SystemTime,
) -> Option<(String, String)> {
    match handler.store.select_user(user.clone(), since) {
        Ok(results) => {
            let (left, right) = split(
                results
                    .into_iter()
                    .map(|rec| {
                        (
                            format!(
                                "{}\t{}\t{}",
                                rec.project,
                                rec.task,
                                human_duration(rec.duration)
                            ),
                            make_table_row(vec![
                                rec.project,
                                rec.task,
                                format!("{}", human_duration(rec.duration)),
                            ]),
                        )
                    })
                    .collect(),
            );
            Some((
                left.join("\n"),
                format!("<table>{}</table>", right.join("\n")),
            ))
        }
        Err(_) => None,
    }
}
