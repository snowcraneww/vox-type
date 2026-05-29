import { useState } from 'react';
import type { AsrConfigStatus } from './types';

interface ModelSettingsViewProps {
  asrConfigStatus: AsrConfigStatus;
  whisperBinaryPath: string;
  whisperModelPath: string;
  asrLanguage: string;
  isInstallingAsr: boolean;
  onBack: () => void;
  onWhisperBinaryPathChange: (value: string) => void;
  onWhisperModelPathChange: (value: string) => void;
  onAsrLanguageChange: (value: string) => void;
  onInstallManagedAsr: () => void;
  onSaveAsrConfig: () => void;
  onRefreshAsrConfig: () => void;
}

export function ModelSettingsView(props: ModelSettingsViewProps) {
  const [activeModel, setActiveModel] = useState<'local' | 'cloud'>('local');

  return (
    <main className="app-shell model-shell">
      <section className="model-panel" aria-labelledby="model-title">
        <header className="model-header">
          <div><span className="product-mark">VoxType</span><h1 id="model-title">模型选择</h1></div>
          <button className="secondary-button" type="button" onClick={props.onBack}>返回主界面</button>
        </header>
        <div className="model-choice-grid" role="tablist" aria-label="当前转写引擎">
          <button className="model-choice" role="tab" type="button" aria-selected={activeModel === 'local'} data-active={activeModel === 'local'} onClick={() => setActiveModel('local')}><span>本地 whisper.cpp</span><strong>{props.asrConfigStatus.ready ? 'Ready' : '未配置'}</strong></button>
          <button className="model-choice" role="tab" type="button" aria-selected={activeModel === 'cloud'} data-active={activeModel === 'cloud'} onClick={() => setActiveModel('cloud')}><span>云端 API</span><strong>下一版接入</strong></button>
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
          <section className="model-config cloud-config" role="tabpanel" aria-label="云端 API 配置">
            <div className="cloud-placeholder"><span>下一版接入</span><strong>API Key、模型名和服务商选择会放在这里。</strong><p>当前版本先保持本地 whisper.cpp 为默认引擎，避免把未完成的云端配置混进日常输入流程。</p></div>
          </section>
        )}
      </section>
    </main>
  );
}
