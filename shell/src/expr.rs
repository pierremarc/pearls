use chrono::Datelike;
use chrono::TimeZone;
use chrono::{offset::Utc, LocalResult};
use humantime;
use pom::parser::{end, is_a, one_of, seq, sym, Parser};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::convert::TryInto;
use std::time;

use crate::parser_ext::{
    ctx_command, err_date_format, err_duration_format, err_ident, err_project_ident, new_context,
    with_error, with_success, ParseCommandError, SharedContext,
};

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum Command {
    Ping,
    Help,
    Add(String),
    Do(String, String, time::Duration),
    Done(String, String, time::Duration),
    Switch(String, String),
    Stop,
    More(time::Duration),
    List,
    Digest(String),
    Since(time::SystemTime),
    Deadline(String, time::SystemTime),
    Provision(String, time::Duration),
    Complete(String, time::SystemTime),
    Note(String, String),
}
// pub enum CommandName {
//     Ping,
//     Add,
//     Do,
//     Done,
//     Switch,
//     Stop,
//     More,
//     List,
//     Digest,
//     Since,
//     Deadline,
//     Provision,
//     Complete,
//     Note,
// }

fn space<'a>() -> Parser<'a, u8, ()> {
    one_of(b" \t").repeat(1..).discard()
}

fn trailing_space<'a>() -> Parser<'a, u8, ()> {
    one_of(b" \t").repeat(0..).discard()
}

fn string<'a>() -> Parser<'a, u8, String> {
    let any = is_a(|_| true);
    let char_string = any.repeat(1..) - (sym(b'.') | end().map(|_| b'.'));
    char_string.convert(|chars| String::from_utf8(chars))
}

