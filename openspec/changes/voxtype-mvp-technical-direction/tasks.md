# VoxType MVP 技术方向任务

状态：待实施。本文只记录 OpenSpec 层面的任务边界，不替代后续详细 implementation plan。

## 1. 项目 scaffold

- [x] 创建 Tauri 2 + React/TypeScript + Rust 项目骨架。
- [x] 建立基础目录结构：`src/`、`src-tauri/`、`src-tauri/src/`。
- [x] 建立格式化、lint/typecheck、测试命令。
- [x] 更新 README 的真实开发命令。

## 2. Rust 核心模块

- [x] 建立 `state` 状态机模块。
- [x] 建立 `config` 本地配置模块。
- [x] 建立 `hotkey` 全局快捷键模块。
- [x] 建立 `recorder` 麦克风录音模块。
- [x] 接入 `cpal` 开始/停止录音采集 proof-of-life。
- [x] 建立 `audio` 音频标准化模块。
- [x] 建立 `asr` adapter，并优先接 whisper.cpp 路线。
- [x] 建立 `insertion` adapter，并优先实现剪贴板粘贴方案。
- [x] 建立统一错误类型和 UI 可展示错误消息。

## 3. UI 与状态反馈

- [x] 建立托盘入口。
- [x] 建立设置页：快捷键、麦克风、模型路径、语言、上屏策略。
- [x] 建立状态提示：空闲、录音中、识别中、上屏中、成功、失败。
- [ ] 确保状态提示不抢走目标应用焦点。

## 4. 验证

- [x] Rust 单元测试覆盖状态机、配置、错误映射、音频处理。
- [x] 使用 mock 覆盖 ASR adapter 和 insertion adapter。
- [x] 前端测试覆盖设置页和状态展示。
- [ ] Windows 手动 E2E 验证：Notepad、VS Code、浏览器输入框。
- [ ] 更新 `docs/harness/progress.md` 和 `docs/harness/feature_list.json` 的证据。
