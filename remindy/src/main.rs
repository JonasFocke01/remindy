// TODO: Clean up code
// TODO: Increase clippy protection
// TODO: Prettify console ouput
// TODO: Play sound on reminder end
// TODO: Make desktop notifications more noticable
// TODO: Save reminder on close, and load on start in json file
// TODO: Repair github pipeline

use std::{
    fmt::Display,
    io::Write,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex},
};

use axum::{
    extract::{rejection::JsonRejection, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use crossterm::{
    cursor,
    event::KeyModifiers,
    execute,
    terminal::{self, disable_raw_mode},
};
use crossterm::{
    event::{poll, read, Event, KeyCode},
    terminal::enable_raw_mode,
};

use duration_string::DurationString;
use notify_rust::Notification;
use serde::{Deserialize, Serialize};

use time::{format_description, Duration, OffsetDateTime, Time, UtcOffset};

#[derive(Clone, Serialize, Deserialize)]
enum ReminderType {
    Duration,
    Time,
}

#[derive(Clone, Serialize, Deserialize)]
struct Reminder {
    name: String,
    start_time: OffsetDateTime,
    reminder_type: ReminderType,
    duration: Duration,
    finish_time: OffsetDateTime,
    finish_notifications_send: bool,
    delete_flag: bool,
    restart_flag: bool,
}

impl Reminder {
    fn display(&mut self) -> String {
        // TODO: Make UTC OFFSET a constant
        let now = OffsetDateTime::now_utc().to_offset(UtcOffset::from_hms(2, 0, 0).unwrap());
        let time_left = self.finish_time - now;
        if !time_left.is_positive() && !self.finish_notifications_send {
            Notification::new()
                .summary(self.name.as_str())
                .urgency(notify_rust::Urgency::Critical)
                .sound_name("alarm-clock_elapsed")
                .show()
                .unwrap();
            self.finish_notifications_send = true;
        }
        format!("{}", self)
    }
    fn restart(&mut self) {
        let now = OffsetDateTime::now_utc().to_offset(UtcOffset::from_hms(2, 0, 0).unwrap());
        self.start_time = now;
        match self.reminder_type {
            ReminderType::Time => {
                self.finish_time = self.finish_time.replace_date(now.date());
                if self.finish_time < now {
                    self.finish_time += Duration::days(1)
                }
                self.duration = self.finish_time - now;
            }
            ReminderType::Duration => {
                self.finish_time = now + self.duration;
            }
        }
        self.delete_flag = false;
        self.restart_flag = false;
    }
    fn snooze(&mut self) {
        let now = OffsetDateTime::now_utc().to_offset(UtcOffset::from_hms(2, 0, 0).unwrap());
        if self.finish_time < now {
            self.finish_time += Duration::minutes(5);
            self.duration += Duration::minutes(5);
        }
    }
}
impl Display for Reminder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let now = OffsetDateTime::now_utc().to_offset(UtcOffset::from_hms(2, 0, 0).unwrap());
        let time_left = self.finish_time - now;
        if time_left.is_positive() {
            let percent_finished = map_range(
                (
                    self.start_time.unix_timestamp() as f64,
                    self.finish_time.unix_timestamp() as f64,
                ),
                (0., 100.),
                now.unix_timestamp() as f64,
            )
            .round() as usize;
            let mut progressbar = String::new();
            for _ in 0..(percent_finished / 5) {
                progressbar.push('=');
            }
            progressbar.push('>');
            write!(
                f,
                "{:>10} {:0>2}:{:0>2}:{:0>2} [{:<21}] {:>11} (duetime: {:?})",
                if self.delete_flag {
                    self.name.clone().to_uppercase()
                } else {
                    self.name.clone()
                },
                time_left.whole_hours() - time_left.whole_days() * 24,
                time_left.whole_minutes() - time_left.whole_hours() * 60,
                time_left.whole_seconds() - time_left.whole_minutes() * 60,
                progressbar,
                if time_left.whole_days() > 0 {
                    format!("(+{} days)", time_left.whole_days())
                } else {
                    "".to_string()
                },
                self.finish_time
            )
        } else if self.restart_flag {
            write!(
                f,
                "?-------------------{} HAS FINISHED-------------------",
                self.name,
            )
        } else {
            write!(
                f,
                "--------------------{} HAS FINISHED-------------------",
                self.name,
            )
        }
    }
}
impl From<ApiReminder> for Reminder {
    fn from(value: ApiReminder) -> Self {
        let now = OffsetDateTime::now_utc().to_offset(UtcOffset::from_hms(2, 0, 0).unwrap());
        Self {
            name: value.name,
            start_time: now,
            duration: now - value.finish_time,
            finish_time: value.finish_time,
            finish_notifications_send: false,
            delete_flag: false,
            restart_flag: false,
            reminder_type: ReminderType::Time,
        }
    }
}

