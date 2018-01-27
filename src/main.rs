#[macro_use]
extern crate clap;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate chrono;

use clap::{Arg, App};
use serde_json::{Error};
use chrono::prelude::*;
use std::fs::File;
use std::io::prelude::*;
use std::env;

static CONFIG_FILENAME : &'static str = ".rpomo.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct Pomodoro{
    start_date_time: String
}

pub fn read_pomodoro() -> Result<Pomodoro, Error> {
    let mut s = String::new();
    File::open(format!("{}/{}", env::home_dir().unwrap().display(), CONFIG_FILENAME)).unwrap().read_to_string(&mut s).unwrap();
    let p: Pomodoro = serde_json::from_str(&s).unwrap();
    Ok(p)
}

pub fn write_pomodoro(p: &Pomodoro) -> Result<(), std::io::Error> {
    let data = serde_json::to_value(p).unwrap().to_string();
    let mut f = File::create(format!("{}/{}", env::home_dir().unwrap().display(), CONFIG_FILENAME)).expect("Unable to create file");
    f.write_all(data.as_bytes())
}

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
             .takes_value(false))
        .arg(Arg::with_name("stop")
             .long("stop")
             .help("Stop a running timer")
             .conflicts_with("start")
             .takes_value(false))
        .get_matches();

    if matches.is_present("start") {
        let utc: DateTime<Local> = Local::now();
        let p = Pomodoro {
            start_date_time: utc.to_rfc3339()
        };
        let _ = write_pomodoro(&p);
    }

    if matches.is_present("status") {
        let mut p = read_pomodoro().unwrap();
        let utc = DateTime::parse_from_rfc3339(&p.start_date_time).unwrap();
        let derp = Local::now().signed_duration_since(utc);
        let minutes = derp.num_minutes();
        let seconds = derp.num_seconds();
        println!("{:02}:{:02}", minutes, seconds-(minutes*60));
    }

    if matches.is_present("stop") {
        println!("{}", "Stop");
    }
}
