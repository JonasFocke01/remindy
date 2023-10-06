use std::sync::{Arc, Mutex};

use crate::reminder::Reminder;

pub fn build_reminder_list(reminders: &Arc<Mutex<Vec<Reminder>>>, cursor_position: usize) -> String {
    let mut result = String::new();
    if let Ok(mut reminders) = reminders.try_lock() {
        for (i, reminder) in reminders.iter_mut().enumerate() {
            result.push_str(format!("{}\n\r", reminder.display(i == cursor_position)).as_str());
        }
    }
    result
}
