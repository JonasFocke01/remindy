#[cfg(not(debug_assertions))]
use home::home_dir;

use reminder::Reminder;

pub mod past_event;
pub mod reminder;

pub const ROOT_PATH: &str = ".remindy";
pub const REMINDER_DB_FILE: &str = "reminders.json";
pub const REMINDER_LIBRARY_FILE: &str = "reminders-library.json";
pub const AUDIO_FILE: &str = "ring_tone.mp3";

pub const PORT: u16 = 6969;

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

pub fn map_range(from_range: (f64, f64), to_range: (f64, f64), s: f64) -> f64 {
    to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
}

pub fn get_reminder_by_id(reminders: &mut [Reminder], id: usize) -> Option<&mut Reminder> {
    reminders.iter_mut().find(|reminder| reminder.id() == id)
}
