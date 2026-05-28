import type { ReactNode } from 'react';
import type { OverlayBackendStatus } from './tauriClient';
import type { AppConfig, AppStatus, AsrConfigStatus, HotkeyRegistrationStatus, RecorderInfo, RecorderRuntimeStatus } from './types';

interface DiagnosticViewProps {
  config: AppConfig;
  status: AppStatus;
  runtimeMessage: string;
  recorderInfo: RecorderInfo | null;
  recordingStatus: RecorderRuntimeStatus;
  isRecording: boolean;
  hotkeyStatus: HotkeyRegistrationStatus;
  overlayBackendStatus: OverlayBackendStatus;
  asrConfigStatus: AsrConfigStatus;
  whisperBinaryPath: string;
  whisperModelPath: string;
  asrLanguage: string;
  isInstallingAsr: boolean;
  deviceSelect: ReactNode;
  diagnosticsPanel: ReactNode;
  onBack: () => void;
  onRefreshHotkeyStatus: () => void;
  onShowOverlayTest: () => void;
  onHideOverlayTest: () => void;
  onWhisperBinaryPathChange: (value: string) => void;
  onWhisperModelPathChange: (value: string) => void;
  onAsrLanguageChange: (value: string) => void;
  onInstallManagedAsr: () => void;
  onSaveAsrConfig: () => void;
  onRefreshAsrConfig: () => void;
  onStartRecording: () => void;
  onStopRecording: () => void;
  onRefreshRecordingStatus: () => void;
  onExportLastRecordingWav: () => void;
  onTranscribeLastRecording: () => void;
  onTranscribeAndInsert: () => void;
  onSimulateDictation: () => void;
  onClipboardInsert: () => void;
}

export function DiagnosticView(props: DiagnosticViewProps) {
  const { config, status, runtimeMessage, recorderInfo, recordingStatus, isRecording, hotkeyStatus, overlayBackendStatus, asrConfigStatus, whisperBinaryPath, whisperModelPath, asrLanguage, isInstallingAsr, deviceSelect, diagnosticsPanel } = props;
  return (
    <main className="app-shell diagnostic-shell">
      <section className="hero diagnostic-hero" aria-labelledby="diagnostic-title">
        <div><p className="eyebrow">DIAGNOSTICS</p><h1 id="diagnostic-title">诊断工作台</h1><p>系统能力、ASR 配置、快捷键和上屏链路。</p></div>
        <button className="secondary-button" type="button" onClick={props.onBack}>返回主界面</button>
      </section>
      <section className="panel" aria-labelledby="settings-title">
        <h2 id="settings-title">运行状态</h2>
        <dl className="settings-grid">
          <div><dt>快捷键</dt><dd>{config.hotkey}</dd></div>
          <div><dt>目标语言</dt><dd>中文优先，兼容英文</dd></div>
          <div><dt>ASR 路线</dt><dd>{config.asrEngine}</dd></div>
          <div><dt>上屏策略</dt><dd>剪贴板粘贴</dd></div>
          <div><dt>全局快捷键</dt><dd>{hotkeyStatus.message}</dd></div>
          <div><dt>桌面浮窗后端</dt><dd>{overlayBackendStatus.lastError ? `${overlayBackendStatus.backend} / ${overlayBackendStatus.lastError}` : overlayBackendStatus.backend}</dd></div>
          <div><dt>麦克风</dt><dd>{recorderInfo ? `${recorderInfo.deviceName} / ${recorderInfo.sampleRate} Hz / ${recorderInfo.channels} 声道` : '等待 Tauri 读取'}</dd></div>
          <div><dt>录音流</dt><dd>{isRecording ? `录音中 / ${recordingStatus.sampleCount} 样本 / ${recordingStatus.durationMs} ms` : '空闲'}</dd></div>
        </dl>
        <div className="button-row"><button type="button" onClick={props.onRefreshHotkeyStatus}>刷新全局快捷键状态</button><button type="button" onClick={props.onShowOverlayTest}>测试桌面浮窗</button><button type="button" onClick={props.onHideOverlayTest}>隐藏桌面浮窗</button></div>
        <div className="config-form single-row-form">{deviceSelect}</div>
      </section>
      <section className="panel" aria-labelledby="asr-config-title">
        <div className="panel-heading-row"><h2 id="asr-config-title">ASR 配置</h2><span className="status-pill" data-phase={asrConfigStatus.ready ? 'succeeded' : 'failed'}>{asrConfigStatus.ready ? '已就绪' : '未就绪'}</span></div>
        <p className="runtime-message">{asrConfigStatus.message}</p>
        <div className="config-form">
          <label className="field"><span>whisper.cpp 可执行文件</span><input value={whisperBinaryPath} onChange={(event) => props.onWhisperBinaryPathChange(event.target.value)} placeholder="例如 C:\tools\whisper.cpp\whisper-cli.exe" /></label>
          <label className="field"><span>Whisper 模型文件</span><input value={whisperModelPath} onChange={(event) => props.onWhisperModelPathChange(event.target.value)} placeholder="例如 C:\models\ggml-small.bin" /></label>
          <label className="field"><span>识别语言</span><input value={asrLanguage} onChange={(event) => props.onAsrLanguageChange(event.target.value)} placeholder="zh" /></label>
        </div>
        <div className="button-row"><button type="button" onClick={props.onInstallManagedAsr} disabled={isInstallingAsr}>{isInstallingAsr ? '正在安装 whisper.cpp' : '一键安装 whisper.cpp'}</button><button type="button" onClick={props.onSaveAsrConfig}>保存 ASR 配置</button><button type="button" onClick={props.onRefreshAsrConfig}>检测 ASR 配置</button></div>
      </section>
      <section className="panel" aria-labelledby="status-title">
        <h2 id="status-title">测试操作</h2>
        <p className="runtime-message">{runtimeMessage}</p>
        <p>{status.message}</p>
        {status.lastTranscript ? <blockquote>{status.lastTranscript}</blockquote> : null}
        <div className="button-row">
          <button type="button" onClick={props.onStartRecording} disabled={isRecording}>开始录音采集</button>
          <button type="button" onClick={props.onStopRecording} disabled={!isRecording}>停止录音采集</button>
          <button type="button" onClick={props.onRefreshRecordingStatus}>刷新录音状态</button>
          <button type="button" onClick={props.onExportLastRecordingWav}>导出最近录音 WAV</button>
          <button type="button" onClick={props.onTranscribeLastRecording}>转写最近录音</button>
          <button type="button" onClick={props.onTranscribeAndInsert}>转写并上屏最近录音</button>
          <button type="button" onClick={props.onSimulateDictation}>模拟一次语音输入闭环</button>
          <button type="button" onClick={props.onClipboardInsert}>测试剪贴板上屏</button>
        </div>
      </section>
      {diagnosticsPanel}
    </main>
  );
}