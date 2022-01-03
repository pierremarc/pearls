use crate::util::{dur, dur_from_ts, st_from_ts, ts};
use rusqlite::{named_params, Connection, Result as SqlResult, Row, ToSql, NO_PARAMS};
use serde::{Deserialize, Serialize};
use std;
use std::fmt;
use std::path::Path;
use std::time;

// struct SqlVec<T>(Vec<T>);

// impl<T: ToSql> ToSql for SqlVec<T> {
//     fn to_sql(&self) -> SqlResult<ToSqlOutput<'_>> {

//     }
// }

#[derive(Serialize, Deserialize, Clone)]
pub struct TaskRecord {
    pub id: i64,
    pub username: String,
    pub start_time: time::SystemTime,
    pub end_time: time::SystemTime,
    pub project: String,
    pub task: String,
}

impl TaskRecord {
    fn from_row(row: &Row) -> SqlResult<TaskRecord> {
        Ok(TaskRecord {
            id: row.get(0)?,
            username: row.get(1)?,
            start_time: st_from_ts(row.get(2)?),
            end_time: st_from_ts(row.get(3)?),
            project: row.get(4)?,
            task: row.get(5)?,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct AggregatedTaskRecord {
    pub project: String,
    pub username: String,
    pub task: String,
    pub start_time: time::SystemTime,
    pub end_time: time::SystemTime,
    pub duration: time::Duration,
}

impl AggregatedTaskRecord {
    fn from_row(row: &Row) -> SqlResult<AggregatedTaskRecord> {
        Ok(AggregatedTaskRecord {
            project: row.get(0)?,
            username: row.get(1)?,
            task: row.get(2)?,
            start_time: st_from_ts(row.get(3)?),
            end_time: st_from_ts(row.get(4)?),
            duration: dur_from_ts(row.get(5)?),
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProjectRecord {
    pub id: i64,
    pub name: String,
    pub username: String,
    pub start_time: time::SystemTime,
    // pub duration: time::Duration, removed in migration 001
    pub end_time: Option<time::SystemTime>,
    pub provision: Option<time::Duration>,
    pub completed: Option<time::SystemTime>,
    pub is_meta: bool,
    pub parent: Option<i64>,
}

impl ProjectRecord {
    fn from_row(row: &Row) -> SqlResult<ProjectRecord> {
        Ok(ProjectRecord {
            id: row.get(0)?,
            name: row.get(1)?,
            username: row.get(2)?,
            start_time: st_from_ts(row.get(3)?),
            end_time: row.get(4).map(st_from_ts).ok(),
            provision: row.get(5).map(dur_from_ts).ok(),
            completed: row.get(6).map(st_from_ts).ok(),
            is_meta: row.get(7).unwrap_or(false),
            parent: row.get(8).ok(),
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NoteRecord {
    pub id: i64,
    pub username: String,
    pub project: String,
    pub created_at: time::SystemTime,
    pub content: String,
}

impl NoteRecord {
    fn from_row(row: &Row) -> SqlResult<NoteRecord> {
        Ok(NoteRecord {
            id: row.get(0)?,
            username: row.get(1)?,
            project: row.get(2)?,
            created_at: st_from_ts(row.get(3)?),
            content: row.get(4)?,
        })
    }
}

pub struct ConnectedStore {
    room_id: String,
    conn: Connection,
}

pub struct Store {
    root_dir: String,
    connections: Vec<ConnectedStore>,
}

#[derive(Debug, Clone)]
pub enum StoreError {
    Open(String),
    Connected(String),
    LogRecord,
    Iter,
    Get,
    Lock,
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::Open(p) => write!(f, "Failed to open connection to {}", p),
            StoreError::Connected(p) => write!(f, "Failed to get a connection to {}", p),
            StoreError::LogRecord => write!(f, "Failed to log record"),
            StoreError::Iter => write!(f, "Failed to iterate"),
            StoreError::Get => write!(f, "Failed to get a record"),
            StoreError::Lock => write!(
                f,
                "Rather obscure, but somewhere we failed to obtain a lock on a mutex..."
            ),
        }
    }
}

impl std::error::Error for StoreError {}

pub enum Name {
    InsertDo,
    InsertNote,
    InsertNotification,
    InsertProject,
    SelectAllProjectInfo,
    SelectCurrentTask,
    SelectCurrentTaskFor,
    SelectEndingTask,
    SelectLatestTaskFor,
    SelectNotes,
    SelectProject,
    SelectProjectDetail,
    SelectProjectInfo,
    SelectUser,
    UpdateCompleted,
    UpdateDeadline,
    UpdateProvision,
    UpdateTaskEnd,
    UpdateMeta,
    UpdateParent,
}

fn sql(name: Name) -> &'static str {
    match name {
        Name::InsertDo => include_str!("sql/insert_do.sql"),
        Name::InsertNote => include_str!("sql/insert_note.sql"),
        Name::InsertNotification => include_str!("sql/insert_notification.sql"),
        Name::InsertProject => include_str!("sql/insert_project.sql"),
        Name::SelectAllProjectInfo => include_str!("sql/select_all_project_info.sql"),
        Name::SelectCurrentTask => include_str!("sql/select_current_task.sql"),
        Name::SelectCurrentTaskFor => include_str!("sql/select_current_task_for.sql"),
        Name::SelectEndingTask => include_str!("sql/select_ending_task.sql"),
        Name::SelectLatestTaskFor => include_str!("sql/select_latest_task_for.sql"),
        Name::SelectNotes => include_str!("sql/select_notes.sql"),
        Name::SelectProject => include_str!("sql/select_project.sql"),
        Name::SelectProjectDetail => include_str!("sql/select_project_detail.sql"),
        Name::SelectProjectInfo => include_str!("sql/select_project_info.sql"),
        Name::SelectUser => include_str!("sql/select_user.sql"),
        Name::UpdateCompleted => include_str!("sql/update_completed.sql"),
        Name::UpdateDeadline => include_str!("sql/update_deadline.sql"),
        Name::UpdateProvision => include_str!("sql/update_provision.sql"),
        Name::UpdateTaskEnd => include_str!("sql/update_task_end.sql"),
        Name::UpdateMeta => include_str!("sql/update_meta.sql"),
        Name::UpdateParent => include_str!("sql/update_parent.sql"),
    }
}

fn migrate(conn: &Connection) {
    let user_version = conn.query_row(
        "SELECT user_version  FROM pragma_user_version();",
        NO_PARAMS,
        |row| row.get::<usize, i64>(0),
    ).expect("Could not get user_version from the database, \nmeans we can't process DB version check and migrations. \nAborting");
    match user_version {
        0 => {
            conn.execute_batch(include_str!("sql/migrations/001.sql"))
                .expect("Failed migration: 001.sql");
            println!("Applied sql/migrations/001.sql");
            migrate(conn);
        }
        1 => {
            conn.execute_batch(include_str!("sql/migrations/002.sql"))
                .expect("Failed migration: 002.sql");
            println!("Applied sql/migrations/002.sql");
            migrate(conn);
        }
        2 => {
            conn.execute_batch(include_str!("sql/migrations/003.sql"))
                .expect("Failed migration: 003.sql");
            println!("Applied sql/migrations/003.sql");
            migrate(conn);
        }
        3 => {
            conn.execute_batch(include_str!("sql/migrations/004.sql"))
                .expect("Failed migration: 004.sql");
            println!("Applied sql/migrations/004.sql");
            migrate(conn);
        }
        _ => println!("Migrate completed, we're at version {}", user_version),
    };
}

pub type StoreResult<T> = Result<T, StoreError>;

impl Store {
    pub fn new(root_dir: String) -> Store {
        Store {
            root_dir,
            connections: vec![],
        }
    }

    pub fn connect_or_create(&mut self, db_name: &str) -> StoreResult<&mut ConnectedStore> {
        let exists = self
            .connections
            .iter()
            .map(|c| &c.room_id)
            .any(|name| name == db_name);
        if exists {
            return self.connected(db_name);
        }
        let root_path = Path::new(&self.root_dir);
        let path = root_path.join(&db_name);
        let conn = Connection::open(path).map_err(|_| StoreError::Open(db_name.into()))?;
        conn.execute("PRAGMA foreign_keys = ON;", NO_PARAMS)
            .expect("Failed to enable foreign_keys");
        conn.execute(include_str!("sql/create_do.sql"), NO_PARAMS)
            .expect("failed creating table do");
        conn.execute(include_str!("sql/create_project.sql"), NO_PARAMS)
            .expect("failed creating table project");
        conn.execute(include_str!("sql/create_notification.sql"), NO_PARAMS)
            .expect("failed creating table notification");
        conn.execute(include_str!("sql/create_cal.sql"), NO_PARAMS)
            .expect("failed creating table cal");

        migrate(&conn);

        self.connections.push(ConnectedStore {
            conn,
            room_id: db_name.into(),
        });
        self.connections
            .last_mut()
            .ok_or_else(|| StoreError::Open(db_name.into()))
    }

    pub fn connected(&mut self, db_name: &str) -> StoreResult<&mut ConnectedStore> {
        self.connections
            .iter_mut()
            .find(|c| c.room_id == db_name)
            .ok_or_else(|| StoreError::Connected(db_name.into()))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut ConnectedStore> {
        self.connections.iter_mut()
    }
}

impl ConnectedStore {
    pub fn room_id(&self) -> String {
        self.room_id.clone()
    }

    fn exec(&self, name: Name, params: &[(&str, &dyn ToSql)]) -> StoreResult<usize> {
        match self.conn.execute_named(sql(name), params) {
            Ok(s) => Ok(s),
            Err(err) => {
                println!("SQLite error: {}", err);
                Err(StoreError::LogRecord)
            }
        }
    }
    fn map_rows<F, T>(&self, name: Name, params: &[(&str, &dyn ToSql)], f: F) -> StoreResult<Vec<T>>
    where
        F: FnMut(&Row) -> SqlResult<T>,
    {
        match self.conn.prepare(sql(name)) {
            Err(err) => {
                println!("SQLite error: {}", err);
                Err(StoreError::Iter)
            }
            Ok(mut stmt) => match stmt.query_map_named(params, f) {
                Err(err) => {
                    println!("SQLite error: {}", err);
                    Err(StoreError::Iter)
                }
                Ok(rows) => Ok(rows
                    .filter_map(|row| match row {
                        Err(err) => {
                            println!("Row Error: {}", err);
                            None
                        }
                        Ok(_) => row.ok(),
                    })
                    .collect()),
            },
        }
    }

    pub fn insert_do(
        &mut self,
        user: String,
        start: time::SystemTime,
        end: time::SystemTime,
        project: String,
        task: String,
    ) -> StoreResult<usize> {
        self.exec(
            Name::InsertDo,
            named_params! {
                ":username": user,
                ":start": ts(&start),
                ":end": ts(&end),
                ":project": project,
                ":task": task,
            },
        )
    }

    pub fn insert_project(
        &mut self,
        username: String,
        name: String,
        start: time::SystemTime,
    ) -> StoreResult<usize> {
        self.exec(
            Name::InsertProject,
            named_params! {
                ":username": username,
                ":name": name,
                ":start": ts(&start),
            },
        )
    }
    pub fn insert_note(
        &mut self,
        project: String,
        username: String,
        content: String,
    ) -> StoreResult<usize> {
        self.exec(
            Name::InsertNote,
            named_params! {
                ":project": project,
                ":username": username,
                ":created_at": ts(&time::SystemTime::now()),
                ":content": content,
            },
        )
    }

    pub fn update_deadline(&mut self, name: String, end: time::SystemTime) -> StoreResult<usize> {
        self.exec(
            Name::UpdateDeadline,
            named_params! {
                ":name": name,
                ":end": ts(&end),
            },
        )
    }

    pub fn update_completed(
        &mut self,
        name: String,
        completed: time::SystemTime,
    ) -> StoreResult<usize> {
        self.exec(
            Name::UpdateCompleted,
            named_params! {
                ":name": name,
                ":completed": ts(&completed),
            },
        )
    }

    pub fn update_provision(
        &mut self,
        name: String,
        provision: time::Duration,
    ) -> StoreResult<usize> {
        self.exec(
            Name::UpdateProvision,
            named_params! {
                ":name": name,
                ":provision": dur(&provision),
            },
        )
    }

    pub fn update_meta(&mut self, name: String, is_meta: bool) -> StoreResult<usize> {
        self.exec(
            Name::UpdateMeta,
            named_params! {
                ":name": name,
                ":is_meta": is_meta,
            },
        )
    }

    pub fn update_parent(&mut self, name: String, parent: i64) -> StoreResult<usize> {
        self.exec(
            Name::UpdateParent,
            named_params! {
                ":name": name,
                ":parent": parent,
            },
        )
    }

    pub fn insert_notification(&mut self, tid: i64, end: time::SystemTime) -> StoreResult<usize> {
        self.exec(
            Name::InsertNotification,
            named_params! {
                ":tid": tid,
                ":end":  ts(&end),
            },
        )
    }

    pub fn select_current_task(&self) -> StoreResult<Vec<TaskRecord>> {
        let now = time::SystemTime::now();
        self.map_rows(
            Name::SelectCurrentTask,
            named_params! {
                ":now": ts(&now),
            },
            TaskRecord::from_row,
        )
    }

    pub fn select_current_task_for(&self, user: String) -> StoreResult<Vec<TaskRecord>> {
        let now = time::SystemTime::now();
        self.map_rows(
            Name::SelectCurrentTaskFor,
            named_params! {
                ":user": user,
                ":now": ts(&now),
            },
            TaskRecord::from_row,
        )
    }

    pub fn select_latest_task_for(&self, user: String) -> StoreResult<Vec<TaskRecord>> {
        self.map_rows(
            Name::SelectLatestTaskFor,
            named_params! {
                ":user": user,
            },
            TaskRecord::from_row,
        )
    }

    pub fn select_all_project_info(&self) -> StoreResult<Vec<ProjectRecord>> {
        self.map_rows(
            Name::SelectAllProjectInfo,
            named_params! {},
            ProjectRecord::from_row,
        )
    }

    pub fn select_project_info(&self, project: String) -> StoreResult<ProjectRecord> {
        self.map_rows(
            Name::SelectProjectInfo,
            named_params! {
                ":project": project,
            },
            ProjectRecord::from_row,
        )
        .and_then(|records| match records.get(0) {
            None => Err(StoreError::Get),
            Some(r) => Ok(r.clone()),
        })
    }

    pub fn select_project(&self, project_name: String) -> StoreResult<Vec<AggregatedTaskRecord>> {
        self.select_project_info(project_name.clone())
            .and_then(|project| {
                if project.is_meta {
                    self.select_all_project_info().map(|projects| {
                        projects
                            .iter()
                            .filter(|p| {
                                p.parent
                                    .map(|parent_id| parent_id == project.id)
                                    .unwrap_or(false)
                            })
                            .filter_map(|project| self.select_project(project.name.clone()).ok())
                            .flatten()
                            .collect()
                    })
                } else {
                    self.map_rows(
                        Name::SelectProject,
                        named_params! {
                            ":project": project_name.clone(),
                        },
                        AggregatedTaskRecord::from_row,
                    )
                }
            })
    }

    pub fn select_project_detail(&self, project_name: String) -> StoreResult<Vec<TaskRecord>> {
        self.select_project_info(project_name.clone())
            .and_then(|project| {
                if project.is_meta {
                    self.select_all_project_info().map(|projects| {
                        projects
                            .iter()
                            .filter(|p| {
                                p.parent
                                    .map(|parent_id| parent_id == project.id)
                                    .unwrap_or(false)
                            })
                            .filter_map(|project| {
                                self.select_project_detail(project.name.clone()).ok()
                            })
                            .flatten()
                            .collect()
                    })
                } else {
                    self.map_rows(
                        Name::SelectProjectDetail,
                        named_params! {
                            ":project": project_name.clone(),
                        },
                        TaskRecord::from_row,
                    )
                }
            })
    }

    pub fn select_notes(&self, project: String) -> StoreResult<Vec<NoteRecord>> {
        self.map_rows(
            Name::SelectNotes,
            named_params! {
                ":project": project,
            },
            NoteRecord::from_row,
        )
    }

    pub fn select_user(
        &self,
        user: String,
        since: time::SystemTime,
    ) -> StoreResult<Vec<AggregatedTaskRecord>> {
        self.map_rows(
            Name::SelectUser,
            named_params! {
                ":user": user,
                ":since": ts(&since),
            },
            AggregatedTaskRecord::from_row,
        )
    }

    pub fn update_task_end(&self, id: i64, end: time::SystemTime) -> StoreResult<usize> {
        self.exec(
            Name::UpdateTaskEnd,
            named_params! {
                ":id": id,
                ":end": ts(&end),
            },
        )
    }

    pub fn select_ending_tasks(&self) -> StoreResult<Vec<TaskRecord>> {
        self.map_rows(
            Name::SelectEndingTask,
            named_params! {
                ":now": ts(&time::SystemTime::now()),
            },
            TaskRecord::from_row,
        )
    }
}
