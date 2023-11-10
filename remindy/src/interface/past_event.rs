use std::fmt::Display;

use colored::Colorize;
use time::OffsetDateTime;

use crate::reminder::{Reminder, OFFSET};

pub enum PastEvent {
    ReminderEnded(Reminder),
    ReminderRepeatToggle(Reminder),
    WrongInput,
    TryResetDateReminder(Reminder),
    ReminderCreated(Reminder),
    ReminderEdited(Reminder),
    ReminderDeleted(Reminder),
    ReminderSnooze(Reminder),
    ReminderPause(Reminder),
    None,
}
impl Display for PastEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[allow(clippy::arithmetic_side_effects)]
            PastEvent::ReminderEnded(reminder) => {
                let now = OffsetDateTime::now_utc().to_offset(OFFSET);
                let finished_ago = now - reminder.finish_time();
                write!(
                    f,
                    "{} {} {:0>2}{}{:0>2}{}{:0>2} {}",
                    reminder.name().green(),
                    "ended".green(),
                    (finished_ago.whole_hours() - finished_ago.whole_days() * 24)
                        .to_string()
                        .green(),
                    ":".green(),
                    (finished_ago.whole_minutes() - finished_ago.whole_hours() * 60)
                        .to_string()
                        .green(),
                    ":".green(),
                    (finished_ago.whole_seconds() - finished_ago.whole_minutes() * 60)
                        .to_string()
                        .green(),
                    "ago".green()
                )
            }
            PastEvent::WrongInput => write!(f, "{}", "Wrong input detected".bright_red()),
            PastEvent::TryResetDateReminder(reminder) => write!(
                f,
                "{} {}",
                reminder.name().bright_green(),
                "cannot be restarted (Date Reminder)".bright_red()
            ),
            PastEvent::ReminderCreated(reminder) => write!(
                f,
                "{} {}",
                reminder.name().bright_green(),
                "created".bright_green()
            ),
            PastEvent::ReminderEdited(reminder) => {
                write!(f, "{} {}", reminder.name().blue(), "edited".blue())
            }
            PastEvent::ReminderDeleted(reminder) => {
                write!(f, "{} {}", reminder.name().red(), "deleted".red())
            }
            PastEvent::ReminderSnooze(reminder) => {
                write!(f, "{} {}", reminder.name().blue(), "snoozed".blue())
            }
            PastEvent::ReminderPause(reminder) => {
                write!(f, "{} {}", reminder.name().blue(), "paused/unpaused".blue())
            }
            PastEvent::ReminderRepeatToggle(reminder) => {
                write!(f, "{} {}", reminder.name().blue(), "repeat toggled".blue())
            }
            PastEvent::None => write!(f, "{}", " ".black()),
        }
    }
}
