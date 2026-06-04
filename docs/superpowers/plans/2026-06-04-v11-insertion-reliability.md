# V11 Insertion Reliability Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a Windows `SendInput(KEYEVENTF_UNICODE)` insertion path with explicit user strategy selection, diagnostics, metadata, and clipboard fallback.

**Architecture:** Rust owns platform insertion adapters. The frontend calls one strategy-aware insertion wrapper so push-to-talk, continuous input, diagnostics, and reinsert actions share behavior. Clipboard remains the default until real desktop evidence supports Auto.

**Tech Stack:** Tauri 2, Rust Windows FFI, React/TypeScript, Vitest, Rust `cargo check` and targeted `cargo test --no-run`.

---

### Task 1: Define Insertion Strategy Types

**Files:**
- Modify: `src/types.ts`
- Modify: `src/tauriClient.ts`
- Test: `src/App.test.tsx`

- [ ] **Step 1: Write the failing frontend behavior test**

Add a test that opens Settings and expects an insertion strategy control with `Clipboard`, `SendInput`, and `Auto`, with `Clipboard` selected by default.

Run: `npm test -- --run src/App.test.tsx -t insertion strategy`
Expected: FAIL because the setting does not exist.

- [ ] **Step 2: Add TypeScript strategy types**

Add `InsertionStrategy = "clipboard" | "sendinput" | "auto"` and command result metadata with requested strategy, actual strategy, fallback flag, and optional error category.

- [ ] **Step 3: Re-run the targeted test**

Run: `npm test -- --run src/App.test.tsx -t insertion strategy`
Expected: PASS after the UI task wires the setting; if this task only adds types, keep the failing test for Task 3.

### Task 2: Add Rust SendInput Adapter

**Files:**
- Create or modify: `src-tauri/src/insertion.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write Rust unit coverage for strategy dispatch**

Add tests for empty text, unsupported non-Windows behavior behind cfg gates, and Auto fallback decision without invoking real OS input in unit tests.

Run: `cargo test --manifest-path src-tauri/Cargo.toml insertion --no-run`
Expected: FAIL until the module exists.

- [ ] **Step 2: Implement `SendInput(KEYEVENTF_UNICODE)` behind Windows cfg**

Use minimal Win32 FFI or the existing project Windows FFI pattern. Do not log raw text. Return structured result metadata.

- [ ] **Step 3: Keep clipboard fallback path intact**

Auto should fall back to existing clipboard insertion only when SendInput returns a structured failure. `Clipboard` should call only the existing path.

- [ ] **Step 4: Verify Rust checks**

Run: `cargo check --manifest-path src-tauri/Cargo.toml --lib` and `cargo test --manifest-path src-tauri/Cargo.toml insertion --no-run`.
Expected: both pass.

### Task 3: Wire Frontend Settings And Calls

**Files:**
- Modify: `src/App.tsx`
- Modify: `src/ModelSettingsView.tsx` or the Settings input page code in `src/App.tsx`
- Modify: `src/tauriClient.ts`
- Test: `src/App.test.tsx`

- [ ] **Step 1: Write failing tests for shared insertion wrapper**

Cover push-to-talk final insertion and diagnostic clipboard test using the new strategy-aware wrapper.

Run: `npm test -- --run src/App.test.tsx -t insertion strategy`
Expected: FAIL until calls are routed through the new wrapper.

- [ ] **Step 2: Add Settings UI**

Place insertion strategy under Settings -> 输入 as a compact third card or row, matching the existing light-green style. Default to Clipboard.

- [ ] **Step 3: Route all final insertions through one helper**

Update push-to-talk, continuous final text, reinsert, simulated dictation, and diagnostic insertion calls to use one frontend helper.

- [ ] **Step 4: Verify frontend checks**

Run: `npm test -- --run src/App.test.tsx`, `npm run typecheck`, and `npm run build`.
Expected: all pass.

### Task 4: Metadata, Diagnostics, And Docs

**Files:**
- Modify: `src/types.ts`
- Modify: transcript history mapping in `src/App.tsx` and Rust persistence if needed
- Modify: `docs/harness/progress.md`
- Modify: `docs/harness/feature_list.json`

- [ ] **Step 1: Add metadata test**

Test that transcript records can show requested/actual insertion strategy and fallback status without exposing extra private text in diagnostics.

- [ ] **Step 2: Render compact metadata**

Use existing `record-meta .stat-pill` style. Do not introduce a new visual language.

- [ ] **Step 3: Update docs and verification evidence**

Record automated checks and manual desktop checklist.

- [ ] **Step 4: Final verification**

Run: `bash init.sh`; `npm test -- --run`; `npm run typecheck`; `npm run build`; `cargo check --manifest-path src-tauri/Cargo.toml --lib`; `cargo test --manifest-path src-tauri/Cargo.toml insertion --no-run`; `python -m json.tool docs/harness/feature_list.json`; `git diff --check`.
