use std::{fmt::Display, fs::write};

use std::{fs::File, io::BufReader};

use crate::interface::past_event::PastEvent;
use crate::{root_path, AUDIO_FILE};

use rodio::{Decoder, OutputStream, Sink};

use colored::Colorize;
use serde::{Deserialize, Serialize};
use time::{format_description, Duration, OffsetDateTime, UtcOffset};

use crate::map_range;

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReminderType {
    Duration,
    Time,
    Date,
    // TODO: Note
}

// TODO: Account for summer/ winter time
pub const OFFSET: UtcOffset = if let Ok(offset) = UtcOffset::from_hms(1, 0, 0) {
    offset
} else {
    panic!("Cant compute UtcOffset")
};

#[allow(clippy::struct_excessive_bools, clippy::struct_field_names)]
#[derive(Clone, Serialize, Deserialize)]
pub struct Reminder {
    name: String,
    description: String,
    start_time: OffsetDateTime,
    reminder_type: ReminderType,
    duration: Duration,
    finish_time: OffsetDateTime,
    finish_notifications_send: bool,
    delete_flag: bool,
    restart_flag: bool,
    paused: bool,
    repeating: bool,
}

impl Reminder {
    pub fn new(
        name: String,
        reminder_type: ReminderType,
        duration: Duration,
        finish_time: OffsetDateTime,
    ) -> Self {
        let start_time = OffsetDateTime::now_utc().to_offset(OFFSET);
        Self {
            name,
            description: String::from("                         "),
            start_time,
            reminder_type,
            duration,
            finish_time,
            finish_notifications_send: false,
            delete_flag: false,
            restart_flag: false,
            paused: false,
            repeating: false,
        }
    }
    #[allow(clippy::arithmetic_side_effects)]
    pub fn from_library(library_reminder: &Self) -> Self {
        let now = OffsetDateTime::now_utc().to_offset(OFFSET);
        if library_reminder.reminder_type == ReminderType::Duration {
            Self {
                name: library_reminder.name().to_string(),
                description: library_reminder.description().to_string(),
                start_time: now,
                reminder_type: library_reminder.reminder_type.clone(),
                duration: library_reminder.duration,
                finish_time: now + library_reminder.duration,
                finish_notifications_send: false,
                delete_flag: false,
                restart_flag: false,
                paused: false,
                repeating: false,
            }
        } else {
            Self {
                name: library_reminder.name().to_string(),
                description: library_reminder.description().to_string(),
                start_time: now,
                reminder_type: library_reminder.reminder_type.clone(),
                duration: library_reminder.finish_time - now,
                finish_time: library_reminder.finish_time.replace_date(now.date()),
                finish_notifications_send: false,
                delete_flag: false,
                restart_flag: false,
                paused: false,
                repeating: false,
            }
        }
    }
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
    pub fn description(&self) -> &str {
        self.description.as_str()
    }
    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }
    pub fn set_reminder_type(&mut self, reminder_type: ReminderType) {
        self.reminder_type = reminder_type;
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
    pub fn repeating(&self) -> bool {
        self.repeating
    }
    pub fn toggle_repeat(&mut self) -> Option<bool> {
        let now = OffsetDateTime::now_utc().to_offset(OFFSET);
        #[allow(clippy::arithmetic_side_effects)]
        let time_left = self.finish_time - now;
        if time_left.is_positive() {
            self.repeating = !self.repeating;
            return Some(self.repeating);
        }
        None
    }
    #[allow(clippy::arithmetic_side_effects)]
    pub fn push_back_end_time_if_paused(&mut self, push_back_amount: Duration) {
        if self.paused {
            self.finish_time += push_back_amount;
        }
    }
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }
    // TODO: declutter this by moving responsibilities into separate funtions
    #[cfg(not(target_os = "macos"))]
    pub fn play_alert_if_needed(&mut self) -> bool {
        use std::process::{Command, Stdio};

        let now = OffsetDateTime::now_utc().to_offset(OFFSET);
        #[allow(clippy::arithmetic_side_effects)]
        let time_left = self.finish_time - now;
        if !time_left.is_positive() && !self.finish_notifications_send {
            self.finish_notifications_send = true;

            // sound
            #[cfg(not(debug_assertions))]
            if let Ok((_stream, audio_stream_handle)) = OutputStream::try_default() {
                let Ok(file) = File::open(format!("{}/{AUDIO_FILE}", root_path())) else {
                    return false;
                };
                let audio_buf = BufReader::new(file);
                let Ok(sink) = Sink::try_new(&audio_stream_handle) else {
                    return false;
                };
                let Ok(audio_source) = Decoder::new(audio_buf) else {
                    return false;
                };
                sink.append(audio_source);
                sink.set_volume(0.7);

                let _trash_bin = msgbox::create(self.name.as_str(), "", msgbox::IconType::Info);

                // This is works only with i3-wm
                let _ = Command::new("i3-msg")
                    .arg("workspace")
                    .arg("musik")
                    .stdout(Stdio::null())
                    .spawn();

                return true;
            }
        }
        false
    }

    #[cfg(target_os = "macos")]
    pub fn play_alert_if_needed(&mut self) -> bool {
        let now = OffsetDateTime::now_utc().to_offset(UtcOffset::from_hms(2, 0, 0).unwrap());
        let time_left = self.finish_time - now;
        if !time_left.is_positive() && !self.finish_notifications_send {
            msgbox::create(self.name.as_str(), "", msgbox::IconType::Info);
            Notification::new()
                .summary(self.name.as_str())
                .show()
                .unwrap();
            self.finish_notifications_send = true;
            return true;
        }
        false
    }
    pub fn restart(&mut self, last_event: &mut PastEvent) {
        let now = OffsetDateTime::now_utc().to_offset(OFFSET);
        self.start_time = now;
        #[allow(clippy::arithmetic_side_effects)]
        match self.reminder_type {
            ReminderType::Time => {
                self.finish_time = self.finish_time.replace_date(now.date());
                while self.finish_time < now {
                    self.finish_time += Duration::days(1);
                }
                self.duration = self.finish_time - now;
            }
            ReminderType::Duration => {
                self.finish_time = now + self.duration;
            }
            ReminderType::Date => {
                *last_event = PastEvent::TryResetDateReminder(self.clone());
            }
        }
        self.delete_flag = false;
        self.restart_flag = false;
        self.finish_notifications_send = false;
    }
    pub fn snooze(&mut self) {
        let now = OffsetDateTime::now_utc().to_offset(OFFSET);
        #[allow(clippy::arithmetic_side_effects)]
        if self.finish_time < now {
            self.finish_time += Duration::minutes(5);
            self.finish_notifications_send = false;
            self.duration += Duration::minutes(5);
            self.finish_notifications_send = false;
        }
    }
    pub fn from_file(filename: &str) -> Option<Vec<Reminder>> {
        if let Ok(reminders_from_file) = std::fs::read_to_string(filename) {
            let Ok(reminders_from_file) =
                serde_json::from_str::<Vec<Reminder>>(reminders_from_file.as_str())
            else {
                return None;
            };
            return Some(reminders_from_file);
        }
        None
    }
    pub fn to_file(filename: &str, reminders: &Vec<Reminder>) {
        let Ok(serialized_reminders) = serde_json::to_string_pretty(reminders) else {
            return;
        };
        let _trash_bin = write(filename, serialized_reminders);
    }
}
impl Display for Reminder {
    #[allow(clippy::arithmetic_side_effects, clippy::too_many_lines)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let now = OffsetDateTime::now_utc().to_offset(OFFSET);
        let time_left = self.finish_time - now;
        let Ok(time_format) = format_description::parse("[hour]:[minute]:[second]") else {
            return Err(std::fmt::Error);
        };
        let Ok(finish_time) = self.finish_time.format(&time_format) else {
            return Err(std::fmt::Error);
        };
        let Ok(date_format) = format_description::parse("[day].[month].[year]") else {
            return Err(std::fmt::Error);
        };
        let Ok(finish_date) = self.finish_time.format(&date_format) else {
            return Err(std::fmt::Error);
        };
        if time_left.is_positive() {
            #[allow(
                clippy::cast_sign_loss,
                clippy::cast_possible_truncation,
                clippy::cast_precision_loss
            )]
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
                "{:>10}{} {} {}{:<21}{} {}",
                self.name.clone().green(),
                if self.restart_flag() {
                    "↻".bright_blue()
                } else if self.delete_flag() {
                    "✗".bright_red()
                } else if self.repeating() {
                    "∞".blue()
                } else {
                    String::from(" ").blue()
                },
                if time_left.whole_days() > 0 {
                    format!(
                        "{:>3}{}",
                        time_left.whole_days().to_string().bright_red(),
                        " days".bright_red(),
                    )
                } else {
                    format!(
                        "{:0>2}{}{:0>2}{}{:0>2}",
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
                    )
                },
                "[".bright_green(),
                if self.paused {
                    progressbar.blue()
                } else {
                    progressbar.bright_red()
                },
                "]".bright_green(),
                if time_left.whole_days() > 0 {
                    finish_date.bright_red()
                } else {
                    format!(" {finish_time} ").bright_red()
                },
            )
        } else {
            write!(
                f,
                "{:>10}{}          {}{:<21}{} {} {}  ",
                self.name.clone().green(),
                if self.repeating() {
                    "∞".blue()
                } else if self.delete_flag() {
                    "✗".bright_red()
                } else if self.restart_flag() {
                    "↻".bright_blue()
                } else {
                    String::from(" ").blue()
                },
                "[".bright_green(),
                "========DONE=========".yellow(),
                "]".bright_green(),
                finish_date.bright_red(),
                finish_time.bright_red(),
            )
        }
    }
}
impl Default for Reminder {
    fn default() -> Self {
        let now = OffsetDateTime::now_utc().to_offset(OFFSET);
        Self {
            name: String::new(),
            description: String::new(),
            start_time: now,
            reminder_type: ReminderType::Time,
            duration: Duration::new(0, 0),
            finish_time: now,
            finish_notifications_send: true,
            delete_flag: false,
            restart_flag: false,
            paused: false,
            repeating: false,
        }
    }
}
impl From<ApiReminder> for Reminder {
    fn from(value: ApiReminder) -> Self {
        let now = OffsetDateTime::now_utc().to_offset(OFFSET);
        Self {
            name: value.name,
            description: value.description,
            start_time: now,
            #[allow(clippy::arithmetic_side_effects)]
            duration: now - value.finish_time,
            finish_time: value.finish_time,
            finish_notifications_send: false,
            delete_flag: false,
            restart_flag: false,
            reminder_type: ReminderType::Time,
            paused: false,
            repeating: false,
        }
    }
}

#[derive(Clone, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct ApiReminder {
    name: String,
    description: String,
    finish_time: OffsetDateTime,
}
