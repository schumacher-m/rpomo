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
    work_count: u8,
    break_count: u8,
    working: bool,
    on_break: bool,
    on_long_break: bool,
}

impl Pomodoro {
    pub fn new() -> Pomodoro {
        Pomodoro {
            start_date_time: "".to_owned(),
            break_date_time: "".to_owned(),
            work_count: 0,
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
                    if self.break_count >= LONG_BREAK_COUNT {
                        duration_diff.num_minutes() >= LONG_BREAK_DURATION as i64
                    } else {
                        duration_diff.num_minutes() >= BREAK_DURATION as i64
                    }
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

        if self.break_count % LONG_BREAK_COUNT == 0 {
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
        self.work_count = self.work_count + 1;
        self.on_break = false;
        self.on_long_break = false;
    }

    pub fn stop(&mut self) {
        self.working = false;
        self.on_break = false;
        self.on_long_break = false;
        self.work_count = 0;
        self.break_count = 0;
        self.start_date_time = "".to_owned();
        self.break_date_time = "".to_owned();
    }

    fn calculate_duration_difference(origin:&String) -> (i64, i64) {
        let utc = DateTime::parse_from_rfc3339(origin).unwrap();
        let derp = Local::now().signed_duration_since(utc);
        (derp.num_minutes(), derp.num_seconds())
    }

    pub fn status(&self) -> String {
        match self {
            &Pomodoro { working: false, on_break: false, on_long_break: false, .. } => {
                "Idle".to_owned()
            },
            &Pomodoro { working: true, on_break: false, on_long_break: false, .. } => {
                let (minutes, _) = Self::calculate_duration_difference(&self.start_date_time);
                format!("Work (#{}): {:01}m/{:01}m", self.work_count, minutes, WORK_DURATION)
            },
            &Pomodoro { working: false, on_break: true, on_long_break: false, .. } => {
                let (minutes, _) = Self::calculate_duration_difference(&self.break_date_time);
                format!("Break (#{}): {:01}m/{:01}m", self.break_count, minutes, BREAK_DURATION)
            },
            &Pomodoro { working: false, on_break: false, on_long_break: true, .. } => {
                let (minutes, _) = Self::calculate_duration_difference(&self.break_date_time);
                format!("Long Break (#{}): {:01}m/{:01}m", self.break_count, minutes, LONG_BREAK_DURATION)
            },
            _ => "???".to_owned()
        }
    }

    pub fn init_from_file() -> Result<Pomodoro, &'static str> {
        let mut s = String::new();
        let file = File::open(Self::default_file_path());

        match file {
            Result::Ok(mut file) => {
                file.read_to_string(&mut s).unwrap();
                let p: Pomodoro = serde_json::from_str(&s).unwrap();
                Ok(p)
            }
            Result::Err(_) => {
                let mut p = Pomodoro::new();
                let _ = p.write_to_file();
                Ok(p)
            }
        }


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
    use chrono::Duration;

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

    #[test]
    fn it_returns_on_exceeding_long_break_time() {
        let mut p = Pomodoro::new();
        let utc: DateTime<Local> = Local::now();
        p.on_break = false;
        p.on_long_break = true;
        p.break_count = LONG_BREAK_COUNT;
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
        assert_eq!(p.work_count, 0);
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
        assert_eq!(p.work_count, 1);
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
        p.start_break();
        assert_eq!(p.break_count, 5);
        assert_eq!(p.on_long_break, false);
        assert_eq!(p.on_break, true);
    }

    #[test]
    fn it_increases_work_count() {
        let mut p = Pomodoro::new();
        p.start_work();
        assert_eq!(p.work_count, 1);
        p.start_work();
        assert_eq!(p.work_count, 2);
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
        assert_eq!(p.status(), "Work (#1): 0m/25m");
        p.start_break();
        assert_eq!(p.status(), "Break (#1): 0m/5m");
        p.start_work();
        assert_eq!(p.status(), "Work (#2): 0m/25m");
        p.start_break();
        assert_eq!(p.status(), "Break (#2): 0m/5m");
        p.start_break();
        assert_eq!(p.status(), "Break (#3): 0m/5m");
        p.start_break();
        assert_eq!(p.status(), "Long Break (#4): 0m/15m");
        p.start_break();
        assert_eq!(p.status(), "Break (#5): 0m/5m");
    }

    #[test]
    fn it_returns_the_default_file_path() {
        // TODO Do we handle the case that a home_dir can't exists?
        let home_dir = env::home_dir().unwrap();
        assert_eq!(Pomodoro::default_file_path(), format!("{}/{}", home_dir.display(), CONFIG_FILENAME));
    }
}
