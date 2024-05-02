#[cfg(not(debug_assertions))]
use json_store_rs::home_dir;
use reminder::Reminder;
use std::path::PathBuf;

pub mod past_event;
pub mod reminder;

pub const ROOT_PATH: &str = ".remindy";
pub const REMINDER_DB_FILE: &str = "reminders.json";
pub const REMINDER_LIBRARY_FILE: &str = "reminders-library.json";
pub const AUDIO_FILE: &str = "ring_tone.mp3";

#[cfg(not(debug_assertions))]
pub fn root_path() -> Result<PathBuf, ()> {
    if let Ok(mut home_dir) = home_dir() {
        home_dir.push(ROOT_PATH);
        return Ok(home_dir);
    }

    Err(())
}

#[cfg(debug_assertions)]
#[allow(clippy::result_unit_err)]
/// # Errors
///
/// Only for compatability reasons, will never fail in debug mode
pub fn root_path() -> Result<PathBuf, ()> {
    Ok(PathBuf::from("dbg_db"))
}

#[must_use]
pub fn map_range(from_range: (f64, f64), to_range: (f64, f64), s: f64) -> f64 {
    to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
}

pub fn get_reminder_by_id(reminders: &mut [Reminder], id: usize) -> Option<&mut Reminder> {
    reminders.iter_mut().find(|reminder| reminder.id() == id)
}
