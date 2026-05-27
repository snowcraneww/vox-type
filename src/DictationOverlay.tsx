import { useEffect, useMemo, useState } from 'react';
import { isTauriRuntime, listenToPushToTalk, type PushToTalkPayload } from './tauriClient';

interface DictationOverlayProps {
  initialPayload?: PushToTalkPayload | null;
}

function overlayCopy(payload: PushToTalkPayload | null) {
  if (!payload) {
    return { title: '待命', detail: '按住 Ctrl+Alt+Space 开始说话', mode: 'idle' };
  }
  if (payload.action === 'startRecording') {
    return { title: '正在听', detail: '松开 Ctrl+Alt+Space 自动转写并上屏', mode: 'recording' };
  }
  if (payload.action === 'stopAndTranscribe') {
    return { title: '正在识别', detail: '正在本地转写并准备上屏', mode: 'transcribing' };
  }
  return { title: '待命', detail: '等待下一次语音输入', mode: 'idle' };
}

export function DictationOverlay({ initialPayload = null }: DictationOverlayProps) {
  const [payload, setPayload] = useState<PushToTalkPayload | null>(initialPayload);
  const copy = useMemo(() => overlayCopy(payload), [payload]);

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
    <main className="overlay-root" data-mode={copy.mode}>
      <section className="mini-dictation" role="status" aria-label={`桌面语音输入状态：${copy.title}`} data-mode={copy.mode}>
        <div className="mini-wave" aria-hidden="true">
          {Array.from({ length: 18 }, (_, index) => <span className="mini-wave-bar" style={{ '--bar-index': index } as React.CSSProperties} key={index} />)}
        </div>
        <div className="mini-copy">
          <strong>{copy.title}</strong>
          <span>{copy.detail}</span>
        </div>
      </section>
    </main>
  );
}
