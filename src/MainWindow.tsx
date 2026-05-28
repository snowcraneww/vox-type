import type { CSSProperties } from 'react';
import type { AppPhase, AppStatus, AsrConfigStatus, HotkeyRegistrationStatus, RecorderInfo } from './types';

interface MainWindowProps {
  status: AppStatus;
  phaseLabel: string;
  asrConfigStatus: AsrConfigStatus;
  hotkeyStatus: HotkeyRegistrationStatus;
  recorderInfo: RecorderInfo | null;
  isRecording: boolean;
  onPrimaryRecordingAction: () => void;
  onOpenDiagnostic: () => void;
  onCopyTranscript: () => void;
  onReinsertTranscript: () => void;
  onClearTranscript: () => void;
}

interface ReadinessItem {
  label: string;
  value: string;
  state: 'ready' | 'warning' | 'busy' | 'error';
}

function readinessToneForPhase(phase: AppPhase): ReadinessItem['state'] {
  if (phase === 'failed') return 'error';
  if (phase === 'recording' || phase === 'transcribing' || phase === 'inserting') return 'busy';
  return 'ready';
}

function SignalMark({ active }: { active: boolean }) {
  return (
    <div className="control-signal" data-active={active} aria-hidden="true">
      {Array.from({ length: 14 }, (_, index) => <span key={index} style={{ '--bar-index': index } as CSSProperties} />)}
    </div>
  );
}

function ModeSelector({ isRecording, onPrimaryRecordingAction }: Pick<MainWindowProps, 'isRecording' | 'onPrimaryRecordingAction'>) {
  return (
    <section className="control-section mode-selector" aria-label="输入模式">
      <div className="section-heading"><span>输入模式</span><strong>选择你现在的说话方式</strong></div>
      <div className="mode-grid">
        <article className="mode-card" data-active={isRecording ? 'false' : 'true'}>
          <div><span className="mode-kicker">短句</span><h2>按住说话</h2><p>按住快捷键说一句，松开后转写并上屏。</p></div>
          <kbd>Ctrl+Alt+Space</kbd>
        </article>
        <article className="mode-card" data-active={isRecording ? 'true' : 'false'}>
          <div><span className="mode-kicker">长句</span><h2>连续输入</h2><p>按一次开始，录音中分段上屏，再按一次停止。</p></div>
          <kbd>Ctrl+Alt+V</kbd>
        </article>
      </div>
      <button className="primary-control-button" data-recording={isRecording} type="button" onClick={onPrimaryRecordingAction}>
        <span className="button-signal" aria-hidden="true"><i /><i /><i /></span>
        <span>{isRecording ? '停止当前录音' : '开始一次语音输入'}</span>
      </button>
    </section>
  );
}

function ReadinessPanel({ items }: { items: ReadinessItem[] }) {
  return (
    <section className="control-section readiness-panel" aria-label="准备状态">
      <div className="section-heading"><span>准备状态</span><strong>系统能力检查</strong></div>
      <dl className="readiness-grid">
        {items.map((item) => <div key={item.label} data-state={item.state}><dt>{item.label}</dt><dd>{item.value}</dd></div>)}
      </dl>
    </section>
  );
}

function RecentTranscript({ text, onCopyTranscript, onReinsertTranscript, onClearTranscript }: Pick<MainWindowProps, 'onCopyTranscript' | 'onReinsertTranscript' | 'onClearTranscript'> & { text: string | null }) {
  const hasText = Boolean(text?.trim());
  return (
    <section className="control-section recent-transcript" aria-label="最近结果">
      <div className="section-heading"><span>最近结果</span><strong>{hasText ? '上一段识别文本' : '还没有识别文本'}</strong></div>
      <p>{hasText ? text : '完成一次语音输入后，最近文本会显示在这里。'}</p>
      <div className="recent-actions">
        <button type="button" onClick={onCopyTranscript} disabled={!hasText}>复制</button>
        <button type="button" onClick={onReinsertTranscript} disabled={!hasText}>重新上屏</button>
        <button type="button" onClick={onClearTranscript} disabled={!hasText}>清空</button>
      </div>
    </section>
  );
}

export function MainWindow({ status, phaseLabel, asrConfigStatus, hotkeyStatus, recorderInfo, isRecording, onPrimaryRecordingAction, onOpenDiagnostic, onCopyTranscript, onReinsertTranscript, onClearTranscript }: MainWindowProps) {
  const readinessItems: ReadinessItem[] = [
    { label: '麦克风', value: recorderInfo?.deviceName ?? '等待设备', state: recorderInfo ? 'ready' : 'warning' },
    { label: '本地识别', value: asrConfigStatus.ready ? 'whisper.cpp 已就绪' : '需要配置 ASR', state: asrConfigStatus.ready ? 'ready' : 'warning' },
    { label: '快捷键', value: hotkeyStatus.registered ? hotkeyStatus.accelerator : '注册失败', state: hotkeyStatus.registered ? 'ready' : 'error' },
    { label: '上屏', value: '剪贴板粘贴', state: readinessToneForPhase(status.phase) },
  ];
  const headline = asrConfigStatus.ready && hotkeyStatus.registered ? '本地语音输入已就绪' : '需要处理配置后使用';

  return (
    <main className="app-shell user-shell">
      <section className="control-center" aria-labelledby="app-title">
        <header className="control-topbar">
          <div className="brand-block"><span className="product-mark">VoxType</span><h1 id="app-title">语音输入控制中心</h1></div>
          <div className="topbar-actions"><span className="topbar-status" data-phase={status.phase}>{phaseLabel}</span><button className="ghost-button" type="button" onClick={onOpenDiagnostic}>诊断</button></div>
        </header>
        <section className="control-hero" aria-label="当前状态">
          <SignalMark active={isRecording || status.phase === 'recording'} />
          <div className="hero-copy" aria-live="polite"><span>{headline}</span><strong>{status.message}</strong></div>
        </section>
        <div className="control-grid">
          <ModeSelector isRecording={isRecording} onPrimaryRecordingAction={onPrimaryRecordingAction} />
          <ReadinessPanel items={readinessItems} />
          <RecentTranscript text={status.lastTranscript} onCopyTranscript={onCopyTranscript} onReinsertTranscript={onReinsertTranscript} onClearTranscript={onClearTranscript} />
        </div>
      </section>
    </main>
  );
}