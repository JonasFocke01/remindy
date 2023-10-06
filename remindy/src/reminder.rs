use std::{
    fmt::Display,
    fs::write,
};

        use std::{fs::File, io::BufReader};

        use crossterm::event::read;
        use notify_rust::Notification;
        use rodio::{Decoder, OutputStream, Sink};

use colored::Colorize;
use serde::{Deserialize, Serialize};
use time::{format_description, Duration, OffsetDateTime, UtcOffset};

use crate::map_range;

#[derive(Clone, Serialize, Deserialize)]
pub enum ReminderType {
    Duration,
    Time,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Reminder {
    name: String,
    start_time: OffsetDateTime,
    reminder_type: ReminderType,
    duration: Duration,
    finish_time: OffsetDateTime,
    finish_notifications_send: bool,
    delete_flag: bool,
    restart_flag: bool,
}

impl Reminder {
    pub fn new(
        name: String,
        reminder_type: ReminderType,
        duration: Duration,
        finish_time: OffsetDateTime,
    ) -> Self {
        let start_time = OffsetDateTime::now_utc().to_offset(UtcOffset::from_hms(2, 0, 0).unwrap());
        Self {
            name,
            start_time,
            reminder_type,
            duration,
            finish_time,
            finish_notifications_send: false,
            delete_flag: false,
            restart_flag: false,
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
    pub fn start_time(&self) -> OffsetDateTime {
        self.start_time
    }
    pub fn set_start_time(&mut self, start_time: OffsetDateTime) {
        self.start_time = start_time;
    }
    pub fn reminder_type(&self) -> &ReminderType {
        &self.reminder_type
    }
    pub fn set_reminder_type(&mut self, reminder_type: ReminderType) {
        self.reminder_type = reminder_type;
    }
    pub fn duration(&self) -> Duration {
        self.duration
    }
    pub fn set_duration(&mut self, duration: Duration) {
        self.duration = duration;
    }
    pub fn finish_time(&self) -> OffsetDateTime {
        self.finish_time
    }
    pub fn set_finish_time(&mut self, finish_time: OffsetDateTime) {
        self.finish_time = finish_time;
    }
    pub fn finish_notifications_send(&self) -> bool {
        self.finish_notifications_send
    }
    pub fn set_finish_notifications_send(&mut self, flag: bool) {
        self.finish_notifications_send = flag;
    }
    pub fn delete_flag(&self) -> bool {
        self.delete_flag
    }
    pub fn set_delete_flag(&mut self, flag: bool) {
        self.delete_flag = flag;
    }
    pub fn restart_flag(&self) -> bool {
        self.restart_flag
    }
    pub fn set_restart_flag(&mut self, flag: bool) {
        self.restart_flag = flag;
    }
    #[cfg(not(target_os = "macos"))]
    pub fn play_alert_if_needed(&mut self) {
        // TODO: Make UTC OFFSET a constant

        let now = OffsetDateTime::now_utc().to_offset(UtcOffset::from_hms(2, 0, 0).unwrap());
        let time_left = self.finish_time - now;
        if !time_left.is_positive() && !self.finish_notifications_send {
            Notification::new()
                .summary(self.name.as_str())
                .urgency(notify_rust::Urgency::Critical)
                .sound_name("alarm-clock_elapsed")
                .show()
                .unwrap();

            // sound
            let (_stream, audio_stream_handle) = OutputStream::try_default().unwrap();
            let audio_file = BufReader::new(File::open("song.mp3").unwrap());
            let sink = Sink::try_new(&audio_stream_handle).unwrap();
            let audio_source = Decoder::new(audio_file).unwrap();
            sink.append(audio_source);
            sink.set_volume(0.7);

            let _trash_bin = msgbox::create(self.name.as_str(), "", msgbox::IconType::Info);

            read().unwrap();

            self.finish_notifications_send = true;
        }
    }

    #[cfg(target_os = "macos")]
    fn play_alert_if_needed(&mut self) {
        // TODO: Make UTC OFFSET a constant
        let now = OffsetDateTime::now_utc().to_offset(UtcOffset::from_hms(2, 0, 0).unwrap());
        let time_left = self.finish_time - now;
        if !time_left.is_positive() && !self.finish_notifications_send {
            msgbox::create(self.name.as_str(), "", msgbox::IconType::Info);
            Notification::new()
                .summary(self.name.as_str())
                .show()
                .unwrap();
            self.finish_notifications_send = true;
        }
    }
    pub fn restart(&mut self) {
        let now = OffsetDateTime::now_utc().to_offset(UtcOffset::from_hms(2, 0, 0).unwrap());
        self.start_time = now;
        match self.reminder_type {
            ReminderType::Time => {
                self.finish_time = self.finish_time.replace_date(now.date());
                if self.finish_time < now {
                    self.finish_time += Duration::days(1)
                }
                self.duration = self.finish_time - now;
            }
            ReminderType::Duration => {
                self.finish_time = now + self.duration;
            }
        }
        self.delete_flag = false;
        self.restart_flag = false;
        self.finish_notifications_send = false;
    }
    pub fn snooze(&mut self) {
        let now = OffsetDateTime::now_utc().to_offset(UtcOffset::from_hms(2, 0, 0).unwrap());
        if self.finish_time < now {
            self.finish_time += Duration::minutes(5);
            self.finish_notifications_send = false;
            self.duration += Duration::minutes(5);
            self.finish_notifications_send = false;
        }
    }
    pub fn from_file(filename: &str) -> Option<Vec<Reminder>> {
        if let Ok(reminders_from_file) = std::fs::read_to_string(filename) {
            let reminders_from_file: Result<Vec<Reminder>, _> =
                serde_json::from_str(reminders_from_file.as_str());
            return Some(reminders_from_file.unwrap());
        }
        None
    }
    pub fn to_file(filename: &str, reminders: Vec<Reminder>) {
        let serialized_reminders =
            serde_json::to_string_pretty(&reminders).unwrap();
        write(filename, serialized_reminders).unwrap();
    }
}
impl Display for Reminder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let now = OffsetDateTime::now_utc().to_offset(UtcOffset::from_hms(2, 0, 0).unwrap());
        let time_left = self.finish_time - now;
        let format =
            format_description::parse("[hour]:[minute]:[second] [day].[month].[year]").unwrap();
        if time_left.is_positive() {
            let percent_finished = map_range(
                (
                    self.start_time.unix_timestamp() as f64,
                    self.finish_time.unix_timestamp() as f64,
                ),
                (0., 100.),
                now.unix_timestamp() as f64,
            )
            .round() as usize;
            let mut progressbar = String::new();
            for _ in 0..(percent_finished / 5) {
                progressbar.push('=');
            }
            progressbar.push('>');
            write!(
                f,
                "{:>10} {:0>2}{}{:0>2}{}{:0>2} {}{:<21}{} {}{}{} {} ",
                if self.delete_flag {
                    self.name.clone().bright_red()
                } else if self.restart_flag {
                    self.name.clone().bright_blue()
                } else {
                    self.name.clone().bright_green()
                },
                (time_left.whole_hours() - time_left.whole_days() * 24)
                    .to_string()
                    .bright_red(),
                ":".bright_red(),
                (time_left.whole_minutes() - time_left.whole_hours() * 60)
                    .to_string()
                    .bright_red(),
                ":".bright_red(),
                (time_left.whole_seconds() - time_left.whole_minutes() * 60)
                    .to_string()
                    .bright_red(),
                "[".bright_green(),
                progressbar.bright_red(),
                "]".bright_green(),
                "(".bright_green(),
                self.finish_time.format(&format).unwrap().bright_red(),
                ")".bright_green(),
                if time_left.whole_days() > 0 {
                    format!(
                        "{}{}{}{}{}",
                        "(".bright_green(),
                        "+".bright_red(),
                        time_left.whole_days().to_string().bright_red(),
                        " days".bright_red(),
                        ")".bright_green()
                    )
                } else {
                    "".to_string()
                },
            )
        } else {
            write!(
                f,
                "{:>10} {:0>2}{}{:0>2}{}{:0>2} {}{:<21}{} {}{}{}  ",
                if self.delete_flag {
                    self.name.clone().bright_red()
                } else if self.restart_flag {
                    self.name.clone().bright_blue()
                } else {
                    self.name.clone().green()
                },
                (time_left.whole_hours() - time_left.whole_days() * 24)
                    .clamp(0, i64::MAX)
                    .to_string()
                    .red(),
                ":".red(),
                (time_left.whole_minutes() - time_left.whole_hours() * 60)
                    .clamp(0, i64::MAX)
                    .to_string()
                    .red(),
                ":".red(),
                (time_left.whole_seconds() - time_left.whole_minutes() * 60)
                    .clamp(0, i64::MAX)
                    .to_string()
                    .red(),
                "[".bright_green(),
                "========DONE=========".yellow(),
                "]".bright_green(),
                "(".bright_green(),
                self.finish_time.format(&format).unwrap().bright_red(),
                ")".bright_green(),
            )
        }
    }
}
impl From<ApiReminder> for Reminder {
    fn from(value: ApiReminder) -> Self {
        let now = OffsetDateTime::now_utc().to_offset(UtcOffset::from_hms(2, 0, 0).unwrap());
        Self {
            name: value.name,
            start_time: now,
            duration: now - value.finish_time,
            finish_time: value.finish_time,
            finish_notifications_send: false,
            delete_flag: false,
            restart_flag: false,
            reminder_type: ReminderType::Time,
        }
    }
}

#[derive(Clone, Deserialize)]
pub struct ApiReminder {
    name: String,
    finish_time: OffsetDateTime,
}