fn letter<'a>() -> Parser<'a, u8, u8> {
    let lc = one_of(b"abcdefghijklmnopqrstuvwxyz");
    let uc = one_of(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    lc | uc
}

fn digit<'a>() -> Parser<'a, u8, u8> {
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

fn fixed_int<'a>(i: usize) -> Parser<'a, u8, u32> {
    digit()
        .repeat(i)
        .convert(|chars| String::from_utf8(chars.to_vec()))
        .map(|s| match s.parse::<u32>() {
            Ok(n) => n,
            Err(_) => 0,
        })
}

fn ident<'a>(ctx: SharedContext) -> Parser<'a, u8, String> {
    let char_string = (letter() | digit() | one_of(b"_-.")).repeat(1..);
    with_error(
        char_string.convert(|chars| String::from_utf8(chars)),
        move || err_ident(ctx.clone()),
    )
}

fn project_ident<'a>(ctx: SharedContext) -> Parser<'a, u8, String> {
    let client = ident(ctx.clone());
    let sep = sym(b'/');
    let name = ident(ctx.clone());
    let all = client - sep + name;
    with_error(all, move || err_project_ident(ctx.clone()))
        .map(|(client, name)| format!("{}/{}", client, name))
}

// fn duration_() -> Parser<u8, time::Duration> {
//     string().map(|s| match humantime::parse_duration(&s) {
//         Ok(d) => d,
//         Err(err) => time::Duration::from_secs(0),
//     })
// }

fn duration<'a>(ctx: SharedContext) -> Parser<'a, u8, time::Duration> {
    let string_parser = string();
    Parser::new(
        move |input: &[u8], start: usize| match string_parser.parse(&input[start..]) {
            Err(e) => Err(e),
            Ok(s) => match humantime::parse_duration(&s) {
                Ok(d) => Ok((d, start + s.len())),
                Err(err) => {
                    err_duration_format(ctx.clone());
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

fn st_from_ts(ts: i64) -> Result<time::SystemTime, impl std::error::Error> {
    ts.try_into()
        .map(|x| time::SystemTime::UNIX_EPOCH + time::Duration::from_millis(x))
}

fn date<'a>(ctx: SharedContext) -> Parser<'a, u8, time::SystemTime> {
    let sep = || one_of(b" -./");
    // YYYY-MM-DD
    let format1 = (fixed_int(4) - sep()) + (fixed_int(2) - sep()) + fixed_int(2);

    // DD-MM[-YYYY]
    let format2 = (fixed_int(2) - sep()) + fixed_int(2) + (sep() + fixed_int(4)).opt();

    let mapped1 = format1.convert(|((y, m), d)| {
        let year = i32::try_from(y).map_err(|_| ParseCommandError::DateFormat)?;
        match Utc.ymd_opt(year, m, d) {
            LocalResult::Single(d) => st_from_ts(d.and_hms(0, 1, 1).timestamp_millis())
                .map_err(|_| ParseCommandError::DateFormat),
            _ => Err(ParseCommandError::DateFormat),
        }
    });

    let mapped2 = format2.convert(|((d, m), opt_y)| {
        let opt_y = opt_y.and_then(|(_, y)| i32::try_from(y).ok());
        let year = match opt_y {
            None => Utc::now().year(),
            Some(y) => y,
        };
        match Utc.ymd_opt(year, m, d) {
            LocalResult::Single(d) => st_from_ts(d.and_hms(0, 1, 1).timestamp_millis())
                .map_err(|_| ParseCommandError::DateFormat),
            _ => Err(ParseCommandError::DateFormat),
        }
    });

    with_error(mapped1 | mapped2, move || err_date_format(ctx.clone()))
}

type CommandParser<'a> = Parser<'a, u8, Command>;

fn ping<'a>(ctx: SharedContext) -> CommandParser<'a> {
    let cn = with_success(seq(b"!ping"), move || ctx_command("ping", ctx.clone()));
    cn.map(|_| Command::Ping)
}

fn help<'a>(ctx: SharedContext) -> CommandParser<'a> {
    let cn = with_success(seq(b"!help") | seq(b"!h"), move || {
        ctx_command("help", ctx.clone())
    });
    cn.map(|_| Command::Help)
}

fn add<'a>(ctx: SharedContext) -> CommandParser<'a> {
    let mctx = ctx.clone();
    let cn = with_success(seq(b"!new") - space(), move || {
        ctx_command("new", mctx.clone())
    });
    let id = project_ident(ctx.clone());
    let all = cn + id;
    all.map(|(_, project_name)| Command::Add(project_name))
}

fn digest<'a>(ctx: SharedContext) -> CommandParser<'a> {
    let mctx = ctx.clone();
    let cn = with_success(seq(b"!digest") - space(), move || {
        ctx_command("digest", mctx.clone())
    });
    let id = project_ident(ctx.clone());
    let all = cn + id;
    all.map(|(_, project_name)| Command::Digest(project_name))
}

fn start<'a>(ctx: SharedContext) -> CommandParser<'a> {
    let mctx = ctx.clone();
    let cn = with_success(seq(b"!do") - space(), move || {
        ctx_command("do", mctx.clone())
    });
    let id = project_ident(ctx.clone()) - space();
    let task = ident(ctx.clone()) - space();
    let d = duration(ctx.clone());
    let all = cn + id + task + d;
    all.map(|(((_, project_name), task), duration)| Command::Do(project_name, task, duration))
        .name("do")
}

fn done<'a>(ctx: SharedContext) -> CommandParser<'a> {
    let mctx = ctx.clone();
    let cn = with_success(seq(b"!done") - space(), move || {
        ctx_command("done", mctx.clone())
    });
    let id = project_ident(ctx.clone()) - space();
    let task = ident(ctx.clone()) - space();
    let d = duration(ctx.clone());
    let all = cn + id + task + d;
    all.map(|(((_, project_name), task), duration)| Command::Done(project_name, task, duration))
        .name("done")
}

fn switch<'a>(ctx: SharedContext) -> CommandParser<'a> {
    let mctx = ctx.clone();
    let cn = with_success(seq(b"!switch") - space(), move || {
        ctx_command("switch", mctx.clone())
    });
    let id = project_ident(ctx.clone()) - space();
    let task = ident(ctx.clone());
    let all = cn + id + task;
    all.map(|((_, project_name), task)| Command::Switch(project_name, task))
        .name("switch")
}

fn stop<'a>(ctx: SharedContext) -> CommandParser<'a> {
    let cn = with_success(seq(b"!stop"), move || ctx_command("stop", ctx.clone()));
    cn.map(|_| Command::Stop).name("stop")
}

fn more<'a>(ctx: SharedContext) -> CommandParser<'a> {
    let mctx = ctx.clone();
    let cn = with_success(seq(b"!more") - space(), move || {
        ctx_command("more", mctx.clone())
    });
    let d = duration(ctx.clone());
    let all = cn + d;
    all.map(|(_, duration)| Command::More(duration))
        .name("more")
}

fn list<'a>(ctx: SharedContext) -> CommandParser<'a> {
    let cn = with_success(seq(b"!ls"), move || ctx_command("ls", ctx.clone()));
    cn.map(|_| Command::List).name("list")
}

fn since<'a>(ctx: SharedContext) -> CommandParser<'a> {
    let mctx = ctx.clone();
    let cn = with_success(seq(b"!since"), move || ctx_command("since", mctx.clone()));
    let t = date(ctx.clone()) | duration(ctx.clone()).map(|d| time::SystemTime::now() - d);
    let all = cn - space() + t;
    all.map(|(_, st)| Command::Since(st)).name("since")
}

fn deadline<'a>(ctx: SharedContext) -> CommandParser<'a> {
    let mctx = ctx.clone();
    let cn = with_success(seq(b"!deadline") - space(), move || {
        ctx_command("deadline", mctx.clone())
    });
    let id = project_ident(ctx.clone()) - space();
    let d = date(ctx.clone());
    let all = cn + id + d;
    all.map(|((_, project_name), d)| Command::Deadline(project_name, d))
        .name("deadline")
}

fn provision<'a>(ctx: SharedContext) -> CommandParser<'a> {
    let mctx = ctx.clone();
    let cn = with_success(seq(b"!provision") - space(), move || {
        ctx_command("provision", mctx.clone())
    });
    let id = project_ident(ctx.clone()) - space();
    let d = duration(ctx.clone());
    let all = cn + id + d;
    all.map(|((_, project_name), d)| Command::Provision(project_name, d))
        .name("provision")
}

fn complete<'a>(ctx: SharedContext) -> CommandParser<'a> {
    let mctx = ctx.clone();
    let cn = with_success(seq(b"!complete") - space(), move || {
        ctx_command("complete", mctx.clone())
    });
    let id = project_ident(ctx.clone()) - space();
    let d = date(ctx.clone()).opt();
    let all = cn + id + d;
    all.map(|((_, project_name), d)| match d {
        Some(d) => Command::Complete(project_name, d),
        None => Command::Complete(project_name, time::SystemTime::now()),
    })
    .name("complete")
}

fn note<'a>(ctx: SharedContext) -> CommandParser<'a> {
    let mctx = ctx.clone();
    let cn = with_success(seq(b"!note") - space(), move || {
        ctx_command("note", mctx.clone())
    });
    let id = project_ident(ctx.clone()) - space();
    let content = string();
    let all = cn + id + content;
    all.map(|((_, project_name), c)| Command::Note(project_name, c))
        .name("note")
}

fn command<'a>(ctx: SharedContext) -> CommandParser<'a> {
    {
        ping(ctx.clone())
            | help(ctx.clone())
            | add(ctx.clone())
            | start(ctx.clone())
            | done(ctx.clone())
            | stop(ctx.clone())
            | list(ctx.clone())
            | digest(ctx.clone())
            | since(ctx.clone())
            | more(ctx.clone())
            | switch(ctx.clone())
            | deadline(ctx.clone())
            | provision(ctx.clone())
            | complete(ctx.clone())
            | note(ctx.clone())
    }
    .name("command")
        - trailing_space()
}

pub fn parse_command<'a>(expr: &'a str) -> Result<Command, ParseCommandError> {
    // let result = command().parse(expr.as_bytes());
    // println!("Parsed \"{}\" {}", expr, result.is_ok());
    // result
    let ctx = new_context();
    match command(ctx.clone()).parse(expr.as_bytes()) {
        Ok(command) => Ok(command),
        Err(_) => {
            let ctx = ctx.borrow();
            match (ctx.command.clone(), ctx.error.clone()) {
                (None, _) => Err(ParseCommandError::NotFound),
                (Some(_), Some(err)) => Err(err),
                (_, _) => Err(ParseCommandError::Mysterious),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::expr::*;

    #[test]
    fn parse_error_is_nice() {
        let expected = String::from("Command does not exists");
        let result = match parse_command("!truc") {
            Ok(_) => String::from("Should not be OK"),
            Err(err) => format!("{}", err),
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn parse_do_ok() {
        let expected: Result<Command, ParseCommandError> = Ok(Command::Do(
            "foo/0".into(),
            "dev".into(),
            time::Duration::from_secs(3 * 60 * 60 + (30 * 60)),
        ));
        assert_eq!(
            parse_command("!do foo/0 dev 3h 30m").is_ok(),
            expected.is_ok()
        );
    }
    #[test]
    fn parse_new_ok() {
        assert_eq!(
            add(new_context()).parse("!new ac/bot".as_bytes()),
            Ok(Command::Add("ac/bot".into(),))
        );
    }
    #[test]
    fn parse_project_ident() {
        assert_eq!(
            project_ident(new_context()).parse("ac/bot".as_bytes()),
            Ok(String::from("ac/bot"))
        );
    }
    #[test]
    fn parse_project_ident_fail() {
        assert_eq!(
            project_ident(new_context()).parse("ac-bot".as_bytes()),
            Err(pom::Error::Custom {
                message: "failed to parse Project identifier".into(),
                position: 0,
                inner: Some(Box::new(pom::Error::Incomplete))
            })
        );
    }
    #[test]
    fn parse_date_iso() {
        assert_eq!(
            date(new_context()).parse("2042-05-29".as_bytes()),
            Ok(st_from_ts(Utc.ymd(2042, 05, 29).and_hms(0, 1, 1).timestamp_millis(),).unwrap())
        );
    }
    #[test]
    fn parse_date_fancy() {
        assert_eq!(
            date(new_context()).parse("29/05/2042".as_bytes()),
            Ok(st_from_ts(Utc.ymd(2042, 05, 29).and_hms(0, 1, 1).timestamp_millis(),).unwrap())
        );
    }
}
