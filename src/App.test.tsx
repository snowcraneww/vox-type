import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { App } from './App';
import { getOverlayBackendStatus, hideDictationOverlay, insertTextWithClipboard, startRecording, stopRecording, transcribeActiveRecordingChunk, transcribeLastRecording, transcribeLastRecordingChunk } from './tauriClient';
import type { PushToTalkPayload } from './tauriClient';

let pushToTalkHandler: ((payload: PushToTalkPayload) => void) | null = null;

vi.mock('./tauriClient', async (importOriginal) => {
  const actual = await importOriginal<typeof import('./tauriClient')>();
  return {
    ...actual,
    isTauriRuntime: () => true,
    getConfig: vi.fn().mockResolvedValue({ hotkey: 'Ctrl+Alt+Space', language: 'zh-CN', asrEngine: 'whisper.cpp', insertionStrategy: 'clipboard', showStatusIndicator: true }),
    getAsrConfigStatus: vi.fn().mockResolvedValue({ whisperBinaryPath: null, whisperModelPath: null, language: 'zh', binaryConfigured: false, modelConfigured: false, binaryExists: false, modelExists: false, ready: false, source: 'default', message: 'ASR 未就绪' }),
    getDefaultInputInfo: vi.fn().mockResolvedValue({ deviceName: 'Test Microphone', sampleRate: 44100, channels: 1 }),
    getUserPreferences: vi.fn().mockResolvedValue({ selectedInputDeviceName: null }),
    getHotkeyStatus: vi.fn().mockResolvedValue({ accelerator: 'Ctrl+Alt+Space', registered: true, message: '全局快捷键已注册：Ctrl+Alt+Space' }),
    getOverlayBackendStatus: vi.fn().mockResolvedValue({ backend: 'native-win32', lastError: null }),
    listInputDevices: vi.fn().mockResolvedValue([{ deviceName: 'Test Microphone', sampleRate: 44100, channels: 1 }]),
    startRecording: vi.fn().mockResolvedValue({ state: 'recording', sampleRate: 44100, channels: 1, sampleCount: 0, durationMs: 0 }),
    stopRecording: vi.fn().mockResolvedValue({ sampleRate: 44100, channels: 1, sampleCount: 32000, durationMs: 2000, asrSampleRate: 16000, asrSampleCount: 32000, asrDurationMs: 2000, peakAmplitude: 12000, rmsAmplitude: 1600 }),
    transcribeLastRecording: vi.fn().mockResolvedValue({ engine: 'whisper.cpp', text: '测试文本' }),
    transcribeActiveRecordingChunk: vi.fn().mockResolvedValue({ transcript: { engine: 'whisper.cpp', text: '实时片段' }, fromSampleIndex: 0, toSampleIndex: 44100, asrSampleCount: 16000 }),
    transcribeLastRecordingChunk: vi.fn().mockResolvedValue({ transcript: { engine: 'whisper.cpp', text: '尾段' }, fromSampleIndex: 44100, toSampleIndex: 52000, asrSampleCount: 2866 }),
    insertTextWithClipboard: vi.fn().mockResolvedValue(undefined),
    hideDictationOverlay: vi.fn().mockResolvedValue(undefined),
    listenToPushToTalk: vi.fn().mockImplementation((handler: (payload: PushToTalkPayload) => void) => {
      pushToTalkHandler = handler;
      return Promise.resolve(() => { pushToTalkHandler = null; });
    }),
  };
});
describe('App', () => {
  beforeEach(() => {
    pushToTalkHandler = null;
    vi.mocked(startRecording).mockClear();
    vi.mocked(stopRecording).mockClear();
    vi.mocked(transcribeLastRecording).mockClear();
    vi.mocked(transcribeActiveRecordingChunk).mockClear();
    vi.mocked(transcribeLastRecordingChunk).mockClear();
    vi.mocked(insertTextWithClipboard).mockClear();
    vi.mocked(hideDictationOverlay).mockClear();
    vi.mocked(getOverlayBackendStatus).mockClear();
  });
  it('renders the user-facing voice input view by default', () => {
    render(<App />);

    expect(screen.getByText('VoxType')).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: '语音直接变成文字' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '开始录音' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '诊断模式' })).toBeInTheDocument();
    expect(screen.getByRole('status', { name: '语音输入状态：已就绪' })).toBeInTheDocument();
    expect(screen.getByText('需处理')).toBeInTheDocument();
    expect(screen.getByText('whisper.cpp')).toBeInTheDocument();
    expect(screen.queryByRole('heading', { name: '诊断工作台' })).not.toBeInTheDocument();
    expect(document.querySelector('.traffic-lights')).not.toBeInTheDocument();
  });


  it('records global shortcut events in diagnostics', async () => {
    render(<App />);

    await screen.findByText(/Ctrl\+Alt\+Space/);
    pushToTalkHandler?.({ state: 'pressed', action: 'startRecording' });
    fireEvent.click(screen.getByRole('button', { name: '诊断模式' }));

    expect(await screen.findByText('收到全局快捷键按下')).toBeInTheDocument();
  });

  it('runs the shortcut closed loop and hides the desktop overlay after release', async () => {
    render(<App />);

    await screen.findByText(/Ctrl\+Alt\+Space/);
    pushToTalkHandler?.({ state: 'pressed', action: 'startRecording' });

    await waitFor(() => expect(startRecording).toHaveBeenCalledTimes(1));

    pushToTalkHandler?.({ state: 'released', action: 'stopAndTranscribe' });

    await waitFor(() => expect(stopRecording).toHaveBeenCalledTimes(1));
    expect(transcribeLastRecording).toHaveBeenCalledTimes(1);
    expect(insertTextWithClipboard).toHaveBeenCalledWith('测试文本');
    await waitFor(() => expect(hideDictationOverlay).toHaveBeenCalledTimes(1));
  });

  it('runs the toggle shortcut closed loop on the second press', async () => {
    render(<App />);

    await screen.findByText(/Ctrl\+Alt\+Space/);
    pushToTalkHandler?.({ state: 'pressed', action: 'toggleStartRecording' });

    await waitFor(() => expect(startRecording).toHaveBeenCalledTimes(1));

    pushToTalkHandler?.({ state: 'pressed', action: 'toggleStopAndTranscribe' });

    await waitFor(() => expect(stopRecording).toHaveBeenCalledTimes(1));
    expect(transcribeActiveRecordingChunk).not.toHaveBeenCalled();
    expect(transcribeLastRecordingChunk).toHaveBeenCalledWith(0);
    expect(transcribeLastRecording).not.toHaveBeenCalled();
    expect(insertTextWithClipboard).toHaveBeenCalledWith('尾段');
    await waitFor(() => expect(hideDictationOverlay).toHaveBeenCalledTimes(1));
  });

  it('streams an active recording chunk while toggle dictation is recording', async () => {
    render(<App />);
    await screen.findByText(/Ctrl\+Alt\+Space/);

    vi.useFakeTimers();
    try {
      await act(async () => {
        pushToTalkHandler?.({ state: 'pressed', action: 'toggleStartRecording' });
        await Promise.resolve();
      });
      expect(startRecording).toHaveBeenCalledTimes(1);

      await act(async () => {
        await vi.advanceTimersByTimeAsync(4200);
        await Promise.resolve();
      });

      expect(transcribeActiveRecordingChunk).toHaveBeenCalledWith(0);
      expect(insertTextWithClipboard).toHaveBeenCalledWith('实时片段');

      await act(async () => {
        pushToTalkHandler?.({ state: 'pressed', action: 'toggleStopAndTranscribe' });
        await Promise.resolve();
      });

      expect(stopRecording).toHaveBeenCalledTimes(1);
      expect(transcribeLastRecordingChunk).toHaveBeenCalledWith(44100);
      expect(insertTextWithClipboard).toHaveBeenCalledWith('尾段');
    } finally {
      vi.useRealTimers();
    }
  });

  it('opens the diagnostic workbench on request', async () => {
    render(<App />);

    fireEvent.click(screen.getByRole('button', { name: '诊断模式' }));

    expect(screen.getByRole('heading', { name: '诊断工作台' })).toBeInTheDocument();
    expect(screen.getByText('开始录音采集')).toBeInTheDocument();
    expect(screen.getByText('停止录音采集')).toBeInTheDocument();
    expect(screen.getByText('转写最近录音')).toBeInTheDocument();
    expect(screen.getByText('测试剪贴板上屏')).toBeInTheDocument();
    expect(screen.getByText('刷新全局快捷键状态')).toBeInTheDocument();
    expect(screen.getByText('测试桌面浮窗')).toBeInTheDocument();
    expect(await screen.findAllByText('native-win32')).toHaveLength(2);
    expect(screen.getByRole('heading', { name: '诊断日志' })).toBeInTheDocument();
  });
});
