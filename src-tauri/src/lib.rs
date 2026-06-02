pub mod asr;
pub mod asr_config;
pub mod asr_installer;
pub mod audio;
pub mod baidu_realtime;
pub mod cloud_asr;
pub mod cloud_asr_config;
pub mod config;
pub mod error;
pub mod hotkey;
pub mod insertion;
pub mod native_overlay;
pub mod overlay;
#[cfg(test)]
mod overlay_tests;
pub mod preferences;
pub mod recorder;
pub mod state;
pub mod tray;
pub mod transcript_history;
pub mod text_postprocess;
pub mod audio_quality;

use asr::{AsrEngine, MockAsrEngine, Transcript, WhisperCppEngine};
use asr_config::{AsrConfig, AsrConfigStatus};
use config::AppConfig;
use insertion::{ClipboardInsertion, InsertionStrategy, MockInsertion};
use preferences::UserPreferences;
use recorder::RecorderManager;
use serde::Serialize;
use state::AppStatus;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

type HotkeyStatusState = Mutex<hotkey::HotkeyRegistrationStatus>;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LiveTranscriptionChunk {
    pub transcript: Transcript,
    pub from_sample_index: usize,
    pub to_sample_index: usize,
    pub asr_sample_count: usize,
}

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
    let config_dir = app_config_dir(&app)?;
    let mut preferences = preferences::load_user_preferences(config_dir.clone());
    preferences.selected_input_device_name = device_name;
    preferences::save_user_preferences(config_dir, preferences)?;
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
fn save_hotkey_preferences(
    app: AppHandle,
    hotkey_status: State<'_, HotkeyStatusState>,
    push_to_talk_hotkey: String,
    toggle_dictation_hotkey: String,
) -> Result<hotkey::HotkeyRegistrationStatus, error::VoxError> {
    hotkey::validate_hotkey_pair(&push_to_talk_hotkey, &toggle_dictation_hotkey)
        .map_err(error::VoxError::Config)?;
    let config_dir = app_config_dir(&app)?;
    let previous_preferences = preferences::load_user_preferences(config_dir.clone());
    let previous_bindings = hotkey::runtime_bindings_from_preferences(&previous_preferences).ok();
    let mut next_preferences = previous_preferences.clone();
    next_preferences.push_to_talk_hotkey = Some(push_to_talk_hotkey.trim().to_string());
    next_preferences.toggle_dictation_hotkey = Some(toggle_dictation_hotkey.trim().to_string());
    let next_bindings = hotkey::runtime_bindings_from_preferences(&next_preferences)
        .map_err(error::VoxError::Config)?;

    app.global_shortcut()
        .unregister_all()
        .map_err(|error| error::VoxError::Hotkey(error.to_string()))?;
    let next_status = register_runtime_hotkeys(&app, next_bindings);
    if !next_status.registered {
        if let Some(bindings) = previous_bindings {
            let restore_status = register_runtime_hotkeys(&app, bindings);
            if let Ok(mut status) = hotkey_status.lock() {
                *status = restore_status;
            }
        }
        return Err(error::VoxError::Hotkey(next_status.message.clone()));
    }

    preferences::save_user_preferences(config_dir, next_preferences)?;
    if let Ok(mut status) = hotkey_status.lock() {
        *status = next_status.clone();
    }
    Ok(next_status)
}

#[tauri::command]
fn save_mode_model_preferences(
    app: AppHandle,
    push_to_talk_model: preferences::TranscriptionModelId,
    toggle_dictation_model: preferences::TranscriptionModelId,
) -> Result<UserPreferences, error::VoxError> {
    let config_dir = app_config_dir(&app)?;
    let mut preferences = preferences::load_user_preferences(config_dir.clone());
    preferences.push_to_talk_model = push_to_talk_model;
    preferences.toggle_dictation_model = toggle_dictation_model;
    preferences::save_user_preferences(config_dir, preferences)
}

#[tauri::command]
fn get_cloud_asr_config_status(
    app: AppHandle,
) -> Result<cloud_asr_config::CloudAsrConfigStatus, error::VoxError> {
    Ok(cloud_asr_config::get_cloud_asr_config_status(
        app_config_dir(&app)?,
    ))
}

