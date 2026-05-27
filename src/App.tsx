import { useEffect, useMemo, useRef, useState } from 'react';
import { getConfig, getDefaultInputInfo, getRecordingStatus, insertTextWithClipboard, isTauriRuntime, simulateDictation, startRecording, stopRecording } from './tauriClient';
import type { AppConfig, AppStatus, RecorderInfo, RecorderRuntimeStatus } from './types';

interface DiagnosticEntry {
  id: number;
  time: string;
  title: string;
  result: 'info' | 'success' | 'warning' | 'error';
  detail: string;
}

const defaultConfig: AppConfig = {
  hotkey: 'Ctrl+Alt+Space',
  language: 'zh-CN',
  asrEngine: 'whisper.cpp',
  insertionStrategy: 'clipboard',
  showStatusIndicator: true,
};

const initialStatus: AppStatus = {
  phase: 'idle',
  message: '等待按住快捷键开始录音',
  lastTranscript: null,
};

export function App() {
  const [config, setConfig] = useState<AppConfig>(defaultConfig);
  const [status, setStatus] = useState<AppStatus>(initialStatus);
  const [recorderInfo, setRecorderInfo] = useState<RecorderInfo | null>(null);
  const [recordingStatus, setRecordingStatus] = useState<RecorderRuntimeStatus>({
    state: 'idle',
    sampleRate: null,
    channels: null,
    sampleCount: 0,
    durationMs: 0,
  });
  const [runtimeMessage, setRuntimeMessage] = useState('浏览器预览模式：系统能力需要在 Tauri 中验证');
  const didLoadRuntime = useRef(false);
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
      {
        ...entry,
        id: Date.now(),
        time: new Date().toLocaleTimeString(),
      },
      ...current.slice(0, 5),
    ]);
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
      setRuntimeMessage(`读取配置失败：${String(error)}`);
      addDiagnostic({ title: '读取配置失败', result: 'error', detail: String(error) });
    });
    void getDefaultInputInfo()
      .then((info) => {
        setRecorderInfo(info);
        addDiagnostic({
          title: '麦克风探测成功',
          result: 'success',
          detail: `${info.deviceName} / ${info.sampleRate} Hz / ${info.channels} 声道。说明 Tauri 已能访问系统默认输入设备。`,
        });
      })
      .catch((error: unknown) => {
        setRuntimeMessage(`读取麦克风失败：${String(error)}`);
        addDiagnostic({ title: '麦克风探测失败', result: 'error', detail: String(error) });
      });
  }, []);

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
        addDiagnostic({
          title: '模拟闭环成功',
          result: 'success',
          detail: `Rust command simulate_dictation 已返回成功。当前是 mock 文本，不代表真实录音已经完成。返回文本：${nextStatus.lastTranscript ?? '无'}`,
        });
        return;
      } catch (error) {
        setStatus({ phase: 'failed', message: `模拟闭环失败：${String(error)}`, lastTranscript: null });
        addDiagnostic({ title: '模拟闭环失败', result: 'error', detail: String(error) });
        return;
      }
    }

    setStatus({
      phase: 'succeeded',
      message: '模拟闭环完成：语音已转成文本并准备上屏',
      lastTranscript: '这是 VoxType 的中文优先语音输入测试。',
    });
    addDiagnostic({
      title: '浏览器 mock 成功',
      result: 'warning',
      detail: '这是纯前端模拟结果。要验证 Rust/Tauri command，请使用 npm run tauri -- dev。',
    });
  }

  async function handleClipboardInsert() {
    const text = status.lastTranscript ?? '这是 VoxType 的剪贴板上屏测试。';
    if (!isTauriRuntime()) {
      setRuntimeMessage('浏览器预览不能调用 Windows 剪贴板上屏，请在 Tauri 中验证');
      addDiagnostic({
        title: '剪贴板上屏未执行',
        result: 'warning',
        detail: '当前是浏览器预览模式。请用 npm run tauri -- dev 启动桌面应用后再测。',
      });
      return;
    }
    try {
      await insertTextWithClipboard(text);
      setStatus({ phase: 'succeeded', message: '已发送剪贴板上屏请求', lastTranscript: text });
      setRuntimeMessage('已发送剪贴板上屏请求；请检查刚才聚焦的输入框是否出现测试文本');
      addDiagnostic({
        title: '剪贴板上屏请求已发送',
        result: 'success',
        detail: '程序已写入剪贴板、发送 Ctrl+V 并尝试恢复旧剪贴板。是否真正进入目标软件，需要看 Notepad、VS Code 或浏览器输入框。',
      });
    } catch (error) {
      setRuntimeMessage(`剪贴板上屏失败：${String(error)}`);
      setStatus({ phase: 'failed', message: `剪贴板上屏失败：${String(error)}`, lastTranscript: null });
      addDiagnostic({ title: '剪贴板上屏失败', result: 'error', detail: String(error) });
    }
  }

  async function handleStartRecording() {
    if (!isTauriRuntime()) {
      addDiagnostic({
        title: '录音未启动',
        result: 'warning',
        detail: '浏览器预览模式不能访问系统麦克风录音流，请使用 npm run tauri -- dev。',
      });
      return;
    }
    try {
      const nextStatus = await startRecording();
      setRecordingStatus(nextStatus);
      setStatus({ phase: 'recording', message: '正在录音，请说一小段话后点击停止录音', lastTranscript: null });
      addDiagnostic({
        title: '录音已启动',
        result: 'success',
        detail: `采样率 ${nextStatus.sampleRate ?? '未知'} Hz，输入声道 ${nextStatus.channels ?? '未知'}。当前只验证录音采集，还不会自动转写。`,
      });
    } catch (error) {
      setStatus({ phase: 'failed', message: `启动录音失败：${String(error)}`, lastTranscript: null });
      addDiagnostic({ title: '启动录音失败', result: 'error', detail: String(error) });
    }
  }

  async function handleStopRecording() {
    if (!isTauriRuntime()) {
      addDiagnostic({
        title: '停止录音未执行',
        result: 'warning',
        detail: '浏览器预览模式没有真实录音流。',
      });
      return;
    }
    try {
      const audio = await stopRecording();
      setRecordingStatus({
        state: 'idle',
        sampleRate: audio.sampleRate,
        channels: audio.channels,
        sampleCount: audio.sampleCount,
        durationMs: audio.durationMs,
      });
      setStatus({
        phase: 'succeeded',
        message: `录音已停止：${audio.durationMs} ms，${audio.sampleCount} 个 mono 样本`,
        lastTranscript: null,
      });
      addDiagnostic({
        title: '录音已停止',
        result: audio.sampleCount > 0 ? 'success' : 'warning',
        detail: `采集到 ${audio.sampleCount} 个 mono 样本，时长 ${audio.durationMs} ms，采样率 ${audio.sampleRate} Hz。下一步才会接 whisper.cpp 转写。`,
      });
    } catch (error) {
      setStatus({ phase: 'failed', message: `停止录音失败：${String(error)}`, lastTranscript: null });
      addDiagnostic({ title: '停止录音失败', result: 'error', detail: String(error) });
    }
  }

  async function handleRefreshRecordingStatus() {
    if (!isTauriRuntime()) {
      return;
    }
    try {
      const nextStatus = await getRecordingStatus();
      setRecordingStatus(nextStatus);
      addDiagnostic({
        title: '录音状态已刷新',
        result: 'info',
        detail: `状态 ${nextStatus.state}，样本 ${nextStatus.sampleCount}，时长 ${nextStatus.durationMs} ms。`,
      });
    } catch (error) {
      addDiagnostic({ title: '刷新录音状态失败', result: 'error', detail: String(error) });
    }
  }

  return (
    <main className="app-shell">
      <section className="hero" aria-labelledby="app-title">
        <div>
          <p className="eyebrow">VoxType MVP</p>
          <h1 id="app-title">本地优先语音输入工具</h1>
        </div>
        <div className="status-pill" data-phase={status.phase}>{phaseLabel}</div>
      </section>

      <section className="panel" aria-labelledby="settings-title">
        <h2 id="settings-title">第一版已确认设置</h2>
        <dl className="settings-grid">
          <div><dt>快捷键</dt><dd>{config.hotkey}</dd></div>
          <div><dt>目标语言</dt><dd>中文优先，兼容英文</dd></div>
          <div><dt>ASR 路线</dt><dd>{config.asrEngine}</dd></div>
          <div><dt>上屏策略</dt><dd>剪贴板粘贴并恢复</dd></div>
          <div><dt>界面形态</dt><dd>托盘 + 设置页 + 状态提示</dd></div>
          <div><dt>隐私默认</dt><dd>音频留在本机</dd></div>
          <div><dt>麦克风</dt><dd>{recorderInfo ? `${recorderInfo.deviceName} / ${recorderInfo.sampleRate} Hz / ${recorderInfo.channels} 声道` : '等待 Tauri 读取'}</dd></div>
          <div><dt>录音流</dt><dd>{recordingStatus.state === 'recording' ? `录音中 / ${recordingStatus.sampleCount} 样本 / ${recordingStatus.durationMs} ms` : '空闲'}</dd></div>
        </dl>
      </section>

      <section className="panel" aria-labelledby="status-title">
        <h2 id="status-title">状态</h2>
        <p className="runtime-message">{runtimeMessage}</p>
        <p>{status.message}</p>
        {status.lastTranscript ? <blockquote>{status.lastTranscript}</blockquote> : null}
        <div className="button-row">
          <button type="button" onClick={handleStartRecording} disabled={recordingStatus.state === 'recording'}>开始录音采集</button>
          <button type="button" onClick={handleStopRecording} disabled={recordingStatus.state !== 'recording'}>停止录音采集</button>
          <button type="button" onClick={handleRefreshRecordingStatus}>刷新录音状态</button>
          <button type="button" onClick={handleSimulateDictation}>模拟一次语音输入闭环</button>
          <button type="button" onClick={handleClipboardInsert}>测试剪贴板上屏</button>
        </div>
      </section>

      <section className="panel" aria-labelledby="diagnostics-title">
        <h2 id="diagnostics-title">诊断日志</h2>
        <ol className="diagnostic-list">
          {diagnostics.map((entry) => (
            <li key={entry.id} data-result={entry.result}>
              <div className="diagnostic-header">
                <strong>{entry.title}</strong>
                <time>{entry.time}</time>
              </div>
              <p>{entry.detail}</p>
            </li>
          ))}
        </ol>
      </section>
    </main>
  );
}
