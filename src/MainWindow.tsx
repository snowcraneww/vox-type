import type { ReactNode } from 'react';
import type { AppStatus, HotkeyRegistrationStatus, ModelReadiness, RecorderInfo, TranscriptRecord, TranscriptStats, TranscriptionModelId } from './types';

const text = {
  controlCenter: '\u8bed\u97f3\u8f93\u5165\u63a7\u5236\u4e2d\u5fc3',
  modelSettings: '\u6a21\u578b\u9009\u62e9',
  diagnostic: '\u8bca\u65ad',
  inputMode: '\u8f93\u5165\u6a21\u5f0f',
  hotkeyStatus: '\u5feb\u6377\u952e\u72b6\u6001',
  hotkeySettings: '\u81ea\u5b9a\u4e49\u5feb\u6377\u952e',
  pushToTalk: '\u6309\u4f4f\u8bf4\u8bdd',
  pushToTalkModel: '\u6309\u4f4f\u8bf4\u8bdd\u6a21\u578b',
  toggleDictation: '\u8fde\u7eed\u8f93\u5165',
  toggleDictationModel: '\u8fde\u7eed\u8f93\u5165\u6a21\u578b',
  registered: '\u5feb\u6377\u952e\u5df2\u6ce8\u518c',
  unregistered: '\u5feb\u6377\u952e\u672a\u6ce8\u518c',
  readiness: '\u51c6\u5907\u72b6\u6001',
  capabilities: '\u7cfb\u7edf\u80fd\u529b',
  microphone: '\u9ea6\u514b\u98ce',
  waitingDevice: '\u7b49\u5f85\u8bbe\u5907',
  paste: '\u4e0a\u5c4f',
  history: '\u8bc6\u522b\u8bb0\u5f55',
  itemUnit: '\u6761',
  empty: '\u7b49\u5f85\u7b2c\u4e00\u6b21\u8bc6\u522b',
  emptyDetail: '\u8f6c\u5199\u7ed3\u679c\u4f1a\u6309\u65f6\u95f4\u5012\u5e8f\u663e\u793a\u5728\u8fd9\u91cc',
  clear: '\u6e05\u7a7a',
  export: '\u5bfc\u51fa',
  clearAll: '\u6e05\u7a7a\u5168\u90e8\u8bc6\u522b\u8bb0\u5f55',
  exportAll: '\u5bfc\u51fa\u8bc6\u522b\u8bb0\u5f55',
  copy: '\u590d\u5236\u6b64\u8bb0\u5f55',
  reinsert: '\u91cd\u65b0\u4e0a\u5c4f\u6b64\u8bb0\u5f55',
  delete: '\u5220\u9664\u6b64\u8bc6\u522b\u8bb0\u5f55',
  countTitle: '\u672c\u6b21\u8fd0\u884c\u8bc6\u522b\u6b21\u6570',
  durationTitle: '\u672c\u6b21\u8fd0\u884c\u7d2f\u8ba1\u5f55\u97f3\u65f6\u957f',
  charsTitle: '\u672c\u6b21\u8fd0\u884c\u7d2f\u8ba1\u8bc6\u522b\u5b57\u6570',
  speedTitle: '\u672c\u6b21\u8fd0\u884c\u5e73\u5747\u8bc6\u522b\u901f\u5ea6',
  charUnit: '\u5b57',
  localModel: 'whisper.cpp',
  baiduShort: '\u767e\u5ea6\u77ed\u8bed\u97f3',
  baiduRealtime: '\u767e\u5ea6\u5b9e\u65f6 WebSocket',
  manual: '\u624b\u52a8',
};

interface MainWindowProps {
  status: AppStatus;
  hotkeyStatus: HotkeyRegistrationStatus;
  pushToTalkHotkey: string;
  toggleDictationHotkey: string;
  pushToTalkModelReadiness: ModelReadiness;
  toggleDictationModelReadiness: ModelReadiness;
  recorderInfo: RecorderInfo | null;
  records: TranscriptRecord[];
  stats: TranscriptStats;
  historyMessage: string | null;
  onOpenDiagnostic: () => void;
  onOpenModelSettings: () => void;
  onCopyRecord: (record: TranscriptRecord) => void;
  onReinsertRecord: (record: TranscriptRecord) => void;
  onDeleteRecord: (id: number) => void;
  onClearRecords: () => void;
  onExportRecords: () => void;
  onOpenHotkeySettings: () => void;
}

type IconName = 'settings' | 'copy' | 'reinsert' | 'delete' | 'count' | 'duration' | 'chars' | 'speed' | 'mode' | 'model';

