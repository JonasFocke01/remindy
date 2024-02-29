use std::cmp::Ordering;
use std::ops::DerefMut;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::{
    fs::File,
    io::Write,
    net::{IpAddr, SocketAddr},
    sync::{Arc, Mutex},
};

use api::{add_reminder, all_reminder, all_reminder_formatted};
use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
    routing::{delete, get, patch, post, put},
    Router,
};
use config::Config;
use json_store_rs::JsonStore;
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};

mod api;
use crate::api::{
    alter_reminder_description, confirm_reminder_finish_event, cut_reminder_duration,
    delete_reminder, force_restart_reminder, get_past_event, pause_reminder,
    push_reminder_duration, rename_reminder, reset_reminder_flags, restart_reminder,
    retime_reminder, snooze_reminder, toggle_reminder_repeat,
};
use reminder::{past_event::PastEvent, reminder::Reminder, root_path, REMINDER_DB_FILE};

#[derive(Serialize, Deserialize, Clone, Default)]
struct Reminders {
    reminders: Vec<Reminder>,
}

impl JsonStore for Reminders {
    fn db_file_path() -> PathBuf {
        let mut root_path = root_path().unwrap_or_default();
        root_path.push(REMINDER_DB_FILE);
        root_path
    }
}

impl std::ops::Deref for Reminders {
    type Target = Vec<Reminder>;
    fn deref(&self) -> &Self::Target {
        &self.reminders
    }
}

impl DerefMut for Reminders {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reminders
    }
}

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() {
    println!("starting...");
    println!("version: {}", env!("CARGO_PKG_VERSION"));
    let config = Config::new();
    let reminders: Arc<Mutex<Reminders>> = if let Ok(reminders) = Reminders::load() {
        Arc::new(Mutex::new(reminders))
    } else {
        // TODO: do something about not deleting existing reminders
        if std::fs::create_dir_all(root_path().unwrap_or_default()).is_ok() {
            if let Ok(mut file) = File::create(
                format!("{:?}/{REMINDER_DB_FILE}", root_path().unwrap_or_default())
                    .as_str()
                    .replace('\"', ""),
            ) {
                let _trash_bin = file.write_all(b"[]");
            }
        }
        Arc::new(Mutex::new(Reminders { reminders: vec![] }))
    };

    let past_event = Arc::new(Mutex::new(PastEvent::None));

    let reminders_clone = Arc::clone(&reminders);
    let past_event_clone = Arc::clone(&past_event);
    thread::spawn(move || loop {
        let Ok(mut reminders) = reminders_clone.lock() else {
            continue;
        };
        reminders.sort_by(|a, b| {
            if a.finish_time().cmp(&b.finish_time()) == Ordering::Less {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        });
        let mut writable = false;
        for reminder in reminders.iter_mut() {
            if reminder.remaining_duration().is_none() {
                if let Ok(mut past_event) = past_event_clone.lock() {
                    if !reminder.needs_confirmation()
                        && !reminder.repeating()
                        && !reminder.already_confirmed()
                    {
                        // TODO: make this configurable
                        // TODO: repair, because no whatsapp is send..
                        let _ = Command::new("curl")
                            .args([
                                "https://api.twilio.com/2010-04-01/Accounts/ACc4a89978184cd77f60c13a8515013754/Messages.json",
                                "-X",
                                "POST",
                                "--data-urlencode",
                                "To=whatsapp:+491733113571",
                                "--data-urlencode",
                                "From=whatsapp:+14155238886",
                                "--data-urlencode",
                                format!("Body=Reminder due:\n{}\n{}", reminder.name(), reminder.description()).as_str(),
                                "-u",
                                "ACc4a89978184cd77f60c13a8515013754:4411046064ba9ef33911da7b225def5f"
                            ])
                            .stdout(Stdio::null())
                            .spawn();
                    }
                    reminder.request_confirmation(&mut past_event);
                    if reminder.repeating() {
                        reminder.restart();
                        writable = true;
                    }
                }
            }
        }
        for reminder in reminders.iter_mut() {
            reminder.push_back_end_time_if_paused(time::Duration::SECOND);
            writable = true;
        }
        if writable {
            write_reminder_db(&mut reminders);
        }
        drop(reminders);
        std::thread::sleep(std::time::Duration::from_secs(1));
    });

    let corslayer = CorsLayer::new().allow_methods(Any).allow_origin(Any);

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
        .route("/reminders/formatted", get(all_reminder_formatted))
        .layer(corslayer)
        .with_state((reminders, past_event));

    #[allow(clippy::panic)]
    let Ok(listener) = tokio::net::TcpListener::bind(&SocketAddr::new(
        IpAddr::V4(config.network().local_ip_as_ipv4()),
        config.network().port_as_u16(),
    ))
    .await
    else {
        panic!("Failed to bind to port {}", config.network().port());
    };
    #[allow(clippy::panic)]
    if axum::serve(listener, app.into_make_service())
        .await
        .is_err()
    {
        panic!("Failed to start server!");
    }
}

#[allow(clippy::missing_errors_doc)]
async fn write_reminder_db_middleware(
    State(reminders): State<Arc<Mutex<Reminders>>>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let result = next.run(req).await;
    if let Ok(mut reminders) = reminders.lock() {
        for reminder in reminders.iter_mut() {
            reminder.push_back_end_time_if_paused(time::Duration::seconds(1));
        }
        write_reminder_db(&mut reminders);
    }
    Ok(result)
}

fn write_reminder_db(reminders: &mut Reminders) {
    let _ = reminders.write();
    print!(" w");
}
