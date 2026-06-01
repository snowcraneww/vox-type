import { useState } from 'react';
import type { AsrConfigStatus, CloudAsrConfigStatus } from './types';

interface ModelSettingsViewProps {
  asrConfigStatus: AsrConfigStatus;
  cloudAsrConfigStatus: CloudAsrConfigStatus;
  cloudProvider: string;
  cloudGroupId: string;
  cloudBaseUrl: string;
  cloudModel: string;
  cloudLanguage: string;
  cloudBaiduCuid: string;
  cloudBaiduFormat: string;
  cloudBaiduSampleRate: string;
  cloudApiKeyInput: string;
  cloudSecretKeyInput: string;
  cloudMessage: string | null;
  whisperBinaryPath: string;
  whisperModelPath: string;
  asrLanguage: string;
  isInstallingAsr: boolean;
  onBack: () => void;
  onWhisperBinaryPathChange: (value: string) => void;
  onWhisperModelPathChange: (value: string) => void;
  onAsrLanguageChange: (value: string) => void;
  onCloudProviderChange: (value: string) => void;
  onCloudGroupIdChange: (value: string) => void;
  onCloudBaseUrlChange: (value: string) => void;
  onCloudModelChange: (value: string) => void;
  onCloudLanguageChange: (value: string) => void;
  onCloudBaiduCuidChange: (value: string) => void;
  onCloudBaiduFormatChange: (value: string) => void;
  onCloudBaiduSampleRateChange: (value: string) => void;
  onCloudApiKeyInputChange: (value: string) => void;
  onCloudSecretKeyInputChange: (value: string) => void;
  onInstallManagedAsr: () => void;
  onSaveAsrConfig: () => void;
  onRefreshAsrConfig: () => void;
  onSaveCloudAsrConfig: () => void;
  onSaveMinimaxApiKey: () => void;
  onSaveBaiduAsrApiKey: () => void;
  onSaveBaiduAsrSecretKey: () => void;
  onTestCloudAsrConfig: () => void;
}

