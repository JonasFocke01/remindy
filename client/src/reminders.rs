use colored::Colorize;
use time::format_description;

use reminder::reminder::Reminder;

pub fn build_reminder_list(reminders: &[Reminder], cursor_position: usize) -> String {
    let mut result = String::new();
    let mut displaying_due = false;
    let Ok(time_format) = format_description::parse("[hour]:[minute]:[second]") else {
        return String::new();
    };
    for (i, reminder) in reminders.iter().enumerate() {
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
                            reminder.description().replace('\n', "\n\r").cyan()
                        )
                        .cyan()
                    } else {
                        format!("{}", reminder.description().replace('\n', "\n\r").cyan()).cyan()
                    }
                } else {
                    format!("{}", reminder.description().replace('\n', "\n\r").cyan()).cyan()
                }
            )
            .as_str(),
        );
    }
    result
}
