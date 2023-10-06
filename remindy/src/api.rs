use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex},
};

use axum::{
    extract::{rejection::JsonRejection, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use colored::Colorize;

use crate::reminder::{ApiReminder, Reminder};

#[derive(Debug)]
// TODO: Fix status (RunninOk is displayed, despite not being okay)
pub enum ApiStatus {
    RunningOk,
    Starting,
    FailedToBindToPort,
    Stopped,
}

impl ApiStatus {
    pub fn as_info_string(&self) -> String {
        let result = format!("{:?}", self);
        match self {
            Self::RunningOk => format!("{}", result.bright_green()),
            Self::Starting => format!("{}", result.green()),
            Self::FailedToBindToPort => format!("{}", result.red()),
            Self::Stopped => format!("{}", result.bright_red()),
        }
    }
}

pub fn spawn_api(reminders: &Arc<Mutex<Vec<Reminder>>>, port: u16) -> Arc<Mutex<ApiStatus>> {
    let status = Arc::new(Mutex::new(ApiStatus::Stopped));
    let return_status = Arc::clone(&status);
    let reminders_axum_clone = Arc::clone(reminders);
    tokio::spawn(async move {
        if let Ok(mut status) = status.lock() {
            *status = ApiStatus::Starting;
        }
        let app = Router::new()
            .route("/reminder", get(all_reminder))
            .route("/reminder", post(add_reminder))
            .with_state(reminders_axum_clone);

        if let Ok(mut status) = status.lock() {
            // This is a lie, because we cant interfere with the awaiting axum server
            *status = ApiStatus::RunningOk;
        }
        if axum::Server::bind(&SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port,
        ))
        .serve(app.into_make_service())
        .await
        .is_err()
        {
            if let Ok(mut status) = status.lock() {
                *status = ApiStatus::FailedToBindToPort;
            }
        } else if let Ok(mut status) = status.lock() {
            *status = ApiStatus::Stopped;
        }
    });
    return_status
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
