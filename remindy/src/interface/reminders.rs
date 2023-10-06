use std::sync::{Arc, Mutex};

use crate::reminder::Reminder;

pub fn build_reminder_list(
    reminders: &Arc<Mutex<Vec<Reminder>>>,
    cursor_position: usize,
) -> String {
    let mut result = String::new();
    if let Ok(mut reminders) = reminders.try_lock() {
        for (i, reminder) in reminders.iter_mut().enumerate() {
            reminder.play_alert_if_needed();
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
                .as_str()
            );
        }
    }
    result
}
