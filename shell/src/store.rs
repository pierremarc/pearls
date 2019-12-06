use crate::expr::Command;
use serde::{Deserialize, Serialize};
use serde_json;
use std::error::Error;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::slice::Iter;
use std::sync::{Arc, RwLock};
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
    log_file: Option<File>,
    records: Vec<Record>,
}

// #[derive(Debug)]
// enum StoreError {
//     Serialization(),
//     Write,
// }

// impl fmt::Display for StoreError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             StoreError::Serialization => write!(f, "Serialization Error"),
//             StoreError::Write => write!(f, "Write Error"),
//         }
//     }
// }

// impl Error for StoreError {}

fn read_log(log: &File) -> Vec<Record> {
    BufReader::new(log)
        .lines()
        .filter_map(|l| match l {
            Err(_) => None,
            Ok(l) => match serde_json::from_str(&l) {
                Err(_) => None,
                Ok(rec) => Some(rec),
            },
        })
        .collect()
}

pub type SharedStore = Arc<RwLock<Box<Store>>>;
impl Store {
    pub fn new(path: &Path) -> SharedStore {
        let log_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path);
        let store = match log_file {
            Ok(log_file) => Box::new(Store {
                records: read_log(&log_file),
                log_file: Some(log_file),
            }),
            Err(_) => Box::new(Store {
                log_file: None,
                records: Vec::new(),
            }),
        };

        Arc::new(RwLock::new(store))
    }

    pub fn log(&mut self, rec: &Record) -> Result<(), io::Error> {
        let ser = serde_json::to_string(rec).expect("Oh my");
        match &mut self.log_file {
            None => Ok(()),
            Some(f) => f.write_all(ser.as_bytes()),
        }
    }

    pub fn iter(&self) -> Iter<Record> {
        self.records.iter()
    }
}
