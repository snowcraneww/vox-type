# VoxType V2 Daily Dictation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build V2 daily dictation UX: a quieter main UI, colorful voice overlay, global push-to-talk path, Simplified Chinese output preference, and persisted input-device choice.

**Architecture:** Implement V2 incrementally. First isolate the visual overlay and UI state model in React, then wire it to existing recording/transcription state, then add Rust hotkey/dictation orchestration and config persistence. Keep diagnostics as the fallback control surface.

**Tech Stack:** React 19, TypeScript, Vitest, Tauri 2, Rust, cpal, whisper.cpp CLI.

---

### Task 1: Document V2 and register harness feature

**Files:**
- Create: `docs/superpowers/specs/2026-05-27-v2-daily-dictation-design.md`
- Modify: `docs/harness/feature_list.json`
- Modify: `docs/harness/progress.md`
- Modify: `docs/harness/session-handoff.md`

- [ ] Write the design spec in Chinese.
- [ ] Add `v2-001` as the single `in_progress` feature.
- [ ] Record that V2 starts from daily dictation UX, not TSF.
- [ ] Run `python -m json.tool docs/harness/feature_list.json`.

### Task 2: Add voice overlay presentation model

**Files:**
- Create: `src/voiceOverlayModel.ts`
- Create: `src/voiceOverlayModel.test.ts`

- [ ] Write failing tests for phase label, transcript preview, and diagnostic hint.
- [ ] Run `npm test -- --run src/voiceOverlayModel.test.ts` and verify RED.
- [ ] Implement `createVoiceOverlayModel`.
- [ ] Re-run the test and verify GREEN.

### Task 3: Add colorful voice overlay component

**Files:**
- Create: `src/VoiceOverlay.tsx`
- Create: `src/VoiceOverlay.test.tsx`
- Modify: `src/styles.css`

- [ ] Write failing tests that render recording, transcribing, success, and failed states.
- [ ] Run `npm test -- --run src/VoiceOverlay.test.tsx` and verify RED.
- [ ] Implement `VoiceOverlay` with a lightweight animated waveform surface.
- [ ] Add CSS for the iridescent wave, reduced-motion fallback, and compact layout.
- [ ] Re-run overlay tests and verify GREEN.

### Task 4: Replace overdone shell styling in main view

**Files:**
- Modify: `src/App.tsx`
- Modify: `src/App.test.tsx`
- Modify: `src/styles.css`

- [ ] Update tests to assert there are no decorative traffic-light controls.
- [ ] Run `npm test -- --run src/App.test.tsx` and verify RED.
- [ ] Replace `mac-window`/`traffic-lights` with quiet tool-window layout.
- [ ] Embed `VoiceOverlay` in the main user view as the V2 visual state surface.
- [ ] Re-run App tests and verify GREEN.

### Task 5: Add Simplified Chinese output preference

**Files:**
- Modify: `src-tauri/src/asr/mod.rs`
- Add or modify Rust tests in the same module.
- Modify docs if command behavior changes.

- [ ] Write a failing Rust test that verifies whisper.cpp arguments include a Simplified Chinese prompt when language starts with `zh`.
- [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml asr::tests::<test_name>` and verify RED.
- [ ] Add prompt argument or equivalent whisper.cpp option.
- [ ] Re-run Rust test and verify GREEN.

### Task 6: Persist selected input device

**Files:**
- Modify: `src-tauri/src/asr_config.rs` or create a focused local config module if broader app config is needed.
- Modify: `src-tauri/src/recorder.rs`
- Modify: `src/App.tsx`
- Modify: `src/tauriClient.ts`
- Modify: `src/types.ts`

- [ ] Write Rust tests for saving/loading selected device name.
- [ ] Verify RED.
- [ ] Implement persistence.
- [ ] Wire frontend initialization to use the saved device.
- [ ] Verify GREEN with Rust and frontend tests.

### Task 7: Global push-to-talk spike behind a narrow command boundary

**Files:**
- Modify or create: `src-tauri/src/hotkey.rs`
- Modify or create: `src-tauri/src/dictation.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify frontend status handling if events are added.

- [ ] Write unit tests for press/release state transitions without registering real OS hotkeys.
- [ ] Implement pure state machine first.
- [ ] Add Tauri command/event boundary for manual triggering.
- [ ] Only after pure tests pass, wire actual global shortcut registration.
- [ ] Manually verify on Windows and record result in `docs/harness/progress.md`.

### Task 8: Full verification and docs sync

**Files:**
- Modify: `README.md`
- Modify: `docs/guide/run-and-understand.md`
- Modify: `docs/guide/code-walkthrough.md`
- Modify: `docs/harness/progress.md`
- Modify: `docs/harness/session-handoff.md`

- [ ] Run `npm test -- --run`.
- [ ] Run `npm run typecheck`.
- [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml`.
- [ ] Run `python -m json.tool docs/harness/feature_list.json`.
- [ ] Run `bash init.sh`.
- [ ] Run `git diff --check`.
- [ ] Update docs with what actually shipped and what remains.