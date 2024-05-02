use std::cmp::Ordering;

use config::Config;
use reminder::reminder::Reminder;

#[allow(clippy::expect_used)]
fn main() {
    let config = Config::new();
    let mut reminders: Vec<Reminder> = vec![];
    if let Ok(response) = reqwest::blocking::get(format!(
        "http://{}:{}/reminders",
        config.network().remote_ip(),
        config.network().port()
    )) {
        if let Ok(data) = response.json() {
            reminders = data;
        }
    }
    reminders.sort_by(|a, b| {
        if a.finish_time().cmp(&b.finish_time()) == Ordering::Less {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    });
    let reminders = reminders
        .iter()
        .filter(|reminder| reminder.remaining_duration().is_some());

    let reminders = reminders.rev();
    let mut reminders = reminders.take(7);

    print!(
        "<span color='#50FF30'>{}</span>  <span color='#00BBBB'>{}</span>  <span color='#50FF30'>{}</span>  <span color='#00BBBB'>{}</span>  <span color='#50FF30'>{}</span>  <span color='#00BBBB'>{}</span>  <span color='#50FF30'>{}</span>",

        format_reminder(reminders.next().expect("Could not read Reminders")),
        format_reminder(reminders.next().expect("Could not read Reminders")),
        format_reminder(reminders.next().expect("Could not read Reminders")),
        format_reminder(reminders.next().expect("Could not read Reminders")),
        format_reminder(reminders.next().expect("Could not read Reminders")),
        format_reminder(reminders.next().expect("Could not read Reminders")),
        format_reminder(reminders.next().expect("Could not read Reminders"))
    );
}

#[allow(clippy::arithmetic_side_effects)]
fn format_reminder(reminder: &Reminder) -> String {
    if let Some(time_left) = reminder.remaining_duration() {
        format!(
            "{} {:0>2}{}{:0>2}{}{:0>2}",
            reminder.name(),
            (time_left.whole_hours() - time_left.whole_days() * 24).to_string(),
            ":",
            (time_left.whole_minutes() - time_left.whole_hours() * 60).to_string(),
            ":",
            (time_left.whole_seconds() - time_left.whole_minutes() * 60).to_string()
        )
    } else {
        String::new()
    }
}
