use serde_json::{Error};
use chrono::Duration;
use chrono::prelude::*;
use std::fs::File;
use std::io::prelude::*;
use std::env;
use std;
use serde_json;

static CONFIG_FILENAME : &'static str = ".rpomo.json";
const WORK_DURATION: u8 = 25;
const BREAK_DURATION: u8 = 5;
const LONG_BREAK_DURATION: u8 = 15;
const LONG_BREAK_COUNT: u8 = 4;

#[derive(Serialize, Deserialize, Debug)]
pub struct Pomodoro{
    start_date_time: String,
    break_date_time: String,
    break_count: u8,
    working: bool,
    on_break: bool,
    on_long_break: bool
}

impl Pomodoro {
    pub fn new() -> Pomodoro {
        let utc: DateTime<Local> = Local::now();
        Pomodoro {
            start_date_time: "".to_owned(),
            break_date_time: "".to_owned(),
            break_count: 0,
            working: false,
            on_break: false,
            on_long_break: false
        }
    }

    pub fn is_exceeding_work_timer(&mut self) -> bool {
        match DateTime::parse_from_rfc3339(&self.start_date_time) {
            Result::Ok(utc) => {
                let duration_diff = Local::now().signed_duration_since(utc);
                if self.is_working() {
                    duration_diff.num_minutes() >= WORK_DURATION as i64
                } else {
                    false
                }
            }
            _ => false
        }
    }

    pub fn is_exceeding_break_timer(&mut self) -> bool {
        match DateTime::parse_from_rfc3339(&self.break_date_time) {
            Result::Ok(utc) => {
                let duration_diff = Local::now().signed_duration_since(utc);
                if self.is_on_break() {
                    duration_diff.num_minutes() >= BREAK_DURATION as i64
                } else {
                    false
                }
            }
            _ => false
        }
    }

    pub fn is_working(&mut self) -> bool {
        self.working
    }

    pub fn is_on_break(&mut self) -> bool {
        self.on_break || self.on_long_break
    }

    pub fn start_break(&mut self) {
        let utc: DateTime<Local> = Local::now();
        self.working = false;
        self.break_date_time = utc.to_rfc3339();
        self.break_count = self.break_count + 1;

        if self.break_count == LONG_BREAK_COUNT {
            self.on_long_break = true;
            self.on_break = false;
        } else {
            self.on_break = true;
            self.on_long_break = false;
        }
    }

    pub fn start_work(&mut self) {
        let utc: DateTime<Local> = Local::now();
        self.working = true;
        self.start_date_time = utc.to_rfc3339();
        self.on_break = false;
        self.on_long_break = false;
    }

    pub fn stop(&mut self) {
        self.working = false;
        self.on_break = false;
        self.on_long_break = false;
        self.break_count = 0;
        self.start_date_time = "".to_owned();
        self.break_date_time = "".to_owned();
    }

    // pub fn is_exceeding_long_break_timer(&mut self) -> bool {
    //     let utc = DateTime::parse_from_rfc3339(&self.break_date_time).unwrap();
    //     let duration_diff = Local::now().signed_duration_since(utc);
    //     // TODO Check for break
    //     duration_diff.num_minutes() >= LONG_BREAK_DURATION as i64
    // }

    pub fn status(&mut self) -> String {
        if !self.working && !self.on_break && !self.on_long_break {
            "Idle".to_owned()
        } else if self.working && !self.on_break && !self.on_long_break {
            let utc = DateTime::parse_from_rfc3339(&self.start_date_time).unwrap();
            let derp = Local::now().signed_duration_since(utc);
            let minutes = derp.num_minutes();
            let seconds = derp.num_seconds();
            format!("Working: {:02}:{:02}", minutes, seconds-(minutes*60))
        } else if !self.working && (self.on_break || self.on_long_break) {
            let utc = DateTime::parse_from_rfc3339(&self.break_date_time).unwrap();
            let derp = Local::now().signed_duration_since(utc);
            let minutes = derp.num_minutes();
            let seconds = derp.num_seconds();
            format!("Break (#{}): {:02}:{:02}", self.break_count, minutes, seconds-(minutes*60))
        } else {
            "???".to_owned()
        }
    }

    pub fn init_from_file() -> Result<Pomodoro, Error> {
        let mut s = String::new();
        let mut file = File::open(Self::default_file_path()).expect("File not found!");
        file.read_to_string(&mut s).unwrap();
        let p: Pomodoro = serde_json::from_str(&s).unwrap();
        Ok(p)
    }

    pub fn write_to_file(&mut self) -> Result<(), std::io::Error> {
        let data = serde_json::to_value(self).unwrap().to_string();
        let mut f = File::create(Self::default_file_path()).expect("Unable to create file");
        f.write_all(data.as_bytes())
    }

