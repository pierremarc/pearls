use crate::expr::Command;
use rusqlite::{named_params, Connection, Error, Result as SqlResult, Row, ToSql, NO_PARAMS};
use serde::{Deserialize, Serialize};
use std;
use std::convert::TryFrom;
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
    UpdateTaskEnd,
    SelectCurrentTask,
    SelectCurrentTaskFor,
    SelectProject,
    SelectUser,
}

fn sql(name: Name) -> &'static str {
    match name {
        Name::InsertDo => include_str!("sql/insert_do.sql"),
        Name::SelectCurrentTask => include_str!("sql/select_current_task.sql"),
        Name::SelectCurrentTaskFor => include_str!("sql/select_current_task_for.sql"),
        Name::UpdateTaskEnd => include_str!("sql/update_task_end.sql"),
        Name::SelectProject => include_str!("sql/select_project.sql"),
        Name::SelectUser => include_str!("sql/select_user.sql"),
    }
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

impl Store {
    pub fn new(path: &Path) -> Result<Store, StoreError> {
        match Connection::open(path) {
            Ok(conn) => {
                conn.execute(include_str!("sql/create_do.sql"), NO_PARAMS)
                    .expect("filed creating table");
                println!("{}", path.display());
                Ok(Store { conn })
            }
            Err(_) => Err(StoreError::Open(String::from(path.to_string_lossy()))),
        }
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

    pub fn log(&mut self, rec: &Record) -> Result<usize, StoreError> {
        match &rec.command {
            Command::Do(project, task, duration) => self.exec(
                Name::InsertDo,
                named_params! {
                    ":username": rec.username,
                    ":start": ts(&rec.time),
                    ":end": ts(&rec.time) + dur(duration),
                    ":project": project,
                    ":task": task,
                },
            ),

            _ => Err(StoreError::LogRecord),
        }
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

    pub fn update_task_end(&self, id: i64) -> Result<usize, StoreError> {
        let now = time::SystemTime::now();
        self.exec(
            Name::UpdateTaskEnd,
            named_params! {
                ":id": id,
                ":now": ts(&now),
            },
        )
    }
}
