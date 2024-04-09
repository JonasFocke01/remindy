use crate::DBFile;
use axum::{
    extract::{rejection::JsonRejection, Path, State},
    http::StatusCode,
    Json,
};
use reminder::{
    get_reminder_by_id,
    past_event::PastEvent,
    reminder::{ApiReminder, Reminder, TimeObject},
};
use std::sync::{Arc, Mutex};

// TODO: move `PastEvent` into DBFile
type ApiState = State<(Arc<Mutex<DBFile>>, Arc<Mutex<PastEvent>>)>;

pub async fn get_past_event(State((_, past_event)): ApiState) -> (StatusCode, Json<PastEvent>) {
    if let Ok(past_event) = past_event.lock() {
        (StatusCode::OK, Json(past_event.clone()))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(PastEvent::None))
    }
}

pub async fn all_reminder(State((reminders, _)): ApiState) -> (StatusCode, Json<Vec<Reminder>>) {
    if let Ok(reminders) = reminders.lock() {
        (StatusCode::OK, Json(reminders.reminders.clone()))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![]))
    }
}

pub async fn all_reminder_formatted(
    State((db_file, _)): ApiState,
) -> (StatusCode, Json<Vec<String>>) {
    let mut result: Vec<String> = vec![];
    if let Ok(db_file) = db_file.lock() {
        for reminder in &db_file.reminders {
            let time_left = reminder.remaining_duration();
            let Ok(time_format) = time::format_description::parse("[hour]:[minute]:[second]")
            else {
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![]));
            };
            result.push(format!(
                "\r{}n\r{}",
                reminder,
                if let Some(time_left) = time_left {
                    if time_left.whole_days() > 0 {
                        let Ok(finish_time) = reminder.finish_time().format(&time_format) else {
                            return (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![]));
                        };
                        format!(
                            "                        {}\n\r{}",
                            finish_time,
                            reminder.description()
                        )
                    } else {
                        reminder.description().replace('\n', "\n\r")
                    }
                } else {
                    reminder.description().replace('\n', "\n\r")
                }
            ));
        }
        (StatusCode::OK, Json(result))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![]))
    }
}

pub async fn add_reminder(
    State((db_file, past_event)): ApiState,
    api_reminder: Result<Json<ApiReminder>, JsonRejection>,
) -> StatusCode {
    // TODO: create fancy middleware, that does this boilerplate stuff
    let Ok(mut db_file) = db_file.lock() else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    let Ok(mut past_event) = past_event.lock() else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    let Ok(Json(api_reminder)) = api_reminder else {
        return StatusCode::UNPROCESSABLE_ENTITY;
    };
    let max_id = db_file
        .reminders
        .iter()
        .map(Reminder::id)
        .max()
        .unwrap_or_default();

    let new_id = max_id.saturating_add(1);

    let new_reminder = Reminder::from_api_reminder(new_id, api_reminder);
    db_file.reminders.push(new_reminder.clone());
    *past_event = PastEvent::ReminderCreated(new_reminder.clone());
    print!("\nn ({}) ", new_reminder.name());
    StatusCode::OK
}

