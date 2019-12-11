use clap::{App, Arg};
use std::path::Path;

mod bot;
mod make;
mod notif;

fn run_bot(homeserver: &str, room_id: &str, username: &str, password: &str, log: &str) {
    let log_path = Path::new(log);
    let rx = bot::start_bot(&log_path, homeserver, room_id, username, password);

    for message in rx.iter() {
        println!("{}", message);
    }
}

fn main() {
    let homeserver = Arg::with_name("homeserver")
        .short("h")
        .long("homeserver")
        .value_name("homeserver")
        .help("Host to connect to")
        .takes_value(true);

    let room = Arg::with_name("room")
        .short("r")
        .long("room")
        .value_name("room")
        .help("Room to join")
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

    let log_path = Arg::with_name("log")
        .short("l")
        .long("log")
        .value_name("log")
        .help("Path to log file (will be created if not exist)")
        .takes_value(true);

    let matches = App::new("Pearls")
        .version("0.1")
        .about("Chat your time")
        .arg(homeserver)
        .arg(room)
        .arg(username)
        .arg(password)
        .arg(log_path)
        .get_matches();

    match (
        matches.value_of("homeserver"),
        matches.value_of("room"),
        matches.value_of("username"),
        matches.value_of("password"),
    ) {
        (Some(hs), Some(rs), Some(us), Some(pa)) => {
            run_bot(
                hs,
                rs,
                us,
                pa,
                matches.value_of("log").unwrap_or("pearls.log"),
            );
        }
        _ => println!("Missing homeserver or username or password"),
    }
}
