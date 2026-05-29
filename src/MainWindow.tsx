import type { AppStatus, AsrConfigStatus, HotkeyRegistrationStatus, RecorderInfo, TranscriptRecord, TranscriptStats } from './types';

interface MainWindowProps {
  status: AppStatus;
  asrConfigStatus: AsrConfigStatus;
  hotkeyStatus: HotkeyRegistrationStatus;
  toggleHotkey: string;
  recorderInfo: RecorderInfo | null;
  records: TranscriptRecord[];
  stats: TranscriptStats;
  onOpenDiagnostic: () => void;
  onOpenModelSettings: () => void;
  onCopyRecord: (record: TranscriptRecord) => void;
  onReinsertRecord: (record: TranscriptRecord) => void;
  onDeleteRecord: (id: number) => void;
  onClearRecords: () => void;
  onExportRecords: () => void;
}

function formatDuration(ms: number) {
  const totalSeconds = Math.floor(ms / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}:${seconds.toString().padStart(2, '0')}`;
}

export function MainWindow({ status, asrConfigStatus, hotkeyStatus, toggleHotkey, recorderInfo, records, stats, onOpenDiagnostic, onOpenModelSettings, onCopyRecord, onReinsertRecord, onDeleteRecord, onClearRecords, onExportRecords }: MainWindowProps) {
  const hotkeysReady = hotkeyStatus.registered;
  const readinessItems = [
    { label: '麦克风', value: recorderInfo?.deviceName ?? '等待设备', state: recorderInfo ? 'ready' : 'warning', title: recorderInfo ? `${recorderInfo.sampleRate} Hz / ${recorderInfo.channels} 声道` : '等待 Tauri 读取默认输入设备。' },
    { label: '本地识别', value: asrConfigStatus.ready ? 'whisper.cpp' : '未配置', state: asrConfigStatus.ready ? 'ready' : 'warning', title: asrConfigStatus.message },
    { label: '上屏', value: 'Clipboard', state: status.phase === 'failed' ? 'error' : 'ready', title: '使用剪贴板写入文本并发送 Ctrl+V。' },
    { label: '云端 API', value: '未配置', state: 'warning', title: '云端 API / 大模型接入计划放在下一版。' },
  ];

  return (
    <main className="app-shell user-shell">
      <section className="control-center v51" aria-labelledby="app-title">
        <header className="control-topbar">
          <div className="brand-block"><span className="product-mark">VoxType</span><h1 id="app-title">语音输入控制中心</h1></div>
          <div className="topbar-actions"><button className="ghost-button" type="button" onClick={onOpenModelSettings}>模型选择</button><button className="ghost-button" type="button" onClick={onOpenDiagnostic}>诊断</button></div>
        </header>
        <div className="v51-top-grid">
          <section className="control-section mode-selector" aria-label="输入模式">
            <div className="section-heading"><span>输入模式</span><strong>快捷键状态</strong></div>
            <article className="mode-card compact" title="按住快捷键说一句，松开后转写并上屏。"><div className="mode-line"><h2>按住说话</h2><kbd>Ctrl+Alt+Space</kbd></div><div className="mode-card-actions"><span className="ready-dot" data-ready={hotkeysReady} title={hotkeysReady ? '快捷键已注册' : '快捷键未注册'} aria-label={hotkeysReady ? '快捷键已注册' : '快捷键未注册'} /><button type="button" disabled title="下一版支持自定义快捷键" aria-label="修改按住说话快捷键">⚙</button></div></article>
            <article className="mode-card compact" title="按一次开始，录音中分段上屏，再按一次停止。"><div className="mode-line"><h2>连续输入</h2><kbd>{toggleHotkey}</kbd></div><div className="mode-card-actions"><span className="ready-dot" data-ready={hotkeysReady} title={hotkeysReady ? '快捷键已注册' : '快捷键未注册'} aria-label={hotkeysReady ? '快捷键已注册' : '快捷键未注册'} /><button type="button" disabled title="下一版支持自定义快捷键" aria-label="修改连续输入快捷键">⚙</button></div></article>
          </section>
          <section className="control-section readiness-panel" aria-label="准备状态">
            <div className="section-heading"><span>准备状态</span><strong>系统能力</strong></div>
            <dl className="readiness-grid">
              {readinessItems.map((item) => <div key={item.label} data-state={item.state} title={item.title}><dt>{item.label}</dt><dd>{item.value}</dd></div>)}
            </dl>
          </section>
        </div>
        <section className="transcript-history" aria-label="识别记录">
          <header className="history-header" data-testid="history-toolbar"><div className="history-title"><span>识别记录</span><strong>{records.length} 条</strong></div><div className="history-stats"><span title="本次运行识别次数"># {stats.count}</span><span title="本次运行累计录音时长">⏱ {formatDuration(stats.totalDurationMs)}</span><span title="本次运行累计识别字数">文 {stats.totalChars}</span><span title="本次运行平均识别速度">⚡ {stats.charsPerMinute}/m</span></div><div className="history-actions"><button type="button" onClick={onClearRecords} disabled={records.length === 0} aria-label="清空全部识别记录" title="清空全部识别记录">清空</button><button type="button" onClick={onExportRecords} disabled={records.length === 0} aria-label="导出识别记录" title="导出识别记录">导出</button></div></header>
          {records.length === 0 ? <div className="empty-history"><span>等待第一次识别</span><strong>转写结果会按时间倒序显示在这里</strong></div> : <ol className="history-list">{records.map((record) => <li key={record.id}><article aria-label={`识别记录 ${record.time}`}><time>{record.time}</time><p>{record.text}</p><div className="record-actions"><button type="button" onClick={() => onCopyRecord(record)} aria-label="复制此记录" title="复制此记录">⧉</button><button type="button" onClick={() => onReinsertRecord(record)} aria-label="重新上屏此记录" title="重新上屏此记录">↩</button><button type="button" onClick={() => onDeleteRecord(record.id)} aria-label="删除此识别记录" title="删除此识别记录">×</button></div></article></li>)}</ol>}
        </section>
      </section>
    </main>
  );
 }
