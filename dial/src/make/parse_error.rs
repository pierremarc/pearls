use html::div;
use shell::parser_ext::ParseCommandError;

fn make_text(err: &ParseCommandError) -> String {
    format!("Oops! {}", err)
}

fn make_html(err: &ParseCommandError) -> String {
    div(format!("Oops! {}", err)).as_string()
}

pub fn parse_error(err: &ParseCommandError) -> Option<(String, String)> {
    Some((make_text(err), make_html(err)))
}