function Icon({ name }: { name: IconName }) {
  const paths: Record<IconName, ReactNode> = {
    settings: <><path d="M12 15.5a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7Z" /><path d="M19.4 15a1.8 1.8 0 0 0 .36 1.98l.05.05a2.2 2.2 0 0 1-3.11 3.11l-.05-.05a1.8 1.8 0 0 0-1.98-.36 1.8 1.8 0 0 0-1.1 1.66V21.5a2.2 2.2 0 0 1-4.4 0v-.08A1.8 1.8 0 0 0 8.05 19.75a1.8 1.8 0 0 0-1.98.36l-.05.05a2.2 2.2 0 0 1-3.11-3.11l.05-.05A1.8 1.8 0 0 0 3.32 15a1.8 1.8 0 0 0-1.66-1.1H1.6a2.2 2.2 0 0 1 0-4.4h.08A1.8 1.8 0 0 0 3.34 8.4a1.8 1.8 0 0 0-.36-1.98l-.05-.05a2.2 2.2 0 0 1 3.11-3.11l.05.05a1.8 1.8 0 0 0 1.98.36h.02A1.8 1.8 0 0 0 9.2 2.02V1.94a2.2 2.2 0 0 1 4.4 0v.08a1.8 1.8 0 0 0 1.1 1.66 1.8 1.8 0 0 0 1.98-.36l.05-.05a2.2 2.2 0 0 1 3.11 3.11l-.05.05a1.8 1.8 0 0 0-.36 1.98v.02a1.8 1.8 0 0 0 1.66 1.1h.08a2.2 2.2 0 0 1 0 4.4h-.08A1.8 1.8 0 0 0 19.4 15Z" /></>,
    copy: <><rect x="8" y="8" width="11" height="11" rx="2" /><path d="M5 15H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2v1" /></>,
    reinsert: <><path d="M9 14 5 10l4-4" /><path d="M5 10h9a5 5 0 0 1 0 10h-2" /></>,
    delete: <><path d="M3 6h18" /><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" /><path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6" /><path d="M10 11v6" /><path d="M14 11v6" /></>,
    count: <><path d="M5 7h14" /><path d="M5 12h14" /><path d="M5 17h14" /><path d="M9 5 7 19" /><path d="M17 5l-2 14" /></>,
    duration: <><circle cx="12" cy="13" r="8" /><path d="M12 9v4l3 2" /><path d="M9 2h6" /></>,
    chars: <><path d="M4 19 10 5h4l6 14" /><path d="M7 14h10" /></>,
    speed: <><path d="M4 14a8 8 0 1 1 16 0" /><path d="M12 14l5-5" /><path d="M4 18h16" /></>,
    mode: <><path d="M7 8h10" /><path d="M7 16h10" /><circle cx="7" cy="8" r="2" /><circle cx="17" cy="16" r="2" /></>,
    model: <><rect x="4" y="4" width="16" height="16" rx="3" /><path d="M9 9h6v6H9z" /><path d="M9 1v3" /><path d="M15 1v3" /><path d="M9 20v3" /><path d="M15 20v3" /></>,
  };
  return <svg className="ui-icon" viewBox="0 0 24 24" aria-hidden="true">{paths[name]}</svg>;
}

function StatPill({ icon, value, title, tone = 'neutral' }: { icon: IconName; value: ReactNode; title: string; tone?: string }) {
  return <span className="stat-pill" data-tone={tone} title={title} aria-label={title}><Icon name={icon} /><span>{value}</span></span>;
}

