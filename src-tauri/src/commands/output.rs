use tauri::State;

use crate::domain::output::OutputBundleDto;
use crate::error::AppErrorDto;
use crate::state::AppState;

/// Returns the current stage and clock so a surface can resync without mutating output.
#[tauri::command]
pub fn output_get(state: State<'_, AppState>) -> Result<OutputBundleDto, AppErrorDto> {
    let output = state.lock_output().map_err(AppErrorDto::from)?;
    Ok(OutputBundleDto {
        stage: output.stage.clone(),
        clock: output.clock.snapshot(),
        speaker_mode: output.speaker_mode,
    })
}
