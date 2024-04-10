use std::{
    io::{stdin, Read, Stdout, Write},
    process::{Command, Stdio},
};

use config::Config;
use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode, KeyModifiers, ModifierKeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use duration_string::DurationString;
use time::{format_description, Date, Duration, OffsetDateTime, PrimitiveDateTime, Time};

use reminder::reminder::{my_local_offset, ApiReminder, Reminder, ReminderType, TimeObject};

/// This Reads any input detected on the terminal window.
/// This will block when a known key combination is found and there are follow up decisions to make
/// for the user.
/// Otherwise, this blocks for one second and returns.
#[allow(clippy::too_many_lines)]
pub fn read_input(
    stdout: &mut Stdout,
    selected_reminder: &Reminder,
    all_reminders: &Vec<Reminder>,
    reminder_amount: usize,
    cursor_position: &mut usize,
    request_client: &reqwest::blocking::Client,
    config: &Config,
) -> bool {
    if poll(std::time::Duration::from_secs(1)).map_or_else(|_| true, |v| v) {
        #[allow(clippy::single_match, clippy::wildcard_enum_match_arm)]
        if let Ok(Event::Key(event)) = read() {
            return match event.code {
                KeyCode::Char('c') => {
                    if event.modifiers.contains(KeyModifiers::CONTROL) {
                        let _trash_bin = execute!(stdout, cursor::Show,);
                        let _trash_bin = disable_raw_mode().is_ok();
                        std::process::exit(0);
                    } else {
                        false
                    }
                }
                KeyCode::Char('n') => {
                    execute!(stdout, cursor::Show,).unwrap();
                    let mut name = String::new();
                    let _trash_bin = enable_raw_mode().is_ok();
                    let _trash_bin = stdout.write_all(b"Name: ");
                    let _trash_bin = stdout.flush();
                    let _trash_bin = disable_raw_mode().is_ok();
                    if stdin().read_line(&mut name).is_err() {
                        return false;
                    };
                    name = name.replace('\n', "");
                    let mut time_input = String::new();
                    let _trash_bin = enable_raw_mode().is_ok();
                    let _trash_bin = stdout.write_all(
                        b"End time or date (1h10m | 15:23 | 8.11.2023 | 8.11.2023 15:23): ",
                    );
                    let _trash_bin = stdout.flush();
                    let _trash_bin = disable_raw_mode().is_ok();
                    if stdin().read_line(&mut time_input).is_err() {
                        return false;
                    };
                    time_input = time_input.replace('\n', "");
                    let now = OffsetDateTime::now_utc().to_offset(my_local_offset());
                    let mut finish_time = OffsetDateTime::now_utc().to_offset(my_local_offset());
                    let reminder_type: ReminderType;
                    #[allow(unused_assignments)]
                    let mut _duration: Duration = Duration::new(0, 0);
                    #[allow(clippy::useless_conversion, clippy::arithmetic_side_effects)]
                    if time_input.chars().all(|e| e.is_ascii_digit() || e == ':') {
                        let Ok(format) = format_description::parse("[hour]:[minute]") else {
                            return false;
                        };
                        let Ok(new_finish_time) = Time::parse(time_input.as_str(), &format) else {
                            return false;
                        };
                        finish_time = finish_time.replace_time(new_finish_time);
                        reminder_type = ReminderType::Time;
                        _duration = finish_time - now;
                    } else if time_input.chars().all(|e| e.is_ascii_digit() || e == '.') {
                        let Ok(format) = format_description::parse("[day].[month].[year]") else {
                            return false;
                        };
                        let Ok(new_finish_date) = Date::parse(time_input.as_str(), &format) else {
                            return false;
                        };
                        finish_time = finish_time.replace_date(new_finish_date);
                        reminder_type = ReminderType::Date;
                    } else if time_input
                        .chars()
                        .all(|e| e.is_ascii_digit() || e == '.' || e == ':' || e == ' ')
                    {
                        let Ok(format) =
                            format_description::parse("[day].[month].[year] [hour]:[minute]")
                        else {
                            return false;
                        };
                        let Ok(new_finish_date_time) =
                            PrimitiveDateTime::parse(time_input.as_str(), &format)
                        else {
                            return false;
                        };
                        finish_time = finish_time.replace_date(new_finish_date_time.date());
                        finish_time = finish_time.replace_time(new_finish_date_time.time());
                        reminder_type = ReminderType::Date;
                    } else {
                        let Ok(parsed_duration_string) = DurationString::from_string(time_input)
                        else {
                            return false;
                        };
                        let parsed_duration: core::time::Duration = parsed_duration_string.into();
                        let Ok(parsed_duration) = parsed_duration.try_into() else {
                            return false;
                        };
                        _duration = Duration::from(parsed_duration);
                        finish_time = now + parsed_duration;
                        reminder_type = ReminderType::Duration;
                    }
                    if request_client
                        .post(format!(
                            "http://{}:{}/reminders",
                            config.network().remote_ip(),
                            config.network().port()
                        ))
                        .json(&ApiReminder {
                            name,
                            description: String::new(),
                            finish_time,
                            reminder_type,
                        })
                        .send()
                        .is_ok()
                    {
                        return true;
                    }
                    false
                }
                KeyCode::Char(' ') => {
                    if request_client
                        .put(format!(
                            "http://{}:{}/reminders/{}/pause",
                            config.network().remote_ip(),
                            config.network().port(),
                            selected_reminder.id()
                        ))
                        .send()
                        .is_ok()
                    {
                        return true;
                    }
                    return false;
                }

                KeyCode::Char('r') => {
                    read_re_mode_input(stdout, selected_reminder, request_client, config);
                    return true;
                }
                KeyCode::Char('k') => {
                    *cursor_position = cursor_position.saturating_sub(1);
                    return false;
                }
                KeyCode::Char('j') => {
                    if *cursor_position != reminder_amount.saturating_sub(1) {
                        *cursor_position = cursor_position.saturating_add(1);
                    }
                    return false;
                }
                KeyCode::Char('d') => {
                    if *cursor_position == reminder_amount.saturating_sub(1)
                        && selected_reminder.delete_flag()
                    {
                        *cursor_position = cursor_position.saturating_sub(1);
                    }
                    if request_client
                        .delete(format!(
                            "http://{}:{}/reminders/{}",
                            config.network().remote_ip(),
                            config.network().port(),
                            selected_reminder.id()
                        ))
                        .send()
                        .is_ok()
                    {
                        return true;
                    }
                    return false;
                }
                KeyCode::Char('s') => {
                    if request_client
                        .put(format!(
                            "http://{}:{}/reminders/{}/snooze",
                            config.network().remote_ip(),
                            config.network().port(),
                            selected_reminder.id()
                        ))
                        .send()
                        .is_ok()
                    {
                        return true;
                    }
                    return false;
                }
                KeyCode::Char('e') => {
                    if request_client
                        .put(format!(
                            "http://{}:{}/reminders/{}/toggle_repeat",
                            config.network().remote_ip(),
                            config.network().port(),
                            selected_reminder.id()
                        ))
                        .send()
                        .is_ok()
                    {
                        return true;
                    }
                    return false;
                }
                KeyCode::Char('s') => {
                    if request_client
                        .put(format!(
                            "http://{}:{}/reminders/{}/toggle_send_e_message",
                            config.network().remote_ip(),
                            config.network().port(),
                            selected_reminder.id()
                        ))
                        .send()
                        .is_ok()
                    {
                        return true;
                    }
                    return false;
                }
                KeyCode::Char('u') => {
                    if request_client
                        .put(format!(
                            "http://{}:{}/reminders/undo",
                            config.network().remote_ip(),
                            config.network().port(),
                        ))
                        .send()
                        .is_ok()
                    {
                        return true;
                    }
                    return false;
                }
                KeyCode::Char('U') => {
                    if request_client
                        .put(format!(
                            "http://{}:{}/reminders/redo",
                            config.network().remote_ip(),
                            config.network().port(),
                        ))
                        .send()
                        .is_ok()
                    {
                        return true;
                    }
                    return false;
                }
                KeyCode::Char('/') => {
                    execute!(stdout, cursor::Show,).unwrap();
                    let mut search_term = String::new();
                    let _trash_bin = enable_raw_mode().is_ok();
                    let _trash_bin = stdout.write_all(b"/");
                    let _trash_bin = stdout.flush();
                    let _trash_bin = disable_raw_mode().is_ok();
                    if stdin().read_line(&mut search_term).is_err() {
                        return false;
                    };
                    search_term = search_term.replace('\n', "").to_lowercase();
                    if let Some(found_reminder_index) = all_reminders.iter().position(|reminder| {
                        reminder
                            .name()
                            .to_lowercase()
                            .contains(search_term.as_str())
                            || reminder
                                .description()
                                .to_lowercase()
                                .contains(search_term.as_str())
                    }) {
                        *cursor_position = found_reminder_index;
                        return false;
                    }
                    return false;
                }
                KeyCode::Enter => {
                    let Ok(process) = Command::new("vipe")
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .spawn()
                    else {
                        return false;
                    };
                    #[allow(clippy::expect_used)]
                    let _trash_bin = process
                        .stdin
                        .expect("Cant send description to editor")
                        .write_all(selected_reminder.description().as_bytes());
                    let mut new_description = String::new();
                    #[allow(clippy::expect_used)]
                    let _trash_bin = process
                        .stdout
                        .expect("Cant read pipe from editor")
                        .read_to_string(&mut new_description);
                    if request_client
                        .patch(format!(
                            "http://{}:{}/reminders/{}/description",
                            config.network().remote_ip(),
                            config.network().port(),
                            selected_reminder.id()
                        ))
                        .json(&new_description)
                        .send()
                        .is_ok()
                    {
                        return true;
                    }
                    return false;
                }
                KeyCode::Char('+') => {
                    let _trash_bin = stdout.write_all(b"Add duration (1h10m15s): ");
                    execute!(stdout, cursor::Show,).unwrap();
                    let _trash_bin = disable_raw_mode().is_ok();
                    let mut time_input = String::new();
                    let _trash_bin = stdin().read_line(&mut time_input);
                    time_input = time_input.replace('\n', "");

                    let _trash_bin = enable_raw_mode().is_ok();
                    let Ok(parsed_duration) = DurationString::from_string(time_input) else {
                        return false;
                    };
                    let duration: core::time::Duration = parsed_duration.into();
                    if request_client
                        .patch(format!(
                            "http://{}:{}/reminders/{}/push_duration",
                            config.network().remote_ip(),
                            config.network().port(),
                            selected_reminder.id()
                        ))
                        .json(&duration)
                        .send()
                        .is_ok()
                    {
                        return true;
                    }
                    return false;
                }
                KeyCode::Char('-') => {
                    let _trash_bin = stdout.write_all(b"Subtract duration (1h10m15s): ");
                    execute!(stdout, cursor::Show,).unwrap();
                    let _trash_bin = disable_raw_mode().is_ok();
                    let mut time_input = String::new();
                    let _trash_bin = stdin().read_line(&mut time_input);
                    time_input = time_input.replace('\n', "");

                    let _trash_bin = enable_raw_mode().is_ok();
                    let Ok(parsed_duration) = DurationString::from_string(time_input) else {
                        return false;
                    };
                    let duration: core::time::Duration = parsed_duration.into();
                    if request_client
                        .patch(format!(
                            "http://{}:{}/reminders/{}/cut_duration",
                            config.network().remote_ip(),
                            config.network().port(),
                            selected_reminder.id()
                        ))
                        .json(&duration)
                        .send()
                        .is_ok()
                    {
                        return true;
                    }
                    return false;
                }
                KeyCode::Esc => {
                    if request_client
                        .put(format!(
                            "http://{}:{}/reminders/reset_flags",
                            config.network().remote_ip(),
                            config.network().port()
                        ))
                        .send()
                        .is_ok()
                    {
                        return true;
                    }
                    return false;
                }
                _ => return false,
            };
        }
        false
    } else {
        false
    }
}
#[allow(clippy::too_many_lines)]
fn read_re_mode_input(
    stdout: &mut Stdout,
    selected_reminder: &Reminder,
    request_client: &reqwest::blocking::Client,
    config: &Config,
) {
    if let Ok(Event::Key(event)) = read() {
        #[allow(clippy::wildcard_enum_match_arm)]
        match event.code {
            KeyCode::Char('s') => {
                let _ = request_client
                    .put(format!(
                        "http://{}:{}/reminders/{}/restart",
                        config.network().remote_ip(),
                        config.network().port(),
                        selected_reminder.id()
                    ))
                    .send();
            }
            KeyCode::Char('n') => {
                let _trash_bin = stdout.write_all(b"New name: ");
                execute!(stdout, cursor::Show,).unwrap();
                let _trash_bin = disable_raw_mode().is_ok();
                let mut name = String::new();
                let _trash_bin = stdin().read_line(&mut name);
                name = name.replace('\n', "");
                let _ = request_client
                    .patch(format!(
                        "http://{}:{}/reminders/{}/rename",
                        config.network().remote_ip(),
                        config.network().port(),
                        selected_reminder.id()
                    ))
                    .json(&name)
                    .send();
            }
            KeyCode::Char('t') => {
                let _trash_bin = stdout.write_all(
                    b"New end time or date (1h10m | 15:23 | 8.11.2023 | 8.11.2023 15:23): ",
                );
                execute!(stdout, cursor::Show,).unwrap();
                let _trash_bin = disable_raw_mode().is_ok();
                let mut time_input = String::new();
                let _trash_bin = stdin().read_line(&mut time_input);
                time_input = time_input.replace('\n', "");
                let now = OffsetDateTime::now_utc().to_offset(my_local_offset());
                let mut finish_time = OffsetDateTime::now_utc().to_offset(my_local_offset());

                let _trash_bin = enable_raw_mode().is_ok();
                #[allow(clippy::useless_conversion)]
                if time_input.chars().all(|e| e.is_ascii_digit() || e == ':') {
                    let Ok(format) = &format_description::parse("[hour]:[minute]") else {
                        return;
                    };
                    let Ok(new_finish_time) = Time::parse(time_input.as_str(), format) else {
                        return;
                    };
                    finish_time = finish_time.replace_time(new_finish_time);
                    #[allow(clippy::arithmetic_side_effects)]
                    let _ = request_client
                        .patch(format!(
                            "http://{}:{}/reminders/{}/retime",
                            config.network().remote_ip(),
                            config.network().port(),
                            selected_reminder.id()
                        ))
                        .json(&TimeObject {
                            reminder_type: ReminderType::Time,
                            finish_time,
                            duration: finish_time - now,
                        })
                        .send();
                } else if time_input
                    .chars()
                    .all(|e| e.is_ascii_digit() || e == '.' || e == ':' || e == ' ')
                {
                    let Ok(format) =
                        format_description::parse("[day].[month].[year] [hour]:[minute]")
                    else {
                        return;
                    };
                    let Ok(new_finish_date_time) =
                        PrimitiveDateTime::parse(time_input.as_str(), &format)
                    else {
                        return;
                    };
                    finish_time = finish_time.replace_date(new_finish_date_time.date());
                    finish_time = finish_time.replace_time(new_finish_date_time.time());
                    #[allow(clippy::arithmetic_side_effects)]
                    let _ = request_client
                        .patch(format!(
                            "http://{}:{}/reminders/{}/retime",
                            config.network().remote_ip(),
                            config.network().port(),
                            selected_reminder.id()
                        ))
                        .json(&TimeObject {
                            reminder_type: ReminderType::Date,
                            finish_time,
                            duration: finish_time - now,
                        })
                        .send();
                } else {
                    let Ok(parsed_duration) = DurationString::from_string(time_input) else {
                        return;
                    };
                    let d: core::time::Duration = parsed_duration.into();
                    let Ok(duration) = d.try_into() else {
                        return;
                    };
                    #[allow(clippy::arithmetic_side_effects)]
                    let _ = request_client
                        .patch(format!(
                            "http://{}:{}/reminders/{}/retime",
                            config.network().remote_ip(),
                            config.network().port(),
                            selected_reminder.id()
                        ))
                        .json(&TimeObject {
                            reminder_type: ReminderType::Duration,
                            finish_time: now + d,
                            duration,
                        })
                        .send();
                }
            }
            _ => (),
        }
    }
}
