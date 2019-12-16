use crate::chrono::Datelike;
use chrono::{DateTime, Duration, Local, TimeZone, Weekday};

pub type LocalTime = DateTime<Local>;

#[derive(Clone, Copy)]
pub enum CalRange {
    Day(LocalTime),
    Week(LocalTime),
    Month(LocalTime),
    Year(LocalTime),
}

pub fn day_of_week(lt: &LocalTime) -> &'static str {
    match lt.weekday() {
        Weekday::Mon => "Monday",
        Weekday::Tue => "Tuesday",
        Weekday::Wed => "Wednesday",
        Weekday::Thu => "Thursday",
        Weekday::Fri => "Friday",
        Weekday::Sat => "Saturday",
        Weekday::Sun => "Sunday",
    }
}

pub fn month_name(lt: &LocalTime) -> &'static str {
    match lt.month() {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Something else...",
    }
}

pub fn day(year: i32, month: u32, day: u32) -> CalRange {
    CalRange::Day(Local.ymd(year, month, day).and_hms(0, 0, 0))
}

pub fn week(year: i32, month: u32, day: u32) -> CalRange {
    let given = Local.ymd(year, month, day).and_hms(0, 0, 0);
    let wd = given.weekday();
    CalRange::Week(
        Local
            .ymd(year, month, day - wd.num_days_from_monday())
            .and_hms(0, 0, 0),
    )
}

pub fn month(year: i32, month: u32) -> CalRange {
    CalRange::Month(Local.ymd(year, month, 1).and_hms(0, 0, 0))
}

pub fn year(year: i32) -> CalRange {
    CalRange::Year(Local.ymd(year, 1, 1).and_hms(0, 0, 0))
}

type Interval = (LocalTime, LocalTime);

fn find_end_of_month(start: LocalTime) -> LocalTime {
    let starting_month = start.month();
    for i in 1..32 {
        let attempt = start + Duration::days(i);
        if attempt.month() != starting_month {
            return attempt;
        }
    }
    return start;
}

fn min(t0: LocalTime, t1: LocalTime) -> LocalTime {
    if t0 < t1 {
        return t0;
    }
    return t1;
}

impl CalRange {
    pub fn interval(self) -> Interval {
        match self {
            CalRange::Day(start) => (start, start + Duration::days(1)),
            CalRange::Week(start) => (start, start + Duration::weeks(1)),
            CalRange::Month(start) => (start, find_end_of_month(start)),
            CalRange::Year(start) => (start, start + Duration::days(365)),
        }
    }

    pub fn next(self) -> CalRange {
        match self {
            CalRange::Day(start) => CalRange::Day(start + Duration::days(1)),
            CalRange::Week(start) => CalRange::Week(start + Duration::weeks(1)),
            CalRange::Month(start) => {
                let next = find_end_of_month(start);
                month(next.year(), next.month())
            }
            CalRange::Year(start) => year(start.year() + 1),
        }
    }

    pub fn iter(self) -> CalRangeIterator {
        match self {
            CalRange::Day(start) => CalRangeIterator {
                start,
                end: start + Duration::days(1),
                step: Box::new(|s| s + Duration::hours(1)),
            },
            CalRange::Week(start) => CalRangeIterator {
                start,
                end: start + Duration::weeks(1),
                step: Box::new(|s| s + Duration::hours(24)),
            },
            CalRange::Month(start) => {
                let end = find_end_of_month(start);
                CalRangeIterator {
                    start,
                    end,
                    step: Box::new(|s| min(s + Duration::days(7), find_end_of_month(s))),
                }
            }
            CalRange::Year(start) => CalRangeIterator {
                start,
                end: start + Duration::days(365),
                step: Box::new(|s| find_end_of_month(s)),
            },
        }
    }
}

type Stepper = dyn Fn(LocalTime) -> LocalTime;

pub struct CalRangeIterator {
    start: LocalTime,
    end: LocalTime,
    step: Box<Stepper>,
}

impl Iterator for CalRangeIterator {
    type Item = Interval;
    fn next(&mut self) -> Option<Interval> {
        if self.start >= self.end {
            return None;
        }
        let end = (self.step)(self.start);
        let interval = (self.start.clone(), end);
        self.start = end;
        Some(interval)
    }
}

#[cfg(test)]
mod tests {
    use crate::cal::*;
    use chrono::{Duration, Local, TimeZone};

    #[test]
    fn iter_day_ok() {
        let s = Local.ymd(2020, 1, 1).and_hms(0, 0, 0);
        let range = day(2020, 1, 1);
        if let Some((start, end)) = range.iter().next() {
            assert_eq!(start, s);
            assert_eq!(end, s + Duration::hours(1));
        }
    }
}
