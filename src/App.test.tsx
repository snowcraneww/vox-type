import { act, fireEvent, render, screen, within, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { App } from './App';
import { getOverlayBackendStatus, hideDictationOverlay, insertTextWithClipboard, saveBaiduAsrApiKey, saveBaiduAsrSecretKey, saveCloudAsrConfig, saveModeModelPreferences, showTranscribingOverlay, startBaiduRealtimeSession, finishBaiduRealtimeSession, startRecording, stopRecording, transcribeActiveRecordingChunk, transcribeLastRecording, transcribeLastRecordingChunk } from './tauriClient';
import type { PushToTalkPayload } from './tauriClient';

let pushToTalkHandler: ((payload: PushToTalkPayload) => void) | null = null;
let baiduRealtimeHandler: ((payload: import('./types').BaiduRealtimeResultEvent) => void) | null = null;

vi.mock('./tauriClient', async (importOriginal) => {
  const actual = await importOriginal<typeof import('./tauriClient')>();
  return {
    ...actual,
    isTauriRuntime: () => true,
    getConfig: vi.fn().mockResolvedValue({ hotkey: 'Ctrl+Alt+Space', language: 'zh-CN', asrEngine: 'whisper.cpp', insertionStrategy: 'clipboard', showStatusIndicator: true }),
    getAsrConfigStatus: vi.fn().mockResolvedValue({ whisperBinaryPath: null, whisperModelPath: null, language: 'zh', binaryConfigured: false, modelConfigured: false, binaryExists: false, modelExists: false, ready: false, source: 'default', message: 'ASR 未就绪' }),
    getCloudAsrConfigStatus: vi.fn().mockResolvedValue({ config: { provider: 'baidu', groupId: null, baseUrl: 'http://vop.baidu.com/server_api', model: '1537', language: 'zh', baiduCuid: 'voxtype-local', baiduFormat: 'pcm', baiduSampleRate: 16000, baiduLmId: null, baiduRealtimeAppId: '10500017', baiduRealtimeEndpoint: 'wss://vop.baidu.com/realtime_asr', baiduRealtimeDevPid: '15372', baiduRealtimeCuid: 'voxtype-local', baiduRealtimeFormat: 'pcm', baiduRealtimeSampleRate: 16000, baiduRealtimeUser: null }, apiKeyConfigured: true, apiKeySource: 'env:BAIDU_ASR_API_KEY', apiKeyPreview: 'ba***ey', secretKeyConfigured: true, secretKeySource: 'env:BAIDU_ASR_SECRET_KEY', secretKeyPreview: 'sk***ey', ready: true, message: '百度短语音 API 配置完整。' }),
    saveCloudAsrConfig: vi.fn().mockImplementation((config) => Promise.resolve({ config, apiKeyConfigured: true, apiKeySource: 'env:BAIDU_ASR_API_KEY', apiKeyPreview: 'ba***ey', secretKeyConfigured: true, secretKeySource: 'env:BAIDU_ASR_SECRET_KEY', secretKeyPreview: 'sk***ey', ready: true, message: '百度短语音 API 配置完整。' })),
    saveModeModelPreferences: vi.fn().mockImplementation((pushToTalkModel, toggleDictationModel) => Promise.resolve({ selectedInputDeviceName: null, pushToTalkHotkey: 'Ctrl+Alt+Space', toggleDictationHotkey: 'Ctrl+Alt+V', pushToTalkModel, toggleDictationModel })),
    saveBaiduAsrApiKey: vi.fn().mockResolvedValue({ config: { provider: 'baidu', groupId: null, baseUrl: 'http://vop.baidu.com/server_api', model: '1537', language: 'zh', baiduCuid: 'voxtype-local', baiduFormat: 'pcm', baiduSampleRate: 16000, baiduLmId: null, baiduRealtimeAppId: '10500017', baiduRealtimeEndpoint: 'wss://vop.baidu.com/realtime_asr', baiduRealtimeDevPid: '15372', baiduRealtimeCuid: 'voxtype-local', baiduRealtimeFormat: 'pcm', baiduRealtimeSampleRate: 16000, baiduRealtimeUser: null }, apiKeyConfigured: true, apiKeySource: 'env:BAIDU_ASR_API_KEY', apiKeyPreview: 'ba***ey', secretKeyConfigured: false, secretKeySource: 'missing', secretKeyPreview: null, ready: false, message: 'Missing BAIDU_ASR_SECRET_KEY.' }),
    saveBaiduAsrSecretKey: vi.fn().mockResolvedValue({ config: { provider: 'baidu', groupId: null, baseUrl: 'http://vop.baidu.com/server_api', model: '1537', language: 'zh', baiduCuid: 'voxtype-local', baiduFormat: 'pcm', baiduSampleRate: 16000, baiduLmId: null, baiduRealtimeAppId: '10500017', baiduRealtimeEndpoint: 'wss://vop.baidu.com/realtime_asr', baiduRealtimeDevPid: '15372', baiduRealtimeCuid: 'voxtype-local', baiduRealtimeFormat: 'pcm', baiduRealtimeSampleRate: 16000, baiduRealtimeUser: null }, apiKeyConfigured: true, apiKeySource: 'env:BAIDU_ASR_API_KEY', apiKeyPreview: 'ba***ey', secretKeyConfigured: true, secretKeySource: 'env:BAIDU_ASR_SECRET_KEY', secretKeyPreview: 'sk***ey', ready: true, message: 'Baidu ready.' }),
    getDefaultInputInfo: vi.fn().mockResolvedValue({ deviceName: 'Test Microphone', sampleRate: 44100, channels: 1 }),
    getUserPreferences: vi.fn().mockResolvedValue({ selectedInputDeviceName: null, pushToTalkHotkey: 'Ctrl+Alt+Space', toggleDictationHotkey: 'Ctrl+Alt+V', pushToTalkModel: 'baidu-short', toggleDictationModel: 'baidu-short' }),
    getHotkeyStatus: vi.fn().mockResolvedValue({ accelerator: 'Ctrl+Alt+Space', registered: true, message: '全局快捷键已注册：Ctrl+Alt+Space' }),
    getOverlayBackendStatus: vi.fn().mockResolvedValue({ backend: 'native-win32', lastError: null }),
    listInputDevices: vi.fn().mockResolvedValue([{ deviceName: 'Test Microphone', sampleRate: 44100, channels: 1 }]),
    startBaiduRealtimeSession: vi.fn().mockResolvedValue({ state: 'streaming', message: 'Baidu realtime WebSocket session started.', startedAtMs: 1000, durationMs: 0, finalText: '' }),
    finishBaiduRealtimeSession: vi.fn().mockResolvedValue({ status: { state: 'finished', message: 'stopped', startedAtMs: 1000, durationMs: 1800, finalText: 'realtime final text' }, text: 'realtime final text', durationMs: 1800, charCount: 6 }),
    cancelBaiduRealtimeSession: vi.fn().mockResolvedValue({ state: 'idle', message: 'cancelled', startedAtMs: null, durationMs: 0, finalText: '' }),
    getBaiduRealtimeSessionStatus: vi.fn().mockResolvedValue({ state: 'idle', message: 'idle', startedAtMs: null, durationMs: 0, finalText: '' }),
    startRecording: vi.fn().mockResolvedValue({ state: 'recording', sampleRate: 44100, channels: 1, sampleCount: 0, durationMs: 0 }),
    stopRecording: vi.fn().mockResolvedValue({ sampleRate: 44100, channels: 1, sampleCount: 32000, durationMs: 2000, asrSampleRate: 16000, asrSampleCount: 32000, asrDurationMs: 2000, peakAmplitude: 12000, rmsAmplitude: 1600 }),
    transcribeLastRecording: vi.fn().mockResolvedValue({ engine: 'whisper.cpp', text: '测试文本' }),
    transcribeActiveRecordingChunk: vi.fn().mockResolvedValue({ transcript: { engine: 'whisper.cpp', text: '实时片段' }, fromSampleIndex: 0, toSampleIndex: 44100, asrSampleCount: 16000 }),
    transcribeLastRecordingChunk: vi.fn().mockResolvedValue({ transcript: { engine: 'whisper.cpp', text: '尾段' }, fromSampleIndex: 44100, toSampleIndex: 52000, asrSampleCount: 2866 }),
    insertTextWithClipboard: vi.fn().mockResolvedValue(undefined),
    hideDictationOverlay: vi.fn().mockResolvedValue(undefined),
    showTranscribingOverlay: vi.fn().mockResolvedValue(undefined),
    listenToPushToTalk: vi.fn().mockImplementation((handler: (payload: PushToTalkPayload) => void) => {
      pushToTalkHandler = handler;
      return Promise.resolve(() => { pushToTalkHandler = null; });
    }),
    listenToBaiduRealtimeResults: vi.fn().mockImplementation((handler: (payload: import('./types').BaiduRealtimeResultEvent) => void) => {
      baiduRealtimeHandler = handler;
      return Promise.resolve(() => { baiduRealtimeHandler = null; });
    }),
  };
});
describe('App', () => {
  async function waitForBaiduReadiness() {
    await waitFor(() => expect(screen.getAllByTitle('百度短语音 API 配置完整。').length).toBeGreaterThanOrEqual(2));
  }

  beforeEach(() => {
    pushToTalkHandler = null;
    baiduRealtimeHandler = null;
    Object.defineProperty(navigator, 'clipboard', {
      configurable: true,
      value: { writeText: vi.fn().mockResolvedValue(undefined) },
    });
    vi.mocked(startBaiduRealtimeSession).mockClear();
    vi.mocked(finishBaiduRealtimeSession).mockClear();
    vi.mocked(startRecording).mockClear();
    vi.mocked(stopRecording).mockClear();
    vi.mocked(transcribeLastRecording).mockClear();
    vi.mocked(transcribeActiveRecordingChunk).mockClear();
    vi.mocked(transcribeLastRecordingChunk).mockClear();
    vi.mocked(insertTextWithClipboard).mockClear();
    vi.mocked(hideDictationOverlay).mockClear();
    vi.mocked(showTranscribingOverlay).mockClear();
    vi.mocked(getOverlayBackendStatus).mockClear();
    vi.mocked(saveCloudAsrConfig).mockClear();
    vi.mocked(saveModeModelPreferences).mockClear();
    vi.mocked(saveBaiduAsrApiKey).mockClear();
    vi.mocked(saveBaiduAsrSecretKey).mockClear();
  });
  it('renders the V5 control center by default', async () => {
    render(<App />);

    expect(screen.getByText('VoxType')).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: '语音输入控制中心' })).toBeInTheDocument();
    expect(screen.getByRole('region', { name: '输入模式' })).toBeInTheDocument();
    expect(screen.getAllByText('按住说话').length).toBeGreaterThan(0);
    expect(screen.getAllByText('连续输入').length).toBeGreaterThan(0);
    expect(screen.getByText('Ctrl+Alt+Space')).toBeInTheDocument();
    expect(screen.getByText('Ctrl+Alt+V')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '模型选择配置' })).toBeInTheDocument();
    expect(screen.getByRole('region', { name: '准备状态' })).toBeInTheDocument();
    expect(screen.getByText('麦克风')).toBeInTheDocument();
    expect(screen.getByText('上屏')).toBeInTheDocument();
    expect(screen.getAllByText('百度短语音 API').length).toBeGreaterThanOrEqual(2);
    expect(screen.queryByText('本地识别')).not.toBeInTheDocument();
    expect(screen.queryByText('云端 API')).not.toBeInTheDocument();
    expect(screen.queryByText('快捷键')).not.toBeInTheDocument();
    expect(screen.queryByRole('region', { name: '动态状态' })).not.toBeInTheDocument();
    expect(screen.queryByTestId('voice-wave')).not.toBeInTheDocument();
    expect(screen.getByRole('region', { name: '识别记录' })).toBeInTheDocument();
    expect(screen.getByTestId('history-toolbar')).toBeInTheDocument();
    expect(screen.getByText('等待第一次识别')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '清空全部识别记录' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '导出识别记录' })).toBeInTheDocument();
    expect(screen.getByTitle('本次运行识别次数')).toHaveTextContent('0 条');
    expect(screen.getByTitle('本次运行累计录音时长')).toHaveTextContent('0:00');
    expect(screen.getByTitle('本次运行累计识别字数')).toHaveTextContent('0 字');
    expect(await screen.findByText('Test Microphone')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '诊断' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: '开始一次语音输入' })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: '安装 whisper.cpp' })).not.toBeInTheDocument();
    expect(screen.queryByRole('heading', { name: '诊断工作台' })).not.toBeInTheDocument();
    expect(document.querySelector('.traffic-lights')).not.toBeInTheDocument();
  });

  it('shows Baidu cloud readiness on the main window when Baidu is configured', async () => {
    const { getCloudAsrConfigStatus } = await import('./tauriClient');
    vi.mocked(getCloudAsrConfigStatus).mockResolvedValueOnce({ config: { provider: 'baidu', groupId: null, baseUrl: 'http://vop.baidu.com/server_api', model: '1537', language: 'zh', baiduCuid: 'voxtype-local', baiduFormat: 'pcm', baiduSampleRate: 16000, baiduLmId: null, baiduRealtimeAppId: '10500017', baiduRealtimeEndpoint: 'wss://vop.baidu.com/realtime_asr', baiduRealtimeDevPid: '15372', baiduRealtimeCuid: 'voxtype-local', baiduRealtimeFormat: 'pcm', baiduRealtimeSampleRate: 16000, baiduRealtimeUser: null }, apiKeyConfigured: true, apiKeySource: 'env:BAIDU_ASR_API_KEY', apiKeyPreview: 'ba***ey', secretKeyConfigured: true, secretKeySource: 'env:BAIDU_ASR_SECRET_KEY', secretKeyPreview: 'sk***ey', ready: true, message: '百度短语音 API 配置完整。' });

    render(<App />);

    await waitFor(() => expect(screen.getAllByText('百度短语音 API').length).toBeGreaterThanOrEqual(2));
    expect(screen.queryByText('MiniMax 未就绪')).not.toBeInTheDocument();
  });

  it('shows shortcut transcription failures in the main transcript area', async () => {
    vi.mocked(transcribeLastRecording).mockRejectedValueOnce({ code: 'asr_failed', message: '语音识别失败：百度 ASR HTTP 404 Not Found：404 page not found' });
    render(<App />);

    await screen.findByText(/Ctrl\+Alt\+Space/);
    await waitForBaiduReadiness();
    pushToTalkHandler?.({ state: 'pressed', action: 'startRecording' });
    await waitFor(() => expect(startRecording).toHaveBeenCalledTimes(1));
    pushToTalkHandler?.({ state: 'released', action: 'stopAndTranscribe' });

    expect(await screen.findByRole('status')).toHaveTextContent('百度 ASR HTTP 404 Not Found');
    expect(screen.queryByText('测试文本')).not.toBeInTheDocument();
  });

  it('clears a previous shortcut transcription error after the next successful transcription', async () => {
    vi.mocked(transcribeLastRecording)
      .mockRejectedValueOnce({ code: 'asr_failed', message: '语音识别失败：百度 ASR HTTP 404 Not Found：404 page not found' })
      .mockResolvedValueOnce({ engine: 'whisper.cpp', text: '恢复成功文本' });
    render(<App />);

    await screen.findByText(/Ctrl\+Alt\+Space/);
    await waitForBaiduReadiness();
    pushToTalkHandler?.({ state: 'pressed', action: 'startRecording' });
    await waitFor(() => expect(startRecording).toHaveBeenCalledTimes(1));
    pushToTalkHandler?.({ state: 'released', action: 'stopAndTranscribe' });

    expect(await screen.findByRole('status')).toHaveTextContent('百度 ASR HTTP 404 Not Found');

    pushToTalkHandler?.({ state: 'pressed', action: 'startRecording' });
    await waitFor(() => expect(startRecording).toHaveBeenCalledTimes(2));
    pushToTalkHandler?.({ state: 'released', action: 'stopAndTranscribe' });

    expect(await screen.findByText('恢复成功文本')).toBeInTheDocument();
    await waitFor(() => expect(screen.queryByRole('status')).not.toBeInTheDocument());
  });


  it('records global shortcut events in diagnostics', async () => {
    render(<App />);

    await screen.findByText(/Ctrl\+Alt\+Space/);
    await waitForBaiduReadiness();
    pushToTalkHandler?.({ state: 'pressed', action: 'startRecording' });
    fireEvent.click(screen.getByRole('button', { name: '诊断' }));

    expect(await screen.findByText('收到全局快捷键按下')).toBeInTheDocument();
  });

  it('opens and closes hotkey settings from one section-level settings button', async () => {
    render(<App />);

    expect(screen.queryByRole('button', { name: '修改按住说话快捷键' })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: '修改连续输入快捷键' })).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: '自定义快捷键' }));

    expect(await screen.findByRole('dialog', { name: '快捷键设置' })).toBeInTheDocument();
    expect(screen.getByLabelText('按住说话快捷键')).toHaveValue('Ctrl+Alt+Space');
    expect(screen.getByLabelText('连续输入快捷键')).toHaveValue('Ctrl+Alt+V');

    fireEvent.click(screen.getByRole('button', { name: '关闭快捷键设置' }));
    expect(screen.queryByRole('dialog', { name: '快捷键设置' })).not.toBeInTheDocument();
  });

  it('runs the shortcut closed loop and hides the desktop overlay after release', async () => {
    render(<App />);

    await screen.findByText(/Ctrl\+Alt\+Space/);
    await waitForBaiduReadiness();
    pushToTalkHandler?.({ state: 'pressed', action: 'startRecording' });

    await waitFor(() => expect(startRecording).toHaveBeenCalledTimes(1));

    pushToTalkHandler?.({ state: 'released', action: 'stopAndTranscribe' });

    await waitFor(() => expect(stopRecording).toHaveBeenCalledTimes(1));
    expect(showTranscribingOverlay).toHaveBeenCalledTimes(1);
    expect(transcribeLastRecording).toHaveBeenCalledWith('baidu-short');
    expect(insertTextWithClipboard).toHaveBeenCalledWith('测试文本');
    await waitFor(() => expect(hideDictationOverlay).toHaveBeenCalledTimes(1));
  });

  it('keeps session transcript records in reverse chronological order with stats and row actions', async () => {
    render(<App />);

    await screen.findByText(/Ctrl\+Alt\+Space/);
    await waitForBaiduReadiness();
    pushToTalkHandler?.({ state: 'pressed', action: 'startRecording' });
    await waitFor(() => expect(startRecording).toHaveBeenCalledTimes(1));
    pushToTalkHandler?.({ state: 'released', action: 'stopAndTranscribe' });

    await screen.findByText('测试文本');
    vi.mocked(transcribeLastRecording).mockResolvedValueOnce({ engine: 'whisper.cpp', text: '第二段文本' });
    pushToTalkHandler?.({ state: 'pressed', action: 'startRecording' });
    await waitFor(() => expect(startRecording).toHaveBeenCalledTimes(2));
    pushToTalkHandler?.({ state: 'released', action: 'stopAndTranscribe' });

    await screen.findByText('第二段文本');
    const firstRecord = screen.getAllByRole('article', { name: /识别记录/ })[0];
    expect(firstRecord).toHaveTextContent('第二段文本');
    expect(screen.getByTitle('本次运行识别次数')).toHaveTextContent('2 条');
    expect(within(screen.getByTestId('history-toolbar')).getByTitle('本次运行累计录音时长')).toHaveTextContent('0:04');
    expect(within(screen.getByTestId('history-toolbar')).getByTitle('本次运行累计识别字数')).toHaveTextContent('9 字');

    fireEvent.click(screen.getAllByRole('button', { name: '重新上屏此记录' })[0]);
    expect(insertTextWithClipboard).toHaveBeenLastCalledWith('第二段文本');

    fireEvent.click(screen.getAllByRole('button', { name: '删除此识别记录' })[0]);
    expect(screen.queryByText('第二段文本')).not.toBeInTheDocument();
    expect(screen.getByText('测试文本')).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: '清空全部识别记录' }));
    expect(screen.getByText('等待第一次识别')).toBeInTheDocument();
  });

  it('shows V7 mode model routing and removes MiniMax from model selection', async () => {
    render(<App />);

    fireEvent.click(screen.getByRole('button', { name: '模型选择配置' }));

    expect(screen.getByRole('heading', { name: '模型选择配置' })).toBeInTheDocument();
    expect(screen.getByRole('region', { name: '输入模式默认模型' })).toBeInTheDocument();
    expect(screen.queryByText('输入模式默认模型')).not.toBeInTheDocument();
    expect(screen.getByText('按住说话')).toBeInTheDocument();
    expect(screen.getByText('连续输入')).toBeInTheDocument();
    expect(screen.queryByText('分别选择')).not.toBeInTheDocument();
    expect(screen.getAllByText('百度短语音 API').length).toBeGreaterThan(0);
    expect(screen.getAllByText('百度实时 WebSocket API').length).toBeGreaterThan(0);
    expect(screen.queryByText('MiniMax')).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '模型配置' }));
    fireEvent.click(screen.getByRole('tab', { name: /本地 whisper\.cpp/ }));
    expect(screen.getByLabelText('whisper.cpp 可执行文件')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('tab', { name: /百度短语音 API/ }));
    expect(screen.getByLabelText('百度 ASR Endpoint')).toHaveValue('http://vop.baidu.com/server_api');
    fireEvent.click(screen.getByRole('tab', { name: /百度实时 WebSocket API/ }));
    expect(screen.getByLabelText('百度 WebSocket AppID')).toHaveValue('10500017');

    fireEvent.click(screen.getByRole('button', { name: '模型选择' }));
    expect(screen.queryByRole('button', { name: '保存默认模型' })).not.toBeInTheDocument();
    fireEvent.click(within(screen.getByRole('group', { name: '连续输入模型默认模型' })).getByRole('button', { name: /WebSocket/ }));

    await waitFor(() => expect(saveModeModelPreferences).toHaveBeenCalledWith('baidu-short', 'baidu-realtime'));
  });


  it('shows visible feedback when exporting transcript records', async () => {
    render(<App />);

    await screen.findByText(/Ctrl\+Alt\+Space/);
    await waitForBaiduReadiness();
    pushToTalkHandler?.({ state: 'pressed', action: 'startRecording' });
    await waitFor(() => expect(startRecording).toHaveBeenCalledTimes(1));
    pushToTalkHandler?.({ state: 'released', action: 'stopAndTranscribe' });

    await screen.findByText('测试文本');
    fireEvent.click(screen.getByRole('button', { name: '导出识别记录' }));

    expect(await screen.findByText('已复制 1 条记录到剪贴板')).toBeInTheDocument();
  });

  it('configures Baidu short speech ASR without rendering the API key secret', async () => {
    render(<App />);

    fireEvent.click(screen.getByRole('button', { name: '模型选择配置' }));
    fireEvent.click(screen.getByRole('button', { name: '模型配置' }));

    expect(screen.getByRole('region', { name: '百度短语音 API 配置' })).toBeInTheDocument();
    expect(screen.getByText('BAIDU_ASR_API_KEY')).toBeInTheDocument();
    expect(screen.getByLabelText('百度 ASR API Key 输入')).toHaveAttribute('type', 'password');
    expect(screen.getByText('BAIDU_ASR_SECRET_KEY')).toBeInTheDocument();
    expect(screen.getByLabelText('百度 ASR Secret Key 输入')).toHaveAttribute('type', 'password');
    expect(screen.getByLabelText('百度 ASR Endpoint')).toHaveValue('http://vop.baidu.com/server_api');
    expect(screen.getByLabelText('百度 ASR dev_pid')).toHaveValue('1537');
    expect(screen.getByLabelText('百度 ASR cuid')).toHaveValue('voxtype-local');
    expect(screen.getByLabelText('百度 ASR 音频格式')).toHaveValue('pcm');
    expect(screen.getByLabelText('百度 ASR 采样率')).toHaveValue(16000);

    fireEvent.click(screen.getByRole('button', { name: '保存百度配置' }));
    await waitFor(() => expect(saveCloudAsrConfig).toHaveBeenCalledWith({
      provider: 'baidu',
      groupId: null,
      baseUrl: 'http://vop.baidu.com/server_api',
      model: '1537',
      language: 'zh',
      baiduCuid: 'voxtype-local',
      baiduFormat: 'pcm',
      baiduSampleRate: 16000,
      baiduLmId: null,
      baiduRealtimeAppId: '10500017',
      baiduRealtimeEndpoint: 'wss://vop.baidu.com/realtime_asr',
      baiduRealtimeDevPid: '15372',
      baiduRealtimeCuid: 'voxtype-local',
      baiduRealtimeFormat: 'pcm',
      baiduRealtimeSampleRate: 16000,
      baiduRealtimeUser: null,
    }));

    const input = screen.getByLabelText('百度 ASR API Key 输入');
    fireEvent.change(input, { target: { value: 'baidu-secret-key' } });
    fireEvent.click(screen.getByRole('button', { name: '保存百度 API Key 到系统环境变量' }));

    await waitFor(() => expect(saveBaiduAsrApiKey).toHaveBeenCalledWith('baidu-secret-key'));
    expect(await screen.findByText('ba***ey')).toBeInTheDocument();
    expect(screen.getByRole('region', { name: '百度短语音 API 配置' })).toBeInTheDocument();
    expect(screen.queryByText('MiniMax')).not.toBeInTheDocument();
    expect(screen.queryByDisplayValue('baidu-secret-key')).not.toBeInTheDocument();
    expect(screen.queryByText('baidu-secret-key')).not.toBeInTheDocument();

    const secretInput = screen.getByLabelText('百度 ASR Secret Key 输入');
    fireEvent.change(secretInput, { target: { value: 'baidu-real-secret' } });
    fireEvent.click(screen.getByRole('button', { name: '保存百度 Secret Key 到系统环境变量' }));

    await waitFor(() => expect(saveBaiduAsrSecretKey).toHaveBeenCalledWith('baidu-real-secret'));
    expect(await screen.findByText('sk***ey')).toBeInTheDocument();
    expect(screen.queryByDisplayValue('baidu-real-secret')).not.toBeInTheDocument();
    expect(screen.queryByText('baidu-real-secret')).not.toBeInTheDocument();
  });

  it('shows explicit feedback when testing Baidu ASR settings', async () => {
    render(<App />);

    fireEvent.click(screen.getByRole('button', { name: '模型选择配置' }));
    fireEvent.click(screen.getByRole('button', { name: '模型配置' }));
    fireEvent.click(screen.getByRole('button', { name: '检测百度配置' }));

    expect(await screen.findByText(/百度配置检测/)).toBeInTheDocument();
    expect(screen.getByText(/百度实时 WebSocket API 已就绪/)).toBeInTheDocument();
    expect(screen.getByRole('region', { name: '百度短语音 API 配置' })).toBeInTheDocument();
  });

  it('shows realtime WebSocket readiness feedback on the realtime config panel', async () => {
    render(<App />);

    fireEvent.click(screen.getByRole('button', { name: '模型选择配置' }));
    fireEvent.click(screen.getByRole('button', { name: '模型配置' }));
    fireEvent.click(screen.getByRole('tab', { name: /百度实时 WebSocket API/ }));
    expect(screen.getByLabelText('百度 WebSocket AppID')).toHaveValue('10500017');
    expect(screen.getByLabelText('百度 WebSocket Endpoint')).toHaveValue('wss://vop.baidu.com/realtime_asr');
    expect(screen.getByLabelText('百度 WebSocket dev_pid')).toHaveValue('15372');
    expect(screen.getByLabelText('百度 WebSocket cuid')).toHaveValue('voxtype-local');
    expect(screen.getByLabelText('百度 WebSocket 音频格式')).toHaveValue('pcm');
    expect(screen.getByLabelText('百度 WebSocket 采样率')).toHaveValue(16000);
    fireEvent.click(screen.getByRole('button', { name: '检测百度配置' }));

    expect(await screen.findByText(/百度实时 WebSocket API 已就绪/)).toBeInTheDocument();
    expect(screen.getByRole('region', { name: '百度实时 WebSocket API 配置' })).toBeInTheDocument();
  });

  it('shows shared Baidu credentials on the realtime WebSocket config panel', async () => {
    render(<App />);

    fireEvent.click(screen.getByRole('button', { name: '模型选择配置' }));
    fireEvent.click(screen.getByRole('button', { name: '模型配置' }));
    fireEvent.click(screen.getByRole('tab', { name: /百度实时 WebSocket API/ }));

    expect(screen.getByRole('region', { name: '百度实时 WebSocket API 配置' })).toBeInTheDocument();
    expect(screen.getByText('BAIDU_ASR_API_KEY')).toBeInTheDocument();
    expect(screen.getByText('BAIDU_ASR_SECRET_KEY')).toBeInTheDocument();
    expect(await screen.findByText('ba***ey')).toBeInTheDocument();
    expect(screen.getByText('sk***ey')).toBeInTheDocument();
    expect(screen.getByLabelText('百度 ASR API Key 输入')).toHaveAttribute('type', 'password');
    expect(screen.getByLabelText('百度 ASR Secret Key 输入')).toHaveAttribute('type', 'password');

    fireEvent.change(screen.getByLabelText('百度 ASR API Key 输入'), { target: { value: 'baidu-realtime-api-key' } });
    fireEvent.click(screen.getByRole('button', { name: '保存百度 API Key 到系统环境变量' }));

    await waitFor(() => expect(saveBaiduAsrApiKey).toHaveBeenCalledWith('baidu-realtime-api-key'));
    expect(screen.queryByDisplayValue('baidu-realtime-api-key')).not.toBeInTheDocument();
    expect(screen.queryByText('baidu-realtime-api-key')).not.toBeInTheDocument();
  });

  it('saves realtime WebSocket fields as part of Baidu model configuration', async () => {
    render(<App />);

    fireEvent.click(screen.getByRole('button', { name: '模型选择配置' }));
    fireEvent.click(screen.getByRole('button', { name: '模型配置' }));
    fireEvent.click(screen.getByRole('tab', { name: /百度实时 WebSocket API/ }));
    fireEvent.change(screen.getByLabelText('百度 WebSocket AppID'), { target: { value: '10500017' } });
    fireEvent.change(screen.getByLabelText('百度 WebSocket dev_pid'), { target: { value: '15372' } });
    fireEvent.click(screen.getByRole('button', { name: '保存百度配置' }));

    await waitFor(() => expect(saveCloudAsrConfig).toHaveBeenCalledWith(expect.objectContaining({
      baiduRealtimeAppId: '10500017',
      baiduRealtimeEndpoint: 'wss://vop.baidu.com/realtime_asr',
      baiduRealtimeDevPid: '15372',
      baiduRealtimeCuid: 'voxtype-local',
      baiduRealtimeFormat: 'pcm',
      baiduRealtimeSampleRate: 16000,
    })));
    expect(await screen.findByText(/百度实时 WebSocket API 已就绪/)).toBeInTheDocument();
  });

  it('lets the user reinsert and clear the latest transcript from the control center', async () => {
    render(<App />);

    await screen.findByText(/Ctrl\+Alt\+Space/);
    await waitForBaiduReadiness();
    pushToTalkHandler?.({ state: 'pressed', action: 'startRecording' });
    await waitFor(() => expect(startRecording).toHaveBeenCalledTimes(1));
    pushToTalkHandler?.({ state: 'released', action: 'stopAndTranscribe' });

    await screen.findByText('测试文本');
    fireEvent.click(screen.getByRole('button', { name: '重新上屏此记录' }));
    expect(insertTextWithClipboard).toHaveBeenLastCalledWith('测试文本');

    fireEvent.click(screen.getByRole('button', { name: '清空全部识别记录' }));
    expect(screen.getByText('等待第一次识别')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: '重新上屏此记录' })).not.toBeInTheDocument();
  });

  it('runs the toggle shortcut closed loop on the second press', async () => {
    render(<App />);

    await screen.findByText(/Ctrl\+Alt\+Space/);
    await waitForBaiduReadiness();
    pushToTalkHandler?.({ state: 'pressed', action: 'toggleStartRecording' });

    await waitFor(() => expect(startRecording).toHaveBeenCalledTimes(1));

    pushToTalkHandler?.({ state: 'pressed', action: 'toggleStopAndTranscribe' });

    await waitFor(() => expect(stopRecording).toHaveBeenCalledTimes(1));
    expect(showTranscribingOverlay).toHaveBeenCalledTimes(1);
    expect(transcribeActiveRecordingChunk).not.toHaveBeenCalled();
    expect(transcribeLastRecordingChunk).not.toHaveBeenCalled();
    expect(transcribeLastRecording).toHaveBeenCalledWith('baidu-short');
    expect(insertTextWithClipboard).toHaveBeenCalledWith('测试文本');
    await waitFor(() => expect(hideDictationOverlay).toHaveBeenCalledTimes(1));
  });

  it('keeps the toggle overlay in transcribing mode while waiting for final model transcription', async () => {
    let resolveTranscript!: (value: { engine: string; text: string }) => void;
    vi.mocked(transcribeLastRecording).mockImplementationOnce(() => new Promise((resolve) => {
      resolveTranscript = resolve;
    }));

    render(<App />);

    await screen.findByText(/Ctrl\+Alt\+Space/);
    await waitForBaiduReadiness();
    pushToTalkHandler?.({ state: 'pressed', action: 'toggleStartRecording' });

    await waitFor(() => expect(startRecording).toHaveBeenCalledTimes(1));
    pushToTalkHandler?.({ state: 'pressed', action: 'toggleStopAndTranscribe' });

    await waitFor(() => expect(stopRecording).toHaveBeenCalledTimes(1));
    await waitFor(() => expect(showTranscribingOverlay).toHaveBeenCalledTimes(1));
    expect(hideDictationOverlay).not.toHaveBeenCalled();
    expect(insertTextWithClipboard).not.toHaveBeenCalledWith('delayed transcript');

    resolveTranscript({ engine: 'baidu-short-speech', text: 'delayed transcript' });
    await waitFor(() => expect(insertTextWithClipboard).toHaveBeenCalledWith('delayed transcript'));
    await waitFor(() => expect(hideDictationOverlay).toHaveBeenCalledTimes(1));
  });

  it('streams an active recording chunk while toggle dictation is recording', async () => {
    render(<App />);
    await screen.findByText(/Ctrl\+Alt\+Space/);
    await waitForBaiduReadiness();

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

      expect(transcribeActiveRecordingChunk).not.toHaveBeenCalled();

      await act(async () => {
        pushToTalkHandler?.({ state: 'pressed', action: 'toggleStopAndTranscribe' });
        await Promise.resolve();
      });

      expect(stopRecording).toHaveBeenCalledTimes(1);
      await vi.waitFor(() => expect(transcribeLastRecording).toHaveBeenCalledWith('baidu-short'));
      await vi.waitFor(() => expect(insertTextWithClipboard).toHaveBeenCalledWith('测试文本'));
      expect(screen.getByText('测试文本')).toBeInTheDocument();
      expect(screen.getByTitle('本次运行识别次数')).toHaveTextContent('1 条');
    } finally {
      vi.useRealTimers();
    }
  });


  it('starts Baidu realtime WebSocket session for continuous input model', async () => {
    render(<App />);

    await waitForBaiduReadiness();
    fireEvent.click(screen.getByRole('button', { name: /\u6a21\u578b\u9009\u62e9\u914d\u7f6e/ }));
    fireEvent.click(within(screen.getByRole('group', { name: /\u8fde\u7eed\u8f93\u5165/ })).getByRole('button', { name: /WebSocket/ }));
    await waitFor(() => expect(saveModeModelPreferences).toHaveBeenCalledWith('baidu-short', 'baidu-realtime'));
    fireEvent.click(screen.getByRole('button', { name: /\u8fd4\u56de\u4e3b\u754c\u9762/ }));

    pushToTalkHandler?.({ state: 'pressed', action: 'toggleStartRecording' });

    await waitFor(() => expect(startBaiduRealtimeSession).toHaveBeenCalledTimes(1));
    expect(startRecording).not.toHaveBeenCalled();
  });

  it('inserts realtime final events and records one merged continuous input row on finish', async () => {
    render(<App />);

    await waitForBaiduReadiness();
    fireEvent.click(screen.getByRole('button', { name: /\u6a21\u578b\u9009\u62e9\u914d\u7f6e/ }));
    fireEvent.click(within(screen.getByRole('group', { name: /\u8fde\u7eed\u8f93\u5165/ })).getByRole('button', { name: /WebSocket/ }));
    await waitFor(() => expect(saveModeModelPreferences).toHaveBeenCalledWith('baidu-short', 'baidu-realtime'));
    fireEvent.click(screen.getByRole('button', { name: /\u8fd4\u56de\u4e3b\u754c\u9762/ }));

    pushToTalkHandler?.({ state: 'pressed', action: 'toggleStartRecording' });
    await waitFor(() => expect(startBaiduRealtimeSession).toHaveBeenCalledTimes(1));
    baiduRealtimeHandler?.({ text: 'realtime final text', isFinal: true, sequence: 1, startedAtMs: 1000, durationMs: 1800 });
    await waitFor(() => expect(insertTextWithClipboard).toHaveBeenCalledWith('realtime final text'));

    pushToTalkHandler?.({ state: 'pressed', action: 'toggleStopAndTranscribe' });

    await waitFor(() => expect(finishBaiduRealtimeSession).toHaveBeenCalledTimes(1));
    expect(screen.getByText('realtime final text')).toBeInTheDocument();
    expect(screen.getByTitle('本次运行识别次数')).toHaveTextContent('1 条');
  });

  it('rejects Baidu realtime WebSocket API for push-to-talk mode in V8', async () => {
    render(<App />);

    await waitForBaiduReadiness();
    fireEvent.click(screen.getByRole('button', { name: /\u6a21\u578b\u9009\u62e9\u914d\u7f6e/ }));
    fireEvent.click(within(screen.getByRole('group', { name: /\u6309\u4f4f\u8bf4\u8bdd/ })).getByRole('button', { name: /WebSocket/ }));
    await waitFor(() => expect(saveModeModelPreferences).toHaveBeenCalledWith('baidu-realtime', 'baidu-short'));
    fireEvent.click(screen.getByRole('button', { name: /\u8fd4\u56de\u4e3b\u754c\u9762/ }));

    pushToTalkHandler?.({ state: 'pressed', action: 'startRecording' });
    await waitFor(() => expect(startRecording).toHaveBeenCalledTimes(1));
    pushToTalkHandler?.({ state: 'released', action: 'stopAndTranscribe' });

    expect(await screen.findByRole('status')).toHaveTextContent(/WebSocket API/);
  });

  it('opens the diagnostic workbench on request', async () => {
    render(<App />);

    fireEvent.click(screen.getByRole('button', { name: '诊断' }));

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
