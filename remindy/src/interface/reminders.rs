use std::sync::{Arc, Mutex};

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
            }
            result.push_str(
                format!(
                    "{}{}{}\n\r",
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
                )
                .as_str(),
            );
        }
    }
    result
}
