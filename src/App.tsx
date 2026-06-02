import { useEffect, useMemo, useRef, useState } from 'react';
import { DiagnosticView } from './DiagnosticView';
import { formatDiagnosticsForCopy } from './diagnostics';
import { HotkeySettingsDialog } from './HotkeySettingsDialog';
import { MainWindow } from './MainWindow';
import { ModelSettingsView } from './ModelSettingsView';
import { formatError } from './errorFormat';
import { cancelBaiduRealtimeSession, exportLastRecordingWav, finishBaiduRealtimeSession, getAsrConfigStatus, getCloudAsrConfigStatus, getConfig, getDefaultInputInfo, getHotkeyStatus, getOverlayBackendStatus, getRecordingStatus, getTranscriptPostprocessConfig, getUserPreferences, hideDictationOverlay, insertTextWithClipboard, installManagedAsr, isTauriRuntime, loadTranscriptHistory, saveTranscriptHistoryEntry, deleteTranscriptHistoryEntry, clearTranscriptHistory, previewTranscriptPostprocess, listenToBaiduRealtimeResults, listenToPushToTalk, listInputDevices, saveAsrConfig, saveBaiduAsrApiKey, saveBaiduAsrSecretKey, saveCloudAsrConfig, saveHotkeyPreferences, saveModeModelPreferences, saveTranscriptPostprocessConfig, setInputDevice, showDictationOverlay, showTranscribingOverlay, simulateDictation, startBaiduRealtimeSession, startRecording, stopRecording, transcribeActiveRecordingChunk, transcribeLastRecording, transcribeLastRecordingChunk } from './tauriClient';
import type { OverlayBackendStatus } from './tauriClient';
import type { AppConfig, AppStatus, AsrConfigStatus, CloudAsrConfigStatus, HotkeyRegistrationStatus, ModelReadiness, RecorderInfo, RecorderRuntimeStatus, BaiduRealtimeResultEvent, AudioQualitySummary, PersistedTranscriptEntry, TranscriptPostprocessConfig, TranscriptRecord, TranscriptStats, TranscriptionModelId } from './types';

interface DiagnosticEntry {
  id: number;
  time: string;
  title: string;
  result: 'info' | 'success' | 'warning' | 'error';
  detail: string;
}

const MAX_DIAGNOSTIC_ENTRIES = 100;
const INSERT_DELAY_MS = 3000;
const LIVE_TRANSCRIPTION_INTERVAL_MS = 4000;

const defaultConfig: AppConfig = {
  hotkey: 'Ctrl+Alt+Space',
  language: 'zh-CN',
  asrEngine: 'whisper.cpp',
  insertionStrategy: 'clipboard',
  showStatusIndicator: true,
};

const defaultPushToTalkHotkey = 'Ctrl+Alt+Space';
const defaultToggleDictationHotkey = 'Ctrl+Alt+V';

const defaultTranscriptPostprocessConfig: TranscriptPostprocessConfig = {
  enabled: true,
  cleanupNoise: true,
  glossary: ['WebSocket', 'whisper.cpp', 'VoxType'],
  replacements: [{ from: 'scale', to: 'skill', enabled: true }],
};

const initialStatus: AppStatus = {
  phase: 'idle',
  message: '等待开始录音',
  lastTranscript: null,
};

const initialAsrConfigStatus: AsrConfigStatus = {
  whisperBinaryPath: null,
  whisperModelPath: null,
  language: 'zh',
  binaryConfigured: false,
  modelConfigured: false,
  binaryExists: false,
  modelExists: false,
  ready: false,
  source: 'default',
  message: '等待 Tauri 读取 ASR 配置。',
};

const initialHotkeyStatus: HotkeyRegistrationStatus = {
  accelerator: defaultPushToTalkHotkey,
  registered: false,
  message: '等待 Tauri 注册全局快捷键。',
};

const initialOverlayBackendStatus: OverlayBackendStatus = {
  backend: 'webview',
  lastError: null,
};

const initialCloudAsrConfigStatus: CloudAsrConfigStatus = {
  config: {
    provider: 'baidu',
    groupId: null,
    baseUrl: 'http://vop.baidu.com/server_api',
    model: '1537',
    language: 'zh',
    baiduCuid: null,
    baiduFormat: null,
    baiduSampleRate: 16000,
    baiduLmId: null,
    baiduRealtimeAppId: '10500017',
    baiduRealtimeEndpoint: 'wss://vop.baidu.com/realtime_asr',
    baiduRealtimeDevPid: '15372',
    baiduRealtimeCuid: 'voxtype-local',
    baiduRealtimeFormat: 'pcm',
    baiduRealtimeSampleRate: 16000,
    baiduRealtimeUser: null,
  },
  apiKeyConfigured: false,
  apiKeySource: 'missing',
  apiKeyPreview: null,
  secretKeyConfigured: false,
  secretKeySource: 'missing',
  secretKeyPreview: null,
  ready: false,
  message: '等待 Tauri 读取百度短语音 API 配置。',
};

