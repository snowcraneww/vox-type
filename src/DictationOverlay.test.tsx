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
  it('renders a compact bottom overlay for active recording', () => {
    render(<DictationOverlay initialPayload={payload({ state: 'pressed', action: 'startRecording' })} />);

    expect(screen.getByRole('status', { name: '桌面语音输入状态：正在听' })).toBeInTheDocument();
    expect(screen.getByText('松开 Ctrl+Alt+Space 自动转写并上屏')).toBeInTheDocument();
    expect(document.querySelectorAll('.mini-wave-bar')).toHaveLength(18);
  });

  it('renders transcribing copy after release', () => {
    render(<DictationOverlay initialPayload={payload({ state: 'released', action: 'stopAndTranscribe' })} />);

    expect(screen.getByRole('status', { name: '桌面语音输入状态：正在识别' })).toBeInTheDocument();
  });
});
