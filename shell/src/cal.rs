use chrono::Datelike;
use chrono::{DateTime, Duration, Local, TimeZone, Weekday};

pub type LocalTime = DateTime<Local>;

#[derive(Clone, Copy, Debug)]
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
    match wd {
        Weekday::Mon => CalRange::Week(given),
        _ => CalRange::Week(given - Duration::days(wd.num_days_from_monday().into())),
    }
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
    for i in 0..33 {
        let attempt = start + Duration::days(i);
        if attempt.month() != starting_month {
            return attempt;
        }
    }
    return start;
}

fn find_end_of_year(start: LocalTime) -> LocalTime {
    let starting_year = start.year();
    let near_end = start + Duration::days(364);
    for i in 1..3 {
        let attempt = near_end + Duration::days(i);
        if attempt.year() != starting_year {
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

fn max(t0: LocalTime, t1: LocalTime) -> LocalTime {
    if t0 > t1 {
        return t0;
    }
    return t1;
}

impl CalRange {
    pub fn interval(&self) -> Interval {
        match *self {
            CalRange::Day(start) => (start, start + Duration::days(1)),
            CalRange::Week(start) => (start, start + Duration::weeks(1)),
            CalRange::Month(start) => (start, find_end_of_month(start)),
            CalRange::Year(start) => (start, find_end_of_year(start)),
        }
    }

    pub fn in_range(&self, t: LocalTime) -> bool {
        let (start, end) = self.interval();
        start <= t && t < end
    }

    pub fn prev(&self) -> CalRange {
        match *self {
            CalRange::Day(start) => CalRange::Day(start - Duration::days(1)),
            CalRange::Week(start) => CalRange::Week(start - Duration::weeks(1)),
            CalRange::Month(start) => match start.month() {
                1 => month(start.year() - 1, 12),
                n => month(start.year(), n),
            },
            CalRange::Year(start) => year(start.year() + 1),
        }
    }

    pub fn next(&self) -> CalRange {
        match *self {
            CalRange::Day(start) => CalRange::Day(start + Duration::days(1)),
            CalRange::Week(start) => CalRange::Week(start + Duration::weeks(1)),
            CalRange::Month(start) => {
                let next = find_end_of_month(start);
                month(next.year(), next.month())
            }
            CalRange::Year(start) => year(start.year() + 1),
        }
    }

    pub fn iter(&self) -> CalRangeIterator {
        match *self {
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
                end: find_end_of_year(start),
                step: Box::new(|s| find_end_of_month(s) + Duration::days(1)),
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

#[derive(Clone)]
pub struct CalendarEvent<T>
where
    T: Clone,
{
    pub start_time: LocalTime,
    pub end_time: LocalTime,
    pub data: T,
}

pub struct Calendar<T>
where
    T: Clone,
{
    events: Vec<CalendarEvent<T>>,
}

impl<T> Calendar<T>
where
    T: Clone,
{
    pub fn new() -> Calendar<T> {
        Calendar { events: Vec::new() }
    }

    pub fn push(&mut self, start_time: LocalTime, end_time: LocalTime, data: T) {
        self.events.push(CalendarEvent {
            start_time,
            end_time,
            data,
        })
    }

    fn find(&self, start: LocalTime, end: LocalTime) -> Vec<CalendarEvent<T>> {
        self.events
            .clone()
            .into_iter()
            .filter(|e| {
                (start < e.start_time && e.start_time < end)
                    || (start < e.end_time && e.end_time < end)
                    || (start < e.start_time && e.end_time < end)
            })
            .collect()
    }

    pub fn start_time(&self) -> LocalTime {
        let initial = Local.ymd(3000, 1, 1).and_hms(0, 0, 0);
        self.events
            .iter()
            .fold(initial, |acc, e| min(acc, e.start_time))
    }

    pub fn end_time(&self) -> LocalTime {
        let initial = Local.ymd(1, 1, 1).and_hms(0, 0, 0);
        self.events
            .iter()
            .fold(initial, |acc, e| max(acc, e.end_time))
    }

    pub fn iter(&self) -> CalendarIterator<'_, T> {
        let s = self.start_time();
        CalendarIterator {
            step: CalendarIteratorStep::Year,
            cur_year: year(s.year()),
            cur_month: month(s.year(), s.month()),
            cur_week: week(s.year(), s.month(), 1),
            iter_week: week(s.year(), s.month(), 1).iter(),
            calendar: self,
        }
    }
}

enum CalendarIteratorStep {
    // Start,
    Year,
    Month,
    Week,
    Day,
}

pub struct CalendarIterator<'c, T>
where
    T: Clone,
{
    step: CalendarIteratorStep,
    cur_year: CalRange,
    cur_month: CalRange,
    cur_week: CalRange,
    iter_week: CalRangeIterator,
    calendar: &'c Calendar<T>,
}

pub enum CalendarItem<T>
where
    T: Clone,
{
    Year(LocalTime),
    Month(LocalTime),
    Week(LocalTime),
    Day(LocalTime, Vec<CalendarEvent<T>>),
    EmptyDay(LocalTime, Vec<CalendarEvent<T>>),
}

impl<'c, T> CalendarIterator<'c, T>
where
    T: Clone,
{
    fn start(&mut self) -> LocalTime {
        self.calendar.start_time()
    }
    fn end(&mut self) -> LocalTime {
        self.calendar.end_time()
    }
}

impl<'c, T> Iterator for CalendarIterator<'c, T>
where
    T: Clone,
{
    type Item = CalendarItem<T>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.step {
            // CalendarIteratorStep::Start => {
            //     self.step = CalendarIteratorStep::Month;
            //     Some(CalendarItem::Year(self.cur_year.interval().0))
            // }
            CalendarIteratorStep::Year => {
                let (start, _) = self.cur_year.interval();
                if start > self.end() {
                    None
                } else {
                    self.step = CalendarIteratorStep::Month;
                    Some(CalendarItem::Year(start))
                }
            }
            CalendarIteratorStep::Month => {
                let (start, end) = self.cur_month.interval();
                if start > self.end() {
                    None
                } else {
                    // println!("Step::Month {}", self.cur_year.in_range(start));
                    if self.cur_year.in_range(start) || self.cur_year.in_range(end) {
                        self.step = CalendarIteratorStep::Week;
                        Some(CalendarItem::Month(start))
                    } else {
                        self.step = CalendarIteratorStep::Year;
                        self.cur_year = self.cur_year.next();
                        self.next()
                    }
                }
            }
            CalendarIteratorStep::Week => {
                let (start, end) = self.cur_week.interval();
                if self.cur_month.in_range(start) || self.cur_month.in_range(end) {
                    self.step = CalendarIteratorStep::Day;
                    Some(CalendarItem::Week(start))
                } else {
                    self.step = CalendarIteratorStep::Month;
                    self.cur_month = self.cur_month.next();
                    if start > self.cur_month.interval().0 {
                        self.cur_week = self.cur_week.prev();
                        self.iter_week = self.cur_week.iter();
                    }
                    self.next()
                }
            }
            CalendarIteratorStep::Day => {
                let day = self.iter_week.next();
                match day {
                    Some((day_start, day_end)) => {
                        if self.cur_month.in_range(day_start) {
                            Some(CalendarItem::Day(
                                day_start,
                                self.calendar.find(day_start, day_end),
                            ))
                        } else {
                            Some(CalendarItem::EmptyDay(
                                day_start,
                                self.calendar.find(day_start, day_end),
                            ))
                        }
                    }
                    None => {
                        let next_week = self.cur_week.next();
                        // if next_week.interval().0 >= self.cur_month.interval().0 {
                        self.cur_week = next_week;
                        // }
                        self.iter_week = self.cur_week.iter();
                        self.step = CalendarIteratorStep::Week;
                        self.next()
                    }
                }
            }
        }
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

    #[test]
    fn week_start_ok() {
        let d0 = 6;
        let d1 = 12;
        let dt0 = Local.ymd(2020, 1, d0).and_hms(0, 0, 0);
        let dt1 = Local.ymd(2020, 1, d1).and_hms(0, 0, 0);
        let w0 = week(dt0.year(), dt0.month(), d0);
        let w1 = week(dt1.year(), dt1.month(), d1);
        let (start0, _) = w0.interval();
        let (start1, _) = w1.interval();
        assert_eq!(start0, dt0);
        assert_eq!(start0, start1);
    }

    #[test]
    fn iter_calendar_ok() {
        let mut cal: Calendar<u32> = Calendar::new();
        let (s0, e0) = (
            Local.ymd(2020, 1, 1).and_hms(12, 0, 0),
            Local.ymd(2020, 1, 1).and_hms(14, 0, 0),
        );
        let (s1, e1) = (
            Local.ymd(2021, 1, 1).and_hms(12, 0, 0),
            Local.ymd(2021, 1, 1).and_hms(14, 0, 0),
        );
        cal.push(s0, e0, 1);
        cal.push(s1, e1, 2);
        let mut prev_year: Option<LocalTime> = None;
        let mut prev_month: Option<LocalTime> = None;
        let mut prev_week: Option<LocalTime> = None;
        for i in cal.iter() {
            match i {
                CalendarItem::Year(d) => {
                    prev_year.map(|pd| assert_ne!(d, pd));
                    println!("\nYear({})", d.year());
                    prev_year = Some(d);
                }
                CalendarItem::Month(d) => {
                    prev_month.map(|pd| assert_ne!(d, pd));
                    println!("\nMonth({})", month_name(&d));
                    prev_month = Some(d);
                }
                CalendarItem::Week(d) => {
                    // prev_week.map(|pd| assert_ne!(d, pd));
                    println!("\nWeek({})", d.day());
                    prev_week = Some(d);
                }
                CalendarItem::Day(d, es) => {
                    let u = es.first().map(|e| e.data).unwrap_or(0);
                    println!("{} {} \t({})", day_of_week(&d), d.day(), u);
                }
                CalendarItem::EmptyDay(d) => {
                    println!("[{} {}]", day_of_week(&d), d.day());
                }
            }
        }
    }
}
