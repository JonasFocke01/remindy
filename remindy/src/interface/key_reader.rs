use std::io::{Stdout, Write};

use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use duration_string::DurationString;
use time::{format_description, Duration, OffsetDateTime, Time, UtcOffset};

use crate::reminder::{Reminder, ReminderType};

pub enum InputResult {
    ExitProgram,
    NewReminder(Reminder),
    AttemptReminderRestart,
    AttemptReminderDelete,
    RenameReminder(String),
    CursorUp,
    CursorDown,
    ResetReminderFlags,
    SnoozeReminder,
    RetimeReminder(TimeObject),
    None,
}
/// This Reads any input detected on the terminal window.
/// This will block when a known key combination is found and there are follow up decisions to make
/// for the user.
/// Otherwise, this blocks for one second and returns.
pub fn read_input(stdout: &mut Stdout) -> InputResult {
    if poll(std::time::Duration::from_secs(1)).unwrap() {
        #[allow(clippy::single_match)]
        match read().unwrap() {
            Event::Key(event) => {
                return match event.code {
                    KeyCode::Char('c') => {
                        if event.modifiers.contains(KeyModifiers::CONTROL) {
                            InputResult::ExitProgram
                        } else {
                            InputResult::None
                        }
                    }
                    KeyCode::Char('n') => {
                        execute!(stdout, cursor::Show,).unwrap();
                        let _trash_bin = disable_raw_mode().is_ok();
                        let mut name = String::new();
                        std::io::stdin().read_line(&mut name).unwrap();
                        name = name.replace('\n', "");

                        let mut time_input = String::new();
                        std::io::stdin().read_line(&mut time_input).unwrap();
                        time_input = time_input.replace('\n', "");
                        let now = OffsetDateTime::now_utc()
                            .to_offset(UtcOffset::from_hms(2, 0, 0).unwrap());
                        let mut finish_time = OffsetDateTime::now_utc()
                            .to_offset(UtcOffset::from_hms(2, 0, 0).unwrap());
                        let reminder_type: ReminderType;
                        let duration: Duration;
                        #[allow(clippy::useless_conversion)]
                        if time_input.chars().all(|e| e.is_ascii_digit() || e == ':') {
                            finish_time = finish_time.replace_time(
                                Time::parse(
                                    time_input.as_str(),
                                    &format_description::parse("[hour]:[minute]").unwrap(),
                                )
                                .unwrap(),
                            );
                            reminder_type = ReminderType::Time;
                            duration = finish_time - now;
                        } else {
                            let d: core::time::Duration =
                                DurationString::from_string(time_input).unwrap().into();
                            duration = Duration::from(d.try_into().unwrap());
                            finish_time = now + d;
                            reminder_type = ReminderType::Duration;
                        }

                        return InputResult::NewReminder(Reminder::new(
                            name,
                            reminder_type,
                            duration,
                            finish_time,
                        ));
                    }
                    KeyCode::Char('r') => match read().unwrap() {
                        Event::Key(event) => match event.code {
                            KeyCode::Char('s') => InputResult::AttemptReminderRestart,
                            KeyCode::Char('n') => {
                                stdout.write_all(b"New name: ").unwrap();
                                execute!(stdout, cursor::Show,).unwrap();
                                let _trash_bin = disable_raw_mode().is_ok();
                                let mut name = String::new();
                                std::io::stdin().read_line(&mut name).unwrap();
                                name = name.replace('\n', "");
                                InputResult::RenameReminder(name)
                            }
                            KeyCode::Char('t') => {
                                stdout.write_all(b"New end time (1h10m | 15:23): ").unwrap();
                                execute!(stdout, cursor::Show,).unwrap();
                                let _trash_bin = disable_raw_mode().is_ok();
                                let mut time_input = String::new();
                                std::io::stdin().read_line(&mut time_input).unwrap();
                                time_input = time_input.replace('\n', "");
                                let now = OffsetDateTime::now_utc()
                                    .to_offset(UtcOffset::from_hms(2, 0, 0).unwrap());
                                let mut finish_time = OffsetDateTime::now_utc()
                                    .to_offset(UtcOffset::from_hms(2, 0, 0).unwrap());

                                let _trash_bin = enable_raw_mode().is_ok();
                                #[allow(clippy::useless_conversion)]
                                if time_input.chars().all(|e| e.is_ascii_digit() || e == ':') {
                                    finish_time = finish_time.replace_time(
                                        Time::parse(
                                            time_input.as_str(),
                                            &format_description::parse("[hour]:[minute]").unwrap(),
                                        )
                                        .unwrap(),
                                    );
                                    InputResult::RetimeReminder(TimeObject {
                                        reminder_type: ReminderType::Time,
                                        finish_time,
                                        duration: finish_time - now,
                                    })
                                } else {
                                    let d: core::time::Duration =
                                        DurationString::from_string(time_input).unwrap().into();
                                    InputResult::RetimeReminder(TimeObject {
                                        reminder_type: ReminderType::Duration,
                                        finish_time: now + d,
                                        duration: Duration::from(d.try_into().unwrap()),
                                    })
                                }
                            }
                            _ => panic!(),
                        },
                        _ => panic!(),
                    },
                    KeyCode::Char('k') => InputResult::CursorUp,
                    KeyCode::Char('j') => InputResult::CursorDown,
                    KeyCode::Char('d') => InputResult::AttemptReminderDelete,
                    KeyCode::Char('s') => InputResult::SnoozeReminder,
                    KeyCode::Esc => InputResult::ResetReminderFlags,
                    // TODO: pause reminder
                    _ => {
                        stdout
                            .write_all(
                                format!("{:?} is a unknown command!\n\r", event.code).as_bytes(),
                            )
                            .unwrap();
                        InputResult::None
                    }
                };
            }
            _ => (),
        }
    }
    InputResult::None
}
pub struct TimeObject {
    pub reminder_type: ReminderType,
    pub finish_time: OffsetDateTime,
    pub duration: Duration,
}
