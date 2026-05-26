# VoxType MVP Scaffold Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 创建最小可运行的 VoxType Tauri 2 + React/TypeScript + Rust 项目骨架，并建立 MVP 核心模块、测试基线和文档入口。

**Architecture:** 保留现有 harness/docs，不使用交互式脚手架覆盖仓库根目录。手写最小 Tauri 2/Vite 配置；Rust 先实现可测试的状态机、配置、ASR adapter、上屏 adapter 和 mock 闭环，真实 whisper.cpp/音频/Windows 上屏作为后续增强点接入。

**Tech Stack:** Rust 1.95、Tauri 2、React 19、TypeScript、Vite、Vitest、npm。

---

## 文件结构

- Create `package.json`：前端和 Tauri 开发命令。
- Create `index.html`、`src/main.tsx`、`src/App.tsx`、`src/styles.css`、`src/types.ts`：React 设置页和状态展示。
- Create `tsconfig.json`、`tsconfig.node.json`、`vite.config.ts`：TypeScript/Vite 配置。
- Create `src-tauri/Cargo.toml`、`src-tauri/build.rs`、`src-tauri/tauri.conf.json`：Tauri/Rust 配置。
- Create `src-tauri/src/*.rs`：Rust MVP 模块 skeleton。
- Modify `README.md`：写真实开发和验证命令。
- Modify `init.sh`：加入产品 scaffold 检查。
- Modify `openspec/changes/voxtype-mvp-technical-direction/tasks.md`：勾选已完成 scaffold 子项。
- Modify `docs/harness/*`：记录证据和下一步。

## Task 1：项目配置和依赖

- [ ] 创建 `package.json`，包含 `dev`、`build`、`preview`、`tauri`、`test`、`typecheck` 脚本。
- [ ] 创建 Vite/TypeScript 配置。
- [ ] 创建 `src-tauri/Cargo.toml`，依赖 `tauri`、`tauri-build`、`serde`、`serde_json`、`thiserror`。
- [ ] 运行 `npm install`。
- [ ] 运行 `cargo fetch --manifest-path src-tauri/Cargo.toml`。

## Task 2：Rust 核心 skeleton

- [ ] 创建 `state.rs`，定义 `AppPhase`、`AppStatus` 和状态转换函数。
- [ ] 创建 `config.rs`，定义 `AppConfig` 默认值：中文优先、whisper.cpp、clipboard。
- [ ] 创建 `error.rs`，定义 `VoxError` 和 `UserFacingError`。
- [ ] 创建 `asr/mod.rs`，定义 `AsrEngine` trait、`MockAsrEngine` 和 `WhisperCppEngine` 占位结构。
- [ ] 创建 `insertion/mod.rs`，定义 `InsertionStrategy` trait、`MockInsertion` 和 `ClipboardInsertion` 占位结构。
- [ ] 创建 `audio.rs`，实现最小时长和 mono PCM 基础工具函数。
- [ ] 创建 `hotkey.rs`、`recorder.rs` 占位模块，明确后续 native 接入边界。
- [ ] 创建 `lib.rs`，暴露 Tauri commands：`get_config`、`get_status`、`simulate_dictation`。

## Task 3：前端设置页和状态展示

- [ ] 创建 `src/types.ts`，与 Rust command 返回结构对齐。
- [ ] 创建 `src/App.tsx`，展示快捷键、语言、ASR、上屏策略和状态。
- [ ] 创建 `src/styles.css`，使用克制工具型布局。
- [ ] 创建 `src/main.tsx`。
- [ ] 添加 Vitest smoke test，验证关键文本渲染。

## Task 4：验证和文档

- [ ] 运行 `npm run typecheck`。
- [ ] 运行 `npm test -- --run`。
- [ ] 运行 `cargo test --manifest-path src-tauri/Cargo.toml`。
- [ ] 运行 `npm run build`。
- [ ] 运行 `bash init.sh` 和 `python -m json.tool docs/harness/feature_list.json`。
- [ ] 更新 README、OpenSpec tasks、harness progress/session handoff。

## 当前边界

- 本计划先完成可运行骨架和 mock 闭环，不承诺今晚完成真实麦克风录音、真实 whisper.cpp 推理和真实 Windows 上屏。
- 如果依赖安装、Tauri 构建或 Windows WebView2 工具链失败，记录 blocker 和已完成范围。