#[tauri::command]
fn save_cloud_asr_config(
    app: AppHandle,
    config: cloud_asr_config::CloudAsrConfig,
) -> Result<cloud_asr_config::CloudAsrConfigStatus, error::VoxError> {
    cloud_asr_config::save_cloud_asr_config(app_config_dir(&app)?, config)
}

#[tauri::command]
fn save_minimax_api_key(
    app: AppHandle,
    api_key: String,
) -> Result<cloud_asr_config::CloudAsrConfigStatus, error::VoxError> {
    cloud_asr_config::save_minimax_api_key_to_user_env(app_config_dir(&app)?, api_key)
}

#[tauri::command]
fn save_baidu_asr_api_key(
    app: AppHandle,
    api_key: String,
) -> Result<cloud_asr_config::CloudAsrConfigStatus, error::VoxError> {
    cloud_asr_config::save_baidu_asr_api_key_to_user_env(app_config_dir(&app)?, api_key)
}

#[tauri::command]
fn save_baidu_asr_secret_key(
    app: AppHandle,
    secret_key: String,
) -> Result<cloud_asr_config::CloudAsrConfigStatus, error::VoxError> {
    cloud_asr_config::save_baidu_asr_secret_key_to_user_env(app_config_dir(&app)?, secret_key)
}

#[tauri::command]
fn start_baidu_realtime_session(
    app: AppHandle,
    recorder: State<'_, RecorderManager>,
    manager: State<'_, baidu_realtime::BaiduRealtimeSessionManager>,
) -> Result<baidu_realtime::BaiduRealtimeSessionStatus, error::VoxError> {
    let cloud_status = cloud_asr_config::get_cloud_asr_config_status(app_config_dir(&app)?);
    let api_key = cloud_asr_config::baidu_asr_api_key_from_env()
        .ok_or_else(|| error::VoxError::Config("未配置 BAIDU_ASR_API_KEY 环境变量".to_string()))?;
    let config = baidu_realtime::config_from_cloud_status(&cloud_status, api_key)?;
    manager.start(app, &recorder, config)
}

#[tauri::command]
fn finish_baidu_realtime_session(
    recorder: State<'_, RecorderManager>,
    manager: State<'_, baidu_realtime::BaiduRealtimeSessionManager>,
) -> Result<baidu_realtime::BaiduRealtimeSessionSummary, error::VoxError> {
    manager.finish(&recorder)
}

#[tauri::command]
fn cancel_baidu_realtime_session(
    recorder: State<'_, RecorderManager>,
    manager: State<'_, baidu_realtime::BaiduRealtimeSessionManager>,
) -> Result<baidu_realtime::BaiduRealtimeSessionStatus, error::VoxError> {
    manager.cancel(&recorder)
}

#[tauri::command]
fn get_baidu_realtime_session_status(
    manager: State<'_, baidu_realtime::BaiduRealtimeSessionManager>,
) -> Result<baidu_realtime::BaiduRealtimeSessionStatus, error::VoxError> {
    manager.status()
}

#[tauri::command]
fn load_transcript_history(
    app: AppHandle,
) -> Result<Vec<transcript_history::PersistedTranscriptEntry>, error::VoxError> {
    transcript_history::load_transcript_history(app_config_dir(&app)?)
}

#[tauri::command]
fn save_transcript_history_entry(
    app: AppHandle,
    entry: transcript_history::PersistedTranscriptEntry,
) -> Result<Vec<transcript_history::PersistedTranscriptEntry>, error::VoxError> {
    transcript_history::save_transcript_history_entry(app_config_dir(&app)?, entry)
}

#[tauri::command]
fn delete_transcript_history_entry(
    app: AppHandle,
    id: String,
) -> Result<Vec<transcript_history::PersistedTranscriptEntry>, error::VoxError> {
    transcript_history::delete_transcript_history_entry(app_config_dir(&app)?, id)
}

#[tauri::command]
fn clear_transcript_history(
    app: AppHandle,
) -> Result<Vec<transcript_history::PersistedTranscriptEntry>, error::VoxError> {
    transcript_history::clear_transcript_history(app_config_dir(&app)?)
}
#[tauri::command]
fn get_transcript_postprocess_config(
    app: AppHandle,
) -> Result<text_postprocess::TranscriptPostprocessConfig, error::VoxError> {
    Ok(text_postprocess::load_transcript_postprocess_config(app_config_dir(&app)?))
}