export function ModelSettingsView(props: ModelSettingsViewProps) {
  const [activeModel, setActiveModel] = useState<'local' | 'cloud'>('local');
  const isBaidu = props.cloudProvider === 'baidu';
  const apiKeyState = props.cloudAsrConfigStatus.apiKeyConfigured ? '已配置' : '未配置';
  const apiKeyDetail = props.cloudAsrConfigStatus.apiKeyPreview ?? '等待环境变量';
  const secretKeyState = props.cloudAsrConfigStatus.secretKeyConfigured ? '已配置' : '未配置';
  const secretKeyDetail = props.cloudAsrConfigStatus.secretKeyPreview ?? '等待环境变量';
  const cloudProviderLabel = isBaidu ? '百度短语音识别' : 'MiniMax';

  return (
    <main className="app-shell model-shell">
      <section className="model-panel" aria-labelledby="model-title">
        <header className="model-header">
          <div><span className="product-mark">VoxType</span><h1 id="model-title">模型选择</h1></div>
          <button className="secondary-button" type="button" onClick={props.onBack}>返回主界面</button>
        </header>
        <div className="model-choice-grid" role="tablist" aria-label="当前转写引擎">
          <button className="model-choice" role="tab" type="button" aria-selected={activeModel === 'local'} data-active={activeModel === 'local'} onClick={() => setActiveModel('local')}><span>本地 whisper.cpp</span><strong>{props.asrConfigStatus.ready ? 'Ready' : '未配置'}</strong></button>
          <button className="model-choice" role="tab" type="button" aria-selected={activeModel === 'cloud'} data-active={activeModel === 'cloud'} onClick={() => setActiveModel('cloud')}><span>云端 API</span><strong>{props.cloudAsrConfigStatus.ready ? `${cloudProviderLabel} Ready` : cloudProviderLabel}</strong></button>
        </div>
        {activeModel === 'local' ? (
          <section className="model-config" role="tabpanel" aria-label="本地 whisper.cpp 配置">
            <label className="field"><span>whisper.cpp 可执行文件</span><input aria-label="whisper.cpp 可执行文件" value={props.whisperBinaryPath} onChange={(event) => props.onWhisperBinaryPathChange(event.target.value)} placeholder="例如 C:\tools\whisper.cpp\whisper-cli.exe" /></label>
            <label className="field"><span>Whisper 模型文件</span><input aria-label="Whisper 模型文件" value={props.whisperModelPath} onChange={(event) => props.onWhisperModelPathChange(event.target.value)} placeholder="例如 C:\models\ggml-small.bin" /></label>
            <label className="field"><span>识别语言</span><input aria-label="识别语言" value={props.asrLanguage} onChange={(event) => props.onAsrLanguageChange(event.target.value)} placeholder="zh" /></label>
            <p className="runtime-message">{props.asrConfigStatus.message}</p>
            <div className="button-row"><button type="button" onClick={props.onInstallManagedAsr} disabled={props.isInstallingAsr}>{props.isInstallingAsr ? '正在安装 whisper.cpp' : '一键安装 whisper.cpp'}</button><button type="button" onClick={props.onRefreshAsrConfig}>检测 ASR 配置</button><button type="button" onClick={props.onSaveAsrConfig}>保存 ASR 配置</button></div>
          </section>
        ) : (
          <section className="model-config cloud-config" role="tabpanel" aria-label={isBaidu ? '百度短语音识别配置' : '云端 API 配置'}>
            <div className="cloud-provider-switch" role="group" aria-label="云端 ASR 服务商">
              <button type="button" data-active={!isBaidu} aria-pressed={!isBaidu} onClick={() => props.onCloudProviderChange('minimax')}>MiniMax</button>
              <button type="button" data-active={isBaidu} aria-pressed={isBaidu} onClick={() => props.onCloudProviderChange('baidu')}>百度短语音识别</button>
            </div>
            {isBaidu ? (
              <>
                <div className="cloud-key-status"><span>百度 ASR API Key</span><strong>{apiKeyState}</strong><code>BAIDU_ASR_API_KEY</code><small>{apiKeyDetail}</small></div>
                <label className="field"><span>百度 ASR API Key</span><input aria-label="百度 ASR API Key 输入" type="password" autoComplete="off" spellCheck={false} value={props.cloudApiKeyInput} onChange={(event) => props.onCloudApiKeyInputChange(event.target.value)} placeholder="粘贴后保存到用户环境变量" /></label>
                <div className="button-row"><button type="button" onClick={props.onSaveBaiduAsrApiKey} disabled={!props.cloudApiKeyInput.trim()}>保存百度 API Key 到系统环境变量</button></div>
                <div className="cloud-key-status"><span>百度 ASR Secret Key</span><strong>{secretKeyState}</strong><code>BAIDU_ASR_SECRET_KEY</code><small>{secretKeyDetail}</small></div>
                <label className="field"><span>百度 ASR Secret Key</span><input aria-label="百度 ASR Secret Key 输入" type="password" autoComplete="off" spellCheck={false} value={props.cloudSecretKeyInput} onChange={(event) => props.onCloudSecretKeyInputChange(event.target.value)} placeholder="粘贴后保存到用户环境变量" /></label>
                <div className="button-row"><button type="button" onClick={props.onSaveBaiduAsrSecretKey} disabled={!props.cloudSecretKeyInput.trim()}>保存百度 Secret Key 到系统环境变量</button></div>
                <label className="field"><span>百度 ASR Endpoint</span><input aria-label="百度 ASR Endpoint" value={props.cloudBaseUrl} onChange={(event) => props.onCloudBaseUrlChange(event.target.value)} /></label>
                <label className="field"><span>百度 ASR dev_pid</span><input aria-label="百度 ASR dev_pid" value={props.cloudModel} onChange={(event) => props.onCloudModelChange(event.target.value)} /></label>
                <label className="field"><span>百度 ASR cuid</span><input aria-label="百度 ASR cuid" value={props.cloudBaiduCuid} onChange={(event) => props.onCloudBaiduCuidChange(event.target.value)} /></label>
                <label className="field"><span>百度 ASR 音频格式</span><input aria-label="百度 ASR 音频格式" value={props.cloudBaiduFormat} onChange={(event) => props.onCloudBaiduFormatChange(event.target.value)} /></label>
                <label className="field"><span>百度 ASR 采样率</span><input aria-label="百度 ASR 采样率" type="number" value={props.cloudBaiduSampleRate} onChange={(event) => props.onCloudBaiduSampleRateChange(event.target.value)} /></label>
                <label className="field"><span>识别语言</span><input aria-label="识别语言" value={props.cloudLanguage} onChange={(event) => props.onCloudLanguageChange(event.target.value)} /></label>
                <p className="runtime-message">{props.cloudMessage ?? props.cloudAsrConfigStatus.message}</p>
                <div className="button-row"><button type="button" onClick={props.onSaveCloudAsrConfig}>保存百度配置</button><button type="button" onClick={props.onTestCloudAsrConfig}>检测百度配置</button></div>
              </>
            ) : (
              <>
            <div className="cloud-key-status"><span>MiniMax API Key</span><strong>{apiKeyState}</strong><code>MINIMAX_API_KEY</code><small>{apiKeyDetail}</small></div>
            <label className="field"><span>MiniMax API Key</span><input aria-label="MiniMax API Key 输入" type="password" autoComplete="off" spellCheck={false} value={props.cloudApiKeyInput} onChange={(event) => props.onCloudApiKeyInputChange(event.target.value)} placeholder="粘贴后保存到用户环境变量" /></label>
            <div className="button-row"><button type="button" onClick={props.onSaveMinimaxApiKey} disabled={!props.cloudApiKeyInput.trim()}>保存 API Key 到系统环境变量</button></div>
            <label className="field"><span>MiniMax Group ID</span><input aria-label="MiniMax Group ID" value={props.cloudGroupId} onChange={(event) => props.onCloudGroupIdChange(event.target.value)} /></label>
            <label className="field"><span>MiniMax Base URL</span><input aria-label="MiniMax Base URL" value={props.cloudBaseUrl} onChange={(event) => props.onCloudBaseUrlChange(event.target.value)} /></label>
            <label className="field"><span>MiniMax 模型</span><input aria-label="MiniMax 模型" value={props.cloudModel} onChange={(event) => props.onCloudModelChange(event.target.value)} /></label>
            <label className="field"><span>识别语言</span><input aria-label="识别语言" value={props.cloudLanguage} onChange={(event) => props.onCloudLanguageChange(event.target.value)} /></label>
            <p className="runtime-message">{props.cloudMessage ?? props.cloudAsrConfigStatus.message}</p>
            <div className="button-row"><button type="button" onClick={props.onSaveCloudAsrConfig}>保存 MiniMax 配置</button><button type="button" onClick={props.onTestCloudAsrConfig}>检测 MiniMax 配置</button></div>
              </>
            )}
          </section>
        )}
      </section>
    </main>
  );
}
