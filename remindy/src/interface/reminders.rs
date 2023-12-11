use std::sync::{Arc, Mutex};

use colored::Colorize;

use crate::reminder::Reminder;

use super::past_event::PastEvent;

pub fn build_reminder_list(
    reminders: &Arc<Mutex<Vec<Reminder>>>,
    cursor_position: usize,
    last_event: &mut PastEvent,
) -> String {
    let mut result = String::new();
    if let Ok(mut reminders) = reminders.try_lock() {
        for (i, reminder) in reminders.iter_mut().enumerate() {
            if reminder.play_alert_if_needed() {
                *last_event = PastEvent::ReminderEnded(reminder.clone());
                if reminder.repeating() {
                    reminder.restart(last_event);
                }
            }
            result.push_str(
                format!(
                    "\r{}{}{}\n\r{}",
                    if i == cursor_position {
                        String::from("[")
                    } else {
                        " ".to_string()
                    },
                    reminder,
                    if i == cursor_position {
                        String::from("]")
                    } else {
                        String::new()
                    },
                    reminder.description().replace('\n', "\n\r").cyan()
                )
                .as_str(),
            );
        }
    }
    result
}
