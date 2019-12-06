use humantime;
use pom::parser::*;
use pom::Error;
use pom::Parser;
use serde::{Deserialize, Serialize};
use std::slice;
use std::time;

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum Command {
    Add(String),
    Start(String, time::Duration),
    Stop,
    List,
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
    one_of(b" \t\r\n").repeat(0..).discard()
}

fn string() -> Parser<u8, String> {
    let any = is_a(|_| true);
    let char_string = any.repeat(0..) - (sym(b'.') | end().map(|_| b'.'));
    char_string.convert(|chars| String::from_utf8(chars))
}

fn letter() -> Parser<u8, u8> {
    let lc = one_of(b"abcdefghijklmnopqrstuvwxyz");
    let uc = one_of(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    (lc | uc)
}

fn number() -> Parser<u8, u8> {
    let n = one_of(b"0123456789");
    n
}

fn ident() -> Parser<u8, String> {
    let char_string = (letter() | number() | one_of(b"_-")).repeat(1..);
    char_string.convert(|(chars)| String::from_utf8(chars))
}

fn duration() -> Parser<u8, time::Duration> {
    string().map(|s| match humantime::parse_duration(&s) {
        Ok(d) => d,
        Err(_) => time::Duration::new(0, 0),
    })
}

fn command_name() -> Parser<u8, String> {
    ((seq(b"add") | seq(b"start") | seq(b"stop") | seq(b"ls")) - (space() | end()))
        .convert(|chars| String::from_utf8(chars.to_vec()))
}

type CommandParser = Parser<u8, Command>;

fn add() -> CommandParser {
    let cn = seq(b"!add") - space();
    let id = ident() - space();
    let all = cn + id;
    all.map(|(_, project_name)| Command::Add(project_name))
}

fn start() -> CommandParser {
    let cn = seq(b"!start") - space();
    let id = ident() - space();
    let d = duration();
    let all = cn + id + d;
    all.map(|((_, project_name), duration)| Command::Start(project_name, duration))
}

fn stop() -> CommandParser {
    let cn = seq(b"!stop");
    cn.map(|_| Command::Stop)
}

fn list() -> CommandParser {
    let cn = seq(b"!ls");
    cn.map(|_| Command::List)
}

fn command() -> CommandParser {
    add() | start() | stop() | list()
}

pub fn parse_command<'a>(expr: &'a str) -> Result<Command, Error> {
    let ptr = expr.as_ptr();
    let len = expr.len();
    let result = {
        unsafe {
            let slice = slice::from_raw_parts(ptr, len);
            command().parse(slice)
        }
    };
    result
}

#[cfg(test)]
mod tests {
    use crate::expr::*;
    #[test]
    fn parse_duration() {
        let input = b"4h  30m";
        let output = duration().parse(input);
        let expected = humantime::parse_duration("4h  30m").unwrap();
        assert_eq!(output, Ok(expected));
    }
    #[test]
    fn parse_command_name() {
        let input = b"add foo";
        let output = command_name().parse(input);
        let expected = String::from("add");
        assert_eq!(output, Ok(expected));
    }

    #[test]
    fn parse_command_ok() {
        assert_eq!(
            parse_command("!add  foo-0"),
            Ok(Command::Add("foo-0".into()))
        );
        assert_eq!(
            parse_command("!start foo-0 3h 30m"),
            Ok(Command::Start(
                "foo-0".into(),
                time::Duration::from_secs(3 * 60 * 60 + (30 * 60))
            ))
        );
        assert_eq!(parse_command("!stop"), Ok(Command::Stop));
        assert_eq!(parse_command("!ls"), Ok(Command::List));
    }
}
