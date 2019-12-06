extern crate chrono;
use chrono::prelude::*;
use std::time::Duration;

pub mod expr;
pub mod store;

#[derive(PartialEq, Clone, Debug)]
pub enum TypeOfWork {
    Dev,
    Admin,
}
#[derive(PartialEq, Clone, Debug)]
pub enum UnitStatus {
    Pending,
    Done,
}

pub type Ident = String;

#[derive(Clone, Debug)]
pub struct TimeUnit {
    id: Ident,
    work: TypeOfWork,
    duration: Duration,
    status: UnitStatus,
}

impl TimeUnit {
    pub fn new(id: Ident, work: TypeOfWork, duration: Duration) -> TimeUnit {
        TimeUnit {
            id,
            work,
            duration,
            status: UnitStatus::Pending,
        }
    }

    pub fn consume(&mut self) -> &mut TimeUnit {
        // TimeUnit {
        //     id: self.id.clone(),
        //     work: self.work,
        //     status: UnitStatus::Done,
        //     duration: self.duration,
        // }
        self.status = UnitStatus::Done;
        self
    }
}

pub fn pendings(units: &Vec<TimeUnit>) -> Vec<TimeUnit> {
    units
        .iter()
        .filter(|u| u.status == UnitStatus::Pending)
        .map(|u| u.clone())
        .collect()
}

pub fn dones(units: &Vec<TimeUnit>) -> Vec<TimeUnit> {
    units
        .iter()
        .filter(|u| u.status == UnitStatus::Done)
        .map(|u| u.clone())
        .collect()
}

pub fn sum_time_units(units: &Vec<TimeUnit>) -> Duration {
    units.iter().map(|u| u.duration).sum()
}

#[derive(Debug, Clone)]
pub struct Shell {
    pub id: Ident,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl Shell {
    pub fn new(id: Ident, start: DateTime<Utc>, end: DateTime<Utc>) -> Shell {
        Shell {
            id: id.clone(),
            start,
            end,
        }
    }
}

#[derive(Debug)]
pub struct SpaceTime {
    shells: Vec<Shell>,
    time_units: Vec<TimeUnit>,
}

impl SpaceTime {
    pub fn new() -> SpaceTime {
        SpaceTime {
            shells: Vec::new(),
            time_units: Vec::new(),
        }
    }

    pub fn add_shell(
        &mut self,
        id: Ident,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> &mut SpaceTime {
        self.shells.push(Shell::new(id, start, end));
        self
    }

    pub fn get_shells(&self) -> &Vec<Shell> {
        &self.shells
    }

    pub fn add_time_unit(
        &mut self,
        id: Ident,
        work: TypeOfWork,
        duration: Duration,
    ) -> &mut SpaceTime {
        self.time_units.push(TimeUnit::new(id, work, duration));
        self
    }

    pub fn find_shell(&self, id: Ident) -> Option<Shell> {
        self.shells
            .iter()
            .find(|shell| shell.id == id)
            .map(|shell| shell.clone())
    }

    pub fn remaining(&self, id: Ident) -> Option<Vec<TimeUnit>> {
        self.find_shell(id.clone()).map(|_| {
            self.time_units
                .iter()
                .filter(|u| u.id == id && u.status == UnitStatus::Pending)
                .map(|u| u.clone())
                .collect()
        })
    }

    pub fn consume_time_unit(&mut self, id: Ident) -> &mut SpaceTime {
        let s = self.find_shell(id.clone());
        if s.is_some() {
            self.time_units
                .iter_mut()
                .filter(|u| u.id == id && u.status == UnitStatus::Pending)
                .map(|u| u.consume())
                .next();
            // .take(1)
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use chrono::prelude::*;
    // use std::time::Duration;

    #[test]
    fn it_works() {
        let mut st = SpaceTime::new();
        let id = String::from("project_0");
        st.add_shell(id.clone(), Utc::now(), Utc.ymd(2050, 1, 1).and_hms(1, 1, 1));
        st.add_time_unit(id.clone(), TypeOfWork::Dev, Duration::from_secs(60 * 60));
        st.add_time_unit(id.clone(), TypeOfWork::Dev, Duration::from_secs(60 * 60));
        st.consume_time_unit(id.clone());
        let empty = Vec::new();
        let mut o = st.remaining(id.clone());
        let r = o.get_or_insert(empty);
        let remaining_time = sum_time_units(&r);
        assert_eq!(Duration::from_secs(60 * 60), remaining_time);
    }
}
