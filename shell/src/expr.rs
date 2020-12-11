use crate::chrono::Datelike;
use crate::chrono::TimeZone;
use chrono::offset::Utc;
use humantime;
use pom::parser::*;
use pom::Error;
use pom::Parser;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::convert::TryInto;
use std::slice;
use std::time;

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum Command {
    Ping,
    Add(String),
    Do(String, String, time::Duration),
    Done(String, String, time::Duration),
    Switch(String, String),
    Stop,
    More(time::Duration),
    List,
    Digest(String),
    Cal(String),
    Since(time::SystemTime),
    ProjectDeadline(String, time::SystemTime),
    ProjectProvision(String, time::Duration),
}

// #[derive(Debug)]
// struct ParseCommandError;

// impl fmt::Display for ParseCommandError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "Command does not exists")
//     }
// }

// impl Error for ParseCommandError {}

fn space() -> Parser<u8, ()> {
    one_of(b" \t").repeat(1..).discard()
}

fn trailing_space() -> Parser<u8, ()> {
    one_of(b" \t").repeat(0..).discard()
}

fn string() -> Parser<u8, String> {
    let any = is_a(|_| true);
    let char_string = any.repeat(1..) - (sym(b'.') | end().map(|_| b'.'));
    char_string.convert(|chars| String::from_utf8(chars))
}

