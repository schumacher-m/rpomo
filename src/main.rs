#[macro_use]
extern crate clap;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate chrono;

pub mod pomodoro;

use clap::{Arg, App};

fn main() {
    let matches = App::new("rpomo")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Rust Pomodoro CLI")
        .arg(Arg::with_name("start")
             .long("start")
             .help("Starts a timer")
             .conflicts_with("stop")
             .takes_value(false))
        .arg(Arg::with_name("status")
             .long("status")
             .help("Status")
             .conflicts_with("start")
             .conflicts_with("stop")
             .conflicts_with("break")
             .takes_value(false))
        .arg(Arg::with_name("stop")
             .long("stop")
             .help("Stop a running timer")
             .conflicts_with("start")
             .takes_value(false))
        .arg(Arg::with_name("break")
             .long("break")
             .help("Starts a break")
             .conflicts_with("start")
             .conflicts_with("stop")
             .takes_value(false))
        .get_matches();

    if matches.is_present("start") {
        let mut p = pomodoro::Pomodoro::new();
        p.start_work();
        let _ = p.write_to_file();
    }

    if matches.is_present("status") {
        match pomodoro::Pomodoro::init_from_file() {
            Result::Ok(mut p) => {
                if p.is_exceeding_work_timer() && p.is_working() {
                    p.start_break();
                    p.write_to_file().expect("Error while writing file!");
                } else if p.is_exceeding_break_timer() && p.is_on_break() {
                    p.start_work();
                    p.write_to_file().expect("Error while writing file!");
                } else {
                    println!("{}", p.status());
                }
            },
            Result::Err(err) => println!("{:?}", err)
        }
    }

    if matches.is_present("break") {
        match pomodoro::Pomodoro::init_from_file() {
            Result::Ok(mut p) => {
                p.start_break();
                let _ = p.write_to_file();
            },
            Result::Err(err) => println!("{:?}", err)
        }
    }

    if matches.is_present("stop") {
        match pomodoro::Pomodoro::init_from_file() {
            Result::Ok(mut p) => {
                p.stop();
                let _ = p.write_to_file();
            },
            Result::Err(err) => println!("{:?}", err)
        }
    }
}
