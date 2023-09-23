
use axum::{
    extract::{rejection::JsonRejection, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use time::{Duration, OffsetDateTime, UtcOffset};

#[derive(Clone, Serialize, Deserialize)]
struct Reminder {
    name: String,
    // start_time: OffsetDateTime,
    finish_time: OffsetDateTime,
}
impl Display for Reminder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let now = OffsetDateTime::now_utc().to_offset(UtcOffset::from_hms(2, 0, 0).unwrap());
        let time_left = self.finish_time - now;
        write!(
            f,
            "{:>10} {:0>2}:{:0>2}:{:0>2} {} {:>11} ",
            self.name,
            time_left.whole_hours() - time_left.whole_days() * 24,
            time_left.whole_minutes() - time_left.whole_hours() * 60,
            time_left.whole_seconds() - time_left.whole_minutes() * 60,
            "Progressbar",
            if time_left.whole_days() > 0 {
                format!("(+{} days)", time_left.whole_days())
            } else {
                "".to_string()
            }
        )
    }
}
}
#[tokio::main]
async fn main() {
    // remindy
    let temp_time = OffsetDateTime::now_utc().to_offset(UtcOffset::from_hms(2, 0, 0).unwrap());
    let reminders: Arc<Mutex<Vec<Reminder>>> = Arc::new(Mutex::new(vec![Reminder {
        name: "foof".to_string(),
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
