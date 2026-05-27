import type { AppStatus } from './types';

export type VoiceOverlayMode = 'idle' | 'recording' | 'transcribing' | 'inserting' | 'succeeded' | 'failed';

export interface VoiceOverlayModel {
  mode: VoiceOverlayMode;
  title: string;
  detail: string;
  level: number;
  transcriptPreview: string | null;
  diagnosticHint: boolean;
}

const MAX_TRANSCRIPT_PREVIEW_LENGTH = 26;

function clampLevel(level: number): number {
  if (!Number.isFinite(level)) {
    return 0;
  }
  return Math.max(0, Math.min(1, level));
}

function previewTranscript(text: string | null): string | null {
  if (!text) {
    return null;
  }
  if (text.length <= MAX_TRANSCRIPT_PREVIEW_LENGTH) {
    return text;
  }
  return `${text.slice(0, MAX_TRANSCRIPT_PREVIEW_LENGTH)}...`;
}

export function createVoiceOverlayModel(status: AppStatus, level: number): VoiceOverlayModel {
  const safeLevel = clampLevel(level);
  const base = {
    level: safeLevel,
    transcriptPreview: previewTranscript(status.lastTranscript),
    diagnosticHint: false,
  };

  switch (status.phase) {
    case 'recording':
      return { ...base, mode: 'recording', title: '正在听', detail: '说完后松开快捷键' };
    case 'transcribing':
      return { ...base, mode: 'transcribing', title: '转写中', detail: '正在本地识别语音' };
    case 'inserting':
      return { ...base, mode: 'inserting', title: '上屏中', detail: '正在把文字送到目标输入框' };
    case 'succeeded':
      return { ...base, mode: 'succeeded', title: '已上屏', detail: status.message || '语音输入完成' };
    case 'failed':
      return { ...base, mode: 'failed', title: '需要处理', detail: status.message || '语音输入失败', diagnosticHint: true };
    case 'idle':
    default:
      return { ...base, mode: 'idle', title: '已就绪', detail: '按住快捷键开始说话' };
  }
}
