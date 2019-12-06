use clap::{App, Arg};
use shell::store::{Record, SharedStore, Store};
use std::path::Path;
use std::sync::Arc;

mod bot;

fn run_bot(homeserver: &str, username: &str, password: &str, log: &str) {
    let log_path = Path::new(log);

    let store = Store::new(&log_path);

    let rx = bot::start_bot(store.clone(), homeserver, username, password);

    for record in rx.iter() {
        match store.try_write() {
            Err(_) => {
                println!("Failed to lock for write");
            }
            Ok(lock) => {
                // lock.get_mut();
            }
        }
    }
}

fn main() {
    let homeserver = Arg::with_name("homeserver")
        .short("h")
        .long("homeserver")
        .value_name("homeserver")
        .help("Host to connect to")
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

    let log_path = Arg::with_name("log file")
        .short("l")
        .long("log")
        .value_name("log")
        .help("Path to log file (will be created if not exist)")
        .takes_value(true);

    let matches = App::new("Pearls")
        .version("0.1")
        .about("Chat your time")
        .arg(homeserver)
        .arg(username)
        .arg(password)
        .arg(log_path)
        .get_matches();

    match (
        matches.value_of("homeserver"),
        matches.value_of("username"),
        matches.value_of("password"),
    ) {
        (Some(hs), Some(us), Some(pa)) => {
            run_bot(hs, us, pa, matches.value_of("log").unwrap_or("pearls.log"));
        }
        _ => println!("Missing homeserver or username or password"),
    }
}
