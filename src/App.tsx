import { useEffect, useMemo, useRef, useState } from 'react';
import { formatDiagnosticsForCopy } from './diagnostics';
import { VoiceOverlay } from './VoiceOverlay';
import { createVoiceOverlayModel } from './voiceOverlayModel';
import { formatError } from './errorFormat';
import { exportLastRecordingWav, getAsrConfigStatus, getConfig, getDefaultInputInfo, getHotkeyStatus, getRecordingStatus, getUserPreferences, hideDictationOverlay, insertTextWithClipboard, installManagedAsr, isTauriRuntime, listenToPushToTalk, listInputDevices, saveAsrConfig, setInputDevice, showDictationOverlay, simulateDictation, startRecording, stopRecording, transcribeActiveRecordingChunk, transcribeLastRecording } from './tauriClient';
import type { AppConfig, AppStatus, AsrConfigStatus, HotkeyRegistrationStatus, RecorderInfo, RecorderRuntimeStatus } from './types';

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

const toggleHotkey = 'Ctrl+Alt+V';

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
  accelerator: 'Ctrl+Alt+Space',
  registered: false,
  message: '等待 Tauri 注册全局快捷键。',
};

export function App() {
  const [viewMode, setViewMode] = useState<'user' | 'diagnostic'>('user');
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
  const [hotkeyStatus, setHotkeyStatus] = useState<HotkeyRegistrationStatus>(initialHotkeyStatus);
  const [whisperBinaryPath, setWhisperBinaryPath] = useState('');
  const [whisperModelPath, setWhisperModelPath] = useState('');
  const [asrLanguage, setAsrLanguage] = useState('zh');
  const [isInstallingAsr, setIsInstallingAsr] = useState(false);
  const didLoadRuntime = useRef(false);
  const nextDiagnosticIdRef = useRef(2);
  const isRecordingRef = useRef(false);
  const liveTimerRef = useRef<number | null>(null);
  const liveCursorRef = useRef(0);
  const liveInFlightRef = useRef(false);
  const liveInsertedTextRef = useRef(false);
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

  function applyAsrConfigStatus(nextStatus: AsrConfigStatus) {
    setAsrConfigStatus(nextStatus);
    setWhisperBinaryPath(nextStatus.whisperBinaryPath ?? '');
    setWhisperModelPath(nextStatus.whisperModelPath ?? '');
    setAsrLanguage(nextStatus.language);
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
  const overlayLevel = isRecording ? 0.72 : status.phase === 'transcribing' || status.phase === 'inserting' ? 0.38 : 0.18;
  const voiceOverlayModel = useMemo(() => createVoiceOverlayModel(status, overlayLevel), [status, overlayLevel]);

  const phaseLabel = useMemo(() => {
    const labels: Record<AppStatus['phase'], string> = {
      idle: '空闲',
      recording: '录音中',
      transcribing: '识别中',
      inserting: '上屏中',
      succeeded: '已完成',
      failed: '失败',
    };
    return labels[status.phase];
  }, [status.phase]);

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

  async function handleStartLiveToggleRecording() {
    const started = await handleStartRecording();
    if (!started) {
      return;
    }
    liveCursorRef.current = 0;
    liveInsertedTextRef.current = false;
    stopLiveTranscriptionTimer();
    setStatus({ phase: 'recording', message: '实验实时输入中：会每隔几秒分段转写并上屏', lastTranscript: null });
    addDiagnostic({ title: '实验实时输入已启动', result: 'info', detail: '当前不是 whisper.cpp 真流式 partial，而是录音中定时切片、转写新增片段并上屏。' });
    liveTimerRef.current = window.setInterval(() => {
      void processLiveTranscriptionChunk({ quietShortChunk: true });
    }, LIVE_TRANSCRIPTION_INTERVAL_MS);
  }

  async function handleStopLiveToggleRecording() {
    stopLiveTranscriptionTimer();
    try {
      await processLiveTranscriptionChunk({ quietShortChunk: true });
      const audio = await stopRecording();
      setRecordingStatus({ state: 'idle', sampleRate: audio.sampleRate, channels: audio.channels, sampleCount: audio.asrSampleCount, durationMs: audio.asrDurationMs });
      if (!liveInsertedTextRef.current) {
        setStatus({ phase: 'transcribing', message: '实时片段未产生文本，正在用整段录音兜底转写', lastTranscript: null });
        const transcript = await transcribeLastRecording();
        await insertTextWithClipboard(transcript.text);
        setStatus({ phase: 'succeeded', message: '切换录音已整段转写并上屏', lastTranscript: transcript.text });
        addDiagnostic({ title: '实验实时兜底上屏完成', result: 'success', detail: `${transcript.engine} 返回文本：${transcript.text}` });
      } else {
        setStatus({ phase: 'succeeded', message: '实验实时输入已停止', lastTranscript: null });
        addDiagnostic({ title: '实验实时输入已停止', result: 'success', detail: `采集到 ${audio.sampleCount} 个 mono 样本，已停止录音流。` });
      }
    } catch (error) {
      const detail = formatError(error);
      setRecordingStatus((current) => ({ ...current, state: 'idle' }));
      setStatus({ phase: 'failed', message: `实验实时输入失败：${detail}`, lastTranscript: null });
      addDiagnostic({ title: '实验实时输入失败', result: 'error', detail });
    } finally {
      await hideDictationOverlay().catch((error: unknown) => {
        addDiagnostic({ title: '桌面浮窗隐藏失败', result: 'error', detail: formatError(error) });
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
      setRecordingStatus({ state: 'idle', sampleRate: audio.sampleRate, channels: audio.channels, sampleCount: audio.asrSampleCount, durationMs: audio.asrDurationMs });
      addDiagnostic({ title: '快捷键录音已停止', result: audio.sampleCount > 0 ? 'success' : 'warning', detail: `采集到 ${audio.sampleCount} 个 mono 样本，峰值音量 ${audio.peakAmplitude}，RMS 音量 ${audio.rmsAmplitude}。` });

      setStatus({ phase: 'transcribing', message: '正在识别刚才的语音', lastTranscript: null });
      const transcript = await transcribeLastRecording();
      setStatus({ phase: 'inserting', message: '正在上屏到当前光标位置', lastTranscript: transcript.text });
      await insertTextWithClipboard(transcript.text);
      setStatus({ phase: 'succeeded', message: '快捷键语音输入完成', lastTranscript: transcript.text });
      addDiagnostic({ title: '快捷键闭环完成', result: 'success', detail: `${transcript.engine} 返回文本并已发送上屏：${transcript.text}` });
    } catch (error) {
      const detail = formatError(error);
      setRecordingStatus((current) => ({ ...current, state: 'idle' }));
      setStatus({ phase: 'failed', message: `快捷键语音输入失败：${detail}`, lastTranscript: null });
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
      setRecordingStatus({ state: 'idle', sampleRate: audio.sampleRate, channels: audio.channels, sampleCount: audio.asrSampleCount, durationMs: audio.asrDurationMs });
      addDiagnostic({ title: '录音已停止', result: audio.sampleCount > 0 ? 'success' : 'warning', detail: `采集到 ${audio.sampleCount} 个 mono 样本，峰值音量 ${audio.peakAmplitude}，RMS 音量 ${audio.rmsAmplitude}。` });

      setStatus({ phase: 'transcribing', message: '正在识别；完成后会给你 3 秒切回目标输入框', lastTranscript: null });
      const transcript = await transcribeLastRecording();
      setStatus({ phase: 'inserting', message: '转写完成，请在 3 秒内切回目标输入框', lastTranscript: transcript.text });
      addDiagnostic({ title: '主界面转写完成，等待上屏', result: 'info', detail: `${transcript.engine} 返回文本：${transcript.text}。` });
      await new Promise((resolve) => window.setTimeout(resolve, INSERT_DELAY_MS));
      await insertTextWithClipboard(transcript.text);
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
      <main className="app-shell user-shell">
        <section className="tool-window" aria-labelledby="app-title">
          <header className="tool-header">
            <div>
              <span className="product-mark">VoxType</span>
              <p>本地优先语音输入</p>
            </div>
            <button className="ghost-button" type="button" onClick={() => setViewMode('diagnostic')}>诊断模式</button>
          </header>

          <div className="voice-dashboard">
            <div className="voice-copy">
              <p className="eyebrow">DICTATION READY</p>
              <h1 id="app-title">语音直接变成文字</h1>
              <p className="voice-subtitle">按住 Ctrl+Alt+Space 说话，或按 Ctrl+Alt+V 开始/停止。停止后会转写并上屏到当前光标位置。</p>
              <div className="status-strip" data-phase={status.phase}><span>{phaseLabel}</span><strong>{status.message}</strong></div>
            </div>
            <div className="record-orb-wrap">
              <VoiceOverlay model={voiceOverlayModel} />
              <button className="record-orb" data-phase={status.phase} type="button" onClick={() => void handlePrimaryRecordingAction()}>
                <span className="record-symbol" />
                <span>{isRecording ? '停止录音' : '开始录音'}</span>
              </button>
            </div>
          </div>

          <div className="transcript-card compact-section">
            <div>
              <span>最近文本</span>
              <p>{status.lastTranscript ?? '还没有识别文本。完成一次录音和转写后会显示在这里。'}</p>
            </div>
            <div className="action-row">
              <button type="button" onClick={handleTranscribeLastRecording}>转写最近录音</button>
              <button type="button" onClick={handleTranscribeAndInsert}>转写并上屏</button>
            </div>
          </div>

          <div className="quick-settings" aria-label="快速设置">
            {renderDeviceSelect(true)}
            <div className="setting-chip"><span>ASR</span><strong>{asrConfigStatus.ready ? '已就绪' : '未就绪'}</strong></div>
            <div className="setting-chip"><span>模型</span><strong>whisper.cpp</strong></div>
            <div className="setting-chip"><span>快捷键</span><strong>{hotkeyStatus.registered ? `${hotkeyStatus.accelerator} / ${toggleHotkey}` : '需处理'}</strong></div>
          </div>

          {!asrConfigStatus.ready ? <div className="setup-banner"><span>{asrConfigStatus.message}</span><button type="button" onClick={handleInstallManagedAsr} disabled={isInstallingAsr}>{isInstallingAsr ? '正在安装' : '一键安装 whisper.cpp'}</button></div> : null}
        </section>
      </main>
    );
  }

  function renderDiagnosticView() {
    return (
      <main className="app-shell diagnostic-shell">
        <section className="hero diagnostic-hero" aria-labelledby="diagnostic-title">
          <div><p className="eyebrow">DIAGNOSTICS</p><h1 id="diagnostic-title">诊断工作台</h1><p>系统能力、ASR 配置、快捷键和上屏链路。</p></div>
          <button className="secondary-button" type="button" onClick={() => setViewMode('user')}>返回主界面</button>
        </section>

        <section className="panel" aria-labelledby="settings-title">
          <h2 id="settings-title">运行状态</h2>
          <dl className="settings-grid">
            <div><dt>快捷键</dt><dd>{config.hotkey}</dd></div>
            <div><dt>目标语言</dt><dd>中文优先，兼容英文</dd></div>
            <div><dt>ASR 路线</dt><dd>{config.asrEngine}</dd></div>
            <div><dt>上屏策略</dt><dd>剪贴板粘贴</dd></div>
            <div><dt>全局快捷键</dt><dd>{hotkeyStatus.message}</dd></div>
            <div><dt>麦克风</dt><dd>{recorderInfo ? `${recorderInfo.deviceName} / ${recorderInfo.sampleRate} Hz / ${recorderInfo.channels} 声道` : '等待 Tauri 读取'}</dd></div>
            <div><dt>录音流</dt><dd>{isRecording ? `录音中 / ${recordingStatus.sampleCount} 样本 / ${recordingStatus.durationMs} ms` : '空闲'}</dd></div>
          </dl>
          <div className="button-row"><button type="button" onClick={handleRefreshHotkeyStatus}>刷新全局快捷键状态</button><button type="button" onClick={handleShowOverlayTest}>测试桌面浮窗</button><button type="button" onClick={handleHideOverlayTest}>隐藏桌面浮窗</button></div>
          <div className="config-form single-row-form">{renderDeviceSelect()}</div>
        </section>

        <section className="panel" aria-labelledby="asr-config-title">
          <div className="panel-heading-row"><h2 id="asr-config-title">ASR 配置</h2><span className="status-pill" data-phase={asrConfigStatus.ready ? 'succeeded' : 'failed'}>{asrConfigStatus.ready ? '已就绪' : '未就绪'}</span></div>
          <p className="runtime-message">{asrConfigStatus.message}</p>
          <div className="config-form">
            <label className="field"><span>whisper.cpp 可执行文件</span><input value={whisperBinaryPath} onChange={(event) => setWhisperBinaryPath(event.target.value)} placeholder="例如 C:\\tools\\whisper.cpp\\whisper-cli.exe" /></label>
            <label className="field"><span>Whisper 模型文件</span><input value={whisperModelPath} onChange={(event) => setWhisperModelPath(event.target.value)} placeholder="例如 C:\\models\\ggml-small.bin" /></label>
            <label className="field"><span>识别语言</span><input value={asrLanguage} onChange={(event) => setAsrLanguage(event.target.value)} placeholder="zh" /></label>
          </div>
          <div className="button-row"><button type="button" onClick={handleInstallManagedAsr} disabled={isInstallingAsr}>{isInstallingAsr ? '正在安装 whisper.cpp' : '一键安装 whisper.cpp'}</button><button type="button" onClick={handleSaveAsrConfig}>保存 ASR 配置</button><button type="button" onClick={handleRefreshAsrConfig}>检测 ASR 配置</button></div>
        </section>

        <section className="panel" aria-labelledby="status-title">
          <h2 id="status-title">测试操作</h2>
          <p className="runtime-message">{runtimeMessage}</p>
          <p>{status.message}</p>
          {status.lastTranscript ? <blockquote>{status.lastTranscript}</blockquote> : null}
          <div className="button-row">
            <button type="button" onClick={handleStartRecording} disabled={isRecording}>开始录音采集</button>
            <button type="button" onClick={handleStopRecording} disabled={!isRecording}>停止录音采集</button>
            <button type="button" onClick={handleRefreshRecordingStatus}>刷新录音状态</button>
            <button type="button" onClick={handleExportLastRecordingWav}>导出最近录音 WAV</button>
            <button type="button" onClick={handleTranscribeLastRecording}>转写最近录音</button>
            <button type="button" onClick={handleTranscribeAndInsert}>转写并上屏最近录音</button>
            <button type="button" onClick={handleSimulateDictation}>模拟一次语音输入闭环</button>
            <button type="button" onClick={handleClipboardInsert}>测试剪贴板上屏</button>
          </div>
        </section>

        {renderDiagnosticsPanel()}
      </main>
    );
  }

  return viewMode === 'user' ? renderUserView() : renderDiagnosticView();
}
