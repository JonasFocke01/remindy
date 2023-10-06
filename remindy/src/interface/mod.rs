pub mod status;
use status::build_status_box;
pub mod key_reader;
use key_reader::read_input;
pub mod reminders;
use reminders::build_reminder_list;
use std::{
    io::{Stdout, Write},
    sync::{Arc, Mutex},
};

use crossterm::{
    cursor, execute,
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use time::{OffsetDateTime, UtcOffset};

use crate::{api::ApiStatus, reminder::Reminder};

use self::key_reader::TimeObject;

pub enum InputAction {
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

impl InputAction {
    fn perform(
        &self,
        stdout: &mut Stdout,
        reminders: &Arc<Mutex<Vec<Reminder>>>,
        cursor_position: &mut usize,
    ) {
        match self {
            InputAction::ExitProgram => {
                execute!(
                    stdout,
                    cursor::Show,
                    terminal::Clear(terminal::ClearType::All),
                    cursor::MoveTo(0, 0)
                )
                .unwrap();
                let _trash_bin = disable_raw_mode().is_ok();
                std::process::exit(0);
            }
            InputAction::NewReminder(reminder) => {
                if let Ok(mut reminders) = reminders.lock() {
                    reminders.push(reminder.clone());
                }
            }
            InputAction::AttemptReminderRestart => {
                if let Ok(mut reminders) = reminders.lock() {
                    let Some(reminder) = reminders.get_mut(*cursor_position) else {
                        return;
                    };
                    if reminder.restart_flag() {
                        reminder.restart();
                    } else {
                        reminder.set_restart_flag(true);
                    }
                }
            }
            InputAction::RenameReminder(name) => {
                if let Ok(mut reminders) = reminders.lock() {
                    let Some(reminder) = reminders.get_mut(*cursor_position) else {
                        return;
                    };
                    reminder.set_name(name.clone());
                    reminder.set_finish_notifications_send(false);
                }
            }
            InputAction::CursorUp => *cursor_position = cursor_position.saturating_sub(1),
            InputAction::CursorDown => {
                if let Ok(reminders) = reminders.lock() {
                    if *cursor_position < reminders.len().saturating_sub(1) {
                        *cursor_position = cursor_position.saturating_add(1);
                    }
                }
            }
            InputAction::ResetReminderFlags => {
                if let Ok(mut reminders) = reminders.lock() {
                    for reminder in reminders.iter_mut() {
                        reminder.set_restart_flag(false);
                        reminder.set_delete_flag(false);
                    }
                }
            }
            InputAction::SnoozeReminder => {
                if let Ok(mut reminders) = reminders.lock() {
                    let Some(reminder) = reminders.get_mut(*cursor_position) else {
                        return;
                    };
                    reminder.snooze();
                }
            }
            InputAction::AttemptReminderDelete => {
                if let Ok(mut reminders) = reminders.lock() {
                    let Some(reminder) = reminders.get_mut(*cursor_position) else {
                        return;
                    };
                    if reminder.delete_flag() {
                        reminders.remove(*cursor_position);
                        if reminders.len() == *cursor_position {
                            *cursor_position = cursor_position.saturating_sub(1);
                        }
                    } else {
                        reminder.set_delete_flag(true);
                    }
                }
            }
            InputAction::RetimeReminder(retime_object) => {
                if let Ok(mut reminders) = reminders.lock() {
                    let Some(reminder) = reminders.get_mut(*cursor_position) else {
                        return;
                    };
                    reminder.set_finish_time(retime_object.finish_time);
                    reminder.set_duration(retime_object.duration);
                    reminder.set_reminder_type(retime_object.reminder_type.clone());
                    reminder.set_finish_notifications_send(false);
                }
            }
            InputAction::None => (),
        };
    }
}

#[allow(clippy::module_name_repetitions)]
pub fn start_interface(reminders: &Arc<Mutex<Vec<Reminder>>>, api_status: &Arc<Mutex<ApiStatus>>) {
    let mut cursor_position: usize = 0;
    let mut stdout = std::io::stdout();
    loop {
        let _trash_bin = enable_raw_mode().is_ok();
        execute!(
            stdout,
            cursor::Hide,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )
        .unwrap();
        if stdout
            .write_all(build_status_box(api_status).as_bytes())
            .is_err()
        {
            return;
        }

        if stdout
            .write_all(build_reminder_list(reminders, cursor_position).as_bytes())
            .is_err()
        {
            return;
        }
        read_input(&mut stdout).perform(&mut stdout, reminders, &mut cursor_position);
        if let Ok(mut reminders) = reminders.try_lock() {
            let now = OffsetDateTime::now_utc();
            if let Ok(offset) = UtcOffset::from_hms(2, 0, 0) {
                now.to_offset(offset);
            }
            if reminders
                .iter()
                .filter(|reminder| reminder.finish_time() < now)
                .collect::<Vec<&Reminder>>()
                .len()
                > 5
                && reminders
                    .get(0)
                    .map_or(Reminder::default(), Reminder::clone)
                    .finish_time()
                    < now
            {
                reminders.remove(0);
            }
            Reminder::to_file("reminders.json", &reminders);
        }
    }
}
