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
