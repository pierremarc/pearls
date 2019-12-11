use crate::bot;
use shell::util::{human_ts, make_table_row, split};

pub fn project(handler: &mut bot::CommandHandler, project: String) -> Option<(String, String)> {
    let available = match handler.store.select_project_info(project.clone(), |row| {
        let _username: String = row.get(2)?;
        let _start: i64 = row.get(3)?;
        let dur: i64 = row.get(4)?;
        Ok(dur / (1000 * 60 * 60))
    }) {
        Err(_) => Vec::new(),
        Ok(ref d) => {
            let v = d;
            v.clone()
        }
    };

    match handler.store.select_project(project.clone(), |row| {
        let username: String = row.get(1)?;
        let task: String = row.get(2)?;
        let spent_millis: i64 = row.get(3)?;
        let spent = human_ts(spent_millis);
        Ok((
            format!("{}\t{}\t{}", username, task, spent),
            make_table_row(vec![username, task, format!("{}", spent)]),
            spent_millis / (1000 * 60 * 60),
        ))
    }) {
        Ok(ref results) => {
            let ((left, right), spent) = (
                split(
                    results
                        .into_iter()
                        .map(|(l, r, _)| (l.clone(), r.clone()))
                        .collect(),
                ),
                results.iter().map(|(_, _, s)| *s).collect::<Vec<i64>>(),
            );
            let done: i64 = spent.iter().fold(0, |acc, x| acc + x);
            let (h0, h1) = available
                .first()
                .map(|n| {
                    (
                        format!("{} hours available, {} hours done\n", n, done),
                        format!(
                            "
                            <div><code>available: {} hours </code> </div>
                            <div><code>done: {} hours </code></div>",
                            n, done
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