function formatDuration(ms: number) {
  const totalSeconds = Math.floor(ms / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}:${seconds.toString().padStart(2, '0')}`;
}

function formatModelLabel(modelId: TranscriptionModelId) {
  if (modelId === 'local-whisper') return text.localModel;
  if (modelId === 'baidu-short') return text.baiduShort;
  return text.baiduRealtime;
}

function formatInputMode(source: TranscriptRecord['inputMode']) {
  if (source === 'push-to-talk') return text.pushToTalk;
  if (source === 'toggle-dictation') return text.toggleDictation;
  return text.manual;
}

export function MainWindow({ status, hotkeyStatus, pushToTalkHotkey, toggleDictationHotkey, pushToTalkModelReadiness, toggleDictationModelReadiness, recorderInfo, records, stats, historyMessage, onOpenDiagnostic, onOpenModelSettings, onCopyRecord, onReinsertRecord, onDeleteRecord, onClearRecords, onExportRecords, onOpenHotkeySettings }: MainWindowProps) {
  const hotkeysReady = hotkeyStatus.registered;
  const readinessItems = [
    { label: text.microphone, value: recorderInfo?.deviceName ?? text.waitingDevice, state: recorderInfo ? 'ready' : 'warning', title: recorderInfo ? `${recorderInfo.sampleRate} Hz / ${recorderInfo.channels}` : 'Waiting for input device.' },
    { label: text.paste, value: 'Clipboard', state: status.phase === 'failed' ? 'error' : 'ready', title: 'Clipboard paste path.' },
    { label: text.pushToTalkModel, value: pushToTalkModelReadiness.label, state: pushToTalkModelReadiness.ready ? 'ready' : 'warning', title: pushToTalkModelReadiness.message },
    { label: text.toggleDictationModel, value: toggleDictationModelReadiness.label, state: toggleDictationModelReadiness.ready ? 'ready' : 'warning', title: toggleDictationModelReadiness.message },
  ];

  return (
    <main className="app-shell user-shell">
      <section className="control-center v51" aria-labelledby="app-title">
        <header className="control-topbar">
          <div className="brand-block"><span className="product-mark">VoxType</span><h1 id="app-title">{text.controlCenter}</h1></div>
          <div className="topbar-actions"><button className="ghost-button" type="button" onClick={onOpenModelSettings}>{text.modelSettings}</button><button className="ghost-button" type="button" onClick={onOpenDiagnostic}>{text.diagnostic}</button></div>
        </header>
        <div className="v51-top-grid">
          <section className="control-section mode-selector" aria-label={text.inputMode}>
            <div className="section-heading"><span>{text.inputMode}</span><strong>{text.hotkeyStatus}</strong><button className="icon-only-button section-action" type="button" onClick={onOpenHotkeySettings} title={text.hotkeySettings} aria-label={text.hotkeySettings}><Icon name="settings" /></button></div>
            <article className="mode-card compact"><div className="mode-line"><h2>{text.pushToTalk}</h2><kbd>{pushToTalkHotkey}</kbd><span className="ready-dot" data-ready={hotkeysReady} title={hotkeysReady ? text.registered : text.unregistered} aria-label={hotkeysReady ? text.registered : text.unregistered} /></div></article>
            <article className="mode-card compact"><div className="mode-line"><h2>{text.toggleDictation}</h2><kbd>{toggleDictationHotkey}</kbd><span className="ready-dot" data-ready={hotkeysReady} title={hotkeysReady ? text.registered : text.unregistered} aria-label={hotkeysReady ? text.registered : text.unregistered} /></div></article>
          </section>
          <section className="control-section readiness-panel" aria-label={text.readiness}>
            <div className="section-heading"><span>{text.readiness}</span><strong>{text.capabilities}</strong></div>
            <dl className="readiness-grid">
              {readinessItems.map((item) => <div key={item.label} data-state={item.state} title={item.title}><dt>{item.label}</dt><dd>{item.value}</dd></div>)}
            </dl>
          </section>
        </div>
        <section className="transcript-history" aria-label={text.history}>
          <header className="history-header" data-testid="history-toolbar"><div className="history-title"><span>{text.history}</span><strong>{records.length} {text.itemUnit}</strong></div><div className="history-stats"><StatPill icon="count" value={stats.count} title={text.countTitle} tone="count" /><StatPill icon="duration" value={formatDuration(stats.totalDurationMs)} title={text.durationTitle} tone="duration" /><StatPill icon="chars" value={stats.totalChars} title={text.charsTitle} tone="chars" /><StatPill icon="speed" value={`${stats.charsPerMinute}/m`} title={text.speedTitle} tone="speed" /></div><div className="history-actions"><button type="button" onClick={onClearRecords} disabled={records.length === 0} aria-label={text.clearAll} title={text.clearAll}>{text.clear}</button><button type="button" onClick={onExportRecords} disabled={records.length === 0} aria-label={text.exportAll} title={text.exportAll}>{text.export}</button></div></header>
          {historyMessage ? <p className="history-message" role="status">{historyMessage}</p> : null}
          {records.length === 0 ? <div className="empty-history"><span>{text.empty}</span><strong>{text.emptyDetail}</strong></div> : <ol className="history-list">{records.map((record) => <li key={record.id}><article aria-label={`${text.history} ${record.time}`}><div className="record-head"><time>{record.time}</time><p>{record.text}</p></div><footer className="record-footer"><div className="record-meta"><StatPill icon="mode" value={formatInputMode(record.inputMode)} title={text.inputMode} tone="mode" /><StatPill icon="model" value={formatModelLabel(record.modelId)} title={text.modelSettings} tone="model" /><StatPill icon="duration" value={formatDuration(record.durationMs)} title={text.durationTitle} tone="duration" /><StatPill icon="chars" value={`${record.charCount} ${text.charUnit}`} title={text.charsTitle} tone="chars" /></div><div className="record-actions"><button type="button" onClick={() => onCopyRecord(record)} aria-label={text.copy} title={text.copy}><Icon name="copy" /></button><button type="button" onClick={() => onReinsertRecord(record)} aria-label={text.reinsert} title={text.reinsert}><Icon name="reinsert" /></button><button type="button" onClick={() => onDeleteRecord(record.id)} aria-label={text.delete} title={text.delete}><Icon name="delete" /></button></div></footer></article></li>)}</ol>}
        </section>
      </section>
    </main>
  );
}
