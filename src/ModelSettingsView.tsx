import { useState } from 'react';
import type { AsrConfigStatus, CloudAsrConfigStatus, ModelReadiness, TranscriptionModelId } from './types';

const text = {
  title: '\u6a21\u578b\u9009\u62e9\u914d\u7f6e',
  back: '\u8fd4\u56de\u4e3b\u754c\u9762',
  defaultModels: '\u8f93\u5165\u6a21\u5f0f\u9ed8\u8ba4\u6a21\u578b',
  selectionPage: '\u6a21\u578b\u9009\u62e9',
  configPage: '\u6a21\u578b\u914d\u7f6e',
  pushToTalk: '\u6309\u4f4f\u8bf4\u8bdd',
  toggleDictation: '\u8fde\u7eed\u8f93\u5165',
  selectPrefix: '\u9009\u62e9',
  selectSuffix: '\u6a21\u578b',
  modelConfig: '\u6a21\u578b\u914d\u7f6e',
  configDetail: '\u9009\u62e9\u4e00\u4e2a\u6a21\u578b\u540e\u7f16\u8f91\u5bf9\u5e94\u914d\u7f6e',
  local: '\u672c\u5730 whisper.cpp',
  executable: 'whisper.cpp \u53ef\u6267\u884c\u6587\u4ef6',
  modelFile: 'Whisper \u6a21\u578b\u6587\u4ef6',
  language: '\u8bc6\u522b\u8bed\u8a00',
  installing: '\u6b63\u5728\u5b89\u88c5 whisper.cpp',
  install: '\u4e00\u952e\u5b89\u88c5 whisper.cpp',
  checkAsr: '\u68c0\u6d4b ASR \u914d\u7f6e',
  saveAsr: '\u4fdd\u5b58 ASR \u914d\u7f6e',
  baiduShort: '\u767e\u5ea6\u77ed\u8bed\u97f3 API',
  baiduShortLabel: '\u767e\u5ea6\u77ed\u8bed\u97f3 API',
  baiduShortConfig: '\u767e\u5ea6\u77ed\u8bed\u97f3 API \u914d\u7f6e',
  apiKey: '\u767e\u5ea6 ASR API Key',
  apiKeyInput: '\u767e\u5ea6 ASR API Key \u8f93\u5165',
  secretKey: '\u767e\u5ea6 ASR Secret Key',
  secretKeyInput: '\u767e\u5ea6 ASR Secret Key \u8f93\u5165',
  saveApiKey: '\u4fdd\u5b58\u767e\u5ea6 API Key \u5230\u7cfb\u7edf\u73af\u5883\u53d8\u91cf',
  saveSecretKey: '\u4fdd\u5b58\u767e\u5ea6 Secret Key \u5230\u7cfb\u7edf\u73af\u5883\u53d8\u91cf',
  endpoint: '\u767e\u5ea6 ASR Endpoint',
  devPid: '\u767e\u5ea6 ASR dev_pid',
  cuid: '\u767e\u5ea6 ASR cuid',
  format: '\u767e\u5ea6 ASR \u97f3\u9891\u683c\u5f0f',
  sampleRate: '\u767e\u5ea6 ASR \u91c7\u6837\u7387',
  lmId: '\u767e\u5ea6 ASR lm_id\uff08\u53ef\u9009\uff09',
  saveBaidu: '\u4fdd\u5b58\u767e\u5ea6\u914d\u7f6e',
  testBaidu: '\u68c0\u6d4b\u767e\u5ea6\u914d\u7f6e',
  waitingEnv: '\u7b49\u5f85\u73af\u5883\u53d8\u91cf',
  pasteSecret: '\u7c98\u8d34\u540e\u4fdd\u5b58\u5230\u7528\u6237\u73af\u5883\u53d8\u91cf',
  baiduRealtime: '\u767e\u5ea6\u5b9e\u65f6 WebSocket API',
  baiduRealtimeLabel: '\u767e\u5ea6\u5b9e\u65f6 WebSocket API',
  realtimeConfig: '\u767e\u5ea6\u5b9e\u65f6 WebSocket API \u914d\u7f6e',
  wsAppId: 'AppID',
  wsEndpoint: 'WebSocket Endpoint',
  wsDevPid: 'dev_pid',
  wsCuid: 'cuid',
  wsFormat: '\u97f3\u9891\u683c\u5f0f',
  wsSampleRate: '\u91c7\u6837\u7387',
  wsUser: 'user\uff08\u53ef\u9009\uff09',
  baiduRealtimeAppIdLabel: '\u767e\u5ea6 WebSocket AppID',
  baiduRealtimeEndpointLabel: '\u767e\u5ea6 WebSocket Endpoint',
  baiduRealtimeDevPidLabel: '\u767e\u5ea6 WebSocket dev_pid',
  baiduRealtimeCuidLabel: '\u767e\u5ea6 WebSocket cuid',
  baiduRealtimeFormatLabel: '\u767e\u5ea6 WebSocket \u97f3\u9891\u683c\u5f0f',
  baiduRealtimeSampleRateLabel: '\u767e\u5ea6 WebSocket \u91c7\u6837\u7387',
  baiduRealtimeUserLabel: '\u767e\u5ea6 WebSocket user',
  lmPlaceholder: '\u81ea\u8bad\u7ec3\u5e73\u53f0\u6a21\u578b ID',
  ready: '\u5df2\u5c31\u7eea',
  notReady: '\u672a\u5c31\u7eea',
};

