pub mod status;
use status::build_status_box;
pub mod key_reader;
use key_reader::read_input;
pub mod reminders;
use reminders::build_reminder_list;
use std::{
    io::Write,
    sync::{Arc, Mutex},
};

use crossterm::{
    cursor, execute,
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use time::{OffsetDateTime, UtcOffset};

use crate::{api::ApiStatus, reminder::Reminder};

use self::key_reader::InputResult;

pub fn start_interface(reminders: Arc<Mutex<Vec<Reminder>>>, api_status: Arc<Mutex<ApiStatus>>) {
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
        stdout
            .write_all(build_status_box(&api_status).as_bytes())
            .unwrap();

        stdout
            .write_all(build_reminder_list(&reminders, cursor_position).as_bytes())
            .unwrap();
        match read_input(&mut stdout) {
            InputResult::ExitProgram => {
                execute!(
                    stdout,
                    cursor::Show,
                    terminal::Clear(terminal::ClearType::All),
                    cursor::MoveTo(0, 0)
                ).unwrap();
                let _trash_bin = disable_raw_mode().is_ok();
                std::process::exit(0);
            }
            InputResult::NewReminder(reminder) => {
                if let Ok(mut reminders) = reminders.lock() {
                    reminders.push(reminder);
                }
            }
            InputResult::AttemptReminderRestart => {
                if let Ok(mut reminders) = reminders.lock() {
                    let reminder = reminders.get_mut(cursor_position).unwrap();
                    if reminder.restart_flag() {
                        reminder.restart()
                    } else {
                        reminder.set_restart_flag(true);
                    }
                }
            }
            InputResult::RenameReminder(name) => {
                if let Ok(mut reminders) = reminders.lock() {
                    let reminder = reminders.get_mut(cursor_position).unwrap();
                    reminder.set_name(name);
                    // TODO: Why true?
                    reminder.set_finish_notifications_send(false);
                }
            }
            InputResult::CursorUp => cursor_position = cursor_position.saturating_sub(1),
            InputResult::CursorDown => {
                if let Ok(reminders) = reminders.lock() {
                    if cursor_position < reminders.len() - 1 {
                        cursor_position = cursor_position.saturating_add(1)
                    }
                }
            }
            InputResult::ResetReminderFlags => {
                if let Ok(mut reminders) = reminders.lock() {
                    for reminder in reminders.iter_mut() {
                        reminder.set_restart_flag(false);
                        reminder.set_delete_flag(false);
                    }
                }
            }
            InputResult::SnoozeReminder => {
                if let Ok(mut reminders) = reminders.lock() {
                    let reminder = reminders.get_mut(cursor_position).unwrap();
                    reminder.snooze();
                }
            }
            InputResult::AttemptReminderDelete => {
                if let Ok(mut reminders) = reminders.lock() {
                    let reminder = reminders.get_mut(cursor_position).unwrap();
                    if reminder.delete_flag() {
                        reminders.remove(cursor_position);
                        if reminders.len() == cursor_position {
                            cursor_position = cursor_position.saturating_sub(1);
                        }
                    } else {
                        reminder.set_delete_flag(true);
                    }
                }
            }
            InputResult::RetimeReminder(retime_object) => {
                if let Ok(mut reminders) = reminders.lock() {
                    let reminder = reminders.get_mut(cursor_position).unwrap();
                    reminder.set_finish_time(retime_object.finish_time);
                    reminder.set_duration(retime_object.duration);
                    reminder.set_reminder_type(retime_object.reminder_type);
                    reminder.set_finish_notifications_send(false);
                }

            }
            InputResult::None => (),
        };
        if let Ok(mut reminders) = reminders.try_lock() {
            let now = OffsetDateTime::now_utc().to_offset(UtcOffset::from_hms(2, 0, 0).unwrap());
            if reminders
                .iter()
                .filter(|reminder| reminder.finish_time() < now)
                .collect::<Vec<&Reminder>>()
                .len()
                > 5
                && reminders.get(0).unwrap().finish_time() < now
            {
                reminders.remove(0);
            }
            Reminder::to_file("reminders.json", reminders.clone());
        }
    }
}
