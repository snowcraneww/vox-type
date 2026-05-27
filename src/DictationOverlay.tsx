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
        <svg className="wave-ripple-svg" aria-hidden="true" viewBox="0 0 300 82" preserveAspectRatio="none">
          <defs>
            <linearGradient id="voice-ripple-gradient" x1="0" x2="1" y1="0" y2="0">
              <stop offset="0%" stopColor="#5ac8fa" />
              <stop offset="32%" stopColor="#7ee7ff" />
              <stop offset="57%" stopColor="#ff2d92" />
              <stop offset="78%" stopColor="#bf5af2" />
              <stop offset="100%" stopColor="#ffd60a" />
            </linearGradient>
            <filter id="voice-ripple-glow" x="-20%" y="-80%" width="140%" height="260%">
              <feGaussianBlur stdDeviation="3.2" result="blur" />
              <feColorMatrix in="blur" type="matrix" values="1 0 0 0 0  0 1 0 0 0  0 0 1 0 0  0 0 0 0.72 0" result="glow" />
              <feMerge>
                <feMergeNode in="glow" />
                <feMergeNode in="SourceGraphic" />
              </feMerge>
            </filter>
          </defs>
          <g filter="url(#voice-ripple-glow)">
            <path className="ripple-line ripple-line-a" pathLength="100" d="M -46 43 C -8 18 24 64 62 41 S 129 16 164 42 S 226 66 265 40 S 330 19 354 43">
              <animate attributeName="d" dur="1.55s" repeatCount="indefinite" values="M -46 43 C -8 18 24 64 62 41 S 129 16 164 42 S 226 66 265 40 S 330 19 354 43; M -46 39 C -4 62 25 22 63 43 S 130 66 166 39 S 229 19 266 43 S 327 64 354 39; M -46 43 C -8 18 24 64 62 41 S 129 16 164 42 S 226 66 265 40 S 330 19 354 43" />
            </path>
            <path className="ripple-line ripple-line-b" pathLength="100" d="M -54 39 C -13 56 20 26 58 39 S 124 56 162 39 S 229 21 268 40 S 326 58 356 37">
              <animate attributeName="d" dur="1.9s" repeatCount="indefinite" values="M -54 39 C -13 56 20 26 58 39 S 124 56 162 39 S 229 21 268 40 S 326 58 356 37; M -54 44 C -12 24 21 58 59 40 S 126 24 164 43 S 230 60 269 39 S 328 21 356 44; M -54 39 C -13 56 20 26 58 39 S 124 56 162 39 S 229 21 268 40 S 326 58 356 37" />
            </path>
            <path className="ripple-line ripple-line-c" pathLength="100" d="M -50 47 C -10 33 24 51 63 47 S 127 31 166 46 S 229 60 269 45 S 329 32 356 48">
              <animate attributeName="d" dur="2.25s" repeatCount="indefinite" values="M -50 47 C -10 33 24 51 63 47 S 127 31 166 46 S 229 60 269 45 S 329 32 356 48; M -50 42 C -9 54 24 35 64 45 S 128 57 168 43 S 230 30 270 46 S 329 57 356 42; M -50 47 C -10 33 24 51 63 47 S 127 31 166 46 S 229 60 269 45 S 329 32 356 48" />
            </path>
            <path className="ripple-line ripple-line-d" pathLength="100" d="M -44 35 C -6 45 21 16 61 35 S 128 55 166 36 S 231 18 268 36 S 328 54 354 34">
              <animate attributeName="d" dur="1.72s" repeatCount="indefinite" values="M -44 35 C -6 45 21 16 61 35 S 128 55 166 36 S 231 18 268 36 S 328 54 354 34; M -44 37 C -6 20 22 52 62 34 S 130 17 168 37 S 230 54 269 35 S 328 19 354 38; M -44 35 C -6 45 21 16 61 35 S 128 55 166 36 S 231 18 268 36 S 328 54 354 34" />
            </path>
          </g>
        </svg>
      </section>
    </main>
  );
}
