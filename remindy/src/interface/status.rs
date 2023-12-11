use std::sync::{Arc, Mutex};

use crate::api::ApiStatus;

use super::past_event::PastEvent;

pub fn build_status_box(api_status: &Arc<Mutex<ApiStatus>>, last_event: &PastEvent) -> String {
    let mut result = String::new();
    result.push_str("           =======================================\n\r");
    result.push_str(format!("           | {:<36}|\n\r", "'j', 'k' -> up, down").as_str());
    result.push_str(format!("           | {:<36}|\n\r", "'n' -> new").as_str());
    result.push_str(format!("           | {:<36}|\n\r", "'rn' -> rename").as_str());
    result.push_str(format!("           | {:<36}|\n\r", "'rt' -> retime").as_str());
    result.push_str(format!("           | {:<36}|\n\r", "'s' -> snooze").as_str());
    result.push_str(format!("           | {:<36}|\n\r", "'l' -> show the library").as_str());
    result.push_str(
        format!(
            "           | {:<36}|\n\r",
            "'d' -> delete (double tab needed)"
        )
        .as_str(),
    );
    result.push_str(
        format!(
            "           | {:<36}|\n\r",
            "'rs' -> restart (double tab needed)"
        )
        .as_str(),
    );
    result.push_str(format!("           | {:<36}|\n\r", "'esc' -> unmark everything").as_str());
    result.push_str(format!("           | {:<36}|\n\r", "'e' -> repeat").as_str());
    result.push_str(format!("           | {:<36}|\n\r", "'CTRL' + 'c' -> exit(0)").as_str());
    if let Ok(api_status) = api_status.lock() {
        result.push_str(
            format!(
                "           | Rest api status: {:<28}|\n\r",
                api_status.as_info_string()
            )
            .as_str(),
        );
    }
    result.push_str(
        format!(
            "           | Version: {:<27}|\n\r",
            env!("CARGO_PKG_VERSION")
        )
        .as_str(),
    );
    result.push_str("           =======================================\n\r");
    result.push_str(format!("            {:^54}\n\n\r", last_event.to_string()).as_str());
    result
}
