use serde::{Deserialize, Serialize};

use time::{Duration, OffsetDateTime, UtcOffset};

#[derive(Clone, Serialize, Deserialize)]
struct Reminder {
    name: String,
    // start_time: OffsetDateTime,
    finish_time: OffsetDateTime,
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

    // terminal display
}

