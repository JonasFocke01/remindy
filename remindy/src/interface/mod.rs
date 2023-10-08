pub mod status;
use status::build_status_box;
pub mod key_reader;
use key_reader::read_input;
pub mod reminders;
use reminders::build_reminder_list;
pub mod past_event;
use past_event::PastEvent;
use std::{
    cmp::Ordering,
    io::{Stdout, Write},
    sync::{Arc, Mutex},
};

use crossterm::{
    cursor, execute,
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use time::OffsetDateTime;

use crate::{
    api::ApiStatus,
    reminder::{Reminder, OFFSET},
    root_path, REMINDER_DB_FILE,
};

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
    PauseReminder,
    None,
}

impl InputAction {
    #[allow(clippy::too_many_lines)]
    fn perform(
        &self,
        stdout: &mut Stdout,
        reminders: &Arc<Mutex<Vec<Reminder>>>,
        cursor_position: &mut usize,
        last_event: &mut PastEvent,
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
                    *last_event = PastEvent::ReminderCreated(reminder.clone());
                }
            }
            InputAction::AttemptReminderRestart => {
                if let Ok(mut reminders) = reminders.lock() {
                    let Some(reminder) = reminders.get_mut(*cursor_position) else {
                        return;
                    };
                    if reminder.restart_flag() {
                        *last_event = PastEvent::ReminderEdited(reminder.clone());
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
                    *last_event = PastEvent::ReminderEdited(reminder.clone());
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
                    *last_event = PastEvent::ReminderSnooze(reminder.clone());
                }
            }
            InputAction::AttemptReminderDelete => {
                if let Ok(mut reminders) = reminders.lock() {
                    let Some(reminder) = reminders.get_mut(*cursor_position) else {
                        return;
                    };
                    if reminder.delete_flag() {
                        *last_event = PastEvent::ReminderDeleted(reminder.clone());
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
                    *last_event = PastEvent::ReminderEdited(reminder.clone());
                }
            }
            InputAction::PauseReminder => {
                if let Ok(mut reminders) = reminders.lock() {
                    let Some(reminder) = reminders.get_mut(*cursor_position) else {
                        return;
                    };
                    reminder.toggle_pause();
                    *last_event = PastEvent::ReminderPause(reminder.clone());
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
    let mut last_event = PastEvent::None;
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
            .write_all(build_status_box(api_status, &last_event).as_bytes())
            .is_err()
        {
            return;
        }

        if stdout
            .write_all(build_reminder_list(reminders, cursor_position, &mut last_event).as_bytes())
            .is_err()
        {
            return;
        }
        read_input(&mut stdout, &mut last_event).perform(
            &mut stdout,
            reminders,
            &mut cursor_position,
            &mut last_event,
        );
        if let Ok(mut reminders) = reminders.try_lock() {
            for reminder in reminders.iter_mut() {
                reminder.push_back_end_time_if_paused(time::Duration::seconds(1));
            }
            let now = OffsetDateTime::now_utc().to_offset(OFFSET);
            if reminders
                .iter()
                .filter(|reminder| reminder.finish_time() < now)
                .collect::<Vec<&Reminder>>()
                .len()
                > 5
                && reminders
                    .last()
                    .map_or(Reminder::default(), Reminder::clone)
                    .finish_time()
                    < now
            {
                let _trash_bin = reminders.pop();
            }
            reminders.sort_by(|a, b| {
                if a.finish_time().cmp(&b.finish_time()) == Ordering::Less {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            });
            Reminder::to_file(
                format!("{}/{REMINDER_DB_FILE}", root_path()).as_str(),
                &reminders,
            );
        }
    }
}