export function App() {
  const [viewMode, setViewMode] = useState<'user' | 'diagnostic' | 'model'>('user');
  const [config, setConfig] = useState<AppConfig>(defaultConfig);
  const [status, setStatus] = useState<AppStatus>(initialStatus);
  const [recorderInfo, setRecorderInfo] = useState<RecorderInfo | null>(null);
  const [inputDevices, setInputDevices] = useState<RecorderInfo[]>([]);
  const [selectedInputDeviceName, setSelectedInputDeviceName] = useState('');
  const [preferredInputDeviceName, setPreferredInputDeviceName] = useState<string | null>(null);
  const [recordingStatus, setRecordingStatus] = useState<RecorderRuntimeStatus>({
    state: 'idle',
    sampleRate: null,
    channels: null,
    sampleCount: 0,
    durationMs: 0,
  });
  const [runtimeMessage, setRuntimeMessage] = useState('浏览器预览模式：系统能力需要在 Tauri 中验证');
  const [asrConfigStatus, setAsrConfigStatus] = useState<AsrConfigStatus>(initialAsrConfigStatus);
  const [cloudAsrConfigStatus, setCloudAsrConfigStatus] = useState<CloudAsrConfigStatus>(initialCloudAsrConfigStatus);
  const [hotkeyStatus, setHotkeyStatus] = useState<HotkeyRegistrationStatus>(initialHotkeyStatus);
  const [overlayBackendStatus, setOverlayBackendStatus] = useState<OverlayBackendStatus>(initialOverlayBackendStatus);
  const [whisperBinaryPath, setWhisperBinaryPath] = useState('');
  const [whisperModelPath, setWhisperModelPath] = useState('');
  const [asrLanguage, setAsrLanguage] = useState('zh');
  const [cloudBaseUrl, setCloudBaseUrl] = useState('http://vop.baidu.com/server_api');
  const [cloudModel, setCloudModel] = useState('1537');
  const [cloudLanguage, setCloudLanguage] = useState('zh');
  const [cloudBaiduCuid, setCloudBaiduCuid] = useState('voxtype-local');
  const [cloudBaiduFormat, setCloudBaiduFormat] = useState('pcm');
  const [cloudBaiduSampleRate, setCloudBaiduSampleRate] = useState('16000');
  const [cloudBaiduLmId, setCloudBaiduLmId] = useState('');
  const [cloudBaiduRealtimeAppId, setCloudBaiduRealtimeAppId] = useState('10500017');
  const [cloudBaiduRealtimeEndpoint, setCloudBaiduRealtimeEndpoint] = useState('wss://vop.baidu.com/realtime_asr');
  const [cloudBaiduRealtimeDevPid, setCloudBaiduRealtimeDevPid] = useState('15372');
  const [cloudBaiduRealtimeCuid, setCloudBaiduRealtimeCuid] = useState('voxtype-local');
  const [cloudBaiduRealtimeFormat, setCloudBaiduRealtimeFormat] = useState('pcm');
  const [cloudBaiduRealtimeSampleRate, setCloudBaiduRealtimeSampleRate] = useState('16000');
  const [cloudBaiduRealtimeUser, setCloudBaiduRealtimeUser] = useState('');
  const [cloudApiKeyInput, setCloudApiKeyInput] = useState('');
  const [cloudSecretKeyInput, setCloudSecretKeyInput] = useState('');
  const [cloudMessage, setCloudMessage] = useState<string | null>(null);
  const [isInstallingAsr, setIsInstallingAsr] = useState(false);
  const [isHotkeyDialogOpen, setIsHotkeyDialogOpen] = useState(false);
  const [pushToTalkHotkey, setPushToTalkHotkey] = useState(defaultPushToTalkHotkey);
  const [toggleDictationHotkeyInput, setToggleDictationHotkeyInput] = useState(defaultToggleDictationHotkey);
  const [pushToTalkModel, setPushToTalkModel] = useState<TranscriptionModelId>('baidu-short');
  const [toggleDictationModel, setToggleDictationModel] = useState<TranscriptionModelId>('baidu-short');
  const [isSavingHotkeys, setIsSavingHotkeys] = useState(false);
  const [hotkeySaveMessage, setHotkeySaveMessage] = useState<string | null>(null);
  const [transcriptRecords, setTranscriptRecords] = useState<TranscriptRecord[]>([]);
  const [historyMessage, setHistoryMessage] = useState<string | null>(null);
  const [transcriptPostprocessConfig, setTranscriptPostprocessConfig] = useState<TranscriptPostprocessConfig>(defaultTranscriptPostprocessConfig);
  const [postprocessReplacementText, setPostprocessReplacementText] = useState('scale => skill');
  const [postprocessGlossaryText, setPostprocessGlossaryText] = useState('WebSocket\nwhisper.cpp\nVoxType');
  const [postprocessPreviewInput, setPostprocessPreviewInput] = useState('scale websocket whisper cpp');
  const [postprocessPreviewOutput, setPostprocessPreviewOutput] = useState<string | null>(null);
  const [postprocessMessage, setPostprocessMessage] = useState<string | null>(null);
  const didLoadRuntime = useRef(false);
  const nextDiagnosticIdRef = useRef(2);
  const nextTranscriptRecordIdRef = useRef(1);
  const isRecordingRef = useRef(false);
  const liveTimerRef = useRef<number | null>(null);
  const liveCursorRef = useRef(0);
  const liveInFlightRef = useRef(false);
  const liveInsertedTextRef = useRef(false);
  const liveSessionTextsRef = useRef<string[]>([]);
  const liveSessionDurationMsRef = useRef(0);
  const lastAudioQualityRef = useRef<AudioQualitySummary | null>(null);
  const asrConfigStatusRef = useRef(asrConfigStatus);
  const cloudAsrConfigStatusRef = useRef(cloudAsrConfigStatus);
  const pushToTalkModelRef = useRef(pushToTalkModel);
  const toggleDictationModelRef = useRef(toggleDictationModel);
  const diagnosticsEndRef = useRef<HTMLDivElement | null>(null);
  const [diagnostics, setDiagnostics] = useState<DiagnosticEntry[]>([
    {
      id: 1,
      time: new Date().toLocaleTimeString(),
      title: '应用启动',
      result: 'info',
      detail: '如果这里显示浏览器预览模式，麦克风、托盘和剪贴板上屏需要改用 npm run tauri -- dev 验证。',
    },
  ]);

  function addDiagnostic(entry: Omit<DiagnosticEntry, 'id' | 'time'>) {
    setDiagnostics((current) => [
      ...current.slice(Math.max(0, current.length - MAX_DIAGNOSTIC_ENTRIES + 1)),
      {
        ...entry,
        id: nextDiagnosticIdRef.current++,
        time: new Date().toLocaleTimeString(),
      },
    ]);
  }

  function getModelReadiness(modelId: TranscriptionModelId, asrStatus = asrConfigStatus, cloudStatus = cloudAsrConfigStatus): ModelReadiness {
    if (modelId === 'local-whisper') {
      return { id: modelId, label: 'whisper.cpp', ready: asrStatus.ready, message: asrStatus.message, availableInV7: true };
    }
    if (modelId === 'baidu-short') {
      return { id: modelId, label: '\u767e\u5ea6\u77ed\u8bed\u97f3 API', ready: cloudStatus.config.provider === 'baidu' && cloudStatus.ready, message: cloudStatus.message, availableInV7: true };
    }
    const cfg = cloudStatus.config;
    const realtimeAppId = cfg.baiduRealtimeAppId?.trim() ?? '';
    const realtimeDevPid = cfg.baiduRealtimeDevPid?.trim() ?? '';
    const realtimeReady = cloudStatus.apiKeyConfigured
      && Boolean(realtimeAppId)
      && /^\d+$/.test(realtimeAppId)
      && cfg.baiduRealtimeEndpoint === 'wss://vop.baidu.com/realtime_asr'
      && Boolean(realtimeDevPid)
      && /^\d+$/.test(realtimeDevPid)
      && Boolean(cfg.baiduRealtimeCuid?.trim())
      && cfg.baiduRealtimeFormat === 'pcm'
      && cfg.baiduRealtimeSampleRate === 16000;
    const message = realtimeReady ? '\u767e\u5ea6\u5b9e\u65f6 WebSocket API \u914d\u7f6e\u5b8c\u6574\u3002' : '\u767e\u5ea6\u5b9e\u65f6 WebSocket API \u9700\u8981 BAIDU_ASR_API_KEY\u3001\u6570\u5b57 AppID\u3001Endpoint\u3001\u6570\u5b57 dev_pid\u3001cuid\u3001pcm \u548c 16000 Hz\u3002';
    return { id: modelId, label: '\u767e\u5ea6\u5b9e\u65f6 WebSocket API', ready: realtimeReady, message, availableInV7: true };
  }

  function assertModelUsable(model: ModelReadiness) {
    if (!model.ready) {
      throw new Error(`${model.label} \u672a\u5c31\u7eea\uff1a${model.message}`);
    }
  }

  function cloudConfigFeedback(status: CloudAsrConfigStatus) {
    const realtime = getModelReadiness('baidu-realtime', asrConfigStatus, status);
    const shortState = status.ready ? '\u767e\u5ea6\u77ed\u8bed\u97f3 API \u5df2\u5c31\u7eea' : `\u767e\u5ea6\u77ed\u8bed\u97f3 API \u672a\u5c31\u7eea\uff1a${status.message}`;
    const realtimeState = realtime.ready ? '\u767e\u5ea6\u5b9e\u65f6 WebSocket API \u5df2\u5c31\u7eea' : `\u767e\u5ea6\u5b9e\u65f6 WebSocket API \u672a\u5c31\u7eea\uff1a${realtime.message}`;
    return `${shortState}\uff1b${realtimeState}`;
  }

  function applyAsrConfigStatus(nextStatus: AsrConfigStatus) {
    setAsrConfigStatus(nextStatus);
    setWhisperBinaryPath(nextStatus.whisperBinaryPath ?? '');
    setWhisperModelPath(nextStatus.whisperModelPath ?? '');
    setAsrLanguage(nextStatus.language);
  }

  function formatReplacementRules(config: TranscriptPostprocessConfig) {
    return config.replacements.map((rule) => `${rule.from} => ${rule.to}`).join('\n');
  }

  function formatGlossary(config: TranscriptPostprocessConfig) {
    return config.glossary.join('\n');
  }

  function applyTranscriptPostprocessConfig(config: TranscriptPostprocessConfig) {
    setTranscriptPostprocessConfig(config);
    setPostprocessReplacementText(formatReplacementRules(config));
    setPostprocessGlossaryText(formatGlossary(config));
  }

  function buildTranscriptPostprocessConfig(): TranscriptPostprocessConfig {
    const replacements = postprocessReplacementText
      .split(/\r?\n/)
      .map((line) => line.trim())
      .filter(Boolean)
      .map((line) => {
        const separator = line.includes('=>') ? '=>' : '->';
        const [from = '', to = ''] = line.split(separator).map((part) => part.trim());
        return { from, to, enabled: Boolean(from && to) };
      })
      .filter((rule) => rule.from && rule.to);
    const glossary = postprocessGlossaryText.split(/\r?\n/).map((line) => line.trim()).filter(Boolean);
    return { ...transcriptPostprocessConfig, glossary, replacements };
  }

  function applyCloudAsrConfigStatus(nextStatus: CloudAsrConfigStatus) {
    const provider = 'baidu';
    const statusConfig = provider === nextStatus.config.provider
      ? nextStatus.config
      : {
          ...nextStatus.config,
          provider,
          baseUrl: 'http://vop.baidu.com/server_api',
          model: '1537',
          baiduCuid: 'voxtype-local',
          baiduFormat: 'pcm',
          baiduSampleRate: provider === 'baidu' ? 16000 : null,
    };
    setCloudAsrConfigStatus({ ...nextStatus, config: statusConfig });
    setCloudBaseUrl(provider === 'baidu' && nextStatus.config.provider !== 'baidu' ? 'http://vop.baidu.com/server_api' : nextStatus.config.baseUrl ?? ('http://vop.baidu.com/server_api'));
    setCloudModel(provider === 'baidu' && nextStatus.config.provider !== 'baidu' ? '1537' : nextStatus.config.model ?? ('1537'));
    setCloudLanguage(nextStatus.config.language ?? 'zh');
    setCloudBaiduCuid(provider === 'baidu' && nextStatus.config.provider !== 'baidu' ? 'voxtype-local' : nextStatus.config.baiduCuid ?? 'voxtype-local');
    setCloudBaiduFormat(provider === 'baidu' && nextStatus.config.provider !== 'baidu' ? 'pcm' : nextStatus.config.baiduFormat ?? 'pcm');
    setCloudBaiduSampleRate(String(provider === 'baidu' && nextStatus.config.provider !== 'baidu' ? 16000 : nextStatus.config.baiduSampleRate ?? 16000));
    setCloudBaiduLmId(nextStatus.config.baiduLmId ?? '');
    setCloudBaiduRealtimeAppId(nextStatus.config.baiduRealtimeAppId ?? '10500017');
    setCloudBaiduRealtimeEndpoint(nextStatus.config.baiduRealtimeEndpoint ?? 'wss://vop.baidu.com/realtime_asr');
    setCloudBaiduRealtimeDevPid(nextStatus.config.baiduRealtimeDevPid ?? '15372');
    setCloudBaiduRealtimeCuid(nextStatus.config.baiduRealtimeCuid ?? 'voxtype-local');
    setCloudBaiduRealtimeFormat(nextStatus.config.baiduRealtimeFormat ?? 'pcm');
    setCloudBaiduRealtimeSampleRate(String(nextStatus.config.baiduRealtimeSampleRate ?? 16000));
    setCloudBaiduRealtimeUser(nextStatus.config.baiduRealtimeUser ?? '');
    setCloudMessage(nextStatus.message);
  }

  useEffect(() => {
    if (didLoadRuntime.current) {
      return;
    }
    didLoadRuntime.current = true;

    if (!isTauriRuntime()) {
      return;
    }

    setRuntimeMessage('Tauri 运行中：可以验证系统能力');
    void getConfig().then(setConfig).catch((error: unknown) => {
      const detail = formatError(error);
      setRuntimeMessage(`读取配置失败：${detail}`);
      addDiagnostic({ title: '读取配置失败', result: 'error', detail });
    });
    void getAsrConfigStatus().then(applyAsrConfigStatus).catch((error: unknown) => {
      const detail = formatError(error);
      setRuntimeMessage(`读取 ASR 配置失败：${detail}`);
      addDiagnostic({ title: '读取 ASR 配置失败', result: 'error', detail });
    });
    void getCloudAsrConfigStatus().then(applyCloudAsrConfigStatus).catch((error: unknown) => {
      const detail = formatError(error);
      addDiagnostic({ title: '读取云端 ASR 配置失败', result: 'error', detail });
    });
    void getDefaultInputInfo()
      .then((info) => {
        setRecorderInfo(info);
        setSelectedInputDeviceName(info.deviceName);
        addDiagnostic({
          title: '麦克风探测成功',
          result: 'success',
          detail: `${info.deviceName} / ${info.sampleRate} Hz / ${info.channels} 声道。`,
        });
      })
      .catch((error: unknown) => {
        const detail = formatError(error);
        setRuntimeMessage(`读取麦克风失败：${detail}`);
        addDiagnostic({ title: '麦克风探测失败', result: 'error', detail });
      });
    void getUserPreferences()
      .then((preferences) => {
        setPreferredInputDeviceName(preferences.selectedInputDeviceName);
        setPushToTalkHotkey(preferences.pushToTalkHotkey ?? defaultPushToTalkHotkey);
        setToggleDictationHotkeyInput(preferences.toggleDictationHotkey ?? defaultToggleDictationHotkey);
        setPushToTalkModel(preferences.pushToTalkModel ?? 'baidu-short');
        setToggleDictationModel(preferences.toggleDictationModel ?? 'baidu-short');
      })
      .catch((error: unknown) => {
        addDiagnostic({ title: '读取用户偏好失败', result: 'error', detail: formatError(error) });
      });
    void getHotkeyStatus()
      .then((nextStatus) => {
        setHotkeyStatus(nextStatus);
        addDiagnostic({ title: nextStatus.registered ? '全局快捷键已注册' : '全局快捷键未就绪', result: nextStatus.registered ? 'success' : 'warning', detail: nextStatus.message });
      })
      .catch((error: unknown) => {
        addDiagnostic({ title: '读取全局快捷键状态失败', result: 'error', detail: formatError(error) });
      });
    void getOverlayBackendStatus()
      .then((nextStatus) => {
        setOverlayBackendStatus(nextStatus);
        addDiagnostic({ title: '桌面浮窗后端已读取', result: nextStatus.lastError ? 'warning' : 'info', detail: nextStatus.lastError ? `${nextStatus.backend}：${nextStatus.lastError}` : nextStatus.backend });
      })
      .catch((error: unknown) => {
        addDiagnostic({ title: '读取桌面浮窗后端失败', result: 'error', detail: formatError(error) });
      });
    void loadTranscriptHistory()
      .then((entries) => {
        setTranscriptRecords(entries.map(mapPersistedEntry));
      })
      .catch((error: unknown) => {
        addDiagnostic({ title: '读取识别记录失败', result: 'warning', detail: formatError(error) });
      });
    void getTranscriptPostprocessConfig()
      .then(applyTranscriptPostprocessConfig)
      .catch((error: unknown) => {
        addDiagnostic({ title: '读取文本优化配置失败', result: 'warning', detail: formatError(error) });
      });
    void listInputDevices()
      .then((devices) => {
        setInputDevices(devices);
        addDiagnostic({
          title: '输入设备列表已读取',
          result: 'info',
          detail: `发现 ${devices.length} 个输入设备：${devices.map((device) => device.deviceName).join('；')}`,
        });
      })
      .catch((error: unknown) => {
        addDiagnostic({ title: '读取输入设备列表失败', result: 'error', detail: formatError(error) });
      });
  }, []);

  useEffect(() => {
    if (!isTauriRuntime() || !preferredInputDeviceName || inputDevices.length === 0 || isRecordingRef.current) {
      return;
    }
    if (selectedInputDeviceName === preferredInputDeviceName) {
      return;
    }
    const hasPreferredDevice = inputDevices.some((device) => device.deviceName === preferredInputDeviceName);
    if (hasPreferredDevice) {
      void handleSelectInputDevice(preferredInputDeviceName);
    } else {
      addDiagnostic({ title: '已保存输入设备不可用', result: 'warning', detail: `没有在当前系统输入设备列表中找到：${preferredInputDeviceName}` });
    }
  }, [preferredInputDeviceName, inputDevices, selectedInputDeviceName]);

  useEffect(() => {
    diagnosticsEndRef.current?.scrollIntoView?.({ block: 'end' });
  }, [diagnostics, viewMode]);

  const isRecording = recordingStatus.state === 'recording';
  isRecordingRef.current = isRecording;
  asrConfigStatusRef.current = asrConfigStatus;
  cloudAsrConfigStatusRef.current = cloudAsrConfigStatus;
  pushToTalkModelRef.current = pushToTalkModel;
  toggleDictationModelRef.current = toggleDictationModel;


  function mapPersistedEntry(entry: PersistedTranscriptEntry): TranscriptRecord {
    return {
      id: entry.id,
      time: new Date(entry.createdAtMs).toLocaleTimeString(),
      text: entry.text,
      durationMs: entry.durationMs,
      inputMode: entry.inputMode,
      modelId: entry.model,
      charCount: entry.characterCount,
      postprocessRulesApplied: entry.postprocessRulesApplied,
      audioQuality: entry.audioQuality,
    };
  }

  async function postprocessText(text: string) {
    if (!isTauriRuntime()) {
      return { text: text.trim(), rulesApplied: 0, noiseRemoved: isLikelyNoiseTranscript(text) };
    }
    try {
      return await previewTranscriptPostprocess(text);
    } catch (error) {
      addDiagnostic({ title: '文本优化失败', result: 'warning', detail: formatError(error) });
      return { text: text.trim(), rulesApplied: 0, noiseRemoved: false };
    }
  }
  const transcriptStats = useMemo<TranscriptStats>(() => {
    const totalDurationMs = transcriptRecords.reduce((sum, record) => sum + record.durationMs, 0);
    const totalChars = transcriptRecords.reduce((sum, record) => sum + record.charCount, 0);
    return {
      count: transcriptRecords.length,
      totalDurationMs,
      totalChars,
      charsPerMinute: totalDurationMs > 0 ? Math.round(totalChars / (totalDurationMs / 60000)) : 0,
    };
  }, [transcriptRecords]);

  async function addTranscriptRecord(text: string, durationMs: number, inputMode: TranscriptRecord['inputMode'], modelId: TranscriptionModelId = inputMode === 'toggle-dictation' ? toggleDictationModel : inputMode === 'push-to-talk' ? pushToTalkModel : 'baidu-short', processedOverride?: { text: string; rulesApplied: number; noiseRemoved: boolean }) {
    const processed = processedOverride ?? await postprocessText(text);
    const normalizedText = processed.text.trim();
    if (!normalizedText || processed.noiseRemoved) {
      return null;
    }
    setHistoryMessage(null);
    const audioQuality = lastAudioQualityRef.current;
    lastAudioQualityRef.current = null;
    const createdAtMs = Date.now();
    const entry: PersistedTranscriptEntry = {
      id: `local-${createdAtMs}-${nextTranscriptRecordIdRef.current++}`,
      text: normalizedText,
      inputMode,
      model: modelId,
      createdAtMs,
      durationMs: Math.max(0, Math.round(durationMs)),
      characterCount: normalizedText.length,
      postprocessRulesApplied: processed.rulesApplied,
      audioQuality,
    };
    const record = mapPersistedEntry(entry);
    setTranscriptRecords((current) => [record, ...current]);
    if (isTauriRuntime()) {
      try {
        const saved = await saveTranscriptHistoryEntry(entry);
        setTranscriptRecords(saved.map(mapPersistedEntry));
      } catch (error) {
        addDiagnostic({ title: '识别记录持久化失败', result: 'warning', detail: formatError(error) });
      }
    }
    return record;
  }

  function isLikelyNoiseTranscript(text: string) {
    const normalizedText = text.trim().replace(/\s+/g, ' ');
    if (!normalizedText) {
      return true;
    }
    const unwrapped = normalizedText.replace(/^[（(]\s*/, '').replace(/\s*[）)]$/, '').trim();
    const isWrapped = unwrapped !== normalizedText;
    const knownNoise = ['字幕:J Chong', '字幕: J Chong', 'J Chong', '不知', '無法 寶寶', '无法 宝宝'];
    return isWrapped && knownNoise.some((pattern) => unwrapped.includes(pattern));
  }

  function rememberLiveSessionText(text: string, durationMs: number) {
    const normalizedText = text.trim();
    if (!normalizedText || isLikelyNoiseTranscript(normalizedText)) {
      return;
    }
    liveSessionTextsRef.current.push(normalizedText);
    liveSessionDurationMsRef.current += Math.max(0, Math.round(durationMs));
  }

  function combinedLiveSessionText() {
    return liveSessionTextsRef.current.join(' ').replace(/\s+/g, ' ').trim();
  }

  async function handleCopyRecord(record: TranscriptRecord) {
    try {
      await navigator.clipboard.writeText(record.text);
      addDiagnostic({ title: '识别记录已复制', result: 'success', detail: `文本：${record.text}` });
    } catch (error) {
      addDiagnostic({ title: '复制识别记录失败', result: 'error', detail: formatError(error) });
    }
  }

  async function handleReinsertRecord(record: TranscriptRecord) {
    if (!isTauriRuntime()) {
      addDiagnostic({ title: '重新上屏未执行', result: 'warning', detail: '浏览器预览模式不能调用 Windows 剪贴板上屏。' });
      return;
    }
    try {
      await insertTextWithClipboard(record.text);
      setStatus({ phase: 'succeeded', message: '识别记录已重新上屏', lastTranscript: record.text });
      addDiagnostic({ title: '识别记录重新上屏请求已发送', result: 'success', detail: `文本：${record.text}` });
    } catch (error) {
      const detail = formatError(error);
      setStatus({ phase: 'failed', message: `重新上屏失败：${detail}`, lastTranscript: null });
      addDiagnostic({ title: '识别记录重新上屏失败', result: 'error', detail });
    }
  }

  async function handleDeleteRecord(id: string) {
    setTranscriptRecords((current) => current.filter((record) => record.id !== id));
    if (isTauriRuntime()) {
      try {
        const saved = await deleteTranscriptHistoryEntry(id);
        setTranscriptRecords(saved.map(mapPersistedEntry));
      } catch (error) {
        addDiagnostic({ title: '删除识别记录持久化失败', result: 'warning', detail: formatError(error) });
      }
    }
  }

  async function handleClearRecords() {
    setTranscriptRecords([]);
    setHistoryMessage(null);
    if (isTauriRuntime()) {
      try {
        await clearTranscriptHistory();
      } catch (error) {
        addDiagnostic({ title: '清空识别记录持久化失败', result: 'warning', detail: formatError(error) });
      }
    }
    addDiagnostic({ title: '识别记录已清空', result: 'info', detail: '识别记录已清空。' });
  }

  async function handleSaveTranscriptPostprocessConfig() {
    const nextConfig = buildTranscriptPostprocessConfig();
    if (!isTauriRuntime()) {
      applyTranscriptPostprocessConfig(nextConfig);
      setPostprocessMessage('浏览器预览模式已暂存文本优化配置');
      return;
    }
    try {
      const saved = await saveTranscriptPostprocessConfig(nextConfig);
      applyTranscriptPostprocessConfig(saved);
      setPostprocessMessage('文本优化配置已保存');
      addDiagnostic({ title: '文本优化配置已保存', result: 'success', detail: `替换规则 ${saved.replacements.length} 条，词表 ${saved.glossary.length} 项。` });
    } catch (error) {
      const detail = formatError(error);
      setPostprocessMessage(`保存文本优化配置失败：${detail}`);
      addDiagnostic({ title: '保存文本优化配置失败', result: 'error', detail });
    }
  }

  async function handlePreviewTranscriptPostprocess() {
    const input = postprocessPreviewInput.trim();
    if (!input) {
      setPostprocessPreviewOutput(null);
      setPostprocessMessage('请输入要预览的文本');
      return;
    }
    if (!isTauriRuntime()) {
      const fallback = input.replace(/scale/g, 'skill').replace(/websocket/gi, 'WebSocket').replace(/whisper cpp/gi, 'whisper.cpp');
      setPostprocessPreviewOutput(fallback);
      setPostprocessMessage('浏览器预览模式使用前端示例规则');
      return;
    }
    try {
      const result = await previewTranscriptPostprocess(input);
      setPostprocessPreviewOutput(result.noiseRemoved ? '' : result.text);
      setPostprocessMessage(`预览完成：应用 ${result.rulesApplied} 条规则${result.noiseRemoved ? '，已过滤噪声文本' : ''}`);
    } catch (error) {
      const detail = formatError(error);
      setPostprocessMessage(`预览文本优化失败：${detail}`);
      addDiagnostic({ title: '预览文本优化失败', result: 'warning', detail });
    }
  }

  async function handleExportRecords() {
    const text = transcriptRecords.map((record) => `[${record.time}] ${record.text}`).join('\n');
    if (!text) {
      setHistoryMessage('没有可导出的识别记录');
      return;
    }
    try {
      await navigator.clipboard.writeText(text);
      setHistoryMessage(`已复制 ${transcriptRecords.length} 条记录到剪贴板`);
      addDiagnostic({ title: '识别记录已导出', result: 'success', detail: `已复制 ${transcriptRecords.length} 条识别记录到剪贴板。` });
    } catch (error) {
      const detail = formatError(error);
      setHistoryMessage(`导出失败：${detail}`);
      addDiagnostic({ title: '导出识别记录失败', result: 'error', detail });
    }
  }

  async function handleSimulateDictation() {
    if (isTauriRuntime()) {
      try {
        const nextStatus = await simulateDictation();
        setStatus(nextStatus);
        addDiagnostic({ title: '模拟闭环成功', result: 'success', detail: `返回文本：${nextStatus.lastTranscript ?? '无'}` });
        return;
      } catch (error) {
        const detail = formatError(error);
        setStatus({ phase: 'failed', message: `模拟闭环失败：${detail}`, lastTranscript: null });
        addDiagnostic({ title: '模拟闭环失败', result: 'error', detail });
        return;
      }
    }

    setStatus({
      phase: 'succeeded',
      message: '模拟闭环完成',
      lastTranscript: '这是 VoxType 的中文优先语音输入测试。',
    });
    addDiagnostic({ title: '浏览器 mock 成功', result: 'warning', detail: '这是纯前端模拟结果。' });
  }

  async function handleClipboardInsert() {
    const text = status.lastTranscript ?? '这是 VoxType 的剪贴板上屏测试。';
    if (!isTauriRuntime()) {
      addDiagnostic({ title: '剪贴板上屏未执行', result: 'warning', detail: '浏览器预览模式不能调用 Windows 剪贴板上屏。' });
      return;
    }
    try {
      await insertTextWithClipboard(text);
      setStatus({ phase: 'succeeded', message: '已发送剪贴板上屏请求', lastTranscript: text });
      addDiagnostic({ title: '剪贴板上屏请求已发送', result: 'success', detail: `文本：${text}` });
    } catch (error) {
      const detail = formatError(error);
      setStatus({ phase: 'failed', message: `剪贴板上屏失败：${detail}`, lastTranscript: null });
      addDiagnostic({ title: '剪贴板上屏失败', result: 'error', detail });
    }
  }

  async function handleCopyLastTranscript() {
    const text = status.lastTranscript?.trim();
    if (!text) return;
    try {
      await navigator.clipboard.writeText(text);
      addDiagnostic({ title: '最近文本已复制', result: 'success', detail: `文本：${text}` });
    } catch (error) {
      addDiagnostic({ title: '复制最近文本失败', result: 'error', detail: formatError(error) });
    }
  }

  async function handleReinsertLastTranscript() {
    const text = status.lastTranscript?.trim();
    if (!text) return;
    await handleClipboardInsert();
  }

  function handleClearLastTranscript() {
    setStatus((current) => ({ ...current, lastTranscript: null }));
    addDiagnostic({ title: '最近文本已清空', result: 'info', detail: '主界面的最近识别文本已清空。' });
  }

  async function handleCopyDiagnostics() {
    const text = formatDiagnosticsForCopy(diagnostics);
    if (!text) {
      return;
    }
    try {
      await navigator.clipboard.writeText(text);
      addDiagnostic({ title: '诊断日志已复制', result: 'success', detail: `已复制 ${diagnostics.length} 条诊断日志。` });
    } catch (error) {
      addDiagnostic({ title: '复制诊断日志失败', result: 'error', detail: formatError(error) });
    }
  }

  async function handleRefreshAsrConfig() {
    if (!isTauriRuntime()) {
      addDiagnostic({ title: 'ASR 配置未检测', result: 'warning', detail: '浏览器预览模式不能读取本机配置。' });
      return;
    }
    try {
      const nextStatus = await getAsrConfigStatus();
      applyAsrConfigStatus(nextStatus);
      addDiagnostic({ title: nextStatus.ready ? 'ASR 配置检测通过' : 'ASR 配置未就绪', result: nextStatus.ready ? 'success' : 'warning', detail: nextStatus.message });
    } catch (error) {
      addDiagnostic({ title: 'ASR 配置检测失败', result: 'error', detail: formatError(error) });
    }
  }


  async function handleShowOverlayTest() {
    if (!isTauriRuntime()) {
      addDiagnostic({ title: '桌面浮窗未显示', result: 'warning', detail: '浏览器预览模式不能显示 Tauri 桌面浮窗。' });
      return;
    }
    try {
      await showDictationOverlay();
      addDiagnostic({ title: '桌面浮窗显示请求已发送', result: 'success', detail: '如果底部没有出现小型彩色语音浮窗，说明 overlay 窗口配置或系统窗口层级需要继续排查。' });
    } catch (error) {
      addDiagnostic({ title: '桌面浮窗显示失败', result: 'error', detail: formatError(error) });
    }
  }

  async function handleHideOverlayTest() {
    if (!isTauriRuntime()) {
      return;
    }
    try {
      await hideDictationOverlay();
      addDiagnostic({ title: '桌面浮窗已隐藏', result: 'info', detail: '已发送隐藏 overlay 窗口请求。' });
    } catch (error) {
      addDiagnostic({ title: '桌面浮窗隐藏失败', result: 'error', detail: formatError(error) });
    }
  }
  async function handleSaveHotkeyPreferences() {
    const nextPushToTalkHotkey = pushToTalkHotkey.trim();
    const nextToggleDictationHotkey = toggleDictationHotkeyInput.trim();
    if (!isTauriRuntime()) {
      setHotkeySaveMessage('\u9700\u8981 Tauri \u684c\u9762\u73af\u5883\u4fdd\u5b58\u5feb\u6377\u952e\u3002');
      addDiagnostic({ title: '\u5feb\u6377\u952e\u672a\u4fdd\u5b58', result: 'warning', detail: '\u6d4f\u89c8\u5668\u9884\u89c8\u6a21\u5f0f\u4e0d\u80fd\u5199\u5165 Tauri \u5e94\u7528\u914d\u7f6e\u3002' });
      return;
    }
    setIsSavingHotkeys(true);
    setHotkeySaveMessage(null);
    try {
      const nextStatus = await saveHotkeyPreferences(nextPushToTalkHotkey, nextToggleDictationHotkey);
      setPushToTalkHotkey(nextPushToTalkHotkey);
      setToggleDictationHotkeyInput(nextToggleDictationHotkey);
      setHotkeyStatus(nextStatus);
      const message = `${nextStatus.message}\u3002\u5df2\u5c1d\u8bd5\u7acb\u5373\u91cd\u65b0\u7ed1\u5b9a\u5168\u5c40\u5feb\u6377\u952e\u3002`;
      setHotkeySaveMessage(message);
      addDiagnostic({ title: nextStatus.registered ? '\u5feb\u6377\u952e\u504f\u597d\u5df2\u4fdd\u5b58' : '\u5feb\u6377\u952e\u504f\u597d\u4fdd\u5b58\u540e\u672a\u5c31\u7eea', result: nextStatus.registered ? 'success' : 'warning', detail: message });
    } catch (error) {
      const detail = formatError(error);
      setHotkeySaveMessage(detail);
      addDiagnostic({ title: '\u5feb\u6377\u952e\u504f\u597d\u4fdd\u5b58\u5931\u8d25', result: 'error', detail });
    } finally {
      setIsSavingHotkeys(false);
    }
  }

  async function handleRefreshHotkeyStatus() {
    if (!isTauriRuntime()) {
      addDiagnostic({ title: '全局快捷键状态未读取', result: 'warning', detail: '浏览器预览模式不能读取 Tauri 全局快捷键注册状态。' });
      return;
    }
    try {
      const nextStatus = await getHotkeyStatus();
      setHotkeyStatus(nextStatus);
      addDiagnostic({ title: nextStatus.registered ? '全局快捷键已注册' : '全局快捷键未就绪', result: nextStatus.registered ? 'success' : 'warning', detail: nextStatus.message });
    } catch (error) {
      addDiagnostic({ title: '读取全局快捷键状态失败', result: 'error', detail: formatError(error) });
    }
  }

  async function handleSaveCloudAsrConfig() {
    if (!isTauriRuntime()) {
      setCloudMessage('浏览器预览模式不能写入 Tauri 应用配置。');
      addDiagnostic({ title: '云端 ASR 配置未保存', result: 'warning', detail: '浏览器预览模式不能写入 Tauri 应用配置。' });
      return;
    }
    const baiduSampleRate = Number.parseInt(cloudBaiduSampleRate, 10);
    try {
      const nextStatus = await saveCloudAsrConfig({
        provider: 'baidu',
        groupId: null,
        baseUrl: cloudBaseUrl.trim() || null,
        model: cloudModel.trim() || null,
        language: cloudLanguage.trim() || 'zh',
        baiduCuid: cloudBaiduCuid.trim() || null,
        baiduFormat: cloudBaiduFormat.trim() || null,
        baiduSampleRate: Number.isFinite(baiduSampleRate) ? baiduSampleRate : null,
        baiduLmId: cloudBaiduLmId.trim() || null,
        baiduRealtimeAppId: cloudBaiduRealtimeAppId.trim() || null,
        baiduRealtimeEndpoint: cloudBaiduRealtimeEndpoint.trim() || null,
        baiduRealtimeDevPid: cloudBaiduRealtimeDevPid.trim() || null,
        baiduRealtimeCuid: cloudBaiduRealtimeCuid.trim() || null,
        baiduRealtimeFormat: cloudBaiduRealtimeFormat.trim() || null,
        baiduRealtimeSampleRate: Number.isFinite(Number.parseInt(cloudBaiduRealtimeSampleRate, 10)) ? Number.parseInt(cloudBaiduRealtimeSampleRate, 10) : null,
        baiduRealtimeUser: cloudBaiduRealtimeUser.trim() || null,
      });
      applyCloudAsrConfigStatus(nextStatus);
      const feedback = cloudConfigFeedback(nextStatus);
      setCloudMessage(`百度配置已保存：${feedback}`);
      const realtimeReady = getModelReadiness('baidu-realtime', asrConfigStatus, nextStatus).ready;
      addDiagnostic({ title: nextStatus.ready && realtimeReady ? '百度配置已保存并就绪' : '百度配置已保存但未完全就绪', result: nextStatus.ready && realtimeReady ? 'success' : 'warning', detail: feedback });
    } catch (error) {
      const detail = formatError(error);
      setCloudMessage(detail);
      addDiagnostic({ title: '云端 ASR 配置保存失败', result: 'error', detail });
    }
  }

  async function handleChangeModeModelPreference(mode: 'push-to-talk' | 'toggle-dictation', nextModel: TranscriptionModelId) {
    const nextPushToTalkModel = mode === 'push-to-talk' ? nextModel : pushToTalkModel;
    const nextToggleDictationModel = mode === 'toggle-dictation' ? nextModel : toggleDictationModel;
    setPushToTalkModel(nextPushToTalkModel);
    setToggleDictationModel(nextToggleDictationModel);
    pushToTalkModelRef.current = nextPushToTalkModel;
    toggleDictationModelRef.current = nextToggleDictationModel;
    if (!isTauriRuntime()) {
      addDiagnostic({ title: '默认模型仅在预览中切换', result: 'warning', detail: '浏览器预览模式不能写入 Tauri 用户偏好。' });
      return;
    }
    try {
      const preferences = await saveModeModelPreferences(nextPushToTalkModel, nextToggleDictationModel);
      const savedPushToTalkModel = preferences.pushToTalkModel ?? nextPushToTalkModel;
      const savedToggleDictationModel = preferences.toggleDictationModel ?? nextToggleDictationModel;
      setPushToTalkModel(savedPushToTalkModel);
      setToggleDictationModel(savedToggleDictationModel);
      pushToTalkModelRef.current = savedPushToTalkModel;
      toggleDictationModelRef.current = savedToggleDictationModel;
      addDiagnostic({ title: '默认模型偏好已保存', result: 'success', detail: `按住说话：${preferences.pushToTalkModel ?? nextPushToTalkModel}；连续输入：${preferences.toggleDictationModel ?? nextToggleDictationModel}` });
    } catch (error) {
      addDiagnostic({ title: '默认模型偏好保存失败', result: 'error', detail: formatError(error) });
    }
  }


  async function handleSaveBaiduAsrApiKey() {
    const apiKey = cloudApiKeyInput.trim();
    if (!apiKey) {
      setCloudMessage('请输入百度 ASR API Key。');
      return;
    }
    if (!isTauriRuntime()) {
      setCloudMessage('浏览器预览模式不能写入系统环境变量。');
      addDiagnostic({ title: '百度 ASR API Key 未保存', result: 'warning', detail: '浏览器预览模式不能写入系统环境变量。' });
      return;
    }
    try {
      const nextStatus = await saveBaiduAsrApiKey(apiKey);
      setCloudApiKeyInput('');
      applyCloudAsrConfigStatus(nextStatus);
      setCloudMessage('百度 ASR API Key 已写入用户环境变量。重启 VoxType 后系统进程也会读取到新值。');
      addDiagnostic({ title: '百度 ASR API Key 已保存', result: nextStatus.apiKeyConfigured ? 'success' : 'warning', detail: '已写入用户环境变量 BAIDU_ASR_API_KEY，未记录密钥明文。' });
    } catch (error) {
      const detail = formatError(error);
      setCloudMessage(detail);
      addDiagnostic({ title: '百度 ASR API Key 保存失败', result: 'error', detail });
    }
  }

  async function handleSaveBaiduAsrSecretKey() {
    const secretKey = cloudSecretKeyInput.trim();
    if (!secretKey) {
      setCloudMessage('请输入百度 ASR Secret Key。');
      return;
    }
    if (!isTauriRuntime()) {
      setCloudMessage('浏览器预览模式不能写入系统环境变量。');
      addDiagnostic({ title: '百度 ASR Secret Key 未保存', result: 'warning', detail: '浏览器预览模式不能写入系统环境变量。' });
      return;
    }
    try {
      const nextStatus = await saveBaiduAsrSecretKey(secretKey);
      setCloudSecretKeyInput('');
      applyCloudAsrConfigStatus(nextStatus);
      setCloudMessage('百度 ASR Secret Key 已写入用户环境变量。重启 VoxType 后系统进程也会读取到新值。');
      addDiagnostic({ title: '百度 ASR Secret Key 已保存', result: nextStatus.secretKeyConfigured ? 'success' : 'warning', detail: '已写入用户环境变量 BAIDU_ASR_SECRET_KEY，未记录密钥明文。' });
    } catch (error) {
      const detail = formatError(error);
      setCloudMessage(detail);
      addDiagnostic({ title: '百度 ASR Secret Key 保存失败', result: 'error', detail });
    }
  }

  async function handleTestCloudAsrConfig() {
    if (!isTauriRuntime()) {
      setCloudMessage('浏览器预览模式不能读取云端 ASR 配置。');
      addDiagnostic({ title: '云端 ASR 配置未检测', result: 'warning', detail: '浏览器预览模式不能读取 Tauri 应用配置。' });
      return;
    }
    try {
      const nextStatus = await getCloudAsrConfigStatus();
      applyCloudAsrConfigStatus(nextStatus);
      const feedback = cloudConfigFeedback(nextStatus);
      const realtimeReady = getModelReadiness('baidu-realtime', asrConfigStatus, nextStatus).ready;
      const message = `百度配置检测完成：${feedback}`;
      setCloudMessage(message);
      addDiagnostic({ title: nextStatus.ready && realtimeReady ? '百度配置检测通过' : '百度配置未完全就绪', result: nextStatus.ready && realtimeReady ? 'success' : 'warning', detail: message });
    } catch (error) {
      const detail = formatError(error);
      setCloudMessage(detail);
      addDiagnostic({ title: '云端 ASR 配置检测失败', result: 'error', detail });
    }
  }

  async function handleSaveAsrConfig() {
    if (!isTauriRuntime()) {
      addDiagnostic({ title: 'ASR 配置未保存', result: 'warning', detail: '浏览器预览模式不能写入 Tauri 应用配置。' });
      return;
    }
    try {
      const nextStatus = await saveAsrConfig({
        whisperBinaryPath: whisperBinaryPath.trim() || null,
        whisperModelPath: whisperModelPath.trim() || null,
        language: asrLanguage.trim() || 'zh',
      });
      applyAsrConfigStatus(nextStatus);
      addDiagnostic({ title: nextStatus.ready ? 'ASR 配置已保存并就绪' : 'ASR 配置已保存但未就绪', result: nextStatus.ready ? 'success' : 'warning', detail: nextStatus.message });
    } catch (error) {
      addDiagnostic({ title: 'ASR 配置保存失败', result: 'error', detail: formatError(error) });
    }
  }

  async function handleInstallManagedAsr() {
    if (!isTauriRuntime()) {
      addDiagnostic({ title: '一键安装未执行', result: 'warning', detail: '浏览器预览模式不能下载和写入本机应用数据目录。' });
      return;
    }
    setIsInstallingAsr(true);
    addDiagnostic({ title: '一键安装已开始', result: 'info', detail: '正在下载 whisper.cpp 和模型。若网络不可达，后续会显示失败原因。' });
    try {
      const nextStatus = await installManagedAsr();
      applyAsrConfigStatus(nextStatus);
      addDiagnostic({ title: nextStatus.ready ? '一键安装完成' : '一键安装完成但配置未就绪', result: nextStatus.ready ? 'success' : 'warning', detail: nextStatus.ready ? 'whisper.cpp 和模型已安装并保存配置。' : nextStatus.message });
    } catch (error) {
      addDiagnostic({ title: '一键安装失败', result: 'error', detail: formatError(error) });
    } finally {
      setIsInstallingAsr(false);
    }
  }

  async function handleStartRecording() {
    if (!isTauriRuntime()) {
      addDiagnostic({ title: '录音未启动', result: 'warning', detail: '浏览器预览模式不能访问系统麦克风录音流。' });
      return false;
    }
    try {
      const nextStatus = await startRecording();
      setRecordingStatus(nextStatus);
      setStatus({ phase: 'recording', message: '正在录音，说完后点击停止录音', lastTranscript: null });
      addDiagnostic({ title: '录音已启动', result: 'success', detail: `采样率 ${nextStatus.sampleRate ?? '未知'} Hz，输入声道 ${nextStatus.channels ?? '未知'}。` });
      return true;
    } catch (error) {
      const detail = formatError(error);
      setStatus({ phase: 'failed', message: `启动录音失败：${detail}`, lastTranscript: null });
      addDiagnostic({ title: '启动录音失败', result: 'error', detail });
      return false;
    }
  }

  function stopLiveTranscriptionTimer() {
    if (liveTimerRef.current !== null) {
      window.clearInterval(liveTimerRef.current);
      liveTimerRef.current = null;
    }
  }

  async function processLiveTranscriptionChunk({ quietShortChunk = false } = {}) {
    if (!isTauriRuntime() || liveInFlightRef.current) {
      return;
    }
    liveInFlightRef.current = true;
    try {
      const chunk = await transcribeActiveRecordingChunk(liveCursorRef.current);
      liveCursorRef.current = chunk.toSampleIndex;
      const text = chunk.transcript.text.trim();
      if (!text) {
        return;
      }
      setStatus({ phase: 'recording', message: '实验实时输入中，继续说话即可分段上屏', lastTranscript: text });
      await insertTextWithClipboard(text);
      rememberLiveSessionText(text, Math.round(chunk.asrSampleCount / 16));
      liveInsertedTextRef.current = true;
      addDiagnostic({ title: '实验实时片段已上屏', result: 'success', detail: `${chunk.transcript.engine} 返回 ${chunk.asrSampleCount} 个 ASR 样本的片段：${text}` });
    } catch (error) {
      const detail = formatError(error);
      if (quietShortChunk && detail.includes('录音片段太短')) {
        return;
      }
      addDiagnostic({ title: '实验实时片段转写失败', result: 'warning', detail });
    } finally {
      liveInFlightRef.current = false;
    }
  }

  async function processFinalLiveTranscriptionChunk() {
    if (!isTauriRuntime()) {
      return;
    }
    try {
      const chunk = await transcribeLastRecordingChunk(liveCursorRef.current);
      liveCursorRef.current = chunk.toSampleIndex;
      const text = chunk.transcript.text.trim();
      if (!text) {
        return;
      }
      await insertTextWithClipboard(text);
      rememberLiveSessionText(text, Math.round(chunk.asrSampleCount / 16));
      liveInsertedTextRef.current = true;
      setStatus({ phase: 'succeeded', message: '实验实时输入尾段已上屏', lastTranscript: text });
      addDiagnostic({ title: '实验实时尾段已上屏', result: 'success', detail: `${chunk.transcript.engine} 返回 ${chunk.asrSampleCount} 个 ASR 样本的尾段：${text}` });
    } catch (error) {
      addDiagnostic({ title: '实验实时尾段转写失败', result: 'warning', detail: formatError(error) });
    }
  }

  async function handleStartLiveToggleRecording() {
    const selectedModel = toggleDictationModelRef.current;
    const model = getModelReadiness(selectedModel, asrConfigStatusRef.current, cloudAsrConfigStatusRef.current);
    try {
      assertModelUsable(model);
    } catch (error) {
      const detail = formatError(error);
      setStatus({ phase: 'failed', message: `连续输入模型不可用：${detail}`, lastTranscript: null });
      setHistoryMessage(`连续输入模型不可用：${detail}`);
      addDiagnostic({ title: '连续输入模型不可用', result: 'error', detail });
      return;
    }
    liveCursorRef.current = 0;
    liveInsertedTextRef.current = false;
    liveSessionTextsRef.current = [];
    liveSessionDurationMsRef.current = 0;
    stopLiveTranscriptionTimer();
    if (selectedModel === 'baidu-realtime') {
      if (!isTauriRuntime()) {
        addDiagnostic({ title: '\u767e\u5ea6\u5b9e\u65f6\u8f93\u5165\u672a\u542f\u52a8', result: 'warning', detail: '\u6d4f\u89c8\u5668\u9884\u89c8\u6a21\u5f0f\u4e0d\u80fd\u8fde\u63a5\u767e\u5ea6\u5b9e\u65f6 WebSocket API\u3002' });
        return;
      }
      try {
        lastAudioQualityRef.current = null;
        const realtimeStatus = await startBaiduRealtimeSession();
        isRecordingRef.current = true;
        setRecordingStatus({ state: 'recording', sampleRate: 16000, channels: 1, sampleCount: 0, durationMs: realtimeStatus.durationMs });
        setStatus({ phase: 'recording', message: '\u767e\u5ea6\u5b9e\u65f6 WebSocket \u8f93\u5165\u4e2d\uff0c\u6700\u7ec8\u7ed3\u679c\u4f1a\u5b9e\u65f6\u4e0a\u5c4f', lastTranscript: null });
        addDiagnostic({ title: '\u767e\u5ea6\u5b9e\u65f6 WebSocket \u8f93\u5165\u5df2\u542f\u52a8', result: 'success', detail: realtimeStatus.message });
      } catch (error) {
        const detail = formatError(error);
        setStatus({ phase: 'failed', message: `\u767e\u5ea6\u5b9e\u65f6\u8f93\u5165\u542f\u52a8\u5931\u8d25\uff1a${detail}`, lastTranscript: null });
        setHistoryMessage(`\u767e\u5ea6\u5b9e\u65f6\u8f93\u5165\u542f\u52a8\u5931\u8d25\uff1a${detail}`);
        addDiagnostic({ title: '\u767e\u5ea6\u5b9e\u65f6 WebSocket \u8f93\u5165\u542f\u52a8\u5931\u8d25', result: 'error', detail });
      }
      return;
    }
    const started = await handleStartRecording();
    if (!started) {
      return;
    }
    if (selectedModel === 'local-whisper') {
      setStatus({ phase: 'recording', message: '实验实时输入中：会每隔几秒分段转写并上屏', lastTranscript: null });
      addDiagnostic({ title: '实验实时输入已启动', result: 'info', detail: '当前不是 whisper.cpp 真流式 partial，而是录音中定时切片、转写新增片段并上屏。' });
      liveTimerRef.current = window.setInterval(() => {
        void processLiveTranscriptionChunk({ quietShortChunk: true });
      }, LIVE_TRANSCRIPTION_INTERVAL_MS);
    } else {
      setStatus({ phase: 'recording', message: '连续输入录音中，停止后使用默认模型整段识别', lastTranscript: null });
      addDiagnostic({ title: '连续输入已启动', result: 'info', detail: `当前连续输入模型为 ${selectedModel}，V7 会停止后整段识别。` });
    }
  }

  async function handleStopLiveToggleRecording() {
    stopLiveTranscriptionTimer();
    const selectedModel = toggleDictationModelRef.current;
    if (selectedModel === 'baidu-realtime') {
      try {
        setStatus({ phase: 'transcribing', message: '\u6b63\u5728\u7ed3\u675f\u767e\u5ea6\u5b9e\u65f6 WebSocket \u8f93\u5165', lastTranscript: null });
        const summary = await finishBaiduRealtimeSession();
        isRecordingRef.current = false;
        setRecordingStatus({ state: 'idle', sampleRate: 16000, channels: 1, sampleCount: 0, durationMs: summary.durationMs });
        const summaryText = summary.text.trim();
        let sessionText = combinedLiveSessionText();
        if (!sessionText && summaryText) {
          if (!liveInsertedTextRef.current) {
            await insertTextWithClipboard(summaryText);
            liveInsertedTextRef.current = true;
          }
          rememberLiveSessionText(summaryText, summary.durationMs);
          sessionText = combinedLiveSessionText();
        }
        if (sessionText) {
          await addTranscriptRecord(sessionText, liveSessionDurationMsRef.current || summary.durationMs, 'toggle-dictation', 'baidu-realtime');
        }
        setStatus({ phase: 'succeeded', message: '\u767e\u5ea6\u5b9e\u65f6 WebSocket \u8f93\u5165\u5df2\u505c\u6b62', lastTranscript: sessionText || summaryText || null });
        addDiagnostic({ title: '\u767e\u5ea6\u5b9e\u65f6 WebSocket \u8f93\u5165\u5df2\u505c\u6b62', result: 'success', detail: sessionText ? '\u6700\u7ec8\u7247\u6bb5\u5df2\u5408\u5e76\u4e3a\u4e00\u6761\u8bc6\u522b\u8bb0\u5f55\u3002' : '\u672c\u6b21\u6ca1\u6709\u53ef\u5199\u5165\u8bc6\u522b\u8bb0\u5f55\u7684\u6700\u7ec8\u6587\u672c\u3002' });
      } catch (error) {
        const detail = formatError(error);
        isRecordingRef.current = false;
        setRecordingStatus((current) => ({ ...current, state: 'idle' }));
        setStatus({ phase: 'failed', message: `\u767e\u5ea6\u5b9e\u65f6 WebSocket \u8f93\u5165\u5931\u8d25\uff1a${detail}`, lastTranscript: null });
        addDiagnostic({ title: '\u767e\u5ea6\u5b9e\u65f6 WebSocket \u8f93\u5165\u5931\u8d25', result: 'error', detail });
      } finally {
        liveSessionTextsRef.current = [];
        liveSessionDurationMsRef.current = 0;
        await hideDictationOverlay().catch((hideError: unknown) => {
          addDiagnostic({ title: '\u684c\u9762\u6d6e\u7a97\u9690\u85cf\u5931\u8d25', result: 'error', detail: formatError(hideError) });
        });
      }
      return;
    }
    try {
      const audio = await stopRecording();
      lastAudioQualityRef.current = audio.audioQuality;
      setRecordingStatus({ state: 'idle', sampleRate: audio.sampleRate, channels: audio.channels, sampleCount: audio.asrSampleCount, durationMs: audio.asrDurationMs });
      setStatus({ phase: 'transcribing', message: '录音已停止，正在处理最后一段', lastTranscript: null });
      addDiagnostic({ title: '实验实时录音已停止', result: 'success', detail: `采集到 ${audio.sampleCount} 个 mono 样本，已立即停止录音流并进入转写状态。` });
      await showTranscribingOverlay().catch((error: unknown) => {
        addDiagnostic({ title: '桌面转写浮窗显示失败', result: 'error', detail: formatError(error) });
      });

      if (selectedModel === 'local-whisper') {
        await processFinalLiveTranscriptionChunk();
      }
      if (!liveInsertedTextRef.current) {
        const transcript = await transcribeLastRecording(selectedModel);
        const processed = await postprocessText(transcript.text);
        const finalText = processed.text.trim();
        if (!finalText || processed.noiseRemoved) {
          return;
        }
        await insertTextWithClipboard(finalText);
        await addTranscriptRecord(finalText, audio.asrDurationMs, 'toggle-dictation', selectedModel, processed);
        setStatus({ phase: 'succeeded', message: '切换录音已整段转写并上屏', lastTranscript: finalText });
        addDiagnostic({ title: '实验实时兜底上屏完成', result: 'success', detail: `${transcript.engine} 返回文本：${finalText}` });
      } else {
        const sessionText = combinedLiveSessionText();
        if (sessionText) {
          await addTranscriptRecord(sessionText, liveSessionDurationMsRef.current || audio.asrDurationMs, 'toggle-dictation', selectedModel);
        }
        setStatus({ phase: 'succeeded', message: '实验实时输入已停止', lastTranscript: sessionText || null });
        addDiagnostic({ title: '实验实时输入已停止', result: 'success', detail: sessionText ? '尾段处理完成，已合并为一条识别记录。' : '尾段处理完成，本次仅产生疑似噪声片段，未写入识别记录。' });
      }
    } catch (error) {
      const detail = formatError(error);
      setRecordingStatus((current) => ({ ...current, state: 'idle' }));
      setStatus({ phase: 'failed', message: `实验实时输入失败：${detail}`, lastTranscript: null });
      addDiagnostic({ title: '实验实时输入失败', result: 'error', detail });
      await hideDictationOverlay().catch((hideError: unknown) => {
        addDiagnostic({ title: '桌面浮窗隐藏失败', result: 'error', detail: formatError(hideError) });
      });
    } finally {
      liveSessionTextsRef.current = [];
      liveSessionDurationMsRef.current = 0;
      await hideDictationOverlay().catch((hideError: unknown) => {
        addDiagnostic({ title: '桌面浮窗隐藏失败', result: 'error', detail: formatError(hideError) });
      });
    }
  }

  async function handleStopTranscribeAndInsertNow() {
    if (!isTauriRuntime()) {
      addDiagnostic({ title: '快捷键闭环未执行', result: 'warning', detail: '浏览器预览模式不能调用录音、whisper.cpp 和剪贴板上屏。' });
      return;
    }
    try {
      const audio = await stopRecording();
      lastAudioQualityRef.current = audio.audioQuality;
      setRecordingStatus({ state: 'idle', sampleRate: audio.sampleRate, channels: audio.channels, sampleCount: audio.asrSampleCount, durationMs: audio.asrDurationMs });
      addDiagnostic({ title: '快捷键录音已停止', result: audio.sampleCount > 0 ? 'success' : 'warning', detail: `采集到 ${audio.sampleCount} 个 mono 样本，峰值音量 ${audio.peakAmplitude}，RMS 音量 ${audio.rmsAmplitude}。` });

      setStatus({ phase: 'transcribing', message: '正在识别刚才的语音', lastTranscript: null });
      await showTranscribingOverlay().catch((error: unknown) => {
        addDiagnostic({ title: '桌面转写浮窗显示失败', result: 'error', detail: formatError(error) });
      });
      const selectedModel = pushToTalkModelRef.current;
      if (selectedModel === 'baidu-realtime') {
        throw new Error('\u767e\u5ea6\u5b9e\u65f6 WebSocket API \u4ec5\u652f\u6301\u8fde\u7eed\u8f93\u5165\u6a21\u5f0f\u3002');
      }
      const model = getModelReadiness(selectedModel, asrConfigStatusRef.current, cloudAsrConfigStatusRef.current);
      assertModelUsable(model);
      const transcript = await transcribeLastRecording(selectedModel);
      const processed = await postprocessText(transcript.text);
      const finalText = processed.text.trim();
      if (!finalText || processed.noiseRemoved) {
        return;
      }
      setStatus({ phase: 'inserting', message: '正在上屏到当前光标位置', lastTranscript: finalText });
      await insertTextWithClipboard(finalText);
      await addTranscriptRecord(finalText, audio.asrDurationMs, 'push-to-talk', selectedModel, processed);
      setStatus({ phase: 'succeeded', message: '快捷键语音输入完成', lastTranscript: finalText });
      addDiagnostic({ title: '快捷键闭环完成', result: 'success', detail: `${transcript.engine} 返回文本并已发送上屏：${finalText}` });
    } catch (error) {
      const detail = formatError(error);
      setRecordingStatus((current) => ({ ...current, state: 'idle' }));
      setStatus({ phase: 'failed', message: `快捷键语音输入失败：${detail}`, lastTranscript: null });
      setHistoryMessage(`语音识别失败：${detail}`);
      addDiagnostic({ title: '快捷键闭环失败', result: 'error', detail });
    } finally {
      await hideDictationOverlay().catch((error: unknown) => {
        addDiagnostic({ title: '桌面浮窗隐藏失败', result: 'error', detail: formatError(error) });
      });
    }
  }

  async function handleSelectInputDevice(deviceName: string) {
    setSelectedInputDeviceName(deviceName);
    if (!isTauriRuntime()) {
      return;
    }
    try {
      const info = await setInputDevice(deviceName || null);
      setRecorderInfo(info);
      addDiagnostic({ title: '输入设备已切换', result: 'success', detail: `后续录音会使用：${info.deviceName} / ${info.sampleRate} Hz / ${info.channels} 声道。` });
    } catch (error) {
      addDiagnostic({ title: '切换输入设备失败', result: 'error', detail: formatError(error) });
    }
  }

  async function handleStopRecording() {
    if (!isTauriRuntime()) {
      addDiagnostic({ title: '停止录音未执行', result: 'warning', detail: '浏览器预览模式没有真实录音流。' });
      return;
    }
    try {
      const audio = await stopRecording();
      lastAudioQualityRef.current = audio.audioQuality;
      setRecordingStatus({ state: 'idle', sampleRate: audio.sampleRate, channels: audio.channels, sampleCount: audio.asrSampleCount, durationMs: audio.asrDurationMs });
      setStatus({ phase: 'succeeded', message: `录音已停止：${audio.durationMs} ms，已准备 ${audio.asrSampleCount} 个 16 kHz ASR 样本`, lastTranscript: null });
      addDiagnostic({ title: '录音已停止', result: audio.sampleCount > 0 ? 'success' : 'warning', detail: `采集到 ${audio.sampleCount} 个 mono 样本，峰值音量 ${audio.peakAmplitude}，RMS 音量 ${audio.rmsAmplitude}。` });
    } catch (error) {
      const detail = formatError(error);
      setStatus({ phase: 'failed', message: `停止录音失败：${detail}`, lastTranscript: null });
      addDiagnostic({ title: '停止录音失败', result: 'error', detail });
    }
  }

  async function handleRefreshRecordingStatus() {
    if (!isTauriRuntime()) {
      return;
    }
    try {
      const nextStatus = await getRecordingStatus();
      setRecordingStatus(nextStatus);
      addDiagnostic({ title: '录音状态已刷新', result: 'info', detail: `状态 ${nextStatus.state}，样本 ${nextStatus.sampleCount}，时长 ${nextStatus.durationMs} ms。` });
    } catch (error) {
      addDiagnostic({ title: '刷新录音状态失败', result: 'error', detail: formatError(error) });
    }
  }

  async function handleTranscribeLastRecording() {
    if (!isTauriRuntime()) {
      addDiagnostic({ title: '真实转写未执行', result: 'warning', detail: '浏览器预览模式不能调用 whisper.cpp。' });
      return;
    }
    try {
      setStatus({ phase: 'transcribing', message: '正在调用 whisper.cpp 转写最近录音', lastTranscript: null });
      const transcript = await transcribeLastRecording();
      await addTranscriptRecord(transcript.text, recordingStatus.durationMs, 'manual', 'baidu-short');
      setStatus({ phase: 'succeeded', message: '真实转写完成', lastTranscript: transcript.text });
      addDiagnostic({ title: '真实转写成功', result: 'success', detail: `${transcript.engine} 返回文本：${transcript.text}` });
    } catch (error) {
      const detail = formatError(error);
      setStatus({ phase: 'failed', message: `真实转写失败：${detail}`, lastTranscript: null });
      addDiagnostic({ title: '真实转写失败', result: 'error', detail });
    }
  }

  async function handleTranscribeAndInsert() {
    if (!isTauriRuntime()) {
      addDiagnostic({ title: '真实闭环未执行', result: 'warning', detail: '浏览器预览模式不能调用 whisper.cpp 和剪贴板上屏。' });
      return;
    }
    try {
      setStatus({ phase: 'transcribing', message: '正在转写；完成后会给你 3 秒切回目标输入框', lastTranscript: null });
      const transcript = await transcribeLastRecording();
      setStatus({ phase: 'inserting', message: '转写完成，请在 3 秒内切回目标输入框', lastTranscript: transcript.text });
      addDiagnostic({ title: '真实转写完成，等待上屏', result: 'info', detail: `${transcript.engine} 返回文本：${transcript.text}。` });
      await new Promise((resolve) => window.setTimeout(resolve, INSERT_DELAY_MS));
      await insertTextWithClipboard(transcript.text);
      await addTranscriptRecord(transcript.text, recordingStatus.durationMs, 'manual', 'baidu-short');
      setStatus({ phase: 'succeeded', message: '真实语音输入闭环完成', lastTranscript: transcript.text });
      addDiagnostic({ title: '真实闭环上屏请求已发送', result: 'success', detail: `文本：${transcript.text}` });
    } catch (error) {
      const detail = formatError(error);
      setStatus({ phase: 'failed', message: `真实闭环失败：${detail}`, lastTranscript: null });
      addDiagnostic({ title: '真实闭环失败', result: 'error', detail });
    }
  }

  async function handleExportLastRecordingWav() {
    if (!isTauriRuntime()) {
      addDiagnostic({ title: '诊断录音未导出', result: 'warning', detail: '浏览器预览模式不能写入 Tauri 应用数据目录。' });
      return;
    }
    try {
      const path = await exportLastRecordingWav();
      addDiagnostic({ title: '诊断录音已导出', result: 'success', detail: `最近一次录音的 16 kHz ASR WAV 已写入：${path}。` });
    } catch (error) {
      addDiagnostic({ title: '导出诊断录音失败', result: 'error', detail: formatError(error) });
    }
  }


  async function handlePrimaryRecordingAction() {
    if (isRecording) {
      await handleStopRecordingAndInsertWithDelay();
    } else {
      await handleStartRecording();
    }
  }

  async function handleStopRecordingAndInsertWithDelay() {
    if (!isTauriRuntime()) {
      addDiagnostic({ title: '语音输入未执行', result: 'warning', detail: '浏览器预览模式不能调用录音、whisper.cpp 和剪贴板上屏。' });
      return;
    }
    try {
      const audio = await stopRecording();
      lastAudioQualityRef.current = audio.audioQuality;
      setRecordingStatus({ state: 'idle', sampleRate: audio.sampleRate, channels: audio.channels, sampleCount: audio.asrSampleCount, durationMs: audio.asrDurationMs });
      addDiagnostic({ title: '录音已停止', result: audio.sampleCount > 0 ? 'success' : 'warning', detail: `采集到 ${audio.sampleCount} 个 mono 样本，峰值音量 ${audio.peakAmplitude}，RMS 音量 ${audio.rmsAmplitude}。` });

      setStatus({ phase: 'transcribing', message: '正在识别；完成后会给你 3 秒切回目标输入框', lastTranscript: null });
      const transcript = await transcribeLastRecording();
      setStatus({ phase: 'inserting', message: '转写完成，请在 3 秒内切回目标输入框', lastTranscript: transcript.text });
      addDiagnostic({ title: '主界面转写完成，等待上屏', result: 'info', detail: `${transcript.engine} 返回文本：${transcript.text}。` });
      await new Promise((resolve) => window.setTimeout(resolve, INSERT_DELAY_MS));
      await insertTextWithClipboard(transcript.text);
      await addTranscriptRecord(transcript.text, audio.asrDurationMs, 'manual', 'baidu-short');
      setStatus({ phase: 'succeeded', message: '语音输入完成', lastTranscript: transcript.text });
      addDiagnostic({ title: '主界面语音输入完成', result: 'success', detail: `文本：${transcript.text}` });
    } catch (error) {
      const detail = formatError(error);
      setRecordingStatus((current) => ({ ...current, state: 'idle' }));
      setStatus({ phase: 'failed', message: `语音输入失败：${detail}`, lastTranscript: null });
      addDiagnostic({ title: '主界面语音输入失败', result: 'error', detail });
    }
  }


  useEffect(() => {
    if (!isTauriRuntime()) {
      return;
    }
    let disposed = false;
    let unlisten: (() => void) | null = null;
    void listenToBaiduRealtimeResults((payload: BaiduRealtimeResultEvent) => {
      if (disposed) {
        return;
      }
      const text = payload.text.trim();
      if (!text) {
        return;
      }
      if (payload.isFinal) {
        void (async () => {
          const processed = await postprocessText(text);
          const finalText = processed.text.trim();
          if (!finalText || processed.noiseRemoved) {
            return;
          }
          liveInsertedTextRef.current = true;
          rememberLiveSessionText(finalText, payload.durationMs);
          setStatus({ phase: 'recording', message: '\u767e\u5ea6\u5b9e\u65f6 WebSocket \u6700\u7ec8\u7247\u6bb5\u5df2\u4e0a\u5c4f', lastTranscript: finalText });
          await insertTextWithClipboard(finalText);
          addDiagnostic({ title: '\u767e\u5ea6\u5b9e\u65f6\u6700\u7ec8\u7247\u6bb5', result: 'success', detail: finalText });
        })().catch((error: unknown) => {
          addDiagnostic({ title: '\u767e\u5ea6\u5b9e\u65f6\u7247\u6bb5\u4e0a\u5c4f\u5931\u8d25', result: 'error', detail: formatError(error) });
        });
      } else {
        setStatus({ phase: 'recording', message: `\u767e\u5ea6\u5b9e\u65f6\u8bc6\u522b\u4e2d\uff1a${text}`, lastTranscript: text });
      }
    })
      .then((nextUnlisten) => {
        if (disposed) {
          nextUnlisten();
          return;
        }
        unlisten = nextUnlisten;
      })
      .catch((error: unknown) => {
        if (!disposed) {
          addDiagnostic({ title: '\u6ce8\u518c\u767e\u5ea6\u5b9e\u65f6\u7ed3\u679c\u76d1\u542c\u5931\u8d25', result: 'error', detail: formatError(error) });
        }
      });
    return () => {
      disposed = true;
      unlisten?.();
      void cancelBaiduRealtimeSession().catch(() => undefined);
    };
  }, []);

  useEffect(() => {
    if (!isTauriRuntime()) {
      return;
    }

    let disposed = false;
    let unlisten: (() => void) | null = null;
    void listenToPushToTalk((payload) => {
      addDiagnostic({
        title: payload.state === 'pressed' ? '收到全局快捷键按下' : '收到全局快捷键松开',
        result: 'info',
        detail: `事件 ${payload.state}，动作 ${payload.action}。如果你在目标输入框里按快捷键但这里没有日志，说明系统没有把快捷键事件交给 VoxType。`,
      });
      if (payload.action === 'toggleStartRecording' && !isRecordingRef.current) {
        void handleStartLiveToggleRecording();
        return;
      }
      if (payload.action === 'startRecording' && !isRecordingRef.current) {
        void handleStartRecording();
        return;
      }
      if (payload.action === 'toggleStopAndTranscribe' && isRecordingRef.current) {
        void handleStopLiveToggleRecording();
        return;
      }
      if (payload.action === 'stopAndTranscribe' && isRecordingRef.current) {
        void handleStopTranscribeAndInsertNow();
      }
      })
      .then((nextUnlisten) => {
        if (disposed) {
          nextUnlisten();
          return;
        }
        unlisten = nextUnlisten;
      })
      .catch((error: unknown) => {
        if (disposed) {
          return;
        }
        addDiagnostic({ title: '注册全局快捷键监听失败', result: 'error', detail: formatError(error) });
      });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  function renderDeviceSelect(compact = false) {
    return (
      <label className={compact ? 'field compact-field' : 'field'}>
        <span>输入设备</span>
        <select value={selectedInputDeviceName} onChange={(event) => void handleSelectInputDevice(event.target.value)} disabled={isRecording}>
          {inputDevices.length === 0 ? <option value="">等待读取输入设备</option> : null}
          {inputDevices.map((device) => <option key={device.deviceName} value={device.deviceName}>{device.deviceName}</option>)}
        </select>
      </label>
    );
  }

  function renderDiagnosticsPanel() {
    return (
      <section className="panel diagnostics-panel" aria-labelledby="diagnostics-title">
        <div className="panel-heading-row">
          <h2 id="diagnostics-title">诊断日志</h2>
          <button className="secondary-button" type="button" onClick={handleCopyDiagnostics}>复制全部日志</button>
        </div>
        <div className="diagnostic-scroll" role="log" aria-live="polite">
          <ol className="diagnostic-list">
            {diagnostics.map((entry) => (
              <li key={entry.id} data-result={entry.result}>
                <div className="diagnostic-header"><strong>{entry.title}</strong><time>{entry.time}</time></div>
                <p>{entry.detail}</p>
              </li>
            ))}
          </ol>
          <div ref={diagnosticsEndRef} />
        </div>
      </section>
    );
  }

  function renderUserView() {
    return (
      <>
      <MainWindow
        status={status}
        hotkeyStatus={hotkeyStatus}
        pushToTalkHotkey={pushToTalkHotkey}
        toggleDictationHotkey={toggleDictationHotkeyInput}
        pushToTalkModelReadiness={getModelReadiness(pushToTalkModel)}
        toggleDictationModelReadiness={getModelReadiness(toggleDictationModel)}
        recorderInfo={recorderInfo}
        records={transcriptRecords}
        stats={transcriptStats}
        historyMessage={historyMessage}
        onOpenDiagnostic={() => setViewMode('diagnostic')}
        onOpenModelSettings={() => setViewMode('model')}
        onCopyRecord={(record) => void handleCopyRecord(record)}
        onReinsertRecord={(record) => void handleReinsertRecord(record)}
        onDeleteRecord={handleDeleteRecord}
        onClearRecords={handleClearRecords}
        onExportRecords={() => void handleExportRecords()}
        onOpenHotkeySettings={() => { setHotkeySaveMessage(null); setIsHotkeyDialogOpen(true); }}
      />
      {isHotkeyDialogOpen ? (
        <HotkeySettingsDialog
          pushToTalkHotkey={pushToTalkHotkey}
          toggleDictationHotkey={toggleDictationHotkeyInput}
          isSaving={isSavingHotkeys}
          message={hotkeySaveMessage}
          onPushToTalkHotkeyChange={setPushToTalkHotkey}
          onToggleDictationHotkeyChange={setToggleDictationHotkeyInput}
          onSave={() => void handleSaveHotkeyPreferences()}
          onClose={() => setIsHotkeyDialogOpen(false)}
        />
      ) : null}
      </>
    );
  }

  function renderModelSettingsView() {
    return (
      <ModelSettingsView
        asrConfigStatus={asrConfigStatus}
        cloudAsrConfigStatus={cloudAsrConfigStatus}
        pushToTalkModel={pushToTalkModel}
        toggleDictationModel={toggleDictationModel}
        modelReadiness={{ 'local-whisper': getModelReadiness('local-whisper'), 'baidu-short': getModelReadiness('baidu-short'), 'baidu-realtime': getModelReadiness('baidu-realtime') }}
        cloudBaseUrl={cloudBaseUrl}
        cloudModel={cloudModel}
        cloudLanguage={cloudLanguage}
        cloudBaiduCuid={cloudBaiduCuid}
        cloudBaiduFormat={cloudBaiduFormat}
        cloudBaiduSampleRate={cloudBaiduSampleRate}
        cloudBaiduLmId={cloudBaiduLmId}
        cloudBaiduRealtimeAppId={cloudBaiduRealtimeAppId}
        cloudBaiduRealtimeEndpoint={cloudBaiduRealtimeEndpoint}
        cloudBaiduRealtimeDevPid={cloudBaiduRealtimeDevPid}
        cloudBaiduRealtimeCuid={cloudBaiduRealtimeCuid}
        cloudBaiduRealtimeFormat={cloudBaiduRealtimeFormat}
        cloudBaiduRealtimeSampleRate={cloudBaiduRealtimeSampleRate}
        cloudBaiduRealtimeUser={cloudBaiduRealtimeUser}
        cloudApiKeyInput={cloudApiKeyInput}
        cloudSecretKeyInput={cloudSecretKeyInput}
        cloudMessage={cloudMessage}
        transcriptPostprocessConfig={transcriptPostprocessConfig}
        postprocessReplacementText={postprocessReplacementText}
        postprocessGlossaryText={postprocessGlossaryText}
        postprocessPreviewInput={postprocessPreviewInput}
        postprocessPreviewOutput={postprocessPreviewOutput}
        postprocessMessage={postprocessMessage}
        whisperBinaryPath={whisperBinaryPath}
        whisperModelPath={whisperModelPath}
        asrLanguage={asrLanguage}
        isInstallingAsr={isInstallingAsr}
        onBack={() => setViewMode('user')}
        onWhisperBinaryPathChange={setWhisperBinaryPath}
        onWhisperModelPathChange={setWhisperModelPath}
        onAsrLanguageChange={setAsrLanguage}
        onPushToTalkModelChange={(value) => void handleChangeModeModelPreference('push-to-talk', value)}
        onToggleDictationModelChange={(value) => void handleChangeModeModelPreference('toggle-dictation', value)}
        onCloudBaseUrlChange={setCloudBaseUrl}
        onCloudModelChange={setCloudModel}
        onCloudLanguageChange={setCloudLanguage}
        onCloudBaiduCuidChange={setCloudBaiduCuid}
        onCloudBaiduFormatChange={setCloudBaiduFormat}
        onCloudBaiduSampleRateChange={setCloudBaiduSampleRate}
        onCloudBaiduLmIdChange={setCloudBaiduLmId}
        onCloudBaiduRealtimeAppIdChange={setCloudBaiduRealtimeAppId}
        onCloudBaiduRealtimeEndpointChange={setCloudBaiduRealtimeEndpoint}
        onCloudBaiduRealtimeDevPidChange={setCloudBaiduRealtimeDevPid}
        onCloudBaiduRealtimeCuidChange={setCloudBaiduRealtimeCuid}
        onCloudBaiduRealtimeFormatChange={setCloudBaiduRealtimeFormat}
        onCloudBaiduRealtimeSampleRateChange={setCloudBaiduRealtimeSampleRate}
        onCloudBaiduRealtimeUserChange={setCloudBaiduRealtimeUser}
        onCloudApiKeyInputChange={setCloudApiKeyInput}
        onCloudSecretKeyInputChange={setCloudSecretKeyInput}
        onTranscriptPostprocessEnabledChange={(value) => setTranscriptPostprocessConfig((current) => ({ ...current, enabled: value }))}
        onTranscriptPostprocessCleanupNoiseChange={(value) => setTranscriptPostprocessConfig((current) => ({ ...current, cleanupNoise: value }))}
        onPostprocessReplacementTextChange={setPostprocessReplacementText}
        onPostprocessGlossaryTextChange={setPostprocessGlossaryText}
        onPostprocessPreviewInputChange={setPostprocessPreviewInput}
        onSaveTranscriptPostprocessConfig={() => void handleSaveTranscriptPostprocessConfig()}
        onPreviewTranscriptPostprocess={() => void handlePreviewTranscriptPostprocess()}
        onInstallManagedAsr={() => void handleInstallManagedAsr()}
        onSaveAsrConfig={() => void handleSaveAsrConfig()}
        onRefreshAsrConfig={() => void handleRefreshAsrConfig()}
        onSaveCloudAsrConfig={() => void handleSaveCloudAsrConfig()}
        onSaveBaiduAsrApiKey={() => void handleSaveBaiduAsrApiKey()}
        onSaveBaiduAsrSecretKey={() => void handleSaveBaiduAsrSecretKey()}
        onTestCloudAsrConfig={() => void handleTestCloudAsrConfig()}
      />
    );
  }

  function renderDiagnosticView() {
    return (
      <DiagnosticView
        config={config}
        status={status}
        runtimeMessage={runtimeMessage}
        recorderInfo={recorderInfo}
        recordingStatus={recordingStatus}
        isRecording={isRecording}
        hotkeyStatus={hotkeyStatus}
        overlayBackendStatus={overlayBackendStatus}
        asrConfigStatus={asrConfigStatus}
        whisperBinaryPath={whisperBinaryPath}
        whisperModelPath={whisperModelPath}
        asrLanguage={asrLanguage}
        isInstallingAsr={isInstallingAsr}
        deviceSelect={renderDeviceSelect()}
        diagnosticsPanel={renderDiagnosticsPanel()}
        onBack={() => setViewMode('user')}
        onRefreshHotkeyStatus={() => void handleRefreshHotkeyStatus()}
        onShowOverlayTest={() => void handleShowOverlayTest()}
        onHideOverlayTest={() => void handleHideOverlayTest()}
        onWhisperBinaryPathChange={setWhisperBinaryPath}
        onWhisperModelPathChange={setWhisperModelPath}
        onAsrLanguageChange={setAsrLanguage}
        onInstallManagedAsr={() => void handleInstallManagedAsr()}
        onSaveAsrConfig={() => void handleSaveAsrConfig()}
        onRefreshAsrConfig={() => void handleRefreshAsrConfig()}
        onStartRecording={() => void handleStartRecording()}
        onStopRecording={() => void handleStopRecording()}
        onRefreshRecordingStatus={() => void handleRefreshRecordingStatus()}
        onExportLastRecordingWav={() => void handleExportLastRecordingWav()}
        onTranscribeLastRecording={() => void handleTranscribeLastRecording()}
        onTranscribeAndInsert={() => void handleTranscribeAndInsert()}
        onSimulateDictation={() => void handleSimulateDictation()}
        onClipboardInsert={() => void handleClipboardInsert()}
      />
    );
  }


  if (viewMode === 'diagnostic') {
    return renderDiagnosticView();
  }
  if (viewMode === 'model') {
    return renderModelSettingsView();
  }
  return renderUserView();
}
