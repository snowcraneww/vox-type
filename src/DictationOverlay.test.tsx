import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { DictationOverlay } from './DictationOverlay';
import type { PushToTalkPayload } from './tauriClient';

function payload(overrides: Partial<PushToTalkPayload>): PushToTalkPayload {
  return {
    state: 'pressed',
    action: 'startRecording',
    ...overrides,
  };
}

describe('DictationOverlay', () => {
  it('renders only flowing ripple lines for active recording', () => {
    render(<DictationOverlay initialPayload={payload({ state: 'pressed', action: 'startRecording' })} />);

    expect(screen.getByRole('status', { name: '桌面语音输入状态：正在录音' })).toBeInTheDocument();
    expect(screen.queryByText('正在听')).not.toBeInTheDocument();
    expect(screen.queryByText('松开 Ctrl+Alt+Space 自动转写并上屏')).not.toBeInTheDocument();
    expect(document.querySelectorAll('.ripple-line')).toHaveLength(4);
    expect(document.querySelector('.ripple-capsule')).toBeInTheDocument();
    expect(document.querySelectorAll('.mini-wave-bar')).toHaveLength(0);
  });

  it('renders transcribing copy after release', () => {
    render(<DictationOverlay initialPayload={payload({ state: 'released', action: 'stopAndTranscribe' })} />);

    expect(screen.getByRole('status', { name: '桌面语音输入状态：正在识别' })).toBeInTheDocument();
    expect(document.querySelector('.wave-ripple')).toHaveAttribute('data-mode', 'transcribing');
    expect(document.querySelectorAll('animate')).toHaveLength(0);
    expect(document.querySelector('.wave-ripple[data-mode="transcribing"] .ripple-line')).toBeInTheDocument();
  });
});
