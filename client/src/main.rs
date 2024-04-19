#[cfg(feature = "music")]
use std::fs::File;
#[cfg(feature = "music")]
use std::io::BufReader;
#[cfg(any(feature = "i3", target_os = "macos"))]
use std::process::{Command, Stdio};
use std::{
    cmp::Ordering,
    io::Write,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use config::Config;
use crossterm::{
    cursor, execute,
    terminal::{self, enable_raw_mode},
};
use reminder::{past_event::PastEvent, reminder::Reminder};
#[cfg(feature = "music")]
use reminder::{root_path, AUDIO_FILE};

mod status_box;
use rodio::OutputStream;
#[cfg(feature = "music")]
use rodio::{Decoder, Sink};
use status_box::build_status_box;

mod reminders;
use reminders::build_reminder_list;

mod key_reader;
use key_reader::read_input;

pub fn main() {
    let config = Arc::new(Mutex::new(Config::new()));
    let mut cursor_position: usize = 0;
    let mut stdout = std::io::stdout();
    let request_client = reqwest::blocking::Client::new();
    let reminders: Arc<Mutex<Vec<Reminder>>> = Arc::new(Mutex::new(vec![]));
    let past_event: Arc<Mutex<PastEvent>> = Arc::new(Mutex::new(PastEvent::None));
    spawn_async_reminder_fetch(
        Arc::clone(&reminders),
        Arc::clone(&past_event),
        Arc::clone(&config),
    );
    loop {
        let Ok(locked_past_event) = past_event.lock() else {
            return;
        };
        let status_box = build_status_box(locked_past_event.clone());
        let status_box = status_box.as_bytes();
        drop(locked_past_event);

        let mut should_fetch_data = false;
        if let Ok(mut reminders) = reminders.lock() {
            let reminder_list = build_reminder_list(&mut reminders, cursor_position);
            let reminder_list = reminder_list.as_bytes();
            let _trash_bin = enable_raw_mode().is_ok();
            execute!(
                stdout,
                cursor::Hide,
                terminal::Clear(terminal::ClearType::All),
                cursor::MoveTo(0, 0)
            )
            .unwrap();
            if stdout.write_all(status_box).is_err() {
                return;
            }
            if stdout.write_all(reminder_list).is_err() {
                return;
            }
            if let Ok(config) = config.lock() {
                if let Some(selected_reminder) = reminders.get(cursor_position) {
                    should_fetch_data = read_input(
                        &mut stdout,
                        selected_reminder,
                        &reminders,
                        reminders.len(),
                        &mut cursor_position,
                        &request_client,
                        &config,
                    );
                } else {
                    should_fetch_data = read_input(
                        &mut stdout,
                        &Reminder::default(),
                        &reminders,
                        reminders.len(),
                        &mut cursor_position,
                        &request_client,
                        &config,
                    );
                }
            };
        }
        if should_fetch_data {
            fetch_data(&reminders, &past_event, &config);
        }
    }
}

fn spawn_async_reminder_fetch(
    reminders: Arc<Mutex<Vec<Reminder>>>,
    past_event: Arc<Mutex<PastEvent>>,
    config: Arc<Mutex<Config>>,
) {
    thread::spawn(move || loop {
        fetch_data(&reminders, &past_event, &config);
        thread::sleep(Duration::from_secs(5));
    });
}

fn fetch_data(
    reminders: &Mutex<Vec<Reminder>>,
    past_event: &Arc<Mutex<PastEvent>>,
    config: &Arc<Mutex<Config>>,
) {
    let config: Config = if let Ok(config) = config.lock() {
        config.clone()
    } else {
        return;
    };
    let request_client = reqwest::blocking::Client::new();
    let mut new_reminders: Vec<Reminder> = vec![];
    if let Ok(response) = reqwest::blocking::get(format!(
        "http://{}:{}/reminders",
        config.network().remote_ip(),
        config.network().port()
    )) {
        if let Ok(data) = response.json() {
            new_reminders = data;
        }
    }
    for reminder in &new_reminders {
        if reminder.needs_confirmation() {
            alert_user(reminder);
            let _ = request_client
                .put(format!(
                    "http://{}:{}/reminders/{}/confirm",
                    config.network().remote_ip(),
                    config.network().port(),
                    reminder.id()
                ))
                .send();
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

    let Ok(response) = reqwest::blocking::get(format!(
        "http://{}:{}/past_event",
        config.network().remote_ip(),
        config.network().port()
    )) else {
        return;
    };
    let Ok(new_past_event) = response.json() else {
        return;
    };
    if let Ok(mut past_event) = past_event.lock() {
        *past_event = new_past_event;
    }
}

fn alert_user(reminder: &Reminder) {
    #[cfg(not(any(feature = "music", feature = "i3", feature = "msg_box")))]
    let _ = reminder;
    if let Ok((_stream, audio_stream_handle)) = OutputStream::try_default() {
        #[cfg(not(any(feature = "music", feature = "i3", feature = "msg_box")))]
        let _ = audio_stream_handle;
        #[cfg(feature = "music")]
        {
            let Ok(file) = File::open(format!(
                "{}/{AUDIO_FILE}",
                format!("{:?}", root_path().unwrap()).replace('"', "")
            )) else {
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
        }

        #[cfg(feature = "msg_box")]
        let _trash_bin = msgbox::create(reminder.name(), "", msgbox::IconType::Info);

        #[cfg(feature = "i3")]
        let _ = Command::new("i3-msg")
            .arg("workspace")
            // TODO: `musik` should be configurable
            .arg("musik")
            .stdout(Stdio::null())
            .spawn();

        #[cfg(all(target_os = "macos", feature = "msg_box"))]
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
