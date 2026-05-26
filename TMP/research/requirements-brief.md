# VoxType Requirements Brief

Last updated: 2026-05-26

Source: `docs/plans/原始需求.md`

## Product Intent

VoxType is intended to be an open source, privacy-friendly desktop voice input tool. The first target platform is Windows. The core user experience is push-to-talk dictation: hold a hotkey, speak, release the hotkey, transcribe locally, and insert the recognized text into the currently focused app.

The original draft proposes Rust + Tauri + React + whisper.cpp. Treat that as a candidate direction, not a final architecture decision.

## Known Facts

- The project is open source.
- The repository license is Apache-2.0, even though the original generated draft mentions MIT.
- Windows is the first-class platform.
- Offline/local-first speech recognition is a product principle, not just an implementation detail.
- The MVP centers on short dictation into arbitrary applications, not meeting transcription or long-form editing.
- The desired interaction is "press and hold to record, release to insert".

## Primary User Scenario

A user is writing in another application such as an editor, chat app, browser, or document tool. They press and hold a configured global hotkey, speak a short phrase or paragraph, release the key, and the recognized text appears at the current caret position without manually copying text from the VoxType UI.

## MVP Behavior

- App runs in the tray or background after startup.
- User can configure or at least learn the global push-to-talk hotkey.
- Pressing the hotkey starts microphone recording.
- Releasing the hotkey stops recording and starts transcription.
- The app gives visible or audible state feedback for idle, recording, transcribing, success, and failure.
- The app inserts text into the previously focused app.
- Audio stays local by default.
- The app fails transparently when microphone permission, model files, hotkey registration, or text insertion fail.

## Non-Goals For MVP

- Full Windows TSF IME integration as the first implementation.
- Cross-platform parity on day one.
- Meeting recording, diarization, summaries, knowledge base, or agent workflows.
- Cloud-only ASR.
- Large plugin systems or account features.
- Shipping copied code from researched repositories.

## Technical Assumptions To Validate

- Tauri 2 can provide a small enough desktop shell for settings, tray, and overlay.
- Rust `cpal` can handle Windows microphone capture reliably enough for MVP.
- A local ASR engine can produce acceptable Chinese and English dictation latency on common Windows hardware.
- Clipboard paste is an acceptable first insertion path if it restores the original clipboard and has clear fallback behavior.
- Windows `SendInput(KEYEVENTF_UNICODE)` can become a more reliable second-phase insertion path before full TSF work.

## Risks

- Text insertion reliability varies by target application, privilege level, focus state, IME state, and clipboard behavior.
- Local ASR model choice affects binary size, download UX, latency, accuracy, GPU/CPU support, and license obligations.
- Overlay windows can steal focus and break insertion if implemented carelessly.
- Global hotkeys can conflict with existing apps and need debounce handling.
- Full TSF integration is complex and should not block the first learning loop.
- GPL/AGPL projects can inform architecture but cannot be copied into an Apache-2.0 project without creating license conflicts.

## Open Questions For Maintainer

- Should the first MVP be Chinese-first, English-first, or bilingual from the start?
- Is offline-only mandatory, or can optional user-configured cloud ASR be a later feature?
- Should the initial text insertion path prioritize reliability or avoiding clipboard use?
- Is a tray-only utility acceptable for MVP, or does the first release need a full settings UI?
- What hardware baseline should define acceptable latency?

