import { render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { DictationOverlay } from './DictationOverlay';
import type { PushToTalkPayload } from './tauriClient';

let pushToTalkHandler: ((payload: PushToTalkPayload) => void) | null = null;

vi.mock('./tauriClient', async (importOriginal) => {
  const actual = await importOriginal<typeof import('./tauriClient')>();
  return {
    ...actual,
    isTauriRuntime: () => true,
    listenToPushToTalk: vi.fn().mockImplementation((handler: (payload: PushToTalkPayload) => void) => {
      pushToTalkHandler = handler;
      return Promise.resolve(() => { pushToTalkHandler = null; });
    }),
  };
});

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
    expect(document.querySelector('.wave-ripple')).toHaveAttribute('data-theme', 'light-green');
    expect(document.querySelector('.wave-ripple')).toHaveAttribute('data-mode', 'recording');
    expect(document.querySelectorAll('.soundwave-bar')).toHaveLength(20);
    expect(document.querySelector('.ripple-capsule')).toBeInTheDocument();
    expect(document.querySelectorAll('.mini-wave-bar')).toHaveLength(0);
  });

  it('keeps the capsule inside the overlay viewport so the border is not clipped', () => {
    render(<DictationOverlay initialPayload={payload({ state: 'pressed', action: 'startRecording' })} />);

    expect(document.querySelector('.wave-ripple-svg')).toHaveAttribute('viewBox', '0 0 132 44');
    expect(document.querySelector('.ripple-capsule')).toHaveAttribute('x', '6.5');
    expect(document.querySelector('.ripple-capsule')).toHaveAttribute('y', '6.5');
    expect(document.querySelector('.ripple-capsule')).toHaveAttribute('height', '31');
  });

  it('renders transcribing dots after push-to-talk release', () => {
    render(<DictationOverlay initialPayload={payload({ state: 'released', action: 'stopAndTranscribe' })} />);

    expect(screen.getByRole('status', { name: '桌面语音输入状态：正在识别' })).toBeInTheDocument();
    expect(document.querySelector('.wave-ripple')).toHaveAttribute('data-theme', 'light-green');
    expect(document.querySelector('.wave-ripple')).toHaveAttribute('data-mode', 'transcribing');
    expect(document.querySelectorAll('.transcribing-dot')).toHaveLength(6);
    expect(document.querySelectorAll('.ripple-line')).toHaveLength(0);
  });

  it('treats toggle start as active recording', () => {
    render(<DictationOverlay initialPayload={payload({ state: 'pressed', action: 'toggleStartRecording' })} />);

    expect(screen.getByRole('status', { name: '桌面语音输入状态：正在录音' })).toBeInTheDocument();
    expect(document.querySelector('.wave-ripple')).toHaveAttribute('data-mode', 'recording');
  });

  it('renders transcribing dots after toggle stop', () => {
    render(<DictationOverlay initialPayload={payload({ state: 'pressed', action: 'toggleStopAndTranscribe' })} />);

    expect(screen.getByRole('status', { name: '桌面语音输入状态：正在识别' })).toBeInTheDocument();
    expect(document.querySelector('.wave-ripple')).toHaveAttribute('data-mode', 'transcribing');
    expect(document.querySelectorAll('.transcribing-dot')).toHaveLength(6);
  });

  it('ignores key release events that do not change dictation state', async () => {
    render(<DictationOverlay initialPayload={payload({ state: 'pressed', action: 'toggleStartRecording' })} />);

    expect(document.querySelector('.wave-ripple')).toHaveAttribute('data-mode', 'recording');

    await waitFor(() => expect(pushToTalkHandler).not.toBeNull());
    pushToTalkHandler?.(payload({ state: 'released', action: 'ignore' }));

    expect(document.querySelector('.wave-ripple')).toHaveAttribute('data-mode', 'recording');
  });
});
