use crate::expr::Command;
use crate::util::{dur, ts};
use rusqlite::{named_params, Connection, Result as SqlResult, Row, ToSql, NO_PARAMS};
use serde::{Deserialize, Serialize};
use std;
use std::fmt;
use std::path::Path;
use std::time;

#[derive(Serialize, Deserialize)]
pub struct Record {
    pub time: time::SystemTime,
    pub username: String,
    pub command: Command,
}

impl Record {
    pub fn new(time: time::SystemTime, username: String, command: Command) -> Record {
        Record {
            time,
            username,
            command,
        }
    }
}

pub struct Store {
    path: String,
    conn: Connection,
}

#[derive(Debug, Clone)]
pub enum StoreError {
    Open(String),
    LogRecord,
    Iter,
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::Open(p) => write!(f, "Failed to open connection to {}", p),
            StoreError::LogRecord => write!(f, "Failed to log record"),
            StoreError::Iter => write!(f, "Failed to iterate"),
        }
    }
}

impl std::error::Error for StoreError {}

pub enum Name {
    InsertDo,
    InsertProject,
    InsertNotification,
    UpdateTaskEnd,
    SelectCurrentTask,
    SelectCurrentTaskFor,
    SelectLatestTaskFor,
    SelectEndingTask,
    SelectProjectInfo,
    SelectProject,
    SelectUser,
}

fn sql(name: Name) -> &'static str {
    match name {
        Name::InsertDo => include_str!("sql/insert_do.sql"),
        Name::InsertProject => include_str!("sql/insert_project.sql"),
        Name::InsertNotification => include_str!("sql/insert_notification.sql"),
        Name::SelectCurrentTask => include_str!("sql/select_current_task.sql"),
        Name::SelectCurrentTaskFor => include_str!("sql/select_current_task_for.sql"),
        Name::SelectLatestTaskFor => include_str!("sql/select_latest_task_for.sql"),
        Name::SelectEndingTask => include_str!("sql/select_ending_task.sql"),
        Name::UpdateTaskEnd => include_str!("sql/update_task_end.sql"),
        Name::SelectProject => include_str!("sql/select_project.sql"),
        Name::SelectProjectInfo => include_str!("sql/select_project_info.sql"),
        Name::SelectUser => include_str!("sql/select_user.sql"),
    }
}

impl Store {
    pub fn new(path: &Path) -> Result<Store, StoreError> {
        match Connection::open(path) {
            Ok(conn) => {
                conn.execute("PRAGMA foreign_keys = ON;", NO_PARAMS)
                    .expect("Failed to enable foreign_keys");
                conn.execute(include_str!("sql/create_do.sql"), NO_PARAMS)
                    .expect("failed creating table do");
                conn.execute(include_str!("sql/create_project.sql"), NO_PARAMS)
                    .expect("failed creating table project");
                conn.execute(include_str!("sql/create_notification.sql"), NO_PARAMS)
                    .expect("failed creating table notification");
                println!("{}", path.display());
                Ok(Store {
                    conn,
                    path: path.to_string_lossy().into(),
                })
            }
            Err(_) => Err(StoreError::Open(String::from(path.to_string_lossy()))),
        }
    }

    pub fn get_path(&self) -> &Path {
        Path::new(&self.path)
    }

    pub fn exec(&self, name: Name, params: &[(&str, &dyn ToSql)]) -> Result<usize, StoreError> {
        match self.conn.execute_named(sql(name), params) {
            Ok(s) => Ok(s),
            Err(err) => {
                println!("SQLite error: {}", err);
                Err(StoreError::LogRecord)
            }
        }
    }
    fn map_rows<F, T>(
        &self,
        name: Name,
        params: &[(&str, &dyn ToSql)],
        f: F,
    ) -> Result<Vec<T>, StoreError>
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
                Ok(rows) => Ok(rows.filter_map(|row| row.ok()).collect()),
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
    ) -> Result<usize, StoreError> {
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
        duration: time::Duration,
    ) -> Result<usize, StoreError> {
        self.exec(
            Name::InsertProject,
            named_params! {
                ":username": username.clone(),
                ":name": name.clone(),
                ":start": ts(&start),
                ":duration":  dur(&duration),
            },
        )
    }

    pub fn insert_notification(&mut self, tid: i64, end: i64) -> Result<usize, StoreError> {
        self.exec(
            Name::InsertNotification,
            named_params! {
                ":tid": tid,
                ":end":  end,
            },
        )
    }

    pub fn select_current_task<F, T>(&self, f: F) -> Result<Vec<T>, StoreError>
    where
        F: FnMut(&Row) -> SqlResult<T>,
    {
        let now = time::SystemTime::now();
        self.map_rows(
            Name::SelectCurrentTask,
            named_params! {
                ":now": ts(&now),
            },
            f,
        )
    }

    pub fn select_current_task_for<F, T>(&self, user: String, f: F) -> Result<Vec<T>, StoreError>
    where
        F: FnMut(&Row) -> SqlResult<T>,
    {
        let now = time::SystemTime::now();
        self.map_rows(
            Name::SelectCurrentTaskFor,
            named_params! {
                ":user": user.clone(),
                ":now": ts(&now),
            },
            f,
        )
    }

    pub fn select_latest_task_for<F, T>(&self, user: String, f: F) -> Result<Vec<T>, StoreError>
    where
        F: FnMut(&Row) -> SqlResult<T>,
    {
        self.map_rows(
            Name::SelectLatestTaskFor,
            named_params! {
                ":user": user.clone(),
            },
            f,
        )
    }

    pub fn select_project_info<F, T>(&self, project: String, f: F) -> Result<Vec<T>, StoreError>
    where
        F: FnMut(&Row) -> SqlResult<T>,
    {
        self.map_rows(
            Name::SelectProjectInfo,
            named_params! {
                ":project": project.clone(),
            },
            f,
        )
    }

    pub fn select_project<F, T>(&self, project: String, f: F) -> Result<Vec<T>, StoreError>
    where
        F: FnMut(&Row) -> SqlResult<T>,
    {
        self.map_rows(
            Name::SelectProject,
            named_params! {
                ":project": project.clone(),
            },
            f,
        )
    }

    pub fn select_user<F, T>(
        &self,
        user: String,
        since: time::SystemTime,
        f: F,
    ) -> Result<Vec<T>, StoreError>
    where
        F: FnMut(&Row) -> SqlResult<T>,
    {
        self.map_rows(
            Name::SelectUser,
            named_params! {
                ":user": user.clone(),
                ":since": ts(&since),
            },
            f,
        )
    }

    pub fn update_task_end(&self, id: i64, end: time::SystemTime) -> Result<usize, StoreError> {
        self.exec(
            Name::UpdateTaskEnd,
            named_params! {
                ":id": id,
                ":end": ts(&end),
            },
        )
    }

    pub fn select_ending_tasks<F, T>(&self, f: F) -> Result<Vec<T>, StoreError>
    where
        F: FnMut(&Row) -> SqlResult<T>,
    {
        self.map_rows(
            Name::SelectEndingTask,
            named_params! {
                ":now": ts(&time::SystemTime::now()),
            },
            f,
        )
    }
}
