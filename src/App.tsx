import { useMemo, useState } from 'react';
import type { AppConfig, AppStatus } from './types';

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
  const [status, setStatus] = useState<AppStatus>(initialStatus);

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

  function simulateDictation() {
    setStatus({
      phase: 'succeeded',
      message: '模拟闭环完成：语音已转成文本并准备上屏',
      lastTranscript: '这是 VoxType 的中文优先语音输入测试。',
    });
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
          <div><dt>快捷键</dt><dd>{defaultConfig.hotkey}</dd></div>
          <div><dt>目标语言</dt><dd>中文优先，兼容英文</dd></div>
          <div><dt>ASR 路线</dt><dd>{defaultConfig.asrEngine}</dd></div>
          <div><dt>上屏策略</dt><dd>剪贴板粘贴并恢复</dd></div>
          <div><dt>界面形态</dt><dd>托盘 + 设置页 + 状态提示</dd></div>
          <div><dt>隐私默认</dt><dd>音频留在本机</dd></div>
        </dl>
      </section>

      <section className="panel" aria-labelledby="status-title">
        <h2 id="status-title">状态</h2>
        <p>{status.message}</p>
        {status.lastTranscript ? <blockquote>{status.lastTranscript}</blockquote> : null}
        <button type="button" onClick={simulateDictation}>模拟一次语音输入闭环</button>
      </section>
    </main>
  );
}
