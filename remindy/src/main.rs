// TODO: Add auto-refreshing reminders

use std::{
    fs::File,
    io::Write,
    sync::{Arc, Mutex},
};

#[cfg(not(debug_assertions))]
use home::home_dir;

use interface::start_interface;

pub const ROOT_PATH: &str = ".remindy";
pub const REMINDER_DB_FILE: &str = "reminders.json";
pub const REMINDER_LIBRARY_FILE: &str = "reminders-library.json";
pub const AUDIO_FILE: &str = "song.mp3";

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
    let reminders: Arc<Mutex<Vec<Reminder>>> = if let Some(reminders) =
        Reminder::from_file(format!("{}/{REMINDER_DB_FILE}", root_path()).as_str())
    {
        Arc::new(Mutex::new(reminders))
    } else {
        if std::fs::create_dir_all(root_path()).is_ok() {
            if let Ok(mut file) = File::create(format!("{}/{REMINDER_DB_FILE}", root_path()).as_str()) {
                let _trash_bin = file.write_all(b"[]");
            }
        }
        Arc::new(Mutex::new(vec![]))
    };

    let api_status: Arc<Mutex<ApiStatus>> = spawn_api(&reminders, 4321);

    start_interface(&reminders, &api_status);
}

fn map_range(from_range: (f64, f64), to_range: (f64, f64), s: f64) -> f64 {
    to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
}

#[cfg(not(debug_assertions))]
pub fn root_path() -> String {
    if let Some(home_dir) = home_dir() {
        if let Some(home_dir) = home_dir.as_os_str().to_str() {
            return format!("{home_dir}/{ROOT_PATH}");
        }
    }
    String::new()
}

#[cfg(debug_assertions)]
pub fn root_path() -> String {
    "dbg_db".to_string()
}
