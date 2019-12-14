use crate::bot;
use shell::util::{dur, human_duration, make_table_row, split};
use std::time;

pub fn project(handler: &mut bot::CommandHandler, project: String) -> Option<(String, String)> {
    let available = handler
        .store
        .select_project_info(project.clone())
        .unwrap_or(Vec::new());

    match handler.store.select_project(project.clone()) {
        Ok(ref recs) => {
            let (left, right) = split(
                recs.into_iter()
                    .map(|rec| {
                        let spent = human_duration(rec.duration);
                        (
                            format!("{}\t{}\t{}", rec.username, rec.task, spent),
                            make_table_row(vec![
                                rec.username.clone(),
                                rec.task.clone(),
                                format!("{}", spent),
                            ]),
                        )
                    })
                    .collect(),
            );
            let done = recs.iter().fold(0, |acc, rec| {
                acc + dur(&rec
                    .end_time
                    .duration_since(rec.start_time)
                    .unwrap_or(time::Duration::from_secs(0)))
            });
            let (h0, h1) = available
                .first()
                .map(|rec| {
                    (
                        format!(
                            "{} hours available, {} hours done\n",
                            dur(&rec.duration) / (1000 * 60 * 60),
                            done / (1000 * 60 * 60)
                        ),
                        format!(
                            "
                            <div><code>available: {} hours </code> </div>
                            <div><code>done: {} hours </code></div>",
                            dur(&rec.duration) / (1000 * 60 * 60),
                            done / (1000 * 60 * 60)
                        ),
                    )
                })
                .unwrap_or((
                    format!("{} done", done),
                    format!("</code><code>done: {} </code>", done),
                ));
            Some((
                h0 + &left.join("\n"),
                h1 + &format!("<table>{}</table>", right.join("\n")),
            ))
        }
        Err(_) => None,
    }
}
