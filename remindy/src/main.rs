// TODO: Clean up code (errorhandling, modules, clippy, ...)
// TODO: Repair github pipeline
// TODO: Propper config in .dotfiles, also, the reminder json should move there
// TODO: Create reminder from library list
// TODO: Add auto-refreshing reminders
// TODO: reminders should be pauseable
// TODO: reminders should autosort (by finish date)

use std::{
    fs::File,
    io::Write,
    sync::{Arc, Mutex},
};

use interface::start_interface;

mod reminder;
use reminder::Reminder;

mod api;
use api::{spawn_api, ApiStatus};

mod interface;

#[tokio::main]
async fn main() {
    let reminders: Arc<Mutex<Vec<Reminder>>> =
        if let Some(reminders) = Reminder::from_file("reminders.json") {
            Arc::new(Mutex::new(reminders))
        } else {
            let mut file = File::create("reminders.json").unwrap();
            file.write_all(b"[]").unwrap();
            Arc::new(Mutex::new(vec![]))
        };

    let api_status: Arc<Mutex<ApiStatus>> = spawn_api(&reminders, 4321);

    start_interface(reminders, api_status);
}

fn map_range(from_range: (f64, f64), to_range: (f64, f64), s: f64) -> f64 {
    to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
}
