use std::cmp::Ordering;

use colored::Colorize;
use time::format_description;

use reminder::reminder::Reminder;

pub fn build_reminder_list(reminders: &mut [Reminder], cursor_position: usize) -> String {
    let mut result = String::new();
    let mut displaying_due = false;
    let Ok(time_format) = format_description::parse("[hour]:[minute]:[second]") else {
        return String::new();
    };
    let Some(longest_name_reminder) = reminders.iter().max_by(|a, b| {
        if a.name().len() > b.name().len() {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    }) else {
        return String::new();
    };
    let longest_name_length: usize = longest_name_reminder.name().len();

    for (i, reminder) in reminders.iter_mut().enumerate() {
        while reminder.name().len() < longest_name_length {
            let modified_name = reminder.name();
            reminder.set_name(format!("{modified_name} "));
        }
        let limited_length_reminder_description = if reminder.description().len() > 80 {
            format!("{:.69}\n\r", reminder.description())
        } else {
            String::from(reminder.description())
        };
        let mut whitespace_enhanced_reminder_description = format!(
            "                        {

            }",
            limited_length_reminder_description.replace('\n', "\n                        ")
        );
        for _ in 0..24 {
            whitespace_enhanced_reminder_description.pop();
        }
        let time_left = reminder.remaining_duration();
        if time_left.is_none() && !displaying_due {
            displaying_due = true;
            result.push_str(format!(" {:-<68}\n", "").blue().to_string().as_str());
        }
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
                if let Some(time_left) = time_left {
                    if time_left.whole_days() > 0 {
                        let Ok(finish_time) = reminder.finish_time().format(&time_format) else {
                            return String::new();
                        };
                        let weekday = reminder.finish_time().weekday();
                        format!(
                            "                        {} {}\n\r{}",
                            finish_time,
                            weekday,
                            whitespace_enhanced_reminder_description
                                .replace('\n', "\n\r")
                                .cyan()
                        )
                        .cyan()
                    } else {
                        format!(
                            "{}",
                            whitespace_enhanced_reminder_description
                                .replace('\n', "\n\r")
                                .cyan()
                        )
                        .cyan()
                    }
                } else {
                    format!(
                        "{}",
                        whitespace_enhanced_reminder_description
                            .replace('\n', "\n\r")
                            .cyan()
                    )
                    .cyan()
                }
            )
            .as_str(),
        );
    }
    result
}
