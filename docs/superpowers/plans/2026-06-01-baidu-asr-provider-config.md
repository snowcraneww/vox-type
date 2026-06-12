# Baidu ASR Provider Config Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Baidu short-speech ASR as a selectable cloud ASR provider configuration while keeping real transcription on local whisper.cpp until Baidu runtime calls are implemented and verified.

**Architecture:** Extend the existing `cloud_asr_config` model from MiniMax-only fields into provider-aware fields. Frontend state remains in `App.tsx`, while `ModelSettingsView.tsx` renders provider-specific forms for MiniMax and Baidu. Secrets are stored only in user environment variables; project config stores endpoint/model/cuid-style non-secret fields.

**Tech Stack:** Tauri 2, Rust, React, TypeScript, Vitest, Windows user environment variables.

---

### Task 1: Document Baidu Official ASR Findings

**Files:**
- Create: `docs/integrations/baidu-asr.md`
- Modify: `docs/harness/research-log.md`

- [ ] **Step 1: Write the integration document**

Create `docs/integrations/baidu-asr.md` with the official endpoint, supported audio shape, request fields, return field, and current VoxType scope.

- [ ] **Step 2: Update research log**

Append a `2026-06-01` Baidu ASR section to `docs/harness/research-log.md` linking the three official docs.

- [ ] **Step 3: Verify docs formatting**

Run: `git diff --check docs/integrations/baidu-asr.md docs/harness/research-log.md`

Expected: exit 0.

### Task 2: Extend Rust Cloud Config

**Files:**
- Modify: `src-tauri/src/cloud_asr_config.rs`

- [ ] **Step 1: Add failing Rust tests**

Add tests proving Baidu config is ready only when `BAIDU_ASR_API_KEY` is present and required non-secret fields are set, and that saved config never contains the key.

- [ ] **Step 2: Run tests to verify red**

Run: `cargo test --manifest-path src-tauri/Cargo.toml cloud_asr_config --no-run`

Expected: fail because Baidu fields/status helpers do not exist yet.

- [ ] **Step 3: Implement provider-aware config**

Add fields for `endpoint`, `dev_pid`, `cuid`, `format`, and `sample_rate`; add `BAIDU_ASR_API_KEY`; normalize MiniMax and Baidu defaults separately.

- [ ] **Step 4: Run Rust verification**

Run: `cargo test --manifest-path src-tauri/Cargo.toml cloud_asr_config --no-run` and `cargo check --manifest-path src-tauri/Cargo.toml --lib`.

Expected: both exit 0.

### Task 3: Extend Frontend Provider Form

**Files:**
- Modify: `src/types.ts`
- Modify: `src/tauriClient.ts`
- Modify: `src/App.tsx`
- Modify: `src/ModelSettingsView.tsx`
- Modify: `src/App.test.tsx`

- [ ] **Step 1: Add failing frontend test**

Add a Vitest case that opens model settings, switches to cloud, selects Baidu ASR, sees Baidu endpoint/dev_pid/cuid/API Key fields, saves provider config, and saves the Baidu key through a password field without rendering the secret.

- [ ] **Step 2: Run test to verify red**

Run: `npm test -- --run src/App.test.tsx`

Expected: fail because Baidu provider controls do not exist.

- [ ] **Step 3: Implement frontend state and UI**

Add provider state, provider selector, Baidu-specific fields, and `saveBaiduAsrApiKey` Tauri client wrapper. Keep MiniMax form unchanged except for being behind provider selection.

- [ ] **Step 4: Run frontend verification**

Run: `npm test -- --run src/App.test.tsx` and `npm run typecheck`.

Expected: both exit 0.

### Task 4: Harness Sync and Final Verification

**Files:**
- Modify: `docs/harness/progress.md`
- Modify: `docs/harness/feature_list.json` if verification expectations change.

- [ ] **Step 1: Update progress**

Record Baidu provider config status, official-doc basis, and that real Baidu transcription remains a follow-up task.

- [ ] **Step 2: Run verification**

Run:

```bash
npm test -- --run src/App.test.tsx
npm run typecheck
cargo test --manifest-path src-tauri/Cargo.toml cloud_asr_config --no-run
cargo check --manifest-path src-tauri/Cargo.toml --lib
python -m json.tool docs/harness/feature_list.json
git diff --check
```

Expected: all commands exit 0. If a known environment issue appears, record it in `docs/harness/progress.md` and `docs/harness/debugging-log.md`.

---

## Self-Review

- Scope matches the approved design: provider configuration and selection only, no real Baidu runtime ASR call.
- Secret handling is explicit: API keys go to user environment variables only.
- Tests cover Rust readiness and frontend UI behavior.
- Documentation captures official Baidu endpoint and request/response shape.
