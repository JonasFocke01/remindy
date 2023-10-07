use std::fmt::Display;

use colored::Colorize;

use crate::reminder::Reminder;

pub enum PastEvent {
    ReminderEnded(Reminder),
    WrongInput,
    ReminderCreated(Reminder),
    ReminderEdited(Reminder),
    ReminderDeleted(Reminder),
    ReminderSnooze(Reminder),
    None,
}
impl Display for PastEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PastEvent::ReminderEnded(reminder) => write!(f, "{} {}", reminder.name().green(), "ended".green()),
            PastEvent::WrongInput => write!(f, "{}", "Wrong input detected".bright_red()),
            PastEvent::ReminderCreated(reminder) => write!(f, "{} {}", reminder.name().bright_green(), "created".bright_green()),
            PastEvent::ReminderEdited(reminder) => write!(f, "{} {}", reminder.name().blue(), "edited".blue()),
            PastEvent::ReminderDeleted(reminder) => write!(f, "{} {}", reminder.name().red(), "deleted".red()),
            PastEvent::ReminderSnooze(reminder) => write!(f, "{} {}", reminder.name().blue(), "snoozed".blue()),
            PastEvent::None => write!(f, "{}", " ".black()),
        }
    }
}