    fn default_file_path() -> String {
        format!("{}/{}", env::home_dir().unwrap().display(), CONFIG_FILENAME)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_returns_on_exceeding_work_time() {
        let mut p = Pomodoro::new();
        let utc: DateTime<Local> = Local::now();
        p.working = true;
        p.start_date_time = (utc - Duration::minutes(WORK_DURATION as i64 + 1)).to_rfc3339();
        assert_eq!(p.is_exceeding_work_timer(), true);
    }

    #[test]
    fn it_returns_false_if_not_working() {
        let mut p = Pomodoro::new();
        let utc: DateTime<Local> = Local::now();
        p.working = false;
        p.start_date_time = (utc - Duration::minutes(WORK_DURATION as i64 + 1)).to_rfc3339();
        assert_eq!(p.is_exceeding_work_timer(), false);
    }

    #[test]
    fn it_returns_on_exceeding_break_time() {
        let mut p = Pomodoro::new();
        let utc: DateTime<Local> = Local::now();
        p.on_break = true;
        p.break_date_time = (utc - Duration::minutes(BREAK_DURATION as i64 + 1)).to_rfc3339();
        assert_eq!(p.is_exceeding_break_timer(), true);
    }

    #[test]
    fn it_returns_false_if_not_break_is_active() {
        let mut p = Pomodoro::new();
        let utc: DateTime<Local> = Local::now();
        p.on_break = false;
        p.break_date_time = (utc - Duration::minutes(BREAK_DURATION as i64 + 1)).to_rfc3339();
        assert_eq!(p.is_exceeding_break_timer(), false);
    }

    fn it_returns_on_exceeding_long_break_time() {
        let mut p = Pomodoro::new();
        let utc: DateTime<Local> = Local::now();
        p.on_long_break = true;
        p.break_date_time = (utc - Duration::minutes(BREAK_DURATION as i64 + 1)).to_rfc3339();
        assert_eq!(p.is_exceeding_break_timer(), false);
        p.break_date_time = (utc - Duration::minutes(LONG_BREAK_DURATION as i64 + 1)).to_rfc3339();
        assert_eq!(p.is_exceeding_break_timer(), true);
    }

    #[test]
    fn it_can_handle_empty_break_date_time() {
        let mut p = Pomodoro::new();
        p.break_date_time = "".to_owned();
        assert_eq!(p.is_exceeding_break_timer(), false);
    }

    #[test]
    fn it_can_handle_empty_work_date_time() {
        let mut p = Pomodoro::new();
        p.start_date_time = "".to_owned();
        assert_eq!(p.is_exceeding_work_timer(), false);
    }

    #[test]
    fn it_returns_state() {
        let mut p = Pomodoro::new();
        p.start_work();
        assert_eq!(p.is_working(), true);
        assert_eq!(p.is_on_break(), false);
        p.start_break();
        assert_eq!(p.is_working(), false);
        assert_eq!(p.is_on_break(), true);
    }

    #[test]
    fn it_triggers_stop() {
        let mut p = Pomodoro::new();
        p.stop();
        assert_eq!(p.working, false);
        assert_eq!(p.on_break, false);
        assert_eq!(p.on_long_break, false);
        assert_eq!(p.break_count, 0);
        assert!(p.start_date_time.is_empty());
        assert!(p.break_date_time.is_empty());
    }

    #[test]
    fn it_triggers_start() {
        let mut p = Pomodoro::new();
        p.start_work();
        assert_eq!(p.working, true);
        assert_eq!(p.on_break, false);
        assert_eq!(p.on_long_break, false);
        assert!(!p.start_date_time.is_empty());
    }

    #[test]
    fn it_triggers_a_long_break() {
        let mut p = Pomodoro::new();
        p.start_work();
        p.start_break();
        p.start_break();
        p.start_break();
        p.start_break();
        assert_eq!(p.break_count, 4);
        assert_eq!(p.on_long_break, true);
        assert_eq!(p.on_break, false);
    }

    #[test]
    fn it_triggers_a_break() {
        let mut p = Pomodoro::new();
        p.start_work();
        p.start_break();
        assert_eq!(p.working, false);
        assert_eq!(p.on_break, true);
        assert_eq!(p.break_count, 1);
        assert_eq!(p.on_long_break, false);
        assert!(!p.break_date_time.is_empty());
    }

    #[test]
    fn it_returns_a_status_string() {
        let mut p = Pomodoro::new();
        assert_eq!(p.status(), "Idle");

        p.start_work();
        assert_eq!(p.status(), "Working: 00:00");

        p.start_break();
        assert_eq!(p.status(), "Break (#1): 00:00");

        p.start_work();
        assert_eq!(p.status(), "Working: 00:00");

        p.start_break();
        assert_eq!(p.status(), "Break (#2): 00:00");

    }

    #[test]
    fn it_returns_the_default_file_path() {
        // TODO Do we handle the case that a home_dir can't exists?
        let home_dir = env::home_dir().unwrap();
        assert_eq!(Pomodoro::default_file_path(), format!("{}/{}", home_dir.display(), CONFIG_FILENAME));
    }
}
