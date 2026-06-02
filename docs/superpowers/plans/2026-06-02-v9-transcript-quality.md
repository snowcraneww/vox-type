# V9 Transcript Quality And History Persistence Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add deterministic transcript post-processing and durable local transcript history.

**Architecture:** Rust owns post-processing, validation, and local history persistence in the Tauri app data directory. React loads persisted history at startup, sends all history mutations through Tauri commands, and applies the same post-processing path before insertion/history updates.

**Tech Stack:** Tauri 2, Rust, React, TypeScript, Vitest, existing user config patterns, existing model routing and transcript history UI.

---

## File Map

- Create: `src-tauri/src/text_postprocess.rs` for config types, processing functions, persistence, preview, and tests.
- Create: `src-tauri/src/transcript_history.rs` for durable history storage, entry validation, retention, and tests.
- Modify: `src-tauri/src/lib.rs` to register post-processing and transcript history Tauri commands.
- Modify: `src/tauriClient.ts` to add postprocess and history commands.
- Modify: `src/types.ts` to add postprocess and persisted history types.
- Modify: `src/ModelSettingsView.tsx` to add a text optimization panel with preview.
- Modify: `src/App.tsx` to load persisted history, persist new records, and route delete/clear through backend commands.
- Modify: `src/App.test.tsx` for quality and persistence tests.
- Modify: `docs/harness/progress.md`, `docs/harness/debugging-log.md`, `docs/harness/feature_list.json`.

## Task 1: Rust Text Postprocess Core

- [ ] Write failing tests in `src-tauri/src/text_postprocess.rs` for `scale => skill`, glossary spelling, whitespace trim, and noise-only text.
- [ ] Implement `ReplacementRule`, `TranscriptPostprocessConfig`, `PostprocessResult`, and `process_transcript`.
- [ ] Ensure replacement rules are applied longest-first.
- [ ] Run: `cargo test --manifest-path src-tauri/Cargo.toml text_postprocess --no-run`.

Expected: the module compiles and tests are included in the test binary.

## Task 2: Postprocess Config Persistence And Preview

- [ ] Add app-data JSON persistence helpers for postprocess config.
- [ ] Add commands: `get_transcript_postprocess_config`, `save_transcript_postprocess_config`, `preview_transcript_postprocess`.
- [ ] Validate save: replacement `from` must not be empty; blank glossary rows are removed.
- [ ] Run: `cargo check --manifest-path src-tauri/Cargo.toml --lib`.

## Task 3: Rust Transcript History Store

- [ ] Create `src-tauri/src/transcript_history.rs`.
- [ ] Define `PersistedTranscriptEntry` with id, text, input mode, model, created time, duration, character count, and postprocess rule count.
- [ ] Add commands: `load_transcript_history`, `save_transcript_history_entry`, `delete_transcript_history_entry`, `clear_transcript_history`.
- [ ] Keep latest 500 entries by default.
- [ ] Add tests for save, load, delete, clear, retention trimming, and corrupt file fallback.
- [ ] Run: `cargo test --manifest-path src-tauri/Cargo.toml transcript_history --no-run`.

## Task 4: Frontend Client And Types

- [ ] Add `TranscriptPostprocessConfig`, `ReplacementRule`, `PostprocessResult`, and `PersistedTranscriptEntry` to `src/types.ts`.
- [ ] Add Tauri client wrappers for the new commands in `src/tauriClient.ts`.
- [ ] Update Vitest mocks for startup history loading and history mutations.
- [ ] Run: `npm run typecheck`.

## Task 5: Text Optimization UI

- [ ] Add a text optimization panel to `src/ModelSettingsView.tsx`.
- [ ] Fields: enabled switch, glossary textarea, replacement textarea using `wrong => correct` lines, cleanup checkbox, preview input, preview output.
- [ ] Save calls `saveTranscriptPostprocessConfig`.
- [ ] Preview calls `previewTranscriptPostprocess`.
- [ ] Add React tests for rendering, saving `scale => skill`, and preview output.
- [ ] Run: `npm test -- --run src/App.test.tsx`.

## Task 6: Runtime Text Quality Integration

- [ ] Apply post-processing to push-to-talk final text before clipboard insertion and history save.
- [ ] Apply post-processing to continuous local/short-speech final text before insertion and history save.
- [ ] Apply post-processing to Baidu realtime `FIN_TEXT` before insertion and merged history save.
- [ ] Add tests proving processed text is inserted and persisted.
- [ ] Keep `MID_TEXT` partial events status-only.

## Task 7: History Persistence UI Integration

- [ ] Load history on app startup and render it in the existing recognition history list.
- [ ] Persist new transcript records immediately after successful recognition flow.
- [ ] Make delete-row call `deleteTranscriptHistoryEntry`.
- [ ] Make clear-all call `clearTranscriptHistory`.
- [ ] Ensure export uses the visible history after persistence load.
- [ ] Add React tests for restart-style load, delete persistence, and clear persistence.

## Task 8: Docs And Verification

- [ ] Update `docs/harness/progress.md` with V9 implementation evidence.
- [ ] Update `docs/harness/debugging-log.md` for issues found.
- [ ] Update `docs/harness/feature_list.json` evidence/status.
- [ ] Run full verification:

```bash
npm test -- --run src/App.test.tsx
npm run typecheck
npm run build
cargo check --manifest-path src-tauri/Cargo.toml --lib
cargo test --manifest-path src-tauri/Cargo.toml text_postprocess --no-run
cargo test --manifest-path src-tauri/Cargo.toml transcript_history --no-run
python -m json.tool docs/harness/feature_list.json
git diff --check
```

## Execution Notes

- Keep V9 deterministic; do not add LLM calls.
- Keep transcript history local-only.
- Do not store raw audio or secrets.
- Do not change the clipboard insertion strategy in V9.
- Do not insert `MID_TEXT` partial text into the target input.
