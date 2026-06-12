# V5.1 Main Window Refinement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refine the V5 main window into a record-first dictation workspace with mode readiness, compact status cards, a reusable VoiceOverlay status capsule, session transcript history, and a model selection page.

**Architecture:** Keep `App.tsx` as the state coordinator. Expand `MainWindow.tsx` into a presentational workspace that receives session transcript records and stats. Add `ModelSettingsView.tsx` as a presentational model configuration page that reuses existing whisper.cpp config handlers. Do not change Rust, ASR, recorder, native overlay, or clipboard insertion logic.

**Tech Stack:** React 19, TypeScript, Vitest, Testing Library, existing Tauri client functions, CSS modules via `src/styles.css`.

---

### Task 1: Session Transcript History State

**Files:**
- Modify: `src/App.test.tsx`
- Modify: `src/App.tsx`
- Modify: `src/types.ts`

- [ ] Write failing tests that complete two dictation paths and expect two records in reverse chronological order.
- [ ] Run `npm test -- --run src/App.test.tsx` and confirm the new test fails because only one latest transcript exists.
- [ ] Add a `TranscriptRecord` type with `id`, `time`, `text`, `durationMs`, and `source`.
- [ ] Add `transcriptRecords` state in `App.tsx` and append records whenever a final text is produced by push-to-talk, toggle tail, fallback full transcription, or manual transcribe-and-insert.
- [ ] Wire record actions: copy, reinsert, delete one, clear all, export text.
- [ ] Run `npm test -- --run src/App.test.tsx` and confirm it passes.

### Task 2: Main Window Layout

**Files:**
- Modify: `src/App.test.tsx`
- Modify: `src/MainWindow.tsx`
- Modify: `src/styles.css`

- [ ] Write failing tests for: “模型选择” button, no old primary start button, mode readiness for both shortcuts, readiness cards without duplicated shortcut card, transcript history actions at top.
- [ ] Run `npm test -- --run src/App.test.tsx` and confirm failure.
- [ ] Update `MainWindow.tsx` props to receive records, stats, model open handler, and record actions.
- [ ] Move transcript list into the lower half of the main window.
- [ ] Replace text-heavy readiness cards with short labels and `title` attributes.
- [ ] Reuse `VoiceOverlay` inside the status capsule with current `voiceOverlayModel`.
- [ ] Update CSS for the new two-column top area and large record list.
- [ ] Run `npm test -- --run src/App.test.tsx` and `npm run typecheck`.

### Task 3: Model Selection Page

**Files:**
- Create: `src/ModelSettingsView.tsx`
- Modify: `src/App.tsx`
- Modify: `src/App.test.tsx`
- Modify: `src/styles.css`

- [ ] Write a failing test that clicks “模型选择” and expects whisper.cpp path inputs, one-click install, detect, save, and cloud API placeholder.
- [ ] Run `npm test -- --run src/App.test.tsx` and confirm failure.
- [ ] Add `viewMode: 'user' | 'diagnostic' | 'model'`.
- [ ] Move ASR config UI from `DiagnosticView` into `ModelSettingsView` while keeping existing handlers.
- [ ] Leave a cloud API placeholder marked as next-version only.
- [ ] Update `DiagnosticView` to show model readiness summary but not own the model config form.
- [ ] Run `npm test -- --run src/App.test.tsx` and `npm run typecheck`.

### Task 4: Documentation and Verification

**Files:**
- Modify: `docs/harness/progress.md`
- Modify: `docs/harness/feature_list.json`

- [ ] Record V5.1 implementation evidence and verification results.
- [ ] Run full verification: `npm test -- --run`, `npm run typecheck`, `npm run build`, `cargo check --manifest-path src-tauri/Cargo.toml --lib`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`, `cargo fmt --all --manifest-path src-tauri/Cargo.toml --check`, `python -m json.tool docs/harness/feature_list.json`, `git diff --check`.
- [ ] Run privacy and encoding scans before commit.
- [ ] Commit with project identity `VoxType <maintainers@voxtype.dev>`.