#[cfg(feature = "i3")]
use std::process::{Command, Stdio};
use std::{
    cmp::Ordering,
    fs::File,
    io::{BufReader, Write},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crossterm::{
    cursor, execute,
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use reminder::{past_event::PastEvent, reminder::Reminder, root_path, AUDIO_FILE, PORT};

mod status_box;
use rodio::{Decoder, OutputStream, Sink};
use status_box::build_status_box;

mod reminders;
use reminders::build_reminder_list;

mod key_reader;
use key_reader::read_input;

// TODO: DOMAIN wants to be configurable
const DOMAIN: &str = "jonrrrs.duckdns.org";

pub fn main() {
    let mut cursor_position: usize = 0;
    let mut stdout = std::io::stdout();
    let request_client = reqwest::blocking::Client::new();
    let reminders: Arc<Mutex<Vec<Reminder>>> = Arc::new(Mutex::new(vec![]));
    let past_event: Arc<Mutex<PastEvent>> = Arc::new(Mutex::new(PastEvent::None));
    spawn_async_reminder_fetch(Arc::clone(&reminders), Arc::clone(&past_event));
    loop {
        let _trash_bin = disable_raw_mode().is_ok();

        let _trash_bin = enable_raw_mode().is_ok();
        execute!(
            stdout,
            cursor::Hide,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )
        .unwrap();

        if let Ok(past_event) = past_event.lock() {
            if stdout
                .write_all(build_status_box(&past_event).as_bytes())
                .is_err()
            {
                return;
            }
        };

        let mut should_fetch_data = false;
        if let Ok(reminders) = reminders.lock() {
            if stdout
                .write_all(build_reminder_list(&reminders, cursor_position).as_bytes())
                .is_err()
            {
                return;
            }
            if let Some(selected_reminder) = reminders.get(cursor_position) {
                should_fetch_data = read_input(
                    &mut stdout,
                    selected_reminder,
                    reminders.len(),
                    &mut cursor_position,
                    &request_client,
                );
            } else {
                should_fetch_data = read_input(
                    &mut stdout,
                    &Reminder::default(),
                    reminders.len(),
                    &mut cursor_position,
                    &request_client,
                );
            }
        }
        if should_fetch_data {
            fetch_data(&reminders, &past_event);
        }
    }
}

fn spawn_async_reminder_fetch(
    reminders: Arc<Mutex<Vec<Reminder>>>,
    past_event: Arc<Mutex<PastEvent>>,
) {
    thread::spawn(move || loop {
        fetch_data(&reminders, &past_event);
        thread::sleep(Duration::from_secs(5));
    });
}

fn fetch_data(reminders: &Mutex<Vec<Reminder>>, past_event: &Arc<Mutex<PastEvent>>) {
    let request_client = reqwest::blocking::Client::new();
    let mut new_reminders: Vec<Reminder> =
        reqwest::blocking::get(format!("http://{DOMAIN}:{PORT}/reminders"))
            .unwrap()
            .json()
            .unwrap();
    for reminder in &new_reminders {
        if reminder.needs_confirmation() {
            alert_user(reminder);
            request_client
                .put(format!(
                    "http://{DOMAIN}:{PORT}/reminders/{}/confirm",
                    reminder.id()
                ))
                .send()
                .unwrap();
        }
    }
    new_reminders.sort_by(|a, b| {
        if a.finish_time().cmp(&b.finish_time()) == Ordering::Less {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    });
    if let Ok(mut reminders) = reminders.lock() {
        *reminders = new_reminders;
    }

    let new_past_event: PastEvent =
        reqwest::blocking::get(format!("http://{DOMAIN}:{PORT}/past_event"))
            .unwrap()
            .json()
            .unwrap();
    if let Ok(mut past_event) = past_event.lock() {
        *past_event = new_past_event;
    }
}

fn alert_user(reminder: &Reminder) {
    if let Ok((_stream, audio_stream_handle)) = OutputStream::try_default() {
        #[cfg(target_os = "linux")]
        {
            let Ok(file) = File::open(format!("{}/{AUDIO_FILE}", root_path())) else {
                return;
            };
            let audio_buf = BufReader::new(file);
            let Ok(sink) = Sink::try_new(&audio_stream_handle) else {
                return;
            };
            let Ok(audio_source) = Decoder::new(audio_buf) else {
                return;
            };
            sink.append(audio_source);
            sink.set_volume(0.7);

            let _trash_bin = msgbox::create(reminder.name(), "", msgbox::IconType::Info);
        }

        #[cfg(feature = "i3")]
        let _ = Command::new("i3-msg")
            .arg("workspace")
            // TODO: `musik` should be configurable
            .arg("musik")
            .stdout(Stdio::null())
            .spawn();

        #[cfg(target_os = "macos")]
        let _ = Command::new("osascript")
            .arg("-e")
            .arg(format!(
                "display notification \"{}\" sound name \"Bottle\"",
                reminder.name()
            ))
            .stdout(Stdio::null())
            .spawn();
    }
}