#[tauri::command]
fn save_transcript_postprocess_config(
    app: AppHandle,
    config: text_postprocess::TranscriptPostprocessConfig,
) -> Result<text_postprocess::TranscriptPostprocessConfig, error::VoxError> {
    text_postprocess::save_transcript_postprocess_config(app_config_dir(&app)?, config)
}

#[tauri::command]
fn preview_transcript_postprocess(
    app: AppHandle,
    text: String,
) -> Result<text_postprocess::PostprocessResult, error::VoxError> {
    let config = text_postprocess::load_transcript_postprocess_config(app_config_dir(&app)?);
    Ok(text_postprocess::process_transcript(&text, &config))
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
    model_id: Option<preferences::TranscriptionModelId>,
) -> Result<Transcript, error::VoxError> {
    let samples = recorder.last_asr_samples()?;
    let selected_model = model_id.unwrap_or(preferences::TranscriptionModelId::BaiduShort);
    if selected_model == preferences::TranscriptionModelId::BaiduRealtime {
        return Err(error::VoxError::Asr(
            "百度实时 WebSocket API 仅支持连续输入模式，请将连续输入模型设为百度实时 WebSocket API 后使用 Ctrl+Alt+V。".to_string(),
        ));
    }
    if selected_model == preferences::TranscriptionModelId::LocalWhisper {
        return transcribe_with_local_whisper(&app, &samples);
    }
    let cloud_status = cloud_asr_config::get_cloud_asr_config_status(app_config_dir(&app)?);
    if cloud_status.config.provider == "baidu" {
        if !cloud_status.ready {
            return Err(error::VoxError::Config(cloud_status.message));
        }
        let config = cloud_status.config;
        let api_key = cloud_asr_config::baidu_asr_api_key_from_env().ok_or_else(|| {
            error::VoxError::Config("未读取到 BAIDU_ASR_API_KEY 环境变量。".to_string())
        })?;
        let secret_key = cloud_asr_config::baidu_asr_secret_key_from_env().ok_or_else(|| {
            error::VoxError::Config("未读取到 BAIDU_ASR_SECRET_KEY 环境变量。".to_string())
        })?;
        let access_token = cloud_asr::fetch_baidu_access_token(&api_key, &secret_key)?;
        let request = cloud_asr::build_baidu_asr_request(
            config.base_url.as_deref().unwrap_or(""),
            &access_token,
            config.model.as_deref().unwrap_or(""),
            config.baidu_cuid.as_deref().unwrap_or(""),
            config.baidu_format.as_deref().unwrap_or(""),
            config
                .baidu_sample_rate
                .unwrap_or(audio::TARGET_SAMPLE_RATE),
            config.baidu_lm_id.as_deref(),
            &samples,
        )?;
        return cloud_asr::transcribe_with_baidu_short_speech(request);
    }

    transcribe_with_local_whisper(&app, &samples)
}

