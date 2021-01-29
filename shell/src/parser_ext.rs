use pom::parser::Parser;
use std::cell::RefCell;
use std::fmt;
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum ParseCommandError {
    NotFound,
    Mysterious,
    DateFormat,
    DurationFormat,
    IdentFormat,
    ProjectIdentFormat,
}

impl fmt::Display for ParseCommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "Command does not exists"),
            Self::Mysterious => write!(f, "Something bad happened..."),
            Self::DateFormat => write!(f, "A date was not well encoded"),
            Self::DurationFormat => write!(f, "A duration was not well encoded"),
            Self::IdentFormat => write!(f, "An identifier was not working for me"),
            Self::ProjectIdentFormat => {
                write!(f, "A project identifier was not working for me")
            }
        }
    }
}

pub struct Context {
    pub command: Option<String>,
    pub error: Option<ParseCommandError>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            command: None,
            error: None,
        }
    }

    pub fn set_command(&mut self, name: &str) {
        self.command = Some(String::from(name));
    }

    pub fn set_error(&mut self, err: ParseCommandError) {
        self.error = Some(err);
    }
}

pub type SharedContext = Rc<RefCell<Context>>;

pub fn new_context() -> SharedContext {
    SharedContext::new(RefCell::new(Context::new()))
}

pub fn err_date_format(ctx: SharedContext) {
    let mut ctx = ctx.borrow_mut();
    ctx.set_error(ParseCommandError::DateFormat);
}

pub fn err_duration_format(ctx: SharedContext) {
    let mut ctx = ctx.borrow_mut();
    ctx.set_error(ParseCommandError::DurationFormat)
}

pub fn err_ident(ctx: SharedContext) {
    let mut ctx = ctx.borrow_mut();
    ctx.set_error(ParseCommandError::IdentFormat)
}

pub fn err_project_ident(ctx: SharedContext) {
    let mut ctx = ctx.borrow_mut();
    ctx.set_error(ParseCommandError::ProjectIdentFormat)
}

pub fn ctx_command(name: &str, ctx: SharedContext) {
    let mut ctx = ctx.borrow_mut();
    ctx.set_command(name);
}

pub fn with_success<'a, I, O, S>(parser: Parser<'a, I, O>, on_success: S) -> Parser<'a, I, O>
where
    I: 'a + PartialEq + Debug,
    O: 'a,
    S: 'a + Fn(),
{
    Parser::new(move |input: &'a [I], start: usize| {
        (parser.method)(input, start).map(|(o, s)| {
            on_success();
            (o, s)
        })
    })
}

pub fn with_error<'a, I, O, E>(parser: Parser<'a, I, O>, on_error: E) -> Parser<'a, I, O>
where
    I: 'a + PartialEq + Debug,
    O: 'a,
    E: 'a + Fn(),
{
    Parser::new(move |input: &'a [I], start: usize| {
        (parser.method)(input, start).map_err(|err| {
            on_error();
            err
        })
    })
}
