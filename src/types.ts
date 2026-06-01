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
  peakAmplitude: number;
  rmsAmplitude: number;
  asrSamples: number[];
  asrSampleRate: number;
  asrSampleCount: number;
  asrDurationMs: number;
}

export interface Transcript {
  text: string;
  engine: string;
}

export type InputModeId = 'push-to-talk' | 'toggle-dictation' | 'manual';
export type TranscriptionModelId = 'local-whisper' | 'baidu-short' | 'baidu-realtime';

export interface ModeModelPreferences {
  pushToTalkModel: TranscriptionModelId;
  toggleDictationModel: TranscriptionModelId;
}

export interface ModelReadiness {
  id: TranscriptionModelId;
  label: string;
  ready: boolean;
  message: string;
  availableInV7: boolean;
}

export interface TranscriptRecord {
  id: number;
  time: string;
  text: string;
  durationMs: number;
  inputMode: InputModeId;
  modelId: TranscriptionModelId;
  charCount: number;
}

export interface TranscriptStats {
  count: number;
  totalDurationMs: number;
  totalChars: number;
  charsPerMinute: number;
}

export interface LiveTranscriptionChunk {
  transcript: Transcript;
  fromSampleIndex: number;
  toSampleIndex: number;
  asrSampleCount: number;
}

export interface AsrConfig {
  whisperBinaryPath: string | null;
  whisperModelPath: string | null;
  language: string;
}

export interface AsrConfigStatus extends AsrConfig {
  binaryConfigured: boolean;
  modelConfigured: boolean;
  binaryExists: boolean;
  modelExists: boolean;
  ready: boolean;
  source: string;
  message: string;
}

export interface UserPreferences {
  selectedInputDeviceName: string | null;
  pushToTalkHotkey: string | null;
  toggleDictationHotkey: string | null;
  pushToTalkModel: TranscriptionModelId | null;
  toggleDictationModel: TranscriptionModelId | null;
}

export interface HotkeyRegistrationStatus {
  accelerator: string;
  registered: boolean;
  message: string;
}

export interface CloudAsrConfig {
  provider: string;
  groupId: string | null;
  baseUrl: string | null;
  model: string | null;
  language: string | null;
  baiduCuid: string | null;
  baiduFormat: string | null;
  baiduSampleRate: number | null;
  baiduLmId: string | null;
  baiduRealtimeEndpoint: string | null;
  baiduRealtimeDevPid: string | null;
  baiduRealtimeCuid: string | null;
  baiduRealtimeFormat: string | null;
  baiduRealtimeSampleRate: number | null;
  baiduRealtimeUser: string | null;
}

export interface CloudAsrConfigStatus {
  config: CloudAsrConfig;
  apiKeyConfigured: boolean;
  apiKeySource: string;
  apiKeyPreview: string | null;
  secretKeyConfigured: boolean;
  secretKeySource: string;
  secretKeyPreview: string | null;
  ready: boolean;
  message: string;
}
