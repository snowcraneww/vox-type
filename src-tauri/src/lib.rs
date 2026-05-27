pub mod asr;
pub mod audio;
pub mod config;
pub mod error;
pub mod hotkey;
pub mod insertion;
pub mod recorder;
pub mod state;
pub mod tray;

use asr::{AsrEngine, MockAsrEngine};
use config::AppConfig;
use insertion::{ClipboardInsertion, InsertionStrategy, MockInsertion};
use recorder::RecorderManager;
use state::AppStatus;
use tauri::State;

#[tauri::command]
fn get_config() -> AppConfig {
    AppConfig::default()
}

#[tauri::command]
fn get_status() -> AppStatus {
    AppStatus::idle()
}

#[tauri::command]
fn simulate_dictation() -> Result<AppStatus, error::VoxError> {
    let engine = MockAsrEngine;
    let insertion = MockInsertion;
    let transcript = engine.transcribe(&[])?;
    insertion.insert_text(&transcript.text)?;
    Ok(AppStatus::succeeded(transcript.text))
}

#[tauri::command]
fn get_default_input_info() -> Result<recorder::RecorderInfo, error::VoxError> {
    recorder::default_input_info()
}

#[tauri::command]
fn start_recording(
    recorder: State<'_, RecorderManager>,
) -> Result<recorder::RecorderRuntimeStatus, error::VoxError> {
    recorder.start()
}

#[tauri::command]
fn stop_recording(
    recorder: State<'_, RecorderManager>,
) -> Result<recorder::RecordedAudio, error::VoxError> {
    recorder.stop()
}

#[tauri::command]
fn get_recording_status(
    recorder: State<'_, RecorderManager>,
) -> Result<recorder::RecorderRuntimeStatus, error::VoxError> {
    recorder.status()
}

#[tauri::command]
fn insert_text_with_clipboard(text: String) -> Result<(), error::VoxError> {
    ClipboardInsertion.insert_text(&text)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(RecorderManager::default())
        .setup(|app| {
            tray::setup_tray(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            get_status,
            simulate_dictation,
            get_default_input_info,
            start_recording,
            stop_recording,
            get_recording_status,
            insert_text_with_clipboard
        ])
        .run(tauri::generate_context!())
        .expect("failed to run VoxType");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulate_dictation_returns_success_status() {
        let status = simulate_dictation().expect("mock dictation should work");
        assert_eq!(status.phase, state::AppPhase::Succeeded);
        assert_eq!(
            status.last_transcript.as_deref(),
            Some("这是 VoxType 的模拟转写结果。")
        );
    }
}
