import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { VoiceOverlay } from './VoiceOverlay';
import type { VoiceOverlayModel } from './voiceOverlayModel';

function model(overrides: Partial<VoiceOverlayModel>): VoiceOverlayModel {
  return {
    mode: 'idle',
    title: '已就绪',
    detail: '按住快捷键开始说话',
    level: 0,
    transcriptPreview: null,
    diagnosticHint: false,
    ...overrides,
  };
}

describe('VoiceOverlay', () => {
  it('renders recording state as an active voice input overlay', () => {
    render(<VoiceOverlay model={model({ mode: 'recording', title: '正在听', detail: '说完后松开快捷键', level: 0.8 })} />);

    const overlay = screen.getByRole('status', { name: '语音输入状态：正在听' });
    expect(overlay).toHaveAttribute('data-mode', 'recording');
    expect(screen.getByText('说完后松开快捷键')).toBeInTheDocument();
    expect(screen.getByTestId('voice-wave')).toHaveAttribute('aria-hidden', 'true');
    expect(document.querySelectorAll('.voice-wave-bar')).toHaveLength(24);
  });

  it('renders transcribing state', () => {
    render(<VoiceOverlay model={model({ mode: 'transcribing', title: '转写中', detail: '正在本地识别语音' })} />);

    expect(screen.getByRole('status', { name: '语音输入状态：转写中' })).toHaveAttribute('data-mode', 'transcribing');
  });

  it('renders transcript preview after success', () => {
    render(<VoiceOverlay model={model({ mode: 'succeeded', title: '已上屏', detail: '语音输入完成', transcriptPreview: '你好，VoxType' })} />);

    expect(screen.getByText('你好，VoxType')).toBeInTheDocument();
  });

  it('renders diagnostic hint on failure', () => {
    render(<VoiceOverlay model={model({ mode: 'failed', title: '需要处理', detail: '模型不存在', diagnosticHint: true })} />);

    expect(screen.getByRole('status', { name: '语音输入状态：需要处理' })).toHaveAttribute('data-mode', 'failed');
    expect(screen.getByText('打开诊断模式查看原因')).toBeInTheDocument();
  });
});
