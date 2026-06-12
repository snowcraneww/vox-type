# ASR 一键安装 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在 VoxType UI 中提供一键安装 whisper.cpp CPU 版和默认 `ggml-base.bin` 模型，并自动保存 ASR 配置。

**Architecture:** 新增 Rust `asr_installer` 模块，负责安装路径规划、下载、解压、模型落盘和配置保存。前端通过 Tauri command 调用安装器，并复用现有 `AsrConfigStatus` 刷新 UI。

**Tech Stack:** Rust、Tauri 2、blocking HTTP download、ZIP extraction、React/TypeScript、Vitest、Cargo test。

---

### Task 1: Rust 安装路径和状态

**Files:**
- Create: `src-tauri/src/asr_installer.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Write failing tests**

测试安装目录规划、已安装文件检测和配置生成。

- [ ] **Step 2: Implement install plan**

实现固定 Windows x64 CPU 版 binary 和 `ggml-base.bin` 模型路径。

### Task 2: 下载和解压

**Files:**
- Modify: `src-tauri/src/asr_installer.rs`

- [ ] **Step 1: Write tests for zip extraction helper**

创建临时 zip，验证能提取 `whisper-cli.exe`。

- [ ] **Step 2: Implement download and unzip**

下载到 app data 的 `downloads/`，解压到 `bin/`。

### Task 3: Tauri command 和前端入口

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/tauriClient.ts`
- Modify: `src/App.tsx`
- Modify: `src/types.ts`

- [ ] **Step 1: Add command**

暴露 `install_managed_asr`，返回 `AsrConfigStatus`。

- [ ] **Step 2: Add UI button**

在 `ASR 配置` 面板新增 `一键安装 whisper.cpp`。

### Task 4: Docs and verification

- [ ] 更新 `docs/guide/run-and-understand.md`。
- [ ] 更新 `docs/guide/code-walkthrough.md`。
- [ ] 更新 `docs/harness/progress.md`、`session-handoff.md`、`feature_list.json`。
- [ ] 运行 `cargo test --manifest-path src-tauri/Cargo.toml`。
- [ ] 运行 `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`。
- [ ] 运行 `npm run typecheck`、`npm test -- --run`、`npm run build`。
- [ ] 运行 `npm run tauri -- build --debug`。
