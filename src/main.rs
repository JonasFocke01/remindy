use notify_rust::Notification;
use std::env;

fn main() {
    Notification::new()
        .summary("Im a notification")
        .body("Hello from the body")
        .show().unwrap();
    }
