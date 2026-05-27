pub mod asr;
pub mod asr_config;
pub mod asr_installer;
pub mod audio;
pub mod config;
pub mod error;
pub mod hotkey;
pub mod insertion;
pub mod preferences;
pub mod recorder;
pub mod state;
pub mod tray;

use asr::{AsrEngine, MockAsrEngine, Transcript, WhisperCppEngine};
use asr_config::{AsrConfig, AsrConfigStatus};
use config::AppConfig;
use insertion::{ClipboardInsertion, InsertionStrategy, MockInsertion};
use preferences::UserPreferences;
use recorder::RecorderManager;
use state::AppStatus;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_global_shortcut::ShortcutState;

type HotkeyStatusState = Mutex<hotkey::HotkeyRegistrationStatus>;

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
fn list_input_devices() -> Result<Vec<recorder::RecorderInfo>, error::VoxError> {
    recorder::list_input_devices()
}

#[tauri::command]
fn set_input_device(
    app: AppHandle,
    recorder: State<'_, RecorderManager>,
    device_name: Option<String>,
) -> Result<recorder::RecorderInfo, error::VoxError> {
    let info = recorder.set_input_device(device_name.clone())?;
    preferences::save_user_preferences(
        app_config_dir(&app)?,
        UserPreferences {
            selected_input_device_name: device_name,
        },
    )?;
    Ok(info)
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
fn get_user_preferences(app: AppHandle) -> Result<UserPreferences, error::VoxError> {
    Ok(preferences::load_user_preferences(app_config_dir(&app)?))
}

#[tauri::command]
fn get_hotkey_status(
    hotkey_status: State<'_, HotkeyStatusState>,
) -> Result<hotkey::HotkeyRegistrationStatus, error::VoxError> {
    hotkey_status
        .lock()
        .map(|status| status.clone())
        .map_err(|error| error::VoxError::Config(format!("读取全局快捷键状态失败：{error}")))
}

#[tauri::command]
fn get_asr_config_status(app: AppHandle) -> Result<AsrConfigStatus, error::VoxError> {
    let config_dir = app_config_dir(&app)?;
    Ok(asr_config::get_asr_config_status(config_dir))
}

#[tauri::command]
fn save_asr_config(app: AppHandle, config: AsrConfig) -> Result<AsrConfigStatus, error::VoxError> {
    let config_dir = app_config_dir(&app)?;
    asr_config::save_asr_config(config_dir, config)
}

#[tauri::command]
fn install_managed_asr(app: AppHandle) -> Result<AsrConfigStatus, error::VoxError> {
    let app_data_dir = app_data_dir(&app)?;
    let config_dir = app_config_dir(&app)?;
    asr_installer::install_managed_asr(app_data_dir, config_dir)
}

#[tauri::command]
fn transcribe_last_recording(
    app: AppHandle,
    recorder: State<'_, RecorderManager>,
) -> Result<Transcript, error::VoxError> {
    let config = asr_config::load_asr_config(app_config_dir(&app)?);
    let status = asr_config::status_from_config(config.clone(), "runtime".to_string());
    if !status.ready {
        return Err(error::VoxError::Config(status.message));
    }
    let samples = recorder.last_asr_samples()?;
    let engine = WhisperCppEngine {
        binary_path: config
            .whisper_binary_path
            .expect("ready config has binary path"),
        model_path: config
            .whisper_model_path
            .expect("ready config has model path"),
        language: config.language,
    };
    engine.transcribe(&samples)
}

#[tauri::command]
fn transcribe_last_recording_and_insert(
    app: AppHandle,
    recorder: State<'_, RecorderManager>,
) -> Result<Transcript, error::VoxError> {
    let transcript = transcribe_last_recording(app, recorder)?;
    ClipboardInsertion.insert_text(&transcript.text)?;
    Ok(transcript)
}

#[tauri::command]
fn export_last_recording_wav(
    app: AppHandle,
    recorder: State<'_, RecorderManager>,
) -> Result<String, error::VoxError> {
    let recording = recorder.last_recording()?;
    let export_dir = app_data_dir(&app)?.join("diagnostics");
    std::fs::create_dir_all(&export_dir)
        .map_err(|error| error::VoxError::Recorder(format!("创建诊断录音目录失败：{error}")))?;
    let path = export_dir.join("last-asr-input.wav");
    asr::write_pcm_wav(&path, &recording.asr_samples)?;
    Ok(path.to_string_lossy().to_string())
}

fn app_config_dir(app: &AppHandle) -> Result<std::path::PathBuf, error::VoxError> {
    app.path()
        .app_config_dir()
        .map_err(|error| error::VoxError::Config(format!("读取应用配置目录失败：{error}")))
}

fn app_data_dir(app: &AppHandle) -> Result<std::path::PathBuf, error::VoxError> {
    app.path()
        .app_data_dir()
        .map_err(|error| error::VoxError::Config(format!("读取应用数据目录失败：{error}")))
}

#[tauri::command]
fn insert_text_with_clipboard(text: String) -> Result<(), error::VoxError> {
    ClipboardInsertion.insert_text(&text)
}

fn setup_push_to_talk_hotkey(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::{Arc, Mutex};

    let state = Arc::new(Mutex::new(hotkey::PushToTalkState::default()));
    let accelerator = hotkey::HotkeyBinding::default().accelerator;
    let handler_state = Arc::clone(&state);

    app.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_shortcuts([accelerator.as_str()])?
            .with_handler(move |app, _shortcut, event| {
                let hotkey_event = match event.state() {
                    ShortcutState::Pressed => hotkey::PushToTalkEvent::Pressed,
                    ShortcutState::Released => hotkey::PushToTalkEvent::Released,
                };
                let action = handler_state
                    .lock()
                    .map(|mut state| state.handle_event(hotkey_event))
                    .unwrap_or(hotkey::PushToTalkAction::Ignore);
                let payload = hotkey::payload_for_event(hotkey_event, action);
                let _ = app.emit("voxtype-push-to-talk", payload);
            })
            .build(),
    )?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(RecorderManager::default())
        .manage(Mutex::new(hotkey::HotkeyRegistrationStatus::failed(
            hotkey::HotkeyBinding::default().accelerator,
            "全局快捷键尚未初始化".to_string(),
        )))
        .setup(|app| {
            let accelerator = hotkey::HotkeyBinding::default().accelerator;
            let next_status = match setup_push_to_talk_hotkey(app.handle()) {
                Ok(()) => hotkey::HotkeyRegistrationStatus::registered(accelerator),
                Err(error) => {
                    eprintln!("failed to register push-to-talk hotkey: {error}");
                    hotkey::HotkeyRegistrationStatus::failed(accelerator, error.to_string())
                }
            };
            if let Some(state) = app.try_state::<HotkeyStatusState>() {
                if let Ok(mut status) = state.lock() {
                    *status = next_status;
                }
            }
            tray::setup_tray(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            get_status,
            simulate_dictation,
            get_default_input_info,
            list_input_devices,
            set_input_device,
            start_recording,
            stop_recording,
            get_recording_status,
            get_user_preferences,
            get_hotkey_status,
            get_asr_config_status,
            save_asr_config,
            install_managed_asr,
            transcribe_last_recording,
            transcribe_last_recording_and_insert,
            export_last_recording_wav,
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
