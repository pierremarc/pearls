// #[macro_use]
// extern crate tower_web;

use clap::{App, Arg};
use std::path::Path;

mod bot;
// mod http;
mod make;
mod notif;

fn run_bot(
    homeserver: &str,
    username: &str,
    password: &str,
    log: &str,
    http_address: &str,
    base_url: &str,
    statict_dir: &str,
) {
    let log_path = Path::new(log);
    let rx = bot::start_bot(log_path, homeserver, username, password, base_url);

    http::start_http(log_path, http_address, statict_dir);

    for message in rx.iter() {
        println!("{}", message);
    }
}

fn main() {
    let homeserver = Arg::with_name("homeserver")
        .short("h")
        .long("homeserver")
        .value_name("homeserver")
        .help("Home server to connect to")
        .takes_value(true);

    let username = Arg::with_name("username")
        .short("u")
        .long("user")
        .value_name("user")
        .help("User id")
        .takes_value(true);

    let password = Arg::with_name("password")
        .short("p")
        .long("password")
        .value_name("password")
        .help("Password")
        .takes_value(true);

    let log_dir = Arg::with_name("log_dir")
        .short("l")
        .long("log_dir")
        .value_name("log_dir")
        .help("Path to a directory where sqlite files will be stored")
        .takes_value(true);

    let static_dir = Arg::with_name("static_dir")
        .short("s")
        .long("static_dir")
        .value_name("static_dir")
        .help("Path static assets directory")
        .takes_value(true);

    let base_url = Arg::with_name("base_url")
        .short("b")
        .long("base_url")
        .value_name("base_url")
        .help("base URL of the HTTP server")
        .takes_value(true);

    let http_address = Arg::with_name("http_address")
        .short("a")
        .long("http_address")
        .value_name("http_address")
        .help("Socket address")
        .takes_value(true);

    let matches = App::new("Pearls")
        .version("0.1")
        .about("Chat your time")
        .arg(homeserver)
        .arg(username)
        .arg(password)
        .arg(log_dir)
        .arg(http_address)
        .arg(base_url)
        .arg(static_dir)
        .get_matches();

    match (
        matches.value_of("homeserver"),
        matches.value_of("username"),
        matches.value_of("password"),
        matches.value_of("http_address"),
        matches.value_of("base_url"),
        matches.value_of("static_dir"),
    ) {
        (Some(hs), Some(us), Some(pa), Some(ha), Some(bu), Some(sd)) => {
            run_bot(
                hs,
                us,
                pa,
                matches.value_of("log_dir").unwrap_or("."),
                ha,
                bu,
                sd,
            );
        }
        _ => println!("Missing homeserver or username or password"),
    }
}
