# Technical Options

Last updated: 2026-05-26

## Recommended Direction

Use Rust + Tauri 2 + React/TypeScript for the main project. Keep Rust responsible for hotkeys, audio, ASR integration, text insertion, config, and process state. Use React for settings and status UI.

This matches the original draft while incorporating lessons from Handy, OpenLess, and faster-whisper-dictation.

## MVP Pipeline

```mermaid
flowchart LR
    A[Hold global hotkey] --> B[Record microphone]
    B --> C[Normalize audio]
    C --> D[Local ASR]
    D --> E[Post-process text lightly]
    E --> F[Insert into active app]
    F --> G[Show success or failure]
```

## Component Choices

| Component | MVP choice | Later option | Rationale |
| --- | --- | --- | --- |
| Desktop shell | Tauri 2 + React/TS | Same | Small app footprint with native Rust escape hatches. |
| Audio capture | Rust `cpal` | Add device management and watchdog | Proven in comparable Rust/Tauri dictation apps. |
| Audio format | 16 kHz mono PCM | Streaming chunks | Common ASR boundary and easy to test. |
| VAD | Minimum duration + silence trim | Silero/ONNX VAD with pre-speech buffer | Avoid blocking the first loop on model complexity. |
| ASR | Evaluate whisper.cpp/transcribe-rs and sherpa-onnx | Multiple engines through an adapter | Avoid locking into `whisper-cli` as a permanent boundary too early. |
| Text insertion | Clipboard paste with restoration | Unicode `SendInput`, then TSF | Fastest MVP with an explicit reliability path. |
| UI | Tray + settings + non-focus-stealing status | Onboarding and model manager | The first app should stay utility-focused. |

## Options Considered

- Rust/Tauri native MVP: recommended.
- Python/faster-whisper prototype: useful for throwaway ASR experiments, not the main app.
- Electron app: good for broad UI products, too heavy for a lightweight input utility.
- TSF IME first: technically attractive but too slow and risky for the first release.

## Next Brainstorming Topic

Before scaffolding product code, decide the first ASR engine experiment and the first insertion path. The most pragmatic default is clipboard paste for MVP insertion and a small engine adapter that can compare whisper.cpp-style and sherpa-onnx-style backends.

