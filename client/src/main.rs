use std::{
    cmp::Ordering,
    fs::File,
    io::{BufReader, Write},
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

// TODO: IP wants to be configurable
const IP: &str = "192.168.2.95";

pub fn main() {
    let mut cursor_position: usize = 0;
    let mut stdout = std::io::stdout();
    let mut request_client = reqwest::blocking::Client::new();
    loop {
        let _trash_bin = disable_raw_mode().is_ok();
        let mut reminders: Vec<Reminder> =
            reqwest::blocking::get(format!("http://{IP}:{PORT}/reminders"))
                .unwrap()
                .json()
                .unwrap();
        for reminder in &reminders {
            if reminder.needs_confirmation() {
                alert_user(reminder);
                request_client
                    .put(format!(
                        "http://{IP}:{PORT}/reminders/{}/confirm",
                        reminder.id()
                    ))
                    .send()
                    .unwrap();
            }
        }
        reminders.sort_by(|a, b| {
            if a.finish_time().cmp(&b.finish_time()) == Ordering::Less {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        });

        let past_event: PastEvent =
            reqwest::blocking::get(format!("http://{IP}:{PORT}/past_event"))
                .unwrap()
                .json()
                .unwrap();

        let _trash_bin = enable_raw_mode().is_ok();
        execute!(
            stdout,
            cursor::Hide,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )
        .unwrap();
        if stdout
            .write_all(build_status_box(&past_event).as_bytes())
            .is_err()
        {
            return;
        }

        if stdout
            .write_all(build_reminder_list(&reminders, cursor_position).as_bytes())
            .is_err()
        {
            return;
        }
        if let Some(selected_reminder) = reminders.get(cursor_position) {
            read_input(
                &mut stdout,
                selected_reminder,
                reminders.len(),
                &mut cursor_position,
                &mut request_client,
            );
        } else {
            read_input(
                &mut stdout,
                &Reminder::default(),
                reminders.len(),
                &mut cursor_position,
                &mut request_client,
            );
        }
    }
}

fn alert_user(reminder: &Reminder) {
    if let Ok((_stream, audio_stream_handle)) = OutputStream::try_default() {
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

        // This is works only with i3-wm
        // let _ = Command::new("i3-msg")
        //     .arg("workspace")
        //     .arg("musik")
        //     .stdout(Stdio::null())
        //     .spawn();
    }
}
