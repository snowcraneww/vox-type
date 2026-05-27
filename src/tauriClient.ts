import { invoke } from '@tauri-apps/api/core';
import type { AppConfig, AppStatus, RecordedAudio, RecorderInfo, RecorderRuntimeStatus, Transcript } from './types';

export async function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>('get_config');
}

export async function getStatus(): Promise<AppStatus> {
  return invoke<AppStatus>('get_status');
}

export async function getDefaultInputInfo(): Promise<RecorderInfo> {
  return invoke<RecorderInfo>('get_default_input_info');
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

export async function simulateDictation(): Promise<AppStatus> {
  return invoke<AppStatus>('simulate_dictation');
}

export async function transcribeLastRecording(): Promise<Transcript> {
  return invoke<Transcript>('transcribe_last_recording');
}

export async function insertTextWithClipboard(text: string): Promise<void> {
  return invoke('insert_text_with_clipboard', { text });
}

export function isTauriRuntime(): boolean {
  return '__TAURI_INTERNALS__' in window;
}
