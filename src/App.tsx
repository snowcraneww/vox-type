import { useEffect, useMemo, useState } from 'react';
import { getConfig, getDefaultInputInfo, insertTextWithClipboard, isTauriRuntime, simulateDictation } from './tauriClient';
import type { AppConfig, AppStatus, RecorderInfo } from './types';

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
  const [runtimeMessage, setRuntimeMessage] = useState('浏览器预览模式：系统能力需要在 Tauri 中验证');

  useEffect(() => {
    if (!isTauriRuntime()) {
      return;
    }

    setRuntimeMessage('Tauri 运行中：可以验证系统能力');
    void getConfig().then(setConfig).catch((error: unknown) => {
      setRuntimeMessage(`读取配置失败：${String(error)}`);
    });
    void getDefaultInputInfo().then(setRecorderInfo).catch((error: unknown) => {
      setRuntimeMessage(`读取麦克风失败：${String(error)}`);
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
        return;
      } catch (error) {
        setStatus({ phase: 'failed', message: `模拟闭环失败：${String(error)}`, lastTranscript: null });
        return;
      }
    }

    setStatus({
      phase: 'succeeded',
      message: '模拟闭环完成：语音已转成文本并准备上屏',
      lastTranscript: '这是 VoxType 的中文优先语音输入测试。',
    });
  }

  async function handleClipboardInsert() {
    const text = status.lastTranscript ?? '这是 VoxType 的剪贴板上屏测试。';
    if (!isTauriRuntime()) {
      setRuntimeMessage('浏览器预览不能调用 Windows 剪贴板上屏，请在 Tauri 中验证');
      return;
    }
    try {
      await insertTextWithClipboard(text);
      setRuntimeMessage('已发送剪贴板上屏请求');
    } catch (error) {
      setRuntimeMessage(`剪贴板上屏失败：${String(error)}`);
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
        </dl>
      </section>

      <section className="panel" aria-labelledby="status-title">
        <h2 id="status-title">状态</h2>
        <p className="runtime-message">{runtimeMessage}</p>
        <p>{status.message}</p>
        {status.lastTranscript ? <blockquote>{status.lastTranscript}</blockquote> : null}
        <div className="button-row">
          <button type="button" onClick={handleSimulateDictation}>模拟一次语音输入闭环</button>
          <button type="button" onClick={handleClipboardInsert}>测试剪贴板上屏</button>
        </div>
      </section>
    </main>
  );
}
