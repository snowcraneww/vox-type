export type AppPhase = 'idle' | 'recording' | 'transcribing' | 'inserting' | 'succeeded' | 'failed';

export interface AppConfig {
  hotkey: string;
  language: string;
  asrEngine: string;
  insertionStrategy: string;
  showStatusIndicator: boolean;
}

export interface AppStatus {
  phase: AppPhase;
  message: string;
  lastTranscript: string | null;
}

export interface RecorderInfo {
  deviceName: string;
  sampleRate: number;
  channels: number;
}

export interface RecorderRuntimeStatus {
  state: 'idle' | 'recording';
  sampleRate: number | null;
  channels: number | null;
  sampleCount: number;
  durationMs: number;
}

export interface RecordedAudio {
  samples: number[];
  sampleRate: number;
  channels: number;
  sampleCount: number;
  durationMs: number;
  asrSampleRate: number;
  asrSampleCount: number;
  asrDurationMs: number;
}
