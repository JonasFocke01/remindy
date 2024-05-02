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
        "<span color='#50FF30'>{}</span>  <span color='#00BBBB'>{}</span>  <span color='#50FF00'>{}</span>  <span color='#00BBBB'>{}</span>  <span color='#50FF30'>{}</span>  <span color='#00BBBB'>{}</span>  <span color='#50FF30'>{}</span>",
        reminders.next().expect("Could not read Reminders"),
        reminders.next().expect("Could not read Reminders"),
        reminders.next().expect("Could not read Reminders"),
        reminders.next().expect("Could not read Reminders"),
        reminders.next().expect("Could not read Reminders"),
        reminders.next().expect("Could not read Reminders"),
        reminders.next().expect("Could not read Reminders")
    );
}
