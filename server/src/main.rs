use std::fs::write;
use std::thread;
use std::{
    fs::File,
    io::Write,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex},
};

use api::{add_reminder, all_reminder};
use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
    routing::{delete, get, patch, post, put},
    Router,
};

mod api;
use crate::api::{
    alter_reminder_description, confirm_reminder_finish_event, cut_reminder_duration,
    delete_reminder, force_restart_reminder, get_past_event, pause_reminder,
    push_reminder_duration, rename_reminder, reset_reminder_flags, restart_reminder,
    retime_reminder, snooze_reminder, toggle_reminder_repeat,
};
use reminder::{past_event::PastEvent, reminder::Reminder, root_path, PORT, REMINDER_DB_FILE};

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
    println!("starting...");
    let reminders: Arc<Mutex<Vec<Reminder>>> = if let Some(reminders) =
        Reminder::from_file(format!("{}/{REMINDER_DB_FILE}", root_path()).as_str())
    {
        Arc::new(Mutex::new(reminders))
    } else {
        // TODO: do something about not deleting existing reminders
        if std::fs::create_dir_all(root_path()).is_ok() {
            if let Ok(mut file) =
                File::create(format!("{}/{REMINDER_DB_FILE}", root_path()).as_str())
            {
                let _trash_bin = file.write_all(b"[]");
            }
        }
        Arc::new(Mutex::new(vec![]))
    };

    let past_event = Arc::new(Mutex::new(PastEvent::None));

    let reminders_clone = Arc::clone(&reminders);
    let past_event_clone = Arc::clone(&past_event);
    thread::spawn(move || loop {
        let Ok(mut reminders) = reminders_clone.lock() else {
            continue;
        };
        let mut writable = false;
        for reminder in reminders.iter_mut() {
            if reminder.remaining_duration().is_none() {
                if let Ok(mut past_event) = past_event_clone.lock() {
                    reminder.request_confirmation(&mut past_event);
                    if reminder.repeating() {
                        reminder.restart();
                        writable = true;
                    }
                }
            }
        }
        if writable {
            write_reminder_db(reminders.clone());
        }
        drop(reminders);
        std::thread::sleep(std::time::Duration::from_secs(1));
    });

    let app = Router::new()
        .route("/reminders", post(add_reminder))
        .route("/reminders/reset_flags", put(reset_reminder_flags))
        .route("/reminders/:id/restart", put(restart_reminder))
        .route("/reminders/:id/force_restart", put(force_restart_reminder))
        .route("/reminders/:id/rename", patch(rename_reminder))
        .route("/reminders/:id/snooze", put(snooze_reminder))
        .route("/reminders/:id/retime", patch(retime_reminder))
        .route("/reminders/:id", delete(delete_reminder))
        .route("/reminders/:id/pause", put(pause_reminder))
        .route("/reminders/:id/toggle_repeat", put(toggle_reminder_repeat))
        .route(
            "/reminders/:id/push_duration",
            patch(push_reminder_duration),
        )
        .route("/reminders/:id/cut_duration", patch(cut_reminder_duration))
        .route(
            "/reminders/:id/description",
            patch(alter_reminder_description),
        )
        .route("/reminders/:id/confirm", put(confirm_reminder_finish_event))
        .layer(axum::middleware::from_fn_with_state(
            Arc::clone(&reminders),
            write_reminder_db_middleware,
        ))
        .route("/past_event", get(get_past_event))
        .route("/reminders", get(all_reminder))
        .with_state((reminders, past_event));

    #[allow(clippy::panic)]
    let Ok(listener) = tokio::net::TcpListener::bind(&SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        PORT,
    ))
    .await
    else {
        panic!("Failed to bind to port {PORT}");
    };
    #[allow(clippy::panic)]
    if axum::serve(listener, app.into_make_service())
        .await
        .is_err()
    {
        panic!("Failed to start server!");
    }
}

pub async fn write_reminder_db_middleware(
    State(reminders): State<Arc<Mutex<Vec<Reminder>>>>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let res = next.run(req).await;
    if let Ok(mut reminders) = reminders.lock() {
        for reminder in reminders.iter_mut() {
            reminder.push_back_end_time_if_paused(time::Duration::seconds(1));
        }
        write_reminder_db(reminders.clone());
    }
    Ok(res)
}

pub fn write_reminder_db(reminders: Vec<Reminder>) {
    let Ok(serialized_reminders) = serde_json::to_string_pretty(&reminders) else {
        panic!("failed to serialize reminders")
    };
    let _trash_bin = write(
        format!("{}/{REMINDER_DB_FILE}", root_path()).as_str(),
        serialized_reminders,
    );
    println!("Wrote reminder db")
}
