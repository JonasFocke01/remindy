use std::sync::{Arc, Mutex};

use colored::Colorize;
use time::{format_description, OffsetDateTime};

use crate::reminder::{Reminder, OFFSET};

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
            let now = OffsetDateTime::now_utc().to_offset(OFFSET);
            #[allow(clippy::arithmetic_side_effects)]
            let time_left = reminder.finish_time() - now;
            let Ok(time_format) = format_description::parse("[hour]:[minute]:[second]") else {
                return String::new();
            };
            result.push_str(
                format!(
                    "\r {}{}{}\n\r{}",
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
                    if time_left.whole_days() > 0 {
                        let Ok(finish_time) = reminder.finish_time().format(&time_format) else {
                            return String::from("kaak");
                        };
                        format!(
                            "                        {}\n\r{}",
                            finish_time,
                            reminder.description().replace('\n', "\n\r").cyan()
                        )
                        .cyan()
                    } else {
                        format!("{}", reminder.description().replace('\n', "\n\r").cyan()).cyan()
                    }
                )
                .as_str(),
            );
        }
    }
    result
}