interface ModelSettingsViewProps {
  asrConfigStatus: AsrConfigStatus;
  cloudAsrConfigStatus: CloudAsrConfigStatus;
  pushToTalkModel: TranscriptionModelId;
  toggleDictationModel: TranscriptionModelId;
  modelReadiness: Record<TranscriptionModelId, ModelReadiness>;
  cloudBaseUrl: string;
  cloudModel: string;
  cloudLanguage: string;
  cloudBaiduCuid: string;
  cloudBaiduFormat: string;
  cloudBaiduSampleRate: string;
  cloudBaiduLmId: string;
  cloudBaiduRealtimeAppId: string;
  cloudBaiduRealtimeEndpoint: string;
  cloudBaiduRealtimeDevPid: string;
  cloudBaiduRealtimeCuid: string;
  cloudBaiduRealtimeFormat: string;
  cloudBaiduRealtimeSampleRate: string;
  cloudBaiduRealtimeUser: string;
  cloudApiKeyInput: string;
  cloudSecretKeyInput: string;
  cloudMessage: string | null;
  whisperBinaryPath: string;
  whisperModelPath: string;
  asrLanguage: string;
  isInstallingAsr: boolean;
  onBack: () => void;
  onPushToTalkModelChange: (value: TranscriptionModelId) => void;
  onToggleDictationModelChange: (value: TranscriptionModelId) => void;
  onWhisperBinaryPathChange: (value: string) => void;
  onWhisperModelPathChange: (value: string) => void;
  onAsrLanguageChange: (value: string) => void;
  onCloudBaseUrlChange: (value: string) => void;
  onCloudModelChange: (value: string) => void;
  onCloudLanguageChange: (value: string) => void;
  onCloudBaiduCuidChange: (value: string) => void;
  onCloudBaiduFormatChange: (value: string) => void;
  onCloudBaiduSampleRateChange: (value: string) => void;
  onCloudBaiduLmIdChange: (value: string) => void;
  onCloudBaiduRealtimeAppIdChange: (value: string) => void;
  onCloudBaiduRealtimeEndpointChange: (value: string) => void;
  onCloudBaiduRealtimeDevPidChange: (value: string) => void;
  onCloudBaiduRealtimeCuidChange: (value: string) => void;
  onCloudBaiduRealtimeFormatChange: (value: string) => void;
  onCloudBaiduRealtimeSampleRateChange: (value: string) => void;
  onCloudBaiduRealtimeUserChange: (value: string) => void;
  onCloudApiKeyInputChange: (value: string) => void;
  onCloudSecretKeyInputChange: (value: string) => void;
  onInstallManagedAsr: () => void;
  onSaveAsrConfig: () => void;
  onRefreshAsrConfig: () => void;
  onSaveCloudAsrConfig: () => void;
  onSaveBaiduAsrApiKey: () => void;
  onSaveBaiduAsrSecretKey: () => void;
  onTestCloudAsrConfig: () => void;
}

const modelOptions: TranscriptionModelId[] = ['local-whisper', 'baidu-short', 'baidu-realtime'];

function displayModelLabel(id: TranscriptionModelId) {
  if (id === 'local-whisper') return text.local;
  if (id === 'baidu-short') return text.baiduShortLabel;
  return text.baiduRealtimeLabel;
}

function ModelButton({ id, active, readiness, onClick }: { id: TranscriptionModelId; active: boolean; readiness: ModelReadiness; onClick: () => void }) {
  return <button className="route-model-button" type="button" data-active={active} data-available={readiness.availableInV7} onClick={onClick} title={readiness.message} aria-pressed={active}><span className="ready-dot" data-ready={readiness.ready} aria-hidden="true" /><span>{displayModelLabel(id)}</span></button>;
}

function ModeModelSelector({ modeName, value, onChange, readiness }: { modeName: string; value: TranscriptionModelId; onChange: (value: TranscriptionModelId) => void; readiness: Record<TranscriptionModelId, ModelReadiness> }) {
  return (
    <div className="mode-model-row">
      <div className="mode-model-label"><span>{text.selectPrefix}<span className="mode-label-pill">{modeName}</span>{text.selectSuffix}</span></div>
      <div className="segmented-models route-models" role="group" aria-label={`${modeName}\u6a21\u578b\u9ed8\u8ba4\u6a21\u578b`}>
        {modelOptions.map((id) => <ModelButton key={id} id={id} active={value === id} readiness={readiness[id]} onClick={() => onChange(id)} />)}
      </div>
    </div>
  );
}

export function ModelSettingsView(props: ModelSettingsViewProps) {
  const [activePage, setActivePage] = useState<'selection' | 'config'>('selection');
  const [activeConfig, setActiveConfig] = useState<TranscriptionModelId>(props.pushToTalkModel);
  const apiKeyState = props.cloudAsrConfigStatus.apiKeyConfigured ? '\u5df2\u914d\u7f6e' : '\u672a\u914d\u7f6e';
  const apiKeyDetail = props.cloudAsrConfigStatus.apiKeyPreview ?? text.waitingEnv;
  const secretKeyState = props.cloudAsrConfigStatus.secretKeyConfigured ? '\u5df2\u914d\u7f6e' : '\u672a\u914d\u7f6e';
  const secretKeyDetail = props.cloudAsrConfigStatus.secretKeyPreview ?? text.waitingEnv;
  const baiduCredentialFields = (
    <div className="secret-grid">
      <div className="secret-block"><div className="cloud-key-status"><span>{text.apiKey}</span><strong>{apiKeyState}</strong><code>BAIDU_ASR_API_KEY</code><small>{apiKeyDetail}</small></div><label className="field"><span>{text.apiKey}</span><input aria-label={text.apiKeyInput} type="password" autoComplete="off" spellCheck={false} value={props.cloudApiKeyInput} onChange={(event) => props.onCloudApiKeyInputChange(event.target.value)} placeholder={text.pasteSecret} /></label><button type="button" onClick={props.onSaveBaiduAsrApiKey} disabled={!props.cloudApiKeyInput.trim()}>{text.saveApiKey}</button></div>
      <div className="secret-block"><div className="cloud-key-status"><span>{text.secretKey}</span><strong>{secretKeyState}</strong><code>BAIDU_ASR_SECRET_KEY</code><small>{secretKeyDetail}</small></div><label className="field"><span>{text.secretKey}</span><input aria-label={text.secretKeyInput} type="password" autoComplete="off" spellCheck={false} value={props.cloudSecretKeyInput} onChange={(event) => props.onCloudSecretKeyInputChange(event.target.value)} placeholder={text.pasteSecret} /></label><button type="button" onClick={props.onSaveBaiduAsrSecretKey} disabled={!props.cloudSecretKeyInput.trim()}>{text.saveSecretKey}</button></div>
    </div>
  );

  return (
    <main className="app-shell model-shell">
      <section className="model-panel" aria-labelledby="model-title">
        <header className="model-header">
          <div className="model-title-line"><span className="product-mark">VoxType</span><h1 id="model-title">{text.title}</h1></div>
          <button className="secondary-button" type="button" onClick={props.onBack}>{text.back}</button>
        </header>
        <nav className="model-page-tabs" aria-label={text.title}>
          <button type="button" data-active={activePage === 'selection'} onClick={() => setActivePage('selection')}>{text.selectionPage}</button>
          <button type="button" data-active={activePage === 'config'} onClick={() => setActivePage('config')}>{text.configPage}</button>
        </nav>
        {activePage === 'selection' ? <section className="model-routing-section" aria-label={text.defaultModels}>
          <div className="mode-routing-card">
            <ModeModelSelector modeName={text.pushToTalk} value={props.pushToTalkModel} onChange={props.onPushToTalkModelChange} readiness={props.modelReadiness} />
            <ModeModelSelector modeName={text.toggleDictation} value={props.toggleDictationModel} onChange={props.onToggleDictationModelChange} readiness={props.modelReadiness} />
          </div>
        </section> : null}
        {activePage === 'config' ? <section className="model-config-section" aria-label={text.modelConfig}>
          <div className="section-heading model-config-heading"><span>{text.modelConfig}</span><strong>{text.configDetail}</strong></div>
          <div className="config-model-switch" role="tablist" aria-label={text.modelConfig}>
            {modelOptions.map((id) => <button key={id} className="config-switch-button" type="button" role="tab" aria-selected={activeConfig === id} data-active={activeConfig === id} data-available={props.modelReadiness[id].availableInV7} onClick={() => setActiveConfig(id)} title={props.modelReadiness[id].message}><span className="ready-dot" data-ready={props.modelReadiness[id].ready} aria-hidden="true" /><span>{displayModelLabel(id)}</span></button>)}
          </div>
          {activeConfig === 'local-whisper' ? <section className="model-config active-config-panel" aria-label={`${text.local} \u914d\u7f6e`}>
            <div className="model-config-title"><div><span>{text.local}</span><strong>{props.asrConfigStatus.ready ? text.ready : text.notReady}</strong></div><span className="ready-dot" data-ready={props.asrConfigStatus.ready} /></div>
            <label className="field"><span>{text.executable}</span><input aria-label={text.executable} value={props.whisperBinaryPath} onChange={(event) => props.onWhisperBinaryPathChange(event.target.value)} placeholder="C:\\tools\\whisper.cpp\\whisper-cli.exe" /></label>
            <label className="field"><span>{text.modelFile}</span><input aria-label={text.modelFile} value={props.whisperModelPath} onChange={(event) => props.onWhisperModelPathChange(event.target.value)} placeholder="C:\\models\\ggml-small.bin" /></label>
            <label className="field"><span>{text.language}</span><input aria-label={text.language} value={props.asrLanguage} onChange={(event) => props.onAsrLanguageChange(event.target.value)} placeholder="zh" /></label>
            <p className="runtime-message">{props.asrConfigStatus.message}</p>
            <div className="button-row"><button type="button" onClick={props.onInstallManagedAsr} disabled={props.isInstallingAsr}>{props.isInstallingAsr ? text.installing : text.install}</button><button type="button" onClick={props.onRefreshAsrConfig}>{text.checkAsr}</button><button type="button" onClick={props.onSaveAsrConfig}>{text.saveAsr}</button></div>
          </section> : null}
          {activeConfig === 'baidu-short' ? <section className="model-config active-config-panel cloud-config" aria-label={text.baiduShortConfig}>
            <div className="model-config-title"><div><span>{text.baiduShort}</span><strong>{props.cloudAsrConfigStatus.ready ? text.ready : text.notReady}</strong></div><span className="ready-dot" data-ready={props.cloudAsrConfigStatus.ready} /></div>
            {baiduCredentialFields}
            <div className="compact-field-grid">
              <label className="field wide"><span>{text.endpoint}</span><input aria-label={text.endpoint} value={props.cloudBaseUrl} onChange={(event) => props.onCloudBaseUrlChange(event.target.value)} /></label>
              <label className="field"><span>{text.devPid}</span><input aria-label={text.devPid} value={props.cloudModel} onChange={(event) => props.onCloudModelChange(event.target.value)} /></label>
              <label className="field"><span>{text.cuid}</span><input aria-label={text.cuid} value={props.cloudBaiduCuid} onChange={(event) => props.onCloudBaiduCuidChange(event.target.value)} /></label>
              <label className="field"><span>{text.format}</span><input aria-label={text.format} value={props.cloudBaiduFormat} onChange={(event) => props.onCloudBaiduFormatChange(event.target.value)} /></label>
              <label className="field"><span>{text.sampleRate}</span><input aria-label={text.sampleRate} type="number" value={props.cloudBaiduSampleRate} onChange={(event) => props.onCloudBaiduSampleRateChange(event.target.value)} /></label>
              <label className="field"><span>{text.lmId}</span><input aria-label={text.lmId} value={props.cloudBaiduLmId} onChange={(event) => props.onCloudBaiduLmIdChange(event.target.value)} placeholder={text.lmPlaceholder} /></label>
              <label className="field"><span>{text.language}</span><input aria-label={text.language} value={props.cloudLanguage} onChange={(event) => props.onCloudLanguageChange(event.target.value)} /></label>
            </div>
            <p className="runtime-message">{props.cloudMessage ?? props.cloudAsrConfigStatus.message}</p>
            <div className="button-row"><button type="button" onClick={props.onSaveCloudAsrConfig}>{text.saveBaidu}</button><button type="button" onClick={props.onTestCloudAsrConfig}>{text.testBaidu}</button></div>
          </section> : null}
          {activeConfig === 'baidu-realtime' ? <section className="model-config active-config-panel cloud-config realtime-config" aria-label={text.realtimeConfig}>
            <div className="model-config-title"><div><span>{text.baiduRealtime}</span><strong>{props.modelReadiness['baidu-realtime'].ready ? text.ready : text.notReady}</strong></div><span className="ready-dot" data-ready={props.modelReadiness['baidu-realtime'].ready} /></div>
            <p className="runtime-message">{props.cloudMessage ?? props.modelReadiness['baidu-realtime'].message}</p>
            {baiduCredentialFields}
            <div className="compact-field-grid">
              <label className="field"><span>{text.wsAppId}</span><input aria-label={text.baiduRealtimeAppIdLabel} value={props.cloudBaiduRealtimeAppId} onChange={(event) => props.onCloudBaiduRealtimeAppIdChange(event.target.value)} /></label>
              <label className="field wide"><span>{text.wsEndpoint}</span><input aria-label={text.baiduRealtimeEndpointLabel} value={props.cloudBaiduRealtimeEndpoint} onChange={(event) => props.onCloudBaiduRealtimeEndpointChange(event.target.value)} /></label>
              <label className="field"><span>{text.wsDevPid}</span><input aria-label={text.baiduRealtimeDevPidLabel} value={props.cloudBaiduRealtimeDevPid} onChange={(event) => props.onCloudBaiduRealtimeDevPidChange(event.target.value)} /></label>
              <label className="field"><span>{text.wsCuid}</span><input aria-label={text.baiduRealtimeCuidLabel} value={props.cloudBaiduRealtimeCuid} onChange={(event) => props.onCloudBaiduRealtimeCuidChange(event.target.value)} /></label>
              <label className="field"><span>{text.wsFormat}</span><input aria-label={text.baiduRealtimeFormatLabel} value={props.cloudBaiduRealtimeFormat} onChange={(event) => props.onCloudBaiduRealtimeFormatChange(event.target.value)} /></label>
              <label className="field"><span>{text.wsSampleRate}</span><input aria-label={text.baiduRealtimeSampleRateLabel} type="number" value={props.cloudBaiduRealtimeSampleRate} onChange={(event) => props.onCloudBaiduRealtimeSampleRateChange(event.target.value)} /></label>
              <label className="field"><span>{text.wsUser}</span><input aria-label={text.baiduRealtimeUserLabel} value={props.cloudBaiduRealtimeUser} onChange={(event) => props.onCloudBaiduRealtimeUserChange(event.target.value)} /></label>
            </div>
            <div className="button-row"><button type="button" onClick={props.onSaveCloudAsrConfig}>{text.saveBaidu}</button><button type="button" onClick={props.onTestCloudAsrConfig}>{text.testBaidu}</button></div>
          </section> : null}
        </section> : null}
      </section>
    </main>
  );
}
