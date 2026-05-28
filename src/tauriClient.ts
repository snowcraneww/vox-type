import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { AppConfig, AppStatus, AsrConfig, AsrConfigStatus, HotkeyRegistrationStatus, LiveTranscriptionChunk, RecordedAudio, RecorderInfo, RecorderRuntimeStatus, Transcript, UserPreferences } from './types';

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

export async function simulateDictation(): Promise<AppStatus> {
  return invoke<AppStatus>('simulate_dictation');
}

export async function transcribeLastRecording(): Promise<Transcript> {
  return invoke<Transcript>('transcribe_last_recording');
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

export async function getAsrConfigStatus(): Promise<AsrConfigStatus> {
  return invoke<AsrConfigStatus>('get_asr_config_status');
}

export async function saveAsrConfig(config: AsrConfig): Promise<AsrConfigStatus> {
  return invoke<AsrConfigStatus>('save_asr_config', { config });
}

export async function installManagedAsr(): Promise<AsrConfigStatus> {
  return invoke<AsrConfigStatus>('install_managed_asr');
}

export async function insertTextWithClipboard(text: string): Promise<void> {
  return invoke('insert_text_with_clipboard', { text });
}

export async function showDictationOverlay(): Promise<void> {
  return invoke('show_dictation_overlay');
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

export function isTauriRuntime(): boolean {
  return '__TAURI_INTERNALS__' in window;
}
