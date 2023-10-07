// TODO: Propper config in .dotfiles, also, the reminder json should move there
// TODO: Add auto-refreshing reminders

use std::{
    fs::File,
    io::Write,
    sync::{Arc, Mutex},
};

use interface::start_interface;

#[warn(
    clippy::pedantic,
    clippy::arithmetic_side_effects,
    clippy::clone_on_ref_ptr,
    clippy::expect_used,
    clippy::float_cmp_const,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::string_add,
    clippy::string_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::unwrap_used,
    clippy::wildcard_enum_match_arm
)]
mod reminder;
use reminder::Reminder;

#[warn(
    clippy::pedantic,
    clippy::arithmetic_side_effects,
    clippy::clone_on_ref_ptr,
    clippy::expect_used,
    clippy::float_cmp_const,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::string_add,
    clippy::string_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::unwrap_used,
    clippy::wildcard_enum_match_arm
)]
mod api;
use api::{spawn_api, ApiStatus};

#[warn(
    clippy::pedantic,
    clippy::arithmetic_side_effects,
    clippy::clone_on_ref_ptr,
    clippy::expect_used,
    clippy::float_cmp_const,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::string_add,
    clippy::string_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::unwrap_used,
    clippy::wildcard_enum_match_arm
)]
mod interface;

#[warn(
    clippy::pedantic,
    clippy::arithmetic_side_effects,
    clippy::clone_on_ref_ptr,
    clippy::expect_used,
    clippy::float_cmp_const,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::string_add,
    clippy::string_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::unwrap_used,
    clippy::wildcard_enum_match_arm
)]
#[tokio::main]
async fn main() {
    let reminders: Arc<Mutex<Vec<Reminder>>> =
        if let Some(reminders) = Reminder::from_file("reminders.json") {
            Arc::new(Mutex::new(reminders))
        } else {
            if let Ok(mut file) = File::create("reminders.json") {
                let _trash_bin = file.write_all(b"[]");
            }
            Arc::new(Mutex::new(vec![]))
        };

    let api_status: Arc<Mutex<ApiStatus>> = spawn_api(&reminders, 4321);

    start_interface(&reminders, &api_status);
}

fn map_range(from_range: (f64, f64), to_range: (f64, f64), s: f64) -> f64 {
    to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
}
