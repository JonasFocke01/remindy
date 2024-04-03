use reminder::{past_event::PastEvent, reminder::my_local_offset};

#[allow(clippy::module_name_repetitions)]
pub fn build_status_box(last_event: PastEvent) -> String {
    let mut result = String::new();
    result.push_str("           =======================================\n\r");
    result.push_str(format!("           | {:<36}|\n\r", "'j', 'k' -> up, down").as_str());
    result.push_str(format!("           | {:<36}|\n\r", "'n' -> new").as_str());
    result.push_str(format!("           | {:<36}|\n\r", "'rn' -> rename").as_str());
    result.push_str(format!("           | {:<36}|\n\r", "'rt' -> retime").as_str());
    result.push_str(format!("           | {:<36}|\n\r", "'s' -> snooze").as_str());
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
    result.push_str(format!("           | {:<36}|\n\r", "'+' -> add to endtime").as_str());
    result.push_str(format!("           | {:<36}|\n\r", "'-' -> subtract from endtime").as_str());
    result.push_str(
        format!(
            "           | {:<36}|\n\r",
            "'s' -> toggle 'send external message'"
        )
        .as_str(),
    );
    result.push_str(format!("           | {:<36}|\n\r", "'ENTER' -> Edit").as_str());
    result.push_str(format!("           | {:<36}|\n\r", "'CTRL' + 'c' -> exit(0)").as_str());
    if my_local_offset().is_utc() {
        for _ in 0..100 {
            result.push_str(
                format!("           | {:<36}|\n\r", "Local Offset NOT found!!!").as_str(),
            );
        }
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
