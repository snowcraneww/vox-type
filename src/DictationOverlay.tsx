import { useEffect, useMemo, useState } from 'react';
import { isTauriRuntime, listenToPushToTalk, type PushToTalkPayload } from './tauriClient';

interface DictationOverlayProps {
  initialPayload?: PushToTalkPayload | null;
}

function overlayMode(payload: PushToTalkPayload | null) {
  if (!payload) {
    return 'idle';
  }
  if (payload.action === 'startRecording') {
    return 'recording';
  }
  if (payload.action === 'stopAndTranscribe') {
    return 'transcribing';
  }
  return 'idle';
}

function overlayLabel(mode: string) {
  if (mode === 'recording') {
    return '桌面语音输入状态：正在录音';
  }
  if (mode === 'transcribing') {
    return '桌面语音输入状态：正在识别';
  }
  return '桌面语音输入状态：待命';
}

export function DictationOverlay({ initialPayload = null }: DictationOverlayProps) {
  const [payload, setPayload] = useState<PushToTalkPayload | null>(initialPayload);
  const mode = useMemo(() => overlayMode(payload), [payload]);
  const isRecording = mode === 'recording';
  const isTranscribing = mode === 'transcribing';

  useEffect(() => {
    if (!isTauriRuntime()) {
      return;
    }
    let disposed = false;
    let unlisten: (() => void) | null = null;
    void listenToPushToTalk((nextPayload) => {
      setPayload(nextPayload);
    }).then((nextUnlisten) => {
      if (disposed) {
        nextUnlisten();
        return;
      }
      unlisten = nextUnlisten;
    }).catch(() => {
      // The main window reports listener permission failures. The overlay stays passive.
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  return (
    <main className="overlay-root" data-mode={mode}>
      <section className="wave-ripple" role="status" aria-label={overlayLabel(mode)} data-mode={mode}>
        <svg className="wave-ripple-svg" aria-hidden="true" viewBox="0 0 120 32" preserveAspectRatio="none">
          <defs>
            <linearGradient id="voice-ripple-gradient" gradientUnits="userSpaceOnUse" x1="-120" x2="120" y1="0" y2="0">
              <stop offset="0%" stopColor="#5ac8fa" />
              <stop offset="18%" stopColor="#7ee7ff" />
              <stop offset="34%" stopColor="#34c759" />
              <stop offset="52%" stopColor="#ffd60a" />
              <stop offset="70%" stopColor="#ff2d92" />
              <stop offset="86%" stopColor="#bf5af2" />
              <stop offset="100%" stopColor="#ffd60a" />
              {isRecording ? <animate attributeName="x1" dur="920ms" repeatCount="indefinite" values="-120;0;-120" /> : null}
              {isRecording ? <animate attributeName="x2" dur="920ms" repeatCount="indefinite" values="120;240;120" /> : null}
            </linearGradient>
            <radialGradient id="voice-capsule-shine" cx="50%" cy="18%" r="76%">
              <stop offset="0%" stopColor="rgba(255,255,255,0.055)" />
              <stop offset="58%" stopColor="rgba(255,255,255,0.018)" />
              <stop offset="100%" stopColor="rgba(255,255,255,0)" />
            </radialGradient>
            <filter id="voice-ripple-glow" x="-8%" y="-55%" width="116%" height="210%">
              <feGaussianBlur stdDeviation="1.25" result="blur" />
              <feColorMatrix in="blur" type="matrix" values="1 0 0 0 0  0 1 0 0 0  0 0 1 0 0  0 0 0 0.72 0" result="glow" />
              <feMerge>
                <feMergeNode in="glow" />
                <feMergeNode in="SourceGraphic" />
              </feMerge>
            </filter>
          </defs>
          <rect className="ripple-capsule" x="0" y="0" width="120" height="32" rx="16" />
          <rect className="ripple-shine" x="4" y="3" width="112" height="26" rx="13" />
          <g className="soundwave-bars" filter="url(#voice-ripple-glow)">
            {Array.from({ length: 20 }, (_, index) => {
              const heights = [4, 7, 10, 5, 12, 8, 3, 11, 6, 13, 9, 4, 12, 7, 10, 5, 8, 11, 6, 3];
              const height = heights[index];
              return (
                <rect
                  className="soundwave-bar"
                  data-index={index}
                  key={index}
                  rx="1.4"
                  x={13 + index * 5}
                  y={16 - height / 2}
                  width="2.4"
                  height={height}
                  style={{ '--bar-index': index } as React.CSSProperties}
                />
              );
            })}
          </g>
          {isTranscribing ? (
            <g className="transcribing-dots" aria-hidden="true">
              <circle className="transcribing-dot" cx="50" cy="16" r="2.3" />
              <circle className="transcribing-dot" cx="60" cy="16" r="2.3" />
              <circle className="transcribing-dot" cx="70" cy="16" r="2.3" />
            </g>
          ) : null}
        </svg>
      </section>
    </main>
  );
}
