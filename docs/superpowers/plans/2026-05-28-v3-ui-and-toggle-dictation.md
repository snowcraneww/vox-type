# V3 UI And Toggle Dictation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让 VoxType V3 拥有更精致的深色主界面/诊断界面，并新增 `Ctrl+Alt+V` 按一下开始、再按一下停止并上屏的切换录音模式。

**Architecture:** Rust 侧在现有 global-shortcut 插件中注册第二个快捷键，并用独立状态机分发 toggle action。前端复用现有录音、转写、剪贴板上屏闭环，只扩展事件类型和界面说明。UI 通过现有 CSS 收敛字号、间距、面板层级和按钮密度。

**Tech Stack:** Rust、Tauri 2、tauri-plugin-global-shortcut、React、TypeScript、Vitest、CSS。

---

### Task 1: 收尾 V2 Harness 状态

**Files:**
- Modify: `docs/harness/feature_list.json`
- Modify: `docs/harness/progress.md`
- Modify: `docs/harness/session-handoff.md`

- [ ] 将 `v2-001` 标记为 `passing`，说明维护者接受当前 overlay 残余边框问题。
- [ ] 新增 `v3-001`，状态为 `in_progress`，范围是 UI 精修、切换录音快捷键和透明边框调研。
- [ ] 运行 `python -m json.tool docs/harness/feature_list.json`。

### Task 2: Rust 第二快捷键

**Files:**
- Modify: `src-tauri/src/hotkey.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] 在 `hotkey.rs` 新增 `ToggleHotkeyBinding`、`ToggleDictationState` 和 `toggleStartRecording` / `toggleStopAndTranscribe` action。
- [ ] 在 `lib.rs` 注册 `Ctrl+Alt+Space` 和 `Ctrl+Alt+V` 两个快捷键。
- [ ] 运行 `cargo test --manifest-path src-tauri/Cargo.toml hotkey`。

### Task 3: 前端切换录音闭环

**Files:**
- Modify: `src/tauriClient.ts`
- Modify: `src/App.tsx`
- Modify: `src/App.test.tsx`

- [ ] 扩展 `PushToTalkPayload.action` 类型。
- [ ] 让 `toggleStartRecording` 调用 `handleStartRecording`。
- [ ] 让 `toggleStopAndTranscribe` 调用 `handleStopTranscribeAndInsertNow`。
- [ ] 在测试里模拟两次 toggle action，断言录音、转写、上屏和隐藏浮窗都被调用。
- [ ] 运行 `npm test -- --run src/App.test.tsx`。

### Task 4: 主界面和诊断模式视觉精修

**Files:**
- Modify: `src/App.tsx`
- Modify: `src/styles.css`
- Test: `src/App.test.tsx`

- [ ] 主界面改为更紧凑的深色工作台：小标题、小按钮、低边框、少卡片。
- [ ] 诊断模式改为紧凑的 grid 面板，降低字号、圆角、阴影和间距。
- [ ] 保持诊断入口、测试按钮、日志复制和 ASR 配置入口不丢失。
- [ ] 运行 `npm test -- --run src/App.test.tsx` 和 `npm run typecheck`。

### Task 5: 文档、验证和提交

**Files:**
- Modify: `README.md`
- Modify: `docs/guide/run-and-understand.md`
- Modify: `docs/harness/debugging-log.md`
- Modify: `docs/harness/progress.md`
- Modify: `docs/harness/feature_list.json`

- [ ] 记录透明窗口白/灰边框调研结论和来源。
- [ ] 说明 V3 两种快捷键模式，注明当前不是实时流式 ASR。
- [ ] 运行完整验证：`npm test -- --run`、`npm run typecheck`、`cargo test --manifest-path src-tauri/Cargo.toml`、`npm run build`、`cargo fmt --all --manifest-path src-tauri/Cargo.toml --check`、`git diff --check`。
- [ ] 运行隐私扫描，确认没有个人信息或密钥进入提交。
- [ ] 用 `VoxType <maintainers@voxtype.dev>` 身份提交。
