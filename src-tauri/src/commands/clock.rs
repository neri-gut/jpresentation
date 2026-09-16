use tauri::{AppHandle, Emitter, State};

use crate::domain::clock::{ClockArmDto, ClockSnapshot};
use crate::domain::output::TIMER_CHANGED;
use crate::error::AppErrorDto;
use crate::state::AppState;

fn emit_clock(app: &AppHandle, snapshot: &ClockSnapshot) {
    let _ = app.emit(TIMER_CHANGED, snapshot);
}

/// Arms a single part. Does not start counting and does not touch stage media.
#[tauri::command]
pub fn clock_arm(
    app: AppHandle,
    state: State<'_, AppState>,
    payload: ClockArmDto,
) -> Result<ClockSnapshot, AppErrorDto> {
    let snapshot = {
        let mut output = state.lock_output().map_err(AppErrorDto::from)?;
        output
            .clock
            .arm(&payload.title, payload.minutes)
            .map_err(AppErrorDto::from)?;
        output.clock.snapshot()
    };
    emit_clock(&app, &snapshot);
    Ok(snapshot)
}

/// Starts or resumes the armed part.
#[tauri::command]
pub fn clock_start(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<ClockSnapshot, AppErrorDto> {
    let snapshot = {
        let mut output = state.lock_output().map_err(AppErrorDto::from)?;
        output.clock.start().map_err(AppErrorDto::from)?;
        output.clock.snapshot()
    };
    emit_clock(&app, &snapshot);
    Ok(snapshot)
}

/// Pauses a running part; elapsed is kept.
#[tauri::command]
pub fn clock_pause(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<ClockSnapshot, AppErrorDto> {
    let snapshot = {
        let mut output = state.lock_output().map_err(AppErrorDto::from)?;
        output.clock.pause().map_err(AppErrorDto::from)?;
        output.clock.snapshot()
    };
    emit_clock(&app, &snapshot);
    Ok(snapshot)
}

/// Clears the current part. Does not arm a next row in this change.
#[tauri::command]
pub fn clock_finish(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<ClockSnapshot, AppErrorDto> {
    let snapshot = {
        let mut output = state.lock_output().map_err(AppErrorDto::from)?;
        output.clock.finish().map_err(AppErrorDto::from)?;
        output.clock.snapshot()
    };
    emit_clock(&app, &snapshot);
    Ok(snapshot)
}
