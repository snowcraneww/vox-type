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
        <svg className="wave-ripple-svg" aria-hidden="true" viewBox="0 0 176 46" preserveAspectRatio="none">
          <defs>
            <linearGradient id="voice-ripple-gradient" gradientUnits="userSpaceOnUse" x1="-176" x2="176" y1="0" y2="0">
              <stop offset="0%" stopColor="#5ac8fa" />
              <stop offset="18%" stopColor="#7ee7ff" />
              <stop offset="34%" stopColor="#34c759" />
              <stop offset="52%" stopColor="#ffd60a" />
              <stop offset="70%" stopColor="#ff2d92" />
              <stop offset="86%" stopColor="#bf5af2" />
              <stop offset="100%" stopColor="#ffd60a" />
              {isRecording ? <animate attributeName="x1" dur="1.15s" repeatCount="indefinite" values="-176;0;-176" /> : null}
              {isRecording ? <animate attributeName="x2" dur="1.15s" repeatCount="indefinite" values="176;352;176" /> : null}
            </linearGradient>
            <radialGradient id="voice-capsule-shine" cx="50%" cy="10%" r="85%">
              <stop offset="0%" stopColor="rgba(255,255,255,0.18)" />
              <stop offset="48%" stopColor="rgba(255,255,255,0.045)" />
              <stop offset="100%" stopColor="rgba(255,255,255,0)" />
            </radialGradient>
            <filter id="voice-ripple-glow" x="-16%" y="-90%" width="132%" height="280%">
              <feGaussianBlur stdDeviation="2.1" result="blur" />
              <feColorMatrix in="blur" type="matrix" values="1 0 0 0 0  0 1 0 0 0  0 0 1 0 0  0 0 0 0.72 0" result="glow" />
              <feMerge>
                <feMergeNode in="glow" />
                <feMergeNode in="SourceGraphic" />
              </feMerge>
            </filter>
          </defs>
          <rect className="ripple-capsule" x="0" y="0" width="176" height="46" rx="23" />
          <rect className="ripple-shine" x="3" y="3" width="170" height="40" rx="20" />
          <g filter="url(#voice-ripple-glow)">
            <path className="ripple-line ripple-line-a" pathLength="100" d="M -8 23 C 0 12 9 34 18 23 S 36 12 45 23 S 63 34 72 23 S 90 12 99 23 S 117 34 126 23 S 144 12 153 23 S 171 34 184 23">
              {isRecording ? <animate attributeName="d" dur="820ms" repeatCount="indefinite" values="M -8 23 C 0 12 9 34 18 23 S 36 12 45 23 S 63 34 72 23 S 90 12 99 23 S 117 34 126 23 S 144 12 153 23 S 171 34 184 23; M -8 23 C 0 33 9 13 18 23 S 36 34 45 23 S 63 12 72 23 S 90 34 99 23 S 117 12 126 23 S 144 34 153 23 S 171 12 184 23; M -8 23 C 0 12 9 34 18 23 S 36 12 45 23 S 63 34 72 23 S 90 12 99 23 S 117 34 126 23 S 144 12 153 23 S 171 34 184 23" /> : null}
            </path>
            <path className="ripple-line ripple-line-b" pathLength="100" d="M -8 24 C 0 17 9 31 18 24 S 36 17 45 24 S 63 31 72 24 S 90 17 99 24 S 117 31 126 24 S 144 17 153 24 S 171 31 184 24">
              {isRecording ? <animate attributeName="d" dur="980ms" repeatCount="indefinite" values="M -8 24 C 0 17 9 31 18 24 S 36 17 45 24 S 63 31 72 24 S 90 17 99 24 S 117 31 126 24 S 144 17 153 24 S 171 31 184 24; M -8 22 C 0 30 9 16 18 22 S 36 30 45 22 S 63 16 72 22 S 90 30 99 22 S 117 16 126 22 S 144 30 153 22 S 171 16 184 22; M -8 24 C 0 17 9 31 18 24 S 36 17 45 24 S 63 31 72 24 S 90 17 99 24 S 117 31 126 24 S 144 17 153 24 S 171 31 184 24" /> : null}
            </path>
            <path className="ripple-line ripple-line-c" pathLength="100" d="M -8 22 C 0 19 9 27 18 22 S 36 19 45 22 S 63 27 72 22 S 90 19 99 22 S 117 27 126 22 S 144 19 153 22 S 171 27 184 22">
              {isRecording ? <animate attributeName="d" dur="1.12s" repeatCount="indefinite" values="M -8 22 C 0 19 9 27 18 22 S 36 19 45 22 S 63 27 72 22 S 90 19 99 22 S 117 27 126 22 S 144 19 153 22 S 171 27 184 22; M -8 25 C 0 28 9 18 18 25 S 36 28 45 25 S 63 18 72 25 S 90 28 99 25 S 117 18 126 25 S 144 28 153 25 S 171 18 184 25; M -8 22 C 0 19 9 27 18 22 S 36 19 45 22 S 63 27 72 22 S 90 19 99 22 S 117 27 126 22 S 144 19 153 22 S 171 27 184 22" /> : null}
            </path>
            <path className="ripple-line ripple-line-d" pathLength="100" d="M -8 23 C 0 15 9 32 18 23 S 36 15 45 23 S 63 32 72 23 S 90 15 99 23 S 117 32 126 23 S 144 15 153 23 S 171 32 184 23">
              {isRecording ? <animate attributeName="d" dur="1.28s" repeatCount="indefinite" values="M -8 23 C 0 15 9 32 18 23 S 36 15 45 23 S 63 32 72 23 S 90 15 99 23 S 117 32 126 23 S 144 15 153 23 S 171 32 184 23; M -8 23 C 0 32 9 15 18 23 S 36 32 45 23 S 63 15 72 23 S 90 32 99 23 S 117 15 126 23 S 144 32 153 23 S 171 15 184 23; M -8 23 C 0 15 9 32 18 23 S 36 15 45 23 S 63 32 72 23 S 90 15 99 23 S 117 32 126 23 S 144 15 153 23 S 171 32 184 23" /> : null}
            </path>
          </g>
        </svg>
      </section>
    </main>
  );
}