#[tokio::main]
async fn main() {
    // remindy
    let temp_time = OffsetDateTime::now_utc().to_offset(UtcOffset::from_hms(2, 0, 0).unwrap());
    let reminders: Arc<Mutex<Vec<Reminder>>> = Arc::new(Mutex::new(vec![
        Reminder {
            name: "foof".to_string(),
            start_time: temp_time,
            duration: Duration::hours(300),
            finish_time: temp_time + Duration::hours(300),
            finish_notifications_send: false,
            delete_flag: false,
            restart_flag: false,
            reminder_type: ReminderType::Time,
        },
        Reminder {
            name: "sees".to_string(),
            start_time: temp_time,
            duration: Duration::hours(300),
            finish_time: temp_time - Duration::hours(300),
            finish_notifications_send: false,
            delete_flag: false,
            restart_flag: false,
            reminder_type: ReminderType::Time,
        },
        Reminder {
            name: "lool".to_string(),
            start_time: temp_time,
            duration: Duration::hours(300),
            finish_time: temp_time + Duration::hours(300),
            finish_notifications_send: false,
            delete_flag: false,
            restart_flag: false,
            reminder_type: ReminderType::Time,
        },
    ]));

    // axum
    let reminders_axum_clone = Arc::clone(&reminders);
    tokio::spawn(async move {
        let app = Router::new()
            .route("/reminder", get(all_reminder))
            .route("/reminder", post(add_reminder))
            .with_state(reminders_axum_clone);

        axum::Server::bind(&SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            4321,
        ))
        .serve(app.into_make_service())
        .await
        .unwrap();
    });

    // terminal display
    let mut cursor_position: usize = 0;
    let _trash_bin = enable_raw_mode().is_ok();
    let mut stdout = std::io::stdout();
    loop {
        execute!(
            stdout,
            cursor::Hide,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )
        .unwrap();
        stdout.write_all(b"Remindy started\n\r").unwrap();
        stdout.write_all(b"===============\n\r").unwrap();
        stdout.write_all(b"'j', 'k' -> up, down\n\r").unwrap();
        stdout.write_all(b"'n' -> new\n\r").unwrap();
        stdout.write_all(b"'rn' -> rename\n\r").unwrap();
        stdout.write_all(b"'rt' -> retime\n\r").unwrap();
        stdout.write_all(b"'s' -> snooze\n\r").unwrap();
        stdout
            .write_all(b"'d' -> delete (double tab needed)\n\r")
            .unwrap();
        stdout
            .write_all(b"'rs' -> restart (double tab needed)\n\r")
            .unwrap();
        stdout.write_all(b"'esc' -> unmark everything\n\r").unwrap();
        stdout
            .write_all(b"'CTRL' + 'c' -> exit(0)\n\r")
            .unwrap();
        if let Ok(mut reminders) = reminders.try_lock() {
            for (i, reminder) in reminders.iter_mut().enumerate() {
                if i == cursor_position {
                    stdout
                        .write_all(format!("{} <--\n\r", reminder.display()).as_bytes())
                        .unwrap();
                } else {
                    stdout
                        .write_all(format!("{}\n\r", reminder.display()).as_bytes())
                        .unwrap();
                }
            }
        }
        if poll(std::time::Duration::from_millis(500)).unwrap() {
            #[allow(clippy::single_match)]
            match read().unwrap() {
                Event::Key(event) => {
                    match event.code {
                        KeyCode::Char('c') => {
                            if event.modifiers.contains(KeyModifiers::CONTROL) {
                                let _trash_bin = disable_raw_mode().is_ok();
                                std::process::exit(0);
                            }
                        }
                        KeyCode::Char('n') => {
                            if let Ok(mut reminders) = reminders.lock() {
                                let now = OffsetDateTime::now_utc()
                                    .to_offset(UtcOffset::from_hms(2, 0, 0).unwrap());
                                reminders.push(Reminder {
                                    name: "NEW".to_string(),
                                    start_time: now,
                                    reminder_type: ReminderType::Duration,
                                    duration: Duration::new(0, 0),
                                    finish_time: now,
                                    finish_notifications_send: true,
                                    delete_flag: false,
                                    restart_flag: false,
                                })
                            }
                        }
                        KeyCode::Char('r') => match read().unwrap() {
                            Event::Key(event) => match event.code {
                                KeyCode::Char('s') => {
                                    if let Ok(mut reminders) = reminders.lock() {
                                        let reminder = reminders.get_mut(cursor_position).unwrap();
                                        if reminder.restart_flag {
                                            reminder.restart()
                                        } else {
                                            reminder.restart_flag = true;
                                        }
                                    }
                                }
                                KeyCode::Char('n') => {
                                    execute!(
                                        stdout,
                                        cursor::Show,
                                        cursor::MoveTo(0, cursor_position as u16)
                                    )
                                    .unwrap();
                                    let _trash_bin = disable_raw_mode().is_ok();
                                    let mut name = String::new();
                                    std::io::stdin().read_line(&mut name).unwrap();
                                    name = name.replace('\n', "");
                                    if let Ok(mut reminders) = reminders.lock() {
                                        let reminder = reminders.get_mut(cursor_position).unwrap();
                                        reminder.name = name;
                                    }
                                    let _trash_bin = enable_raw_mode().is_ok();
                                }
                                KeyCode::Char('t') => {
                                    let _trash_bin = disable_raw_mode().is_ok();
                                    let mut time_input = String::new();
                                    std::io::stdin().read_line(&mut time_input).unwrap();
                                    time_input = time_input.replace('\n', "");
                                    if let Ok(mut reminders) = reminders.lock() {
                                        let reminder = reminders.get_mut(cursor_position).unwrap();
                                        reminder.name = time_input.clone();
                                    }
                                    let now = OffsetDateTime::now_utc()
                                        .to_offset(UtcOffset::from_hms(2, 0, 0).unwrap());
                                    let mut finish_time = OffsetDateTime::now_utc()
                                        .to_offset(UtcOffset::from_hms(2, 0, 0).unwrap());
                                    if let Ok(mut reminders) = reminders.lock() {
                                        let reminder = reminders.get_mut(cursor_position).unwrap();
                                        #[allow(clippy::useless_conversion)]
                                        if time_input.chars().all(|e| e.is_ascii_digit()) {
                                            finish_time = finish_time.replace_time(
                                                Time::parse(
                                                    time_input.as_str(),
                                                    &format_description::parse("[hour]:[minute]")
                                                        .unwrap(),
                                                )
                                                .unwrap(),
                                            );
                                            reminder.reminder_type = ReminderType::Time;
                                            reminder.start_time = now;
                                            reminder.finish_time = finish_time;
                                            reminder.duration = finish_time - now;
                                        } else {
                                            let d: core::time::Duration =
                                                DurationString::from_string(time_input)
                                                    .unwrap()
                                                    .into();
                                            reminder.duration =
                                                Duration::from(d.try_into().unwrap());
                                            reminder.finish_time = now + d;
                                            reminder.reminder_type = ReminderType::Duration;
                                        }
                                    }
                                    let _trash_bin = enable_raw_mode().is_ok();
                                }
                                _ => (),
                            },
                            _ => (),
                        },
                        KeyCode::Char('k') => cursor_position = cursor_position.saturating_sub(1),
                        KeyCode::Char('j') => {
                            if let Ok(reminders) = reminders.lock() {
                                if cursor_position < reminders.len() - 1 {
                                    cursor_position = cursor_position.saturating_add(1)
                                }
                            }
                        }
                        KeyCode::Char('d') => {
                            if let Ok(mut reminders) = reminders.lock() {
                                let reminder = reminders.get_mut(cursor_position).unwrap();
                                if reminder.delete_flag {
                                    reminders.remove(cursor_position);
                                    if reminders.len() == cursor_position {
                                        cursor_position = cursor_position.saturating_sub(1);
                                    }
                                } else {
                                    reminder.delete_flag = true;
                                }
                            }
                        }
                        KeyCode::Char('s') => {
                            if let Ok(mut reminders) = reminders.lock() {
                                let reminder = reminders.get_mut(cursor_position).unwrap();
                                reminder.snooze();
                            }
                        }
                        KeyCode::Esc => {
                            if let Ok(mut reminders) = reminders.lock() {
                                for reminder in reminders.iter_mut() {
                                    reminder.restart_flag = false;
                                    reminder.delete_flag = false;
                                }
                            }
                        }
                        // TODO: pause reminder
                        _ => stdout
                            .write_all(
                                format!("{:?} is a unknown command!\n\r", event.code).as_bytes(),
                            )
                            .unwrap(),
                    }
                }
                _ => (),
            }
        }
    }
}

async fn all_reminder(
    State(reminders): State<Arc<Mutex<Vec<Reminder>>>>,
) -> (StatusCode, Json<Vec<Reminder>>) {
    if let Ok(reminders) = reminders.lock() {
        (StatusCode::OK, Json(reminders.clone()))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![]))
    }
}

#[derive(Clone, Deserialize)]
struct ApiReminder {
    name: String,
    finish_time: OffsetDateTime,
}
async fn add_reminder(
    State(reminders): State<Arc<Mutex<Vec<Reminder>>>>,
    new_reminder: Result<Json<ApiReminder>, JsonRejection>,
) -> StatusCode {
    if let Ok(mut reminders) = reminders.lock() {
        if let Ok(Json(new_reminder)) = new_reminder {
            reminders.push(new_reminder.into());
            StatusCode::OK
        } else {
            StatusCode::UNPROCESSABLE_ENTITY
        }
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

fn map_range(from_range: (f64, f64), to_range: (f64, f64), s: f64) -> f64 {
    to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
}