fn transcribe_with_local_whisper(
    app: &AppHandle,
    samples: &[i16],
) -> Result<Transcript, error::VoxError> {
    let config = asr_config::load_asr_config(app_config_dir(&app)?);
    let status = asr_config::status_from_config(config.clone(), "runtime".to_string());
    if !status.ready {
        return Err(error::VoxError::Config(status.message));
    }
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
fn transcribe_active_recording_chunk(
    app: AppHandle,
    recorder: State<'_, RecorderManager>,
    from_sample_index: usize,
) -> Result<LiveTranscriptionChunk, error::VoxError> {
    let config = asr_config::load_asr_config(app_config_dir(&app)?);
    let status = asr_config::status_from_config(config.clone(), "runtime".to_string());
    if !status.ready {
        return Err(error::VoxError::Config(status.message));
    }
    let (samples, to_sample_index) = recorder.active_asr_samples_from(from_sample_index)?;
    if samples.len() < audio::TARGET_SAMPLE_RATE as usize {
        return Err(error::VoxError::Recorder(
            "录音片段太短，暂不转写".to_string(),
        ));
    }
    let engine = WhisperCppEngine {
        binary_path: config
            .whisper_binary_path
            .expect("ready config has binary path"),
        model_path: config
            .whisper_model_path
            .expect("ready config has model path"),
        language: config.language,
    };
    let transcript = engine.transcribe(&samples)?;
    Ok(LiveTranscriptionChunk {
        transcript,
        from_sample_index,
        to_sample_index,
        asr_sample_count: samples.len(),
    })
}

#[tauri::command]
fn transcribe_last_recording_chunk(
    app: AppHandle,
    recorder: State<'_, RecorderManager>,
    from_sample_index: usize,
) -> Result<LiveTranscriptionChunk, error::VoxError> {
    let config = asr_config::load_asr_config(app_config_dir(&app)?);
    let status = asr_config::status_from_config(config.clone(), "runtime".to_string());
    if !status.ready {
        return Err(error::VoxError::Config(status.message));
    }
    let (samples, to_sample_index) = recorder.last_asr_samples_from(from_sample_index)?;
    if samples.is_empty() {
        return Err(error::VoxError::Recorder(
            "录音尾段没有新增 ASR 样本".to_string(),
        ));
    }
    let engine = WhisperCppEngine {
        binary_path: config
            .whisper_binary_path
            .expect("ready config has binary path"),
        model_path: config
            .whisper_model_path
            .expect("ready config has model path"),
        language: config.language,
    };
    let transcript = engine.transcribe(&samples)?;
    Ok(LiveTranscriptionChunk {
        transcript,
        from_sample_index,
        to_sample_index,
        asr_sample_count: samples.len(),
    })
}

#[tauri::command]
fn transcribe_last_recording_and_insert(
    app: AppHandle,
    recorder: State<'_, RecorderManager>,
) -> Result<Transcript, error::VoxError> {
    let transcript = transcribe_last_recording(app, recorder, None)?;
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
#[tauri::command]
fn show_dictation_overlay(app: AppHandle) -> Result<(), error::VoxError> {
    overlay::show_dictation_overlay(&app)
}

#[tauri::command]
fn show_transcribing_overlay(app: AppHandle) -> Result<(), error::VoxError> {
    overlay::show_transcribing_overlay(&app)
}

#[tauri::command]
fn hide_dictation_overlay(app: AppHandle) -> Result<(), error::VoxError> {
    overlay::hide_dictation_overlay(&app)
}

#[tauri::command]
fn get_overlay_backend_status() -> overlay::OverlayBackendStatus {
    overlay::backend_status()
}

fn register_hotkey_handler(
    app: &AppHandle,
    bindings: hotkey::RuntimeHotkeyBindings,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::{Arc, Mutex};

    let push_to_talk_state = Arc::new(Mutex::new(hotkey::PushToTalkState::default()));
    let toggle_state = Arc::new(Mutex::new(hotkey::ToggleDictationState::default()));
    let push_to_talk_shortcut = bindings
        .push_to_talk
        .parse::<tauri_plugin_global_shortcut::Shortcut>()?;
    let toggle_shortcut = bindings
        .toggle_dictation
        .parse::<tauri_plugin_global_shortcut::Shortcut>()?;
    let push_to_talk_shortcut_id = push_to_talk_shortcut.id();
    let toggle_shortcut_id = toggle_shortcut.id();
    let handler_push_to_talk_state = Arc::clone(&push_to_talk_state);
    let handler_toggle_state = Arc::clone(&toggle_state);

    app.global_shortcut().on_shortcuts(
        [push_to_talk_shortcut, toggle_shortcut],
        move |app, shortcut, event| {
            let hotkey_event = match event.state() {
                ShortcutState::Pressed => hotkey::PushToTalkEvent::Pressed,
                ShortcutState::Released => hotkey::PushToTalkEvent::Released,
            };
            let shortcut_id = shortcut.id();
            let action = if shortcut_id == toggle_shortcut_id {
                match hotkey_event {
                    hotkey::PushToTalkEvent::Pressed => handler_toggle_state
                        .lock()
                        .map(|mut state| state.handle_pressed())
                        .unwrap_or(hotkey::PushToTalkAction::Ignore),
                    hotkey::PushToTalkEvent::Released => handler_toggle_state
                        .lock()
                        .map(|state| state.handle_released())
                        .unwrap_or(hotkey::PushToTalkAction::Ignore),
                }
            } else if shortcut_id == push_to_talk_shortcut_id {
                handler_push_to_talk_state
                    .lock()
                    .map(|mut state| state.handle_event(hotkey_event))
                    .unwrap_or(hotkey::PushToTalkAction::Ignore)
            } else {
                hotkey::PushToTalkAction::Ignore
            };
            let payload = hotkey::payload_for_event(hotkey_event, action);
            match action {
                hotkey::PushToTalkAction::StartRecording
                | hotkey::PushToTalkAction::ToggleStartRecording => {
                    if let Err(error) = overlay::show_dictation_overlay(app) {
                        eprintln!("failed to show dictation overlay: {error}");
                    }
                }
                hotkey::PushToTalkAction::StopAndTranscribe
                | hotkey::PushToTalkAction::ToggleStopAndTranscribe => {
                    if let Err(error) = overlay::show_transcribing_overlay(app) {
                        eprintln!("failed to keep dictation overlay visible: {error}");
                    }
                }
                hotkey::PushToTalkAction::Ignore => {}
            }
            let _ = app.emit("voxtype-push-to-talk", payload);
        },
    )?;

    Ok(())
}

fn register_runtime_hotkeys(
    app: &AppHandle,
    bindings: hotkey::RuntimeHotkeyBindings,
) -> hotkey::HotkeyRegistrationStatus {
    let accelerator = bindings.push_to_talk.clone();
    match register_hotkey_handler(app, bindings) {
        Ok(()) => hotkey::HotkeyRegistrationStatus::registered(accelerator),
        Err(error) => {
            eprintln!("failed to register push-to-talk hotkey: {error}");
            hotkey::HotkeyRegistrationStatus::failed(accelerator, error.to_string())
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(RecorderManager::default())
        .manage(baidu_realtime::BaiduRealtimeSessionManager::default())
        .manage(Mutex::new(hotkey::HotkeyRegistrationStatus::failed(
            hotkey::HotkeyBinding::default().accelerator,
            "全局快捷键尚未初始化".to_string(),
        )))
        .setup(|app| {
            app.handle()
                .plugin(tauri_plugin_global_shortcut::Builder::new().build())?;
            let preferences = preferences::load_user_preferences(app_config_dir(app.handle())?);
            let bindings = hotkey::runtime_bindings_from_preferences(&preferences)
                .map_err(error::VoxError::Config)?;
            let next_status = register_runtime_hotkeys(app.handle(), bindings);
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
            save_hotkey_preferences,
            save_mode_model_preferences,
            load_transcript_history,
            save_transcript_history_entry,
            delete_transcript_history_entry,
            clear_transcript_history,
            get_transcript_postprocess_config,
            save_transcript_postprocess_config,
            preview_transcript_postprocess,
            get_asr_config_status,
            save_asr_config,
            get_cloud_asr_config_status,
            save_cloud_asr_config,
            save_minimax_api_key,
            save_baidu_asr_api_key,
            save_baidu_asr_secret_key,
            start_baidu_realtime_session,
            finish_baidu_realtime_session,
            cancel_baidu_realtime_session,
            get_baidu_realtime_session_status,
            install_managed_asr,
            transcribe_last_recording,
            transcribe_active_recording_chunk,
            transcribe_last_recording_chunk,
            transcribe_last_recording_and_insert,
            export_last_recording_wav,
            insert_text_with_clipboard,
            show_dictation_overlay,
            show_transcribing_overlay,
            hide_dictation_overlay,
            get_overlay_backend_status
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

    #[test]
    fn live_transcription_chunk_records_sample_range() {
        let chunk = LiveTranscriptionChunk {
            transcript: Transcript {
                text: "你好".to_string(),
                engine: "mock".to_string(),
            },
            from_sample_index: 10,
            to_sample_index: 42,
            asr_sample_count: 32,
        };

        assert_eq!(chunk.from_sample_index, 10);
        assert_eq!(chunk.to_sample_index, 42);
        assert_eq!(chunk.transcript.text, "你好");
    }
}
