use std::{
    error::Error,
    fmt::Display,
    process::Command,
};

#[cfg(feature = "colored")]
use colored::Colorize;
use serde::{Deserialize, Serialize};
use time::{format_description, Duration, OffsetDateTime, UtcOffset};

use crate::{map_range, past_event::PastEvent};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimeObject {
    pub reminder_type: ReminderType,
    pub finish_time: OffsetDateTime,
    pub duration: Duration,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReminderType {
    Duration,
    Time,
    Date,
    // TODO: Note
}

#[must_use]
pub fn my_local_offset() -> UtcOffset {
    let Ok(system_offset_command_output) = Command::new("date").arg("+'%:z'").output() else {
        println!("Error: Could not compute local offset");
        return UtcOffset::UTC;
    };
    let system_offset = system_offset_command_output.stdout;
    let Some(sliced_system_offset) = system_offset.get(3) else {
        println!("Error: Could not compute local offset");
        return UtcOffset::UTC;
    };
    let sliced_system_offset = &[*sliced_system_offset];
    let Ok(system_offset_hours) = std::str::from_utf8(sliced_system_offset) else {
        println!("Error: Could not compute local offset");
        return UtcOffset::UTC;
    };
    let Ok(system_offset_hours) = system_offset_hours.parse() else {
        println!("Error: Could not compute local offset");
        return UtcOffset::UTC;
    };
    match UtcOffset::from_hms(system_offset_hours, 0, 0) {
        Ok(offset) => offset,
        Err(e) => {
            println!("Error: Could not compute local offset");
            println!("{:?}", e.source());
            UtcOffset::UTC
        }
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    id: usize,
    name: String,
    description: String,
    start_time: OffsetDateTime,
    reminder_type: ReminderType,
    whole_duration: Duration,
    finish_time: OffsetDateTime,
    needs_confirmation: bool,
    already_confirmed: bool,
    delete_flag: bool,
    restart_flag: bool,
    paused: bool,
    repeating: bool,
    send_e_message: bool,
}

impl Reminder {
    #[must_use]
    pub fn new(
        id: usize,
        name: String,
        reminder_type: ReminderType,
        duration: Duration,
        finish_time: OffsetDateTime,
    ) -> Self {
        let start_time = OffsetDateTime::now_utc().to_offset(my_local_offset());
        Self {
            id,
            name,
            description: String::from("                         "),
            start_time,
            reminder_type,
            whole_duration: duration,
            finish_time,
            needs_confirmation: false,
            already_confirmed: false,
            delete_flag: false,
            restart_flag: false,
            paused: false,
            repeating: false,
            send_e_message: true,
        }
    }
    #[must_use]
    pub fn from_api_reminder(id: usize, value: ApiReminder) -> Self {
        let now = OffsetDateTime::now_utc().to_offset(my_local_offset());
        Self {
            id,
            name: value.name,
            description: value.description,
            start_time: now,
            #[allow(clippy::arithmetic_side_effects)]
            whole_duration: value.finish_time - now,
            finish_time: value.finish_time,
            needs_confirmation: false,
            already_confirmed: false,
            delete_flag: false,
            restart_flag: false,
            reminder_type: value.reminder_type,
            paused: false,
            repeating: false,
            send_e_message: true,
        }
    }
    #[must_use]
    pub fn id(&self) -> usize {
        self.id
    }
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
    #[must_use]
    pub fn description(&self) -> &str {
        self.description.as_str()
    }
    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }
    pub fn set_reminder_type(&mut self, reminder_type: ReminderType) {
        self.reminder_type = reminder_type;
    }
    #[must_use]
    pub fn send_e_message(&self) -> bool {
        self.send_e_message
    }
    pub fn toggle_send_e_message(&mut self) {
        self.send_e_message = !self.send_e_message;
    }
    #[must_use]
    #[allow(clippy::arithmetic_side_effects)]
    pub fn remaining_duration(&self) -> Option<Duration> {
        let now = OffsetDateTime::now_utc().to_offset(my_local_offset());
        let difference = self.finish_time - now;
        if difference.is_positive() {
            Some(difference)
        } else {
            None
        }
    }
    #[must_use]
    #[allow(
        clippy::arithmetic_side_effects,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    pub fn remaining_percent(&self) -> usize {
        let difference = self.finish_time - self.start_time;
        if difference.is_positive() {
            let now = OffsetDateTime::now_utc().to_offset(my_local_offset());
            map_range(
                (
                    self.start_time.unix_timestamp() as f64,
                    self.finish_time.unix_timestamp() as f64,
                ),
                (0., 100.),
                now.unix_timestamp() as f64,
            )
            .round() as usize
        } else {
            100
        }
    }
    pub fn set_whole_duration(&mut self, whole_duration: Duration) {
        self.whole_duration = whole_duration;
        self.already_confirmed = false;
    }
    #[must_use]
    pub fn already_confirmed(&self) -> bool {
        self.already_confirmed
    }
    #[must_use]
    pub fn finish_time(&self) -> OffsetDateTime {
        self.finish_time
    }
    pub fn set_finish_time(&mut self, finish_time: OffsetDateTime) {
        self.already_confirmed = false;
        self.finish_time = finish_time;
    }
    #[must_use]
    pub fn needs_confirmation(&self) -> bool {
        self.needs_confirmation
    }
    pub fn confirm_finish_event(&mut self) {
        self.needs_confirmation = false;
        if self.remaining_duration().is_none() {
            self.already_confirmed = true;
        }
    }
    pub fn request_confirmation(&mut self, last_event: &mut PastEvent) {
        if !self.already_confirmed {
            *last_event = PastEvent::ReminderEnded(self.clone());
            self.needs_confirmation = true;
        }
    }
    #[must_use]
    pub fn delete_flag(&self) -> bool {
        self.delete_flag
    }
    pub fn set_delete_flag(&mut self, flag: bool) {
        self.delete_flag = flag;
    }
    #[must_use]
    pub fn restart_flag(&self) -> bool {
        self.restart_flag
    }
    #[must_use]
    pub fn paused(&self) -> bool {
        self.paused
    }
    pub fn set_restart_flag(&mut self, flag: bool) {
        self.restart_flag = flag;
    }
    #[must_use]
    pub fn repeating(&self) -> bool {
        self.repeating
    }
    pub fn toggle_repeat(&mut self) -> Option<bool> {
        let now = OffsetDateTime::now_utc().to_offset(my_local_offset());
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
    pub fn restart(&mut self) {
        let now = OffsetDateTime::now_utc().to_offset(my_local_offset());
        self.start_time = now;
        #[allow(clippy::arithmetic_side_effects)]
        match self.reminder_type {
            ReminderType::Time => {
                self.finish_time = self.finish_time.replace_date(now.date());
                while self.finish_time < now {
                    self.finish_time += Duration::days(1);
                }
                self.whole_duration = self.finish_time - now;
            }
            ReminderType::Duration => {
                self.finish_time = now + self.whole_duration;
            }
            ReminderType::Date => {}
        }
        self.delete_flag = false;
        self.restart_flag = false;
        self.already_confirmed = false;
    }
    pub fn snooze(&mut self) {
        let now = OffsetDateTime::now_utc().to_offset(my_local_offset());
        #[allow(clippy::arithmetic_side_effects)]
        if self.finish_time < now {
            self.finish_time += Duration::minutes(5);
            self.start_time += Duration::minutes(5);
            self.needs_confirmation = false;
            self.already_confirmed = false;
        }
    }
    #[must_use]
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
}
#[cfg(not(feature = "colored"))]
impl Display for Reminder {
    #[allow(clippy::arithmetic_side_effects, clippy::too_many_lines)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let now = OffsetDateTime::now_utc().to_offset(my_local_offset());
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
            let mut progressbar = String::new();
            for _ in 0..(self.remaining_percent() / 5) {
                progressbar.push('=');
            }
            progressbar.push('>');
            write!(
                f,
                "{}{} {} {}",
                self.name.clone(),
                if self.restart_flag() {
                    "↻"
                } else if self.delete_flag() {
                    "✗"
                } else if self.repeating() {
                    "∞"
                } else {
                    " "
                },
                if time_left.whole_days() > 0 {
                    format!("{:>3}{}", time_left.whole_days().to_string(), " days",)
                } else {
                    format!(
                        "{:0>2}{}{:0>2}{}{:0>2}",
                        (time_left.whole_hours() - time_left.whole_days() * 24).to_string(),
                        ":",
                        (time_left.whole_minutes() - time_left.whole_hours() * 60).to_string(),
                        ":",
                        (time_left.whole_seconds() - time_left.whole_minutes() * 60).to_string()
                    )
                },
                if time_left.whole_days() > 0 {
                    finish_date
                } else {
                    format!(" {finish_time} ")
                },
            )
        } else {
            write!(
                f,
                "{}{} {} {}  ",
                self.name.clone(),
                if self.repeating() {
                    "∞"
                } else if self.delete_flag() {
                    "✗"
                } else if self.restart_flag() {
                    "↻"
                } else {
                    " "
                },
                finish_date,
                finish_time,
            )
        }
    }
}
#[cfg(feature = "colored")]
impl Display for Reminder {
    #[allow(clippy::arithmetic_side_effects, clippy::too_many_lines)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let now = OffsetDateTime::now_utc().to_offset(my_local_offset());
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
            let mut progressbar = String::new();
            for _ in 0..(self.remaining_percent() / 5) {
                progressbar.push('=');
            }
            progressbar.push('>');
            write!(
                f,
                "{:>10}{} {} {}{:<21}{} {} {}",
                if self.repeating() {
                    self.name.clone().green().clear()
                } else {
                    self.name.clone().green()
                },
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
                if self.send_e_message() {
                    "ޔ".blue()
                } else {
                    " ".blue()
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
        let now = OffsetDateTime::now_utc().to_offset(my_local_offset());
        Self {
            id: 0,
            name: String::new(),
            description: String::new(),
            start_time: now,
            reminder_type: ReminderType::Time,
            whole_duration: Duration::new(0, 0),
            finish_time: now,
            needs_confirmation: false,
            already_confirmed: false,
            delete_flag: false,
            restart_flag: false,
            paused: false,
            repeating: false,
            send_e_message: true,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct ApiReminder {
    pub name: String,
    pub description: String,
    pub finish_time: OffsetDateTime,
    pub reminder_type: ReminderType,
}
