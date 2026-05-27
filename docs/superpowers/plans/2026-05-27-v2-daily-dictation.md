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

- [x] Write the design spec in Chinese.
- [x] Add `v2-001` as the single `in_progress` feature.
- [x] Record that V2 starts from daily dictation UX, not TSF.
- [x] Run `python -m json.tool docs/harness/feature_list.json`.

### Task 2: Add voice overlay presentation model

**Files:**
- Create: `src/voiceOverlayModel.ts`
- Create: `src/voiceOverlayModel.test.ts`

- [x] Write failing tests for phase label, transcript preview, and diagnostic hint.
- [x] Run `npm test -- --run src/voiceOverlayModel.test.ts` and verify RED.
- [x] Implement `createVoiceOverlayModel`.
- [x] Re-run the test and verify GREEN.

### Task 3: Add colorful voice overlay component

**Files:**
- Create: `src/VoiceOverlay.tsx`
- Create: `src/VoiceOverlay.test.tsx`
- Modify: `src/styles.css`

- [x] Write failing tests that render recording, transcribing, success, and failed states.
- [x] Run `npm test -- --run src/VoiceOverlay.test.tsx` and verify RED.
- [x] Implement `VoiceOverlay` with a lightweight animated waveform surface.
- [x] Add CSS for the iridescent wave, reduced-motion fallback, and compact layout.
- [x] Re-run overlay tests and verify GREEN.

### Task 4: Replace overdone shell styling in main view

**Files:**
- Modify: `src/App.tsx`
- Modify: `src/App.test.tsx`
- Modify: `src/styles.css`

- [x] Update tests to assert there are no decorative traffic-light controls.
- [x] Run `npm test -- --run src/App.test.tsx` and verify RED.
- [x] Replace `mac-window`/`traffic-lights` with quiet tool-window layout.
- [x] Embed `VoiceOverlay` in the main user view as the V2 visual state surface.
- [x] Re-run App tests and verify GREEN.

### Task 5: Add Simplified Chinese output preference

**Files:**
- Modify: `src-tauri/src/asr/mod.rs`
- Add or modify Rust tests in the same module.
- Modify docs if command behavior changes.

- [x] Write a failing Rust test that verifies whisper.cpp arguments include a Simplified Chinese prompt when language starts with `zh`.
- [x] Run `cargo test --manifest-path src-tauri/Cargo.toml asr::tests::<test_name>` and verify RED.
- [x] Add prompt argument or equivalent whisper.cpp option.
- [x] Re-run Rust test and verify GREEN.

### Task 6: Persist selected input device

**Files:**
- Modify: `src-tauri/src/asr_config.rs` or create a focused local config module if broader app config is needed.
- Modify: `src-tauri/src/recorder.rs`
- Modify: `src/App.tsx`
- Modify: `src/tauriClient.ts`
- Modify: `src/types.ts`

- [x] Write Rust tests for saving/loading selected device name.
- [x] Verify RED.
- [x] Implement persistence.
- [x] Wire frontend initialization to use the saved device.
- [x] Verify GREEN with Rust and frontend tests.

### Task 7: Global push-to-talk spike behind a narrow command boundary

**Files:**
- Modify or create: `src-tauri/src/hotkey.rs`
- Modify or create: `src-tauri/src/dictation.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify frontend status handling if events are added.

- [x] Write unit tests for press/release state transitions without registering real OS hotkeys.
- [x] Implement pure state machine first.
- [x] Add Tauri event boundary for push-to-talk events.
- [x] Only after pure tests pass, wire actual global shortcut registration.
- [ ] Manually verify on Windows and record result in `docs/harness/progress.md`.

### Task 8: Full verification and docs sync

**Files:**
- Modify: `README.md`
- Modify: `docs/guide/run-and-understand.md`
- Modify: `docs/guide/code-walkthrough.md`
- Modify: `docs/harness/progress.md`
- Modify: `docs/harness/session-handoff.md`

- [x] Run `npm test -- --run`.
- [x] Run `npm run typecheck`.
- [x] Run `cargo test --manifest-path src-tauri/Cargo.toml`.
- [x] Run `python -m json.tool docs/harness/feature_list.json`.
- [x] Run `bash init.sh`.
- [x] Run `git diff --check`.
- [x] Update docs with what actually shipped and what remains.

## Completion audit notes

- Code and automated verification for V2 are complete as of commit `874fe6c` plus this plan/spec sync.
- The remaining unchecked item is real Windows manual verification of `Ctrl+Alt+Space` global push-to-talk in a target application. Automated tests prove the Rust state machine, event boundary, frontend typecheck, build, and unit tests; they do not prove OS-level focus and global shortcut behavior.
- Keep `v2-001` as `in_progress` until that manual verification is recorded in `docs/harness/progress.md` and `docs/harness/feature_list.json`.
