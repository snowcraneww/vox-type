import { useEffect, useMemo, useState } from 'react';
import { isTauriRuntime, listenToPushToTalk, type PushToTalkPayload } from './tauriClient';

interface DictationOverlayProps {
  initialPayload?: PushToTalkPayload | null;
}

function overlayMode(payload: PushToTalkPayload | null) {
  if (!payload) {
    return 'idle';
  }
  if (payload.action === 'startRecording' || payload.action === 'toggleStartRecording') {
    return 'recording';
  }
  if (payload.action === 'stopAndTranscribe' || payload.action === 'toggleStopAndTranscribe') {
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
      if (nextPayload.action === 'ignore') {
        return;
      }
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
    <main className="overlay-root" data-mode={mode} data-theme="light-green">
      <section className="wave-ripple" role="status" aria-label={overlayLabel(mode)} data-mode={mode} data-theme="light-green">
        <svg className="wave-ripple-svg" aria-hidden="true" viewBox="0 0 120 32" preserveAspectRatio="none">
          <defs>
            <linearGradient id="voice-ripple-gradient" gradientUnits="userSpaceOnUse" x1="-120" x2="120" y1="0" y2="0">
              <stop offset="0%" stopColor="#2c7a3f" />
              <stop offset="18%" stopColor="#4fb866" />
              <stop offset="34%" stopColor="#7bdc8b" />
              <stop offset="52%" stopColor="#c7efbd" />
              <stop offset="70%" stopColor="#68b876" />
              <stop offset="86%" stopColor="#236b38" />
              <stop offset="100%" stopColor="#dff5d8" />
              {isRecording ? <animate attributeName="x1" dur="920ms" repeatCount="indefinite" values="-120;0;-120" /> : null}
              {isRecording ? <animate attributeName="x2" dur="920ms" repeatCount="indefinite" values="120;240;120" /> : null}
            </linearGradient>
            <filter id="voice-ripple-glow" x="-8%" y="-55%" width="116%" height="210%">
              <feGaussianBlur stdDeviation="1.25" result="blur" />
              <feColorMatrix in="blur" type="matrix" values="1 0 0 0 0  0 1 0 0 0  0 0 1 0 0  0 0 0 0.72 0" result="glow" />
              <feMerge>
                <feMergeNode in="glow" />
                <feMergeNode in="SourceGraphic" />
              </feMerge>
            </filter>
            <clipPath id="voice-capsule-clip">
              <rect x="0.5" y="0.5" width="119" height="31" rx="15.5" />
            </clipPath>
          </defs>
          <rect className="ripple-capsule" x="0.5" y="0.5" width="119" height="31" rx="15.5" />
          <g className="overlay-contents" clipPath="url(#voice-capsule-clip)">
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
            <g className="transcribing-dots" aria-hidden="true" filter="url(#voice-ripple-glow)">
              {[40, 48, 56, 64, 72, 80].map((cx, index) => (
                <circle className={`transcribing-dot transcribing-dot-${index}`} cx={cx} cy="16" data-index={index} key={cx} r="2.15">
                  <animate attributeName="r" begin={`${index * 0.10}s`} dur="1.25s" repeatCount="indefinite" values="1.85;2.75;2.1;1.85" />
                  <animate attributeName="opacity" begin={`${index * 0.10}s`} dur="1.25s" repeatCount="indefinite" values="0.32;1;0.58;0.32" />
                </circle>
              ))}
            </g>
          ) : null}
          </g>
        </svg>
      </section>
    </main>
  );
}
