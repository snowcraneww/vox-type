# V3 UI 与切换录音模式设计

## 目标

V3 解决两个问题：主界面和诊断模式仍像测试台，日常输入只支持“按住说话”。本阶段把界面收敛成更精致的深色工具窗口，并新增第二个全局快捷键：按一次开始录音，再按一次停止、转写并上屏。

## 用户体验

- 主界面只展示日常使用所需信息：当前状态、最近文本、输入设备、ASR 状态、两种快捷键。
- 诊断模式保留所有测试按钮，但视觉上改成低噪声的工程面板，不再使用粗大的卡片和字号。
- `Ctrl+Alt+Space` 继续保持按住说话：按下开始，松开停止并上屏。
- `Ctrl+Alt+V` 新增切换录音：第一次按下开始，第二次按下停止并上屏。
- 当前 whisper.cpp 接入不是流式 ASR，因此 V3 的“切换录音模式”不会假装边说边出字；它仍在停止后统一转写和上屏。真正实时流式输入进入后续版本。
- 如果 Tauri/WebView2 透明 overlay 在 Windows 上仍露出浅色边缘，V3 允许用黑色窗口背景兜底，让视觉退化为“黑色胶囊浮窗”，优先保证不出现白色方框。

## 技术设计

- Rust `hotkey` 模块新增 `ToggleDictationState` 和默认快捷键 `Ctrl+Alt+V`。
- Tauri global-shortcut 插件一次注册两个快捷键，并根据触发的 shortcut 分发为不同 action。
- 前端沿用 `voxtype-push-to-talk` 事件通道，新增 `toggleStartRecording` 和 `toggleStopAndTranscribe` action。
- 前端复用现有闭环：`startRecording` -> `stopRecording` -> `transcribeLastRecording` -> `insertTextWithClipboard` -> `hideDictationOverlay`。
- UI 只做 CSS/结构微调，不引入新依赖，不更换技术栈。

## 透明浮窗边框调研结论

Tauri 2 + Windows WebView2 透明窗口在不同系统、WebView2 版本和焦点状态下存在已知渲染限制。当前仓库已设置 `transparent: true`、`decorations: false`、页面透明背景和小尺寸 overlay，但仍可能看到浅色边缘或 ghost 背景。V3 记录该风险，不把它作为阻塞项。后续若继续攻关，需要做 Windows 原生窗口样式或 WebView2 controller 背景色实验。

## 验收标准

- Rust 快捷键状态机测试覆盖两种快捷键。
- 前端测试覆盖切换录音模式闭环。
- `npm run typecheck`、`npm test -- --run`、`cargo test --manifest-path src-tauri/Cargo.toml`、`npm run build` 通过。
- 文档同步：`feature_list.json`、`progress.md`、`debugging-log.md` 和运行指南更新。
