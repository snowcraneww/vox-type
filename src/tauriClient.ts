import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { AppConfig, AppStatus, AsrConfig, AudioPreprocessConfig, AsrConfigStatus, BaiduRealtimeResultEvent, BaiduRealtimeSessionStatus, BaiduRealtimeSessionSummary, BuildInfo, CloudAsrConfig, CloudAsrConfigStatus, HotkeyRegistrationStatus, InsertionResult, InsertionStrategy, LiveTranscriptionChunk, RecordedAudio, RecorderInfo, RecorderRuntimeStatus, Transcript, TranscriptPostprocessConfig, PostprocessResult, PersistedTranscriptEntry, SenseVoiceConfig, SenseVoiceConfigStatus, TranscriptionModelId, UserPreferences } from './types';

export interface PushToTalkPayload {
  state: 'pressed' | 'released';
  action: 'startRecording' | 'stopAndTranscribe' | 'toggleStartRecording' | 'toggleStopAndTranscribe' | 'ignore';
}

export interface OverlayBackendStatus {
  backend: string;
  lastError: string | null;
}

export async function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>('get_config');
}

export async function getStatus(): Promise<AppStatus> {
  return invoke<AppStatus>('get_status');
}

export async function getDefaultInputInfo(): Promise<RecorderInfo> {
  return invoke<RecorderInfo>('get_default_input_info');
}

export async function listInputDevices(): Promise<RecorderInfo[]> {
  return invoke<RecorderInfo[]>('list_input_devices');
}

export async function setInputDevice(deviceName: string | null): Promise<RecorderInfo> {
  return invoke<RecorderInfo>('set_input_device', { deviceName });
}

export async function startRecording(): Promise<RecorderRuntimeStatus> {
  return invoke<RecorderRuntimeStatus>('start_recording');
}

export async function stopRecording(): Promise<RecordedAudio> {
  return invoke<RecordedAudio>('stop_recording');
}

export async function getRecordingStatus(): Promise<RecorderRuntimeStatus> {
  return invoke<RecorderRuntimeStatus>('get_recording_status');
}

export async function getUserPreferences(): Promise<UserPreferences> {
  return invoke<UserPreferences>('get_user_preferences');
}

export async function getHotkeyStatus(): Promise<HotkeyRegistrationStatus> {
  return invoke<HotkeyRegistrationStatus>('get_hotkey_status');
}

export async function saveHotkeyPreferences(pushToTalkHotkey: string, toggleDictationHotkey: string): Promise<HotkeyRegistrationStatus> {
  return invoke<HotkeyRegistrationStatus>('save_hotkey_preferences', { pushToTalkHotkey, toggleDictationHotkey });
}

export async function saveModeModelPreferences(pushToTalkModel: TranscriptionModelId, toggleDictationModel: TranscriptionModelId): Promise<UserPreferences> {
  return invoke<UserPreferences>('save_mode_model_preferences', { pushToTalkModel, toggleDictationModel });
}

export async function saveInsertionStrategyPreference(insertionStrategy: InsertionStrategy): Promise<UserPreferences> {
  return invoke<UserPreferences>('save_insertion_strategy_preference', { insertionStrategy });
}

export async function simulateDictation(): Promise<AppStatus> {
  return invoke<AppStatus>('simulate_dictation');
}

export async function transcribeLastRecording(modelId?: TranscriptionModelId): Promise<Transcript> {
  return invoke<Transcript>('transcribe_last_recording', { modelId });
}

export async function transcribeActiveRecordingChunk(fromSampleIndex: number): Promise<LiveTranscriptionChunk> {
  return invoke<LiveTranscriptionChunk>('transcribe_active_recording_chunk', { fromSampleIndex });
}

export async function transcribeLastRecordingChunk(fromSampleIndex: number): Promise<LiveTranscriptionChunk> {
  return invoke<LiveTranscriptionChunk>('transcribe_last_recording_chunk', { fromSampleIndex });
}

export async function transcribeLastRecordingAndInsert(): Promise<Transcript> {
  return invoke<Transcript>('transcribe_last_recording_and_insert');
}

export async function exportLastRecordingWav(): Promise<string> {
  return invoke<string>('export_last_recording_wav');
}


export async function getAudioPreprocessConfig(): Promise<AudioPreprocessConfig> {
  return invoke<AudioPreprocessConfig>('get_audio_preprocess_config');
}

export async function saveAudioPreprocessConfig(config: AudioPreprocessConfig): Promise<AudioPreprocessConfig> {
  return invoke<AudioPreprocessConfig>('save_audio_preprocess_config', { config });
}

export async function getAsrConfigStatus(): Promise<AsrConfigStatus> {
  return invoke<AsrConfigStatus>('get_asr_config_status');
}

export async function getCloudAsrConfigStatus(): Promise<CloudAsrConfigStatus> {
  return invoke<CloudAsrConfigStatus>('get_cloud_asr_config_status');
}

export async function saveCloudAsrConfig(config: CloudAsrConfig): Promise<CloudAsrConfigStatus> {
  return invoke<CloudAsrConfigStatus>('save_cloud_asr_config', { config });
}

