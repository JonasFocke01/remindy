use std::io::{stdin, Stdout, Write};

use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use duration_string::DurationString;
use time::{format_description, Duration, OffsetDateTime, Time};

use crate::reminder::{Reminder, ReminderType, OFFSET};

use super::{past_event::PastEvent, InputAction};

/// This Reads any input detected on the terminal window.
/// This will block when a known key combination is found and there are follow up decisions to make
/// for the user.
/// Otherwise, this blocks for one second and returns.
#[allow(clippy::too_many_lines)]
pub fn read_input(stdout: &mut Stdout, last_event: &mut PastEvent) -> InputAction {
    if poll(std::time::Duration::from_secs(1)).map_or_else(|_| true, |v| v) {
        // TODO: This format should be a const
        let Ok(format) = format_description::parse("[hour]:[minute]") else {
            return InputAction::None;
        };
        #[allow(clippy::single_match, clippy::wildcard_enum_match_arm)]
        if let Ok(Event::Key(event)) = read() {
            return match event.code {
                KeyCode::Char('c') => {
                    if event.modifiers.contains(KeyModifiers::CONTROL) {
                        InputAction::ExitProgram
                    } else {
                        *last_event = PastEvent::WrongInput;
                        InputAction::None
                    }
                }
                KeyCode::Char('n') => {
                    execute!(stdout, cursor::Show,).unwrap();
                    let _trash_bin = disable_raw_mode().is_ok();
                    let mut name = String::new();
                    // Something should print to ask for the input
                    if stdin().read_line(&mut name).is_err() {
                        return InputAction::None;
                    };
                    name = name.replace('\n', "");
                    let mut time_input = String::new();
                    // Something should print to ask for the input
                    if stdin().read_line(&mut time_input).is_err() {
                        return InputAction::None;
                    };
                    time_input = time_input.replace('\n', "");
                    let now = OffsetDateTime::now_utc().to_offset(OFFSET);
                    let mut finish_time = OffsetDateTime::now_utc().to_offset(OFFSET);
                    let reminder_type: ReminderType;
                    #[allow(unused_assignments)]
                    let mut duration: Duration = Duration::new(0, 0);
                    #[allow(clippy::useless_conversion, clippy::arithmetic_side_effects)]
                    if time_input.chars().all(|e| e.is_ascii_digit() || e == ':') {
                        let Ok(new_finish_time) = Time::parse(time_input.as_str(), &format) else {
                            *last_event = PastEvent::WrongInput;
                            return InputAction::None;
                        };
                        finish_time = finish_time.replace_time(new_finish_time);
                        reminder_type = ReminderType::Time;
                        duration = finish_time - now;
                    } else {
                        let Ok(parsed_duration_string) = DurationString::from_string(time_input)
                        else {
                            *last_event = PastEvent::WrongInput;
                            return InputAction::None;
                        };
                        let parsed_duration: core::time::Duration = parsed_duration_string.into();
                        let Ok(parsed_duration) = parsed_duration.try_into() else {
                            *last_event = PastEvent::WrongInput;
                            return InputAction::None;
                        };
                        duration = Duration::from(parsed_duration);
                        finish_time = now + parsed_duration;
                        reminder_type = ReminderType::Duration;
                    }
                    InputAction::NewReminder(Reminder::new(
                        name,
                        reminder_type,
                        duration,
                        finish_time,
                    ))
                }
                KeyCode::Char('r') => read_re_mode_input(stdout),
                KeyCode::Char('k') => InputAction::CursorUp,
                KeyCode::Char('j') => InputAction::CursorDown,
                KeyCode::Char('d') => InputAction::AttemptReminderDelete,
                KeyCode::Char('s') => InputAction::SnoozeReminder,
                KeyCode::Char('l') => {
                    execute!(stdout, cursor::Show,).unwrap();
                    let _trash_bin = disable_raw_mode().is_ok();
                    let Some(library_reminders) = Reminder::from_file("reminders-library.json")
                    else {
                        return InputAction::None;
                    };
                    for (i, reminder) in library_reminders.iter().enumerate() {
                        println!("\r({i:0>2}) {:<10}", reminder.name(),);
                    }
                    print!("\rWhat reminder you want to add: ");
                    let mut index_input = String::new();
                    if stdin().read_line(&mut index_input).is_err() {
                        return InputAction::None;
                    };
                    if let Ok(to_add_index) = index_input.replace('\n', "").parse::<usize>() {
                        let Some(to_add_reminder) = library_reminders.get(to_add_index) else {
                            return InputAction::None;
                        };
                        *last_event = PastEvent::ReminderCreated(to_add_reminder.clone());
                        return InputAction::NewReminder(Reminder::from_library(to_add_reminder));
                    }
                    InputAction::None
                }
                KeyCode::Esc => InputAction::ResetReminderFlags,
                // TODO: pause reminder
                _ => InputAction::None,
            };
        }
    }
    InputAction::None
}
fn read_re_mode_input(stdout: &mut Stdout) -> InputAction {
    if let Ok(Event::Key(event)) = read() {
        #[allow(clippy::wildcard_enum_match_arm)]
        match event.code {
            KeyCode::Char('s') => InputAction::AttemptReminderRestart,
            KeyCode::Char('n') => {
                let _trash_bin = stdout.write_all(b"New name: ");
                execute!(stdout, cursor::Show,).unwrap();
                let _trash_bin = disable_raw_mode().is_ok();
                let mut name = String::new();
                let _trash_bin = stdin().read_line(&mut name);
                name = name.replace('\n', "");
                InputAction::RenameReminder(name)
            }
            KeyCode::Char('t') => {
                let _trash_bin = stdout.write_all(b"New end time (1h10m | 15:23): ");
                execute!(stdout, cursor::Show,).unwrap();
                let _trash_bin = disable_raw_mode().is_ok();
                let mut time_input = String::new();
                let _trash_bin = stdin().read_line(&mut time_input);
                time_input = time_input.replace('\n', "");
                let now = OffsetDateTime::now_utc().to_offset(OFFSET);
                let mut finish_time = OffsetDateTime::now_utc().to_offset(OFFSET);

                let _trash_bin = enable_raw_mode().is_ok();
                #[allow(clippy::useless_conversion)]
                if time_input.chars().all(|e| e.is_ascii_digit() || e == ':') {
                    // TODO: format wants to be a const
                    let Ok(format) = &format_description::parse("[hour]:[minute]") else {
                        return InputAction::None;
                    };
                    let Ok(new_finish_time) = Time::parse(time_input.as_str(), format) else {
                        return InputAction::None;
                    };
                    finish_time = finish_time.replace_time(new_finish_time);
                    #[allow(clippy::arithmetic_side_effects)]
                    InputAction::RetimeReminder(TimeObject {
                        reminder_type: ReminderType::Time,
                        finish_time,
                        duration: finish_time - now,
                    })
                } else {
                    let Ok(parsed_duration) = DurationString::from_string(time_input) else {
                        return InputAction::None;
                    };
                    let d: core::time::Duration = parsed_duration.into();
                    let Ok(duration) = d.try_into() else {
                        return InputAction::None;
                    };
                    InputAction::RetimeReminder(TimeObject {
                        reminder_type: ReminderType::Duration,
                        #[allow(clippy::arithmetic_side_effects)]
                        finish_time: now + d,
                        duration,
                    })
                }
            }
            _ => InputAction::None,
        }
    } else {
        InputAction::None
    }
}
pub struct TimeObject {
    pub reminder_type: ReminderType,
    pub finish_time: OffsetDateTime,
    pub duration: Duration,
}
