import { describe, expect, it } from 'vitest';
import { createVoiceOverlayModel } from './voiceOverlayModel';
import type { AppStatus } from './types';

function status(phase: AppStatus['phase'], message = '状态消息', lastTranscript: string | null = null): AppStatus {
  return { phase, message, lastTranscript };
}

describe('createVoiceOverlayModel', () => {
  it('maps recording state to listening copy and active visual mode', () => {
    const model = createVoiceOverlayModel(status('recording', '正在录音'), 0.72);

    expect(model.mode).toBe('recording');
    expect(model.title).toBe('正在听');
    expect(model.detail).toBe('说完后松开快捷键');
    expect(model.level).toBe(0.72);
    expect(model.diagnosticHint).toBe(false);
  });

  it('shows transcript preview after success and trims long text', () => {
    const model = createVoiceOverlayModel(status('succeeded', '已完成', '这是一段很长的语音输入测试文本，用来确认浮窗不会撑破布局。'), 0);

    expect(model.mode).toBe('succeeded');
    expect(model.title).toBe('已上屏');
    expect(model.transcriptPreview).toBe('这是一段很长的语音输入测试文本，用来确认浮窗不会撑破...');
  });

  it('exposes diagnostic hint for failures', () => {
    const model = createVoiceOverlayModel(status('failed', '真实转写失败：模型不存在'), 0.2);

    expect(model.mode).toBe('failed');
    expect(model.title).toBe('需要处理');
    expect(model.detail).toBe('真实转写失败：模型不存在');
    expect(model.diagnosticHint).toBe(true);
  });
});
