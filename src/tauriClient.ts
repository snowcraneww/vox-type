import { invoke } from '@tauri-apps/api/core';
import type { AppConfig, AppStatus, AsrConfig, AsrConfigStatus, RecordedAudio, RecorderInfo, RecorderRuntimeStatus, Transcript, UserPreferences } from './types';

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

export async function simulateDictation(): Promise<AppStatus> {
  return invoke<AppStatus>('simulate_dictation');
}

export async function transcribeLastRecording(): Promise<Transcript> {
  return invoke<Transcript>('transcribe_last_recording');
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

export function isTauriRuntime(): boolean {
  return '__TAURI_INTERNALS__' in window;
}