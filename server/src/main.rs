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
use crossterm::{cursor, execute};
use serde::{Deserialize, Serialize};

use time::{Duration, OffsetDateTime, UtcOffset};

#[derive(Clone, Serialize, Deserialize)]
struct Reminder {
    name: String,
    start_time: OffsetDateTime,
    duration: Duration,
    finish_time: OffsetDateTime,
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
                "{:>10} {:0>2}:{:0>2}:{:0>2} [{:<21}] {:>11} ",
                self.name,
                time_left.whole_hours() - time_left.whole_days() * 24,
                time_left.whole_minutes() - time_left.whole_hours() * 60,
                time_left.whole_seconds() - time_left.whole_minutes() * 60,
                progressbar,
                if time_left.whole_days() > 0 {
                    format!("(+{} days)", time_left.whole_days())
                } else {
                    "".to_string()
                }
            )
        } else {
            write!(f, "----------------------{} HAS FINISHED----------------------", self.name,)
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
        }
    }
}

#[tokio::main]
async fn main() {
    // remindy
    let temp_time = OffsetDateTime::now_utc().to_offset(UtcOffset::from_hms(2, 0, 0).unwrap());
    let reminders: Arc<Mutex<Vec<Reminder>>> = Arc::new(Mutex::new(vec![Reminder {
        name: "foof".to_string(),
        start_time: temp_time,
        duration: Duration::hours(300),
        finish_time: temp_time + Duration::hours(300),
    }]));

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
    let mut to_revert_lines: u16;
    loop {
        if let Ok(reminders) = reminders.try_lock() {
            let mut stdout = std::io::stdout();
            stdout.flush().unwrap();
            println!("\rreminders: ");
            to_revert_lines = 1;
            for reminder in reminders.iter() {
                to_revert_lines += 1;
                println!("{}", reminder);
            }
            execute!(stdout, cursor::MoveUp(to_revert_lines), cursor::Hide).unwrap();
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