export async function saveBaiduAsrApiKey(apiKey: string): Promise<CloudAsrConfigStatus> {
  return invoke<CloudAsrConfigStatus>('save_baidu_asr_api_key', { apiKey });
}

export async function saveBaiduAsrSecretKey(secretKey: string): Promise<CloudAsrConfigStatus> {
  return invoke<CloudAsrConfigStatus>('save_baidu_asr_secret_key', { secretKey });
}

export async function saveAsrConfig(config: AsrConfig): Promise<AsrConfigStatus> {
  return invoke<AsrConfigStatus>('save_asr_config', { config });
}

export async function installManagedAsr(): Promise<AsrConfigStatus> {
  return invoke<AsrConfigStatus>('install_managed_asr');
}

export async function getSenseVoiceConfigStatus(): Promise<SenseVoiceConfigStatus> {
  return invoke<SenseVoiceConfigStatus>('get_sensevoice_config_status');
}

export async function saveSenseVoiceConfig(config: SenseVoiceConfig): Promise<SenseVoiceConfigStatus> {
  return invoke<SenseVoiceConfigStatus>('save_sensevoice_config', { config });
}

export async function installManagedSenseVoice(): Promise<SenseVoiceConfigStatus> {
  return invoke<SenseVoiceConfigStatus>('install_managed_sensevoice');
}


export async function startBaiduRealtimeSession(): Promise<BaiduRealtimeSessionStatus> {
  return invoke<BaiduRealtimeSessionStatus>('start_baidu_realtime_session');
}

export async function finishBaiduRealtimeSession(): Promise<BaiduRealtimeSessionSummary> {
  return invoke<BaiduRealtimeSessionSummary>('finish_baidu_realtime_session');
}

export async function cancelBaiduRealtimeSession(): Promise<BaiduRealtimeSessionStatus> {
  return invoke<BaiduRealtimeSessionStatus>('cancel_baidu_realtime_session');
}

export async function getBaiduRealtimeSessionStatus(): Promise<BaiduRealtimeSessionStatus> {
  return invoke<BaiduRealtimeSessionStatus>('get_baidu_realtime_session_status');
}

export async function insertTextWithClipboard(text: string): Promise<void> {
  return invoke('insert_text_with_clipboard', { text });
}

export async function insertText(text: string, strategy: InsertionStrategy): Promise<InsertionResult> {
  return invoke<InsertionResult>('insert_text', { text, strategy });
}

export async function getBuildInfo(): Promise<BuildInfo> {
  return invoke<BuildInfo>('get_build_info');
}

export async function showDictationOverlay(): Promise<void> {
  return invoke('show_dictation_overlay');
}

export async function showTranscribingOverlay(): Promise<void> {
  return invoke('show_transcribing_overlay');
}

export async function hideDictationOverlay(): Promise<void> {
  return invoke('hide_dictation_overlay');
}

export async function getOverlayBackendStatus(): Promise<OverlayBackendStatus> {
  return invoke<OverlayBackendStatus>('get_overlay_backend_status');
}

export async function listenToPushToTalk(handler: (payload: PushToTalkPayload) => void): Promise<() => void> {
  return listen<PushToTalkPayload>('voxtype-push-to-talk', (event) => handler(event.payload));
}


export async function listenToBaiduRealtimeResults(handler: (payload: BaiduRealtimeResultEvent) => void): Promise<() => void> {
  return listen<BaiduRealtimeResultEvent>('voxtype-baidu-realtime-result', (event) => handler(event.payload));
}

export function isTauriRuntime(): boolean {
  return '__TAURI_INTERNALS__' in window;
}

export async function loadTranscriptHistory(): Promise<PersistedTranscriptEntry[]> {
  return invoke<PersistedTranscriptEntry[]>('load_transcript_history');
}

export async function saveTranscriptHistoryEntry(entry: PersistedTranscriptEntry): Promise<PersistedTranscriptEntry[]> {
  return invoke<PersistedTranscriptEntry[]>('save_transcript_history_entry', { entry });
}

export async function deleteTranscriptHistoryEntry(id: string): Promise<PersistedTranscriptEntry[]> {
  return invoke<PersistedTranscriptEntry[]>('delete_transcript_history_entry', { id });
}

export async function clearTranscriptHistory(): Promise<PersistedTranscriptEntry[]> {
  return invoke<PersistedTranscriptEntry[]>('clear_transcript_history');
}

export async function getTranscriptPostprocessConfig(): Promise<TranscriptPostprocessConfig> {
  return invoke<TranscriptPostprocessConfig>('get_transcript_postprocess_config');
}

export async function saveTranscriptPostprocessConfig(config: TranscriptPostprocessConfig): Promise<TranscriptPostprocessConfig> {
  return invoke<TranscriptPostprocessConfig>('save_transcript_postprocess_config', { config });
}

export async function previewTranscriptPostprocess(text: string): Promise<PostprocessResult> {
  return invoke<PostprocessResult>('preview_transcript_postprocess', { text });
}
