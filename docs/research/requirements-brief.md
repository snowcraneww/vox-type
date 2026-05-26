# VoxType Requirements Brief

Last updated: 2026-05-26

This brief converts the original AI-generated draft in `docs/plans/原始需求.md` into project requirements. The draft is input, not authority. The repository license is Apache-2.0.

## Goal

VoxType is an open source, Windows-first, local-first voice input tool. The first useful behavior is push-to-talk dictation into the current app: hold a global hotkey, speak, release, transcribe locally, and insert the text at the caret.

## MVP Scope

- Background/tray desktop app.
- Configurable or documented global push-to-talk hotkey.
- Microphone recording while the hotkey is held.
- Local speech-to-text after release.
- Text insertion into the previously focused application.
- Clear state feedback for idle, recording, transcribing, success, and failure.
- Privacy-first defaults: no audio leaves the machine unless a later optional cloud provider is explicitly enabled.

## Deferred

- Full TSF IME integration.
- Meeting transcription, diarization, summaries, notes, and agents.
- Cross-platform parity.
- Cloud-first recognition.
- Plugin systems or accounts.

## Main Risks

- Text insertion reliability across Windows apps.
- ASR latency, accuracy, model download size, and hardware support.
- Hotkey conflicts and debounce behavior.
- Overlay windows stealing focus.
- License contamination from GPL/AGPL reference projects.

## Maintainer Decisions Still Needed

- Chinese-first, English-first, or bilingual MVP.
- Offline-only versus optional cloud provider in later phases.
- Acceptable first insertion method: clipboard paste or direct Unicode input.
- Minimum hardware target and acceptable release-to-text latency.

