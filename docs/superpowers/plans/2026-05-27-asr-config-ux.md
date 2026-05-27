# ASR 配置体验 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让 VoxType 在界面内检测、保存和使用 whisper.cpp 配置，避免普通用户只能依赖环境变量。

**Architecture:** 新增 Rust `asr_config` 模块作为单一配置边界，负责读取应用内 JSON 配置、环境变量 fallback、路径检测和保存。前端通过 Tauri commands 获取状态、保存配置，并统一格式化结构化错误。

**Tech Stack:** Rust、Tauri 2 commands、React/TypeScript、Vitest、Cargo test。

---

### Task 1: Rust ASR 配置模块

**Files:**
- Create: `src-tauri/src/asr_config.rs`
- Modify: `src-tauri/src/lib.rs`

- [x] **Step 1: Write failing tests**

覆盖未配置、路径存在、路径不存在三类状态。

- [x] **Step 2: Verify RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml asr_config -- --nocapture`

Expected: FAIL because `status_from_config` is missing.

- [ ] **Step 3: Implement config load/save/status**

应用内配置优先，环境变量只作为 fallback。

- [ ] **Step 4: Add Tauri commands**

暴露 `get_asr_config_status` 和 `save_asr_config`，并让 `transcribe_last_recording` 使用同一配置来源。

### Task 2: Frontend ASR 配置 UI

**Files:**
- Modify: `src/App.tsx`
- Modify: `src/tauriClient.ts`
- Modify: `src/types.ts`
- Create: `src/errorFormat.ts`

- [ ] **Step 1: Write failing frontend test**

覆盖结构化 Tauri error 不再显示 `[object Object]`。

- [ ] **Step 2: Implement UI and client calls**

新增路径输入、语言输入、保存配置、检测配置按钮。

### Task 3: 中文文档对齐

**Files:**
- Modify: `docs/guide/run-and-understand.md`
- Modify: `docs/guide/code-walkthrough.md`
- Modify: `docs/harness/progress.md`
- Modify: `docs/harness/session-handoff.md`
- Modify: `docs/harness/feature_list.json`

- [ ] **Step 1: Update user-facing run guide**

说明“界面配置优先，环境变量 fallback”。

- [ ] **Step 2: Update harness progress**

记录 ASR 配置体验进入 MVP。

### Task 4: Verification

- [ ] `cargo test --manifest-path src-tauri/Cargo.toml`
- [ ] `npm run typecheck`
- [ ] `npm test -- --run`
- [ ] `npm run build`

