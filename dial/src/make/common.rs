use crate::bot;
use html::{div, h4, li, paragraph, ul, Element};
use shell::store::ProjectRecord;
use strsim::levenshtein;

type ScoredName = (String, usize);
pub struct Candidates(Vec<ScoredName>, Option<String>);

impl Candidates {
    fn empty() -> Candidates {
        Candidates(Vec::new(), None)
    }

    pub fn take(&self, n: usize) -> Vec<String> {
        self.0
            .iter()
            .take(n)
            .map(|(name, _)| name.clone())
            .collect()
    }

    pub fn as_text(&self, desc: &str) -> String {
        let title = self.1.clone().map_or(
            String::from("This project does not exist, similar project names are:"),
            |name| {
                format!(
                    "Project \"{}\" does not exist, similar project names are:",
                    name
                )
            },
        );
        let list: String = self
            .take(5)
            .iter()
            .map(|c| format!("\n  - {}", c))
            .collect();
        format!("{}\n{}\n{}", title, list, desc)
    }

    pub fn as_html(&self, desc: &str) -> String {
        let title = self.1.clone().map_or(
            h4("This project does not exist, similar project names are:"),
            |name| {
                h4(format!(
                    "Project \"{}\" does not exist, similar project names are:",
                    name
                ))
            },
        );
        let list_items: Vec<Element> = self.take(5).iter().map(|c| li(c)).collect();
        let list = ul(list_items);
        div(vec![title, list, paragraph(desc)]).as_string()
    }
}

fn get_candidates(handler: &mut bot::CommandHandler, project: &str) -> Candidates {
    match handler.store.select_all_project_info() {
        Err(_) => Candidates::empty(),
        Ok(rows) => {
            let mut names: Vec<ScoredName> = rows
                .iter()
                .map(|record| (record.name.clone(), levenshtein(project, &record.name)))
                .collect();

            names.sort_by_key(|(_, n)| *n);
            // names.iter().take(5).map(|(name, _)| name.clone()).collect()
            Candidates(names, Some(project.into()))
        }
    }
}

pub fn select_project(
    handler: &mut bot::CommandHandler,
    name: &str,
) -> Result<ProjectRecord, Candidates> {
    match handler.store.select_project_info(name.into()) {
        Err(_) => Err(Candidates::empty()),
        Ok(rows) => match rows.get(0) {
            None => Err(get_candidates(handler, name)),
            Some(rec) => Ok(rec.clone()),
        },
    }
}
