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
            .map(|(name, score)| name.clone())
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
            .take(8)
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
        let list_items: Vec<Element> = self.take(8).iter().map(|c| li(c)).collect();
        let list = ul(list_items);
        div(vec![title, list, paragraph(desc)]).as_string()
    }
}

fn get_project_name_parts(project_name: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = project_name.split("/").collect();
    parts.get(0).and_then(|client_name| {
        parts
            .get(1)
            .map(|project_name| (String::from(*client_name), String::from(*project_name)))
    })
}

fn get_candidates(handler: &mut bot::CommandHandler, project: &str) -> Candidates {
    match (
        get_project_name_parts(project),
        handler.store.select_all_project_info(),
    ) {
        (None, Err(_)) => Candidates::empty(),
        (None, Ok(_)) => Candidates::empty(),
        (Some(_), Err(_)) => Candidates::empty(),
        (Some((ref_client, ref_project)), Ok(rows)) => {
            let mut names: Vec<ScoredName> = rows
                .iter()
                .filter_map(|record| {
                    get_project_name_parts(&record.name).map(|(client_name, project_name)| {
                        let client_score = levenshtein(&ref_client, &client_name) * 10 * 2;
                        let project_score = levenshtein(&ref_project, &project_name) * 10 / 3;
                        (record.name.clone(), client_score + project_score)
                    })
                })
                .collect();

            names.sort_by_key(|(_, score)| *score);
            // names.iter().take(5).map(|(name, _)| name.clone()).collect()
            Candidates(names, Some(project.into()))
        }
    }
}

pub fn select_project(
    handler: &mut bot::CommandHandler,
    name: &str,
) -> Result<ProjectRecord, Candidates> {
    handler
        .store
        .select_project_info(name.into())
        .map_err(|_| get_candidates(handler, name))
}

const IS_META_DISCLAIMER: &str =
    "This is a meta project, you must assign work to its child projects.";

fn project_list_string(projects: &Vec<&ProjectRecord>) -> String {
    projects
        .iter()
        .map(|project| format!("→ {}", project.name))
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn check_meta(
    handler: &mut bot::CommandHandler,
    project: &ProjectRecord,
) -> Option<(String, String)> {
    match project.is_meta {
        false => None,
        true => handler
            .store
            .select_all_project_info()
            .ok()
            .map(|projects| {
                let projects = projects
                    .iter()
                    .filter(|p| p.parent.map(|parent| parent == project.id).unwrap_or(false))
                    .collect::<Vec<_>>();

                (
                    format!("{}\n{}", IS_META_DISCLAIMER, project_list_string(&projects)),
                    String::new(),
                )
            }),
    }
}
