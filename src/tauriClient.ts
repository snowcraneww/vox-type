import { invoke } from '@tauri-apps/api/core';
import type { AppConfig, AppStatus, RecorderInfo } from './types';

export async function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>('get_config');
}

export async function getStatus(): Promise<AppStatus> {
  return invoke<AppStatus>('get_status');
}

export async function getDefaultInputInfo(): Promise<RecorderInfo> {
  return invoke<RecorderInfo>('get_default_input_info');
}

export async function simulateDictation(): Promise<AppStatus> {
  return invoke<AppStatus>('simulate_dictation');
}

export async function insertTextWithClipboard(text: string): Promise<void> {
  return invoke('insert_text_with_clipboard', { text });
}

export function isTauriRuntime(): boolean {
  return '__TAURI_INTERNALS__' in window;
}
