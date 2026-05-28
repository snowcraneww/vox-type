import type { VoiceOverlayModel } from './voiceOverlayModel';

interface VoiceOverlayProps {
  model: VoiceOverlayModel;
}

export function VoiceOverlay({ model }: VoiceOverlayProps) {
  const levelStyle = { '--voice-level': model.level.toFixed(2) } as React.CSSProperties;
  const waveBars = Array.from({ length: 20 }, (_, index) => index);

  return (
    <aside className="voice-overlay" data-mode={model.mode} role="status" aria-label={`语音输入状态：${model.title}`} style={levelStyle}>
      <div className="voice-wave-stage" data-mode={model.mode} data-testid="voice-wave" aria-hidden="true">
        <div className="voice-wave-bars">
          {waveBars.map((bar) => (
            <span className="voice-wave-bar" key={bar} style={{ '--bar-index': bar } as React.CSSProperties} />
          ))}
        </div>
        <span className="voice-wave-ribbon ribbon-a" />
        <span className="voice-wave-ribbon ribbon-b" />
        <span className="voice-glow-core" />
      </div>
      <div className="voice-overlay-copy">
        <strong>{model.title}</strong>
        <span>{model.detail}</span>
        {model.transcriptPreview ? <p>{model.transcriptPreview}</p> : null}
        {model.diagnosticHint ? <em>打开诊断模式查看原因</em> : null}
      </div>
    </aside>
  );
}