pub async fn restart_reminder(
    State((db_file, past_event)): ApiState,
    Path(id): Path<usize>,
) -> StatusCode {
    let Ok(mut db_file) = db_file.lock() else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    let Ok(mut past_event) = past_event.lock() else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    if let Some(reminder) = get_reminder_by_id(&mut db_file.reminders, id) {
        if reminder.restart_flag() {
            *past_event = PastEvent::ReminderEdited(reminder.clone());
            reminder.restart();
        } else {
            reminder.set_restart_flag(true);
        }
        print!("\nrs ({})", reminder.name());
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

pub async fn force_restart_reminder(
    State((db_file, _)): ApiState,
    Path(id): Path<usize>,
) -> StatusCode {
    let Ok(mut db_file) = db_file.lock() else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    if let Some(reminder) = get_reminder_by_id(&mut db_file.reminders, id) {
        reminder.restart();
        print!("\nfrs ({}) ", reminder.name());
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

pub async fn rename_reminder(
    State((db_file, past_event)): ApiState,
    Path(id): Path<usize>,
    name: Result<Json<String>, JsonRejection>,
) -> StatusCode {
    let Ok(mut db_file) = db_file.lock() else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    let Ok(mut past_event) = past_event.lock() else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    let Ok(Json(name)) = name else {
        return StatusCode::UNPROCESSABLE_ENTITY;
    };
    if let Some(reminder) = get_reminder_by_id(&mut db_file.reminders, id) {
        reminder.set_name(name.clone());
        *past_event = PastEvent::ReminderEdited(reminder.clone());
        print!("\nrn ({name}) ");
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

pub async fn reset_reminder_flags(State((db_file, _)): ApiState) -> StatusCode {
    let Ok(mut db_file) = db_file.lock() else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    for reminder in &mut db_file.reminders {
        reminder.set_restart_flag(false);
        reminder.set_delete_flag(false);
    }
    print!("\nrrf ");
    StatusCode::OK
}

pub async fn snooze_reminder(
    State((db_file, past_event)): ApiState,
    Path(id): Path<usize>,
) -> StatusCode {
    let Ok(mut db_file) = db_file.lock() else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    let Ok(mut past_event) = past_event.lock() else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    if let Some(reminder) = get_reminder_by_id(&mut db_file.reminders, id) {
        reminder.snooze();
        *past_event = PastEvent::ReminderSnooze(reminder.clone());
        print!("\ns ({}) ", reminder.name());
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

pub async fn delete_reminder(
    State((db_file, past_event)): ApiState,
    Path(id): Path<usize>,
) -> StatusCode {
    let Ok(mut db_file) = db_file.lock() else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    let Ok(mut past_event) = past_event.lock() else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    let reminders_clone = db_file.reminders.clone();
    if let Some(reminder) = get_reminder_by_id(&mut db_file.reminders, id) {
        print!("\nd ({}) ", reminder.name());
        if reminder.delete_flag() {
            *past_event = PastEvent::ReminderDeleted(reminder.clone());
            let Some(index) = reminders_clone
                .iter()
                .position(|s_reminder| s_reminder.id() == reminder.id())
            else {
                return StatusCode::NOT_FOUND;
            };
            db_file.reminders.remove(index);
        } else {
            reminder.set_delete_flag(true);
        }
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

pub async fn retime_reminder(
    State((db_file, past_event)): ApiState,
    Path(id): Path<usize>,
    retime_object: Result<Json<TimeObject>, JsonRejection>,
) -> StatusCode {
    let Ok(mut db_file) = db_file.lock() else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    let Ok(mut past_event) = past_event.lock() else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    let Ok(Json(retime_object)) = retime_object else {
        return StatusCode::UNPROCESSABLE_ENTITY;
    };
    if let Some(reminder) = get_reminder_by_id(&mut db_file.reminders, id) {
        reminder.set_finish_time(retime_object.finish_time);
        reminder.set_whole_duration(retime_object.duration);
        reminder.set_reminder_type(retime_object.reminder_type.clone());
        *past_event = PastEvent::ReminderEdited(reminder.clone());
        print!("\nrt ({}) ", reminder.name());
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

pub async fn pause_reminder(
    State((db_file, past_event)): ApiState,
    Path(id): Path<usize>,
) -> StatusCode {
    let Ok(mut db_file) = db_file.lock() else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    let Ok(mut past_event) = past_event.lock() else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    if let Some(reminder) = get_reminder_by_id(&mut db_file.reminders, id) {
        reminder.toggle_pause();
        *past_event = PastEvent::ReminderPause(reminder.clone());
        print!("\n' ' ({}) ", reminder.name());
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

pub async fn toggle_reminder_repeat(
    State((db_file, past_event)): ApiState,
    Path(id): Path<usize>,
) -> StatusCode {
    let Ok(mut db_file) = db_file.lock() else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    let Ok(mut past_event) = past_event.lock() else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    if let Some(reminder) = get_reminder_by_id(&mut db_file.reminders, id) {
        if let Some(toggled) = reminder.toggle_repeat() {
            if toggled {
                *past_event = PastEvent::ReminderRepeatToggle(reminder.clone());
            }
        }
        print!("\ne ({}) ", reminder.name());
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

pub async fn push_reminder_duration(
    State((db_file, _)): ApiState,
    Path(id): Path<usize>,
    amount_to_add: Result<Json<core::time::Duration>, JsonRejection>,
) -> StatusCode {
    let Ok(mut db_file) = db_file.lock() else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    let Ok(Json(amount_to_add)) = amount_to_add else {
        return StatusCode::UNPROCESSABLE_ENTITY;
    };
    if let Some(reminder) = get_reminder_by_id(&mut db_file.reminders, id) {
        let Ok(duration): Result<time::Duration, _> = amount_to_add.try_into() else {
            return StatusCode::INTERNAL_SERVER_ERROR;
        };
        reminder.set_finish_time(reminder.finish_time().saturating_add(duration));
        print!("\n+ ({}) ", reminder.name());
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

pub async fn cut_reminder_duration(
    State((db_file, _)): ApiState,
    Path(id): Path<usize>,
    amount_to_subtract: Result<Json<core::time::Duration>, JsonRejection>,
) -> StatusCode {
    let Ok(mut db_file) = db_file.lock() else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    let Ok(Json(amount_to_subtract)) = amount_to_subtract else {
        return StatusCode::UNPROCESSABLE_ENTITY;
    };
    if let Some(reminder) = get_reminder_by_id(&mut db_file.reminders, id) {
        let Ok(duration): Result<time::Duration, _> = amount_to_subtract.try_into() else {
            return StatusCode::INTERNAL_SERVER_ERROR;
        };
        reminder.set_finish_time(reminder.finish_time().saturating_sub(duration));
        print!("\n- ({}) ", reminder.name());
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

pub async fn alter_reminder_description(
    State((db_file, _)): ApiState,
    Path(id): Path<usize>,
    new_description: Result<Json<String>, JsonRejection>,
) -> StatusCode {
    let Ok(mut db_file) = db_file.lock() else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    let Ok(Json(new_description)) = new_description else {
        return StatusCode::UNPROCESSABLE_ENTITY;
    };
    if let Some(reminder) = get_reminder_by_id(&mut db_file.reminders, id) {
        reminder.set_description(new_description.clone());
        print!("\n\\n ({}) ", reminder.name());
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

pub async fn confirm_reminder_finish_event(
    State((db_file, _)): ApiState,
    Path(id): Path<usize>,
) -> StatusCode {
    let Ok(mut db_file) = db_file.lock() else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    if let Some(reminder) = get_reminder_by_id(&mut db_file.reminders, id) {
        reminder.confirm_finish_event();
        print!("\nc ({}) ", reminder.name());
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

pub async fn pop_reminder_history(State((db_file, _)): ApiState) -> StatusCode {
    let Ok(mut db_file) = db_file.lock() else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };

    let current_reminders = db_file.reminders.clone();
    db_file.redoable_history.push(current_reminders);
    if let Some(history_reminders) = db_file.history.pop() {
        db_file.reset_history_on_change = true;
        db_file.reminders = history_reminders;
        return StatusCode::OK;
    };
    StatusCode::INTERNAL_SERVER_ERROR
}

pub async fn pop_reminder_redo_history(State((db_file, _)): ApiState) -> StatusCode {
    let Ok(mut db_file) = db_file.lock() else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };

    let current_reminders = db_file.reminders.clone();
    db_file.history.push(current_reminders);
    if let Some(redo_history_reminders) = db_file.redoable_history.pop() {
        db_file.reset_history_on_change = true;
        db_file.reminders = redo_history_reminders;
        return StatusCode::OK;
    };
    StatusCode::INTERNAL_SERVER_ERROR
}