fn letter() -> Parser<u8, u8> {
    let lc = one_of(b"abcdefghijklmnopqrstuvwxyz");
    let uc = one_of(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    lc | uc
}

fn digit() -> Parser<u8, u8> {
    let n = one_of(b"0123456789");
    n
}

// fn integer() -> Parser<u8, u32> {
//     digit()
//         .repeat(1..)
//         .convert(|chars| String::from_utf8(chars.to_vec()))
//         .map(|s| match s.parse::<u32>() {
//             Ok(n) => n,
//             Err(_) => 0,
//         })
// }

fn fixed_int(i: usize) -> Parser<u8, u32> {
    digit()
        .repeat(i)
        .convert(|chars| String::from_utf8(chars.to_vec()))
        .map(|s| match s.parse::<u32>() {
            Ok(n) => n,
            Err(_) => 0,
        })
}

fn ident() -> Parser<u8, String> {
    let char_string = (letter() | digit() | one_of(b"_-/#@")).repeat(1..);
    char_string.convert(|chars| String::from_utf8(chars))
}

// fn duration_() -> Parser<u8, time::Duration> {
//     string().map(|s| match humantime::parse_duration(&s) {
//         Ok(d) => d,
//         Err(err) => time::Duration::from_secs(0),
//     })
// }

fn duration() -> Parser<u8, time::Duration> {
    let string_parser = string();
    Parser::new(
        move |input: &[u8], start: usize| match string_parser.parse(&input[start..]) {
            Err(e) => Err(e),
            Ok(s) => match humantime::parse_duration(&s) {
                Ok(d) => Ok((d, start + s.len())),
                Err(err) => {
                    println!("err({}) {}", s, err);
                    Err(pom::Error::Custom {
                        message: format!("HumanTimeError {}", err),
                        position: start,
                        inner: None,
                    })
                }
            },
        },
    )
}

fn st_from_ts(ts: i64) -> time::SystemTime {
    time::SystemTime::UNIX_EPOCH + time::Duration::from_millis(ts.try_into().unwrap())
}

fn date() -> Parser<u8, time::SystemTime> {
    let sep = || one_of(b" -./");
    // YYYY-MM-DD
    let format1 = (fixed_int(4) - sep()) + (fixed_int(2) - sep()) + fixed_int(2);

    // DD-MM[-YYYY]
    let format2 = (fixed_int(2) - sep()) + fixed_int(2) + (-sep() + fixed_int(4)).opt();

    let mapped1 = format1.map(|((y, m), d)| {
        st_from_ts(
            Utc.ymd(i32::try_from(y).unwrap_or(i32::max_value()), m, d)
                .and_hms(0, 1, 1)
                .timestamp_millis(),
        )
    });

    let mapped2 = format2.map(|((d, m), opt_y)| {
        let (_, y) = opt_y.unwrap_or_else(|| {
            (
                true,
                Utc::now().year().try_into().unwrap_or(u32::max_value()),
            )
        });
        st_from_ts(
            Utc.ymd(i32::try_from(y).unwrap_or(i32::max_value()), m, d)
                .and_hms(0, 1, 1)
                .timestamp_millis(),
        )
    });

    mapped1 | mapped2
}

type CommandParser = Parser<u8, Command>;

fn ping() -> CommandParser {
    let cn = seq(b"!ping");
    cn.map(|_| Command::Ping).name("ping")
}

fn add() -> CommandParser {
    let cn = seq(b"!new") - space();
    let id = ident();
    let all = cn + id;
    all.map(|(_, project_name)| Command::Add(project_name))
        .name("new")
}

fn digest() -> CommandParser {
    let cn = seq(b"!digest") - space();
    let id = ident();
    let all = cn + id;
    all.map(|(_, project_name)| Command::Digest(project_name))
        .name("digest")
}

fn cal() -> CommandParser {
    let cn = seq(b"!cal") - space();
    let id = ident();
    let all = cn + id;
    all.map(|(_, project_name)| Command::Cal(project_name))
        .name("cal")
}

fn start() -> CommandParser {
    let cn = seq(b"!do") - space();
    let id = ident() - space();
    let task = ident() - space();
    let d = duration();
    let all = cn + id + task + d;
    all.map(|(((_, project_name), task), duration)| Command::Do(project_name, task, duration))
        .name("do")
}

fn done() -> CommandParser {
    let cn = seq(b"!done") - space();
    let id = ident() - space();
    let task = ident() - space();
    let d = duration();
    let all = cn + id + task + d;
    all.map(|(((_, project_name), task), duration)| Command::Done(project_name, task, duration))
        .name("done")
}

fn switch() -> CommandParser {
    let cn = seq(b"!switch") - space();
    let id = ident() - space();
    let task = ident();
    let all = cn + id + task;
    all.map(|((_, project_name), task)| Command::Switch(project_name, task))
        .name("switch")
}

fn stop() -> CommandParser {
    let cn = seq(b"!stop");
    cn.map(|_| Command::Stop).name("stop")
}

fn more() -> CommandParser {
    let cn = seq(b"!more") - space();
    let d = duration();
    let all = cn + d;
    all.map(|(_, duration)| Command::More(duration))
        .name("more")
}

fn list() -> CommandParser {
    let cn = seq(b"!ls");
    cn.map(|_| Command::List).name("list")
}

fn since() -> CommandParser {
    let cn = seq(b"!since");
    let t = date() | duration().map(|d| time::SystemTime::now() - d);
    let all = cn - space() + t;
    all.map(|(_, st)| Command::Since(st)).name("since")
}

fn deadline() -> CommandParser {
    let cn = seq(b"!deadline") - space();
    let id = ident() - space();
    let d = date();
    let all = cn + id + d;
    all.map(|((_, project_name), d)| Command::ProjectDeadline(project_name, d))
        .name("deadline")
}

fn provision() -> CommandParser {
    let cn = seq(b"!provision") - space();
    let id = ident() - space();
    let d = duration();
    let all = cn + id + d;
    all.map(|((_, project_name), d)| Command::ProjectProvision(project_name, d))
        .name("provision")
}

fn command() -> CommandParser {
    {
        ping()
            | add()
            | start()
            | done()
            | stop()
            | list()
            | digest()
            | since()
            | more()
            | switch()
            | cal()
            | deadline()
            | provision()
    }
    .name("command")
        - trailing_space()
}

pub fn parse_command<'a>(expr: &'a str) -> Result<Command, Error> {
    println!("enter parser {}", expr);
    let ptr = expr.as_ptr();
    let len = expr.len();
    let result = {
        unsafe {
            let slice = slice::from_raw_parts(ptr, len);
            command().parse(slice)
        }
    };
    println!("leaving parser {} {}", expr, result.is_ok());
    result
}

#[cfg(test)]
mod tests {
    use crate::expr::*;
    #[test]
    fn parse_do_ok() {
        assert_eq!(
            parse_command("!do foo-0 dev 3h 30m"),
            Ok(Command::Do(
                "foo-0".into(),
                "dev".into(),
                time::Duration::from_secs(3 * 60 * 60 + (30 * 60))
            ))
        );
    }
    #[test]
    fn parse_new_ok() {
        assert_eq!(
            parse_command("!new ac-bot"),
            Ok(Command::Add("ac-bot".into(),))
        );
    }
}
