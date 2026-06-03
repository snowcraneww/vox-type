import type { TranscriptPostprocessConfig } from './types';

const text = {
  title: '\u6587\u672c\u4f18\u5316',
  back: '\u8fd4\u56de\u4e3b\u754c\u9762',
  eyebrow: '\u8f6c\u5199\u540e\u5904\u7406',
  detail: '\u8ba9\u8bed\u97f3\u8bc6\u522b\u7ed3\u679c\u5728\u4e0a\u5c4f\u548c\u5165\u5e93\u524d\u5148\u505a\u786e\u5b9a\u6027\u6e05\u7406',
  enablePostprocess: '\u542f\u7528\u6587\u672c\u4f18\u5316',
  enableHint: '\u5bf9\u65b0\u8bc6\u522b\u6587\u672c\u5e94\u7528\u4e0b\u65b9\u89c4\u5219',
  cleanupNoise: '\u8fc7\u6ee4\u5b57\u5e55\u7c7b\u566a\u58f0',
  cleanupHint: '\u79fb\u9664\u5e38\u89c1\u975e\u8bed\u97f3\u5185\u5bb9\u7247\u6bb5',
  replacements: '\u66ff\u6362\u89c4\u5219',
  replacementsHint: '\u6bcf\u884c\u4e00\u6761\uff0c\u683c\u5f0f\uff1a\u539f\u6587 => \u66ff\u6362\u6587\u672c',
  glossary: '\u4e13\u6709\u8bcd\u8868',
  glossaryHint: '\u6bcf\u884c\u4e00\u4e2a\u8bcd\uff0c\u7528\u4e8e\u4fee\u6b63\u5e38\u89c1\u5927\u5c0f\u5199\u548c\u5199\u6cd5',
  preview: '\u6548\u679c\u9884\u89c8',
  previewInput: '\u9884\u89c8\u8f93\u5165',
  previewOutput: '\u9884\u89c8\u7ed3\u679c',
  saveTextOptimization: '\u4fdd\u5b58\u6587\u672c\u4f18\u5316',
  previewTextOptimization: '\u4fdd\u5b58\u5e76\u9884\u89c8',
  replacementPlaceholder: 'scale => skill',
  glossaryPlaceholder: 'WebSocket\nwhisper.cpp\nVoxType',
};

interface TextOptimizationViewProps {
  transcriptPostprocessConfig: TranscriptPostprocessConfig;
  postprocessReplacementText: string;
  postprocessGlossaryText: string;
  postprocessPreviewInput: string;
  postprocessPreviewOutput: string | null;
  postprocessMessage: string | null;
  onBack: () => void;
  onTranscriptPostprocessEnabledChange: (value: boolean) => void;
  onTranscriptPostprocessCleanupNoiseChange: (value: boolean) => void;
  onPostprocessReplacementTextChange: (value: string) => void;
  onPostprocessGlossaryTextChange: (value: string) => void;
  onPostprocessPreviewInputChange: (value: string) => void;
  onSaveTranscriptPostprocessConfig: () => void;
  onPreviewTranscriptPostprocess: () => void;
}

function ToggleCard({ checked, title, hint, onChange }: { checked: boolean; title: string; hint: string; onChange: (value: boolean) => void }) {
  return (
    <label className="optimization-toggle-card">
      <span><strong>{title}</strong><small>{hint}</small></span>
      <input type="checkbox" checked={checked} onChange={(event) => onChange(event.target.checked)} />
    </label>
  );
}

export function TextOptimizationView(props: TextOptimizationViewProps) {
  return (
    <main className="app-shell model-shell optimization-shell">
      <section className="model-panel optimization-panel" aria-labelledby="optimization-title">
        <header className="model-header optimization-header">
          <div className="model-title-line"><span className="product-mark">VoxType</span><h1 id="optimization-title">{text.title}</h1></div>
          <button className="secondary-button" type="button" onClick={props.onBack}>{text.back}</button>
        </header>
        <section className="optimization-hero" aria-label={text.eyebrow}>
          <span>{text.eyebrow}</span>
          <p>{text.detail}</p>
        </section>
        <section className="optimization-card optimization-toggles" aria-label={text.title}>
          <ToggleCard checked={props.transcriptPostprocessConfig.enabled} title={text.enablePostprocess} hint={text.enableHint} onChange={props.onTranscriptPostprocessEnabledChange} />
          <ToggleCard checked={props.transcriptPostprocessConfig.cleanupNoise} title={text.cleanupNoise} hint={text.cleanupHint} onChange={props.onTranscriptPostprocessCleanupNoiseChange} />
        </section>
        <section className="optimization-grid">
          <label className="field optimization-card"><span>{text.replacements}</span><small>{text.replacementsHint}</small><textarea aria-label={text.replacements} value={props.postprocessReplacementText} onChange={(event) => props.onPostprocessReplacementTextChange(event.target.value)} placeholder={text.replacementPlaceholder} /></label>
          <label className="field optimization-card"><span>{text.glossary}</span><small>{text.glossaryHint}</small><textarea aria-label={text.glossary} value={props.postprocessGlossaryText} onChange={(event) => props.onPostprocessGlossaryTextChange(event.target.value)} placeholder={text.glossaryPlaceholder} /></label>
        </section>
        <section className="optimization-card preview-card" aria-label={text.preview}>
          <div className="section-heading model-config-heading"><span>{text.preview}</span><strong>{text.previewOutput}</strong></div>
          <label className="field"><span>{text.previewInput}</span><input aria-label={text.previewInput} value={props.postprocessPreviewInput} onChange={(event) => props.onPostprocessPreviewInputChange(event.target.value)} /></label>
          <output className="postprocess-output" aria-label={text.previewOutput}>{props.postprocessPreviewOutput ?? text.previewOutput}</output>
        </section>
        {props.postprocessMessage ? <p className="runtime-message optimization-message">{props.postprocessMessage}</p> : null}
        <div className="button-row optimization-actions"><button type="button" onClick={props.onSaveTranscriptPostprocessConfig}>{text.saveTextOptimization}</button><button type="button" onClick={props.onPreviewTranscriptPostprocess}>{text.previewTextOptimization}</button></div>
      </section>
    </main>
  );
}
