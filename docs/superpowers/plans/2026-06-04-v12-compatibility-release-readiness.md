# V12 Desktop Compatibility And Release Readiness Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create repeatable desktop compatibility verification and release-readiness checks after V11 insertion reliability lands.

**Architecture:** Keep implementation light. Store compatibility evidence in docs, add only minimal app diagnostics for build/version metadata if needed, and wrap existing build commands instead of adding a new release system.

**Tech Stack:** Tauri 2, Rust, React/TypeScript, npm scripts, repository docs, existing harness JSON.

---

### Task 1: Compatibility Matrix Document

**Files:**
- Create: `docs/guide/desktop-compatibility-matrix.md`
- Modify: `docs/harness/feature_list.json`
- Modify: `docs/harness/progress.md`

- [ ] **Step 1: Draft the matrix structure**

Create a table with target app, target type, clipboard result, SendInput result, Auto result, known caveats, and evidence date. Include VS Code, Notepad, browser text fields, an IME-heavy chat target, and one elevated/permission-boundary target.

- [ ] **Step 2: Add V12 status references**

Reference the matrix from `v12-001` evidence and from progress once manual checks are run.

- [ ] **Step 3: Validate docs**

Run: `python -m json.tool docs/harness/feature_list.json`
Expected: PASS.

### Task 2: Release Build And Version Checklist

**Files:**
- Create: `docs/guide/release-checklist.md`
- Modify: `README.md` if public commands change

- [ ] **Step 1: Document release commands**

Document the exact commands for clean install, tests, typecheck, build, Rust check, and Tauri release build. Mark any command that is environment-dependent.

- [ ] **Step 2: Add release artifact expectations**

Record expected installer artifact paths, version naming, and which artifacts should not be committed.

- [ ] **Step 3: Add privacy scan checklist**

Include checks for API keys, tokens, local user paths, audio files, model binaries, and logs.

### Task 3: Optional Build Metadata In Diagnostics

**Files:**
- Modify: `src-tauri/src/lib.rs` or an existing config/status module
- Modify: `src/DiagnosticView.tsx`
- Test: `src/App.test.tsx`

- [ ] **Step 1: Write a failing test for build metadata display**

Test that Settings -> 诊断 can show app version/build channel without using a button-like status style.

Run: `npm test -- --run src/App.test.tsx -t build metadata`
Expected: FAIL until metadata is wired.

- [ ] **Step 2: Add minimal metadata command or reuse existing package version**

Expose only non-sensitive metadata: app version, build profile/channel if available. Do not expose local paths.

- [ ] **Step 3: Verify frontend checks**

Run: `npm test -- --run src/App.test.tsx`, `npm run typecheck`, and `npm run build`.
Expected: all pass.

### Task 4: V12 Closeout Evidence And TSF Decision

**Files:**
- Modify: `docs/guide/desktop-compatibility-matrix.md`
- Modify: `docs/harness/feature_list.json`
- Modify: `docs/harness/progress.md`
- Optionally create: `docs/research/tsf-decision-after-v12.md`

- [ ] **Step 1: Record manual desktop evidence**

Fill compatibility results for each target app and each insertion strategy after V11 exists. Include failure modes and fallback behavior.

- [ ] **Step 2: Write TSF decision note**

State whether SendInput plus clipboard fallback is enough for the next public release, or whether V13 should be a TSF feasibility spike.

- [ ] **Step 3: Final verification**

Run: `bash init.sh`; `npm test -- --run`; `npm run typecheck`; `npm run build`; `cargo check --manifest-path src-tauri/Cargo.toml --lib`; `python -m json.tool docs/harness/feature_list.json`; `git diff --check`.
