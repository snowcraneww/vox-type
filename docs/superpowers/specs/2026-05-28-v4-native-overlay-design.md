# V4 原生桌面浮窗设计

## 目标

V4 只解决一个问题：Windows 桌面底部语音浮窗的黑色胶囊外侧不应再出现 Tauri/WebView2 带来的白色、灰色或浅色矩形边框。功能链路、ASR、上屏策略和主界面不在本版本扩展范围内。

## 用户体验

- 用户按 `Ctrl+Alt+Space` 或 `Ctrl+Alt+V` 时，屏幕底部只看到一个小型黑色胶囊和五彩声波。
- 胶囊外侧不出现白底、灰框、WebView 矩形、标题栏残影或黑色大块背景。
- overlay 不抢焦点，不进入任务栏，保持置顶。
- 当前识别和上屏行为保持不变：`Ctrl+Alt+Space` 是按住说话，`Ctrl+Alt+V` 是切换录音和实验分段上屏。
- 如果原生 overlay 在某台机器上失败，应用必须自动退回现有 Tauri WebView overlay，而不是影响录音和转写。

## 已知问题判断

当前仓库已经做过普通 CSS 和 Tauri 基础配置处理：

- `dictation-overlay` 窗口已设置 `transparent: true`。
- `dictation-overlay` 窗口已设置 `decorations: false`。
- overlay 页面首帧已经在 `index.html` 里设置透明背景。
- React/CSS 里已经把 overlay 根节点和可见内容缩小到接近 `120 x 32`。

维护者仍能看到浅色外框，说明问题很可能来自 Windows + WebView2 + Tauri 宿主窗口透明合成，而不是普通 CSS 边框。公开资料也显示 `shadow: false`、显示后 resize/repaint、WebView 背景色和原生 layered window 都是社区常见排查方向。

## 推荐架构

V4 采用分层 spike，不直接删除现有 WebView overlay。

### 第一层：Tauri WebView overlay 补丁

先保留现有 React overlay，只补最小平台设置：

- 在 `src-tauri/tauri.conf.json` 的 `dictation-overlay` 窗口加入 `shadow: false`。
- 在 Rust `show_dictation_overlay` 中尝试调用 Tauri window shadow/background 相关 API；如果 API 当前版本不可用，就只保留配置项和注释。
- 在显示前后做一次受控的尺寸/位置刷新，验证是否能解决“resize 后才透明”的问题。
- 记录诊断日志，让维护者知道当前 backend 是 `webview`。

这一层成功，则 V4 不需要引入 Win32 原生窗口。

### 第二层：Windows 原生 layered overlay 原型

如果第一层仍有白/灰框，再新增 Windows-only 原生 overlay backend：

- 新增 `src-tauri/src/native_overlay.rs`，只在 `cfg(windows)` 下编译。
- 使用 Win32 popup/layered/topmost 窗口绘制浮窗，不承载 WebView。
- 建议窗口样式：`WS_POPUP`，扩展样式包含 `WS_EX_LAYERED`、`WS_EX_TOOLWINDOW`、`WS_EX_TOPMOST`、`WS_EX_NOACTIVATE`；点击穿透是否启用单独评估，避免挡住用户操作。
- 第一版原生绘制只实现静态/轻量动画的黑色胶囊和声波，不追求完全复制 React SVG 动效。
- Tauri 主流程仍通过现有 `show_dictation_overlay` / `hide_dictation_overlay` command 调用统一 facade，由 facade 决定使用 `native-win32` 还是 `webview`。

### 第三层：诊断和回退

- 新增 overlay backend 状态：`webview`、`webview-shadowless`、`native-win32`、`fallback-webview`。
- 诊断模式显示当前 backend 和最近一次显示/隐藏错误。
- 如果原生 overlay 创建、绘制或隐藏失败，自动回退到 WebView overlay，并写入诊断日志。

## 非目标

- 不在 V4 改 ASR 模型、流式识别、VAD 或文本合并。
- 不在 V4 改剪贴板上屏为 TSF。
- 不在 V4 继续大改主界面视觉。
- 不在 V4 支持 macOS/Linux 原生 overlay；非 Windows 继续使用 WebView overlay。
- 不把原生 overlay 作为唯一方案，直到维护者在真实 Windows 桌面验收通过。

## 风险

- Tauri 的 `shadow: false` 可能只解决部分机器，不保证消除所有 WebView2 合成残影。
- Win32 layered window 会引入 unsafe 代码、消息循环、DPI、多显示器和生命周期风险。
- 原生窗口动画如果实现过重，可能影响录音期间 CPU 占用。
- 视觉问题无法只靠自动测试证明，必须保留人工验收步骤。

## 验收标准

- 自动验证通过：`npm test -- --run`、`npm run typecheck`、`cargo test --manifest-path src-tauri/Cargo.toml`、`cargo fmt --all --manifest-path src-tauri/Cargo.toml --check`、`cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`、`npm run build`。
- 诊断模式可以显示当前 overlay backend。
- 如果启用原生 backend，原生创建失败时会自动回退 WebView overlay。
- 维护者人工验证：底部浮窗外侧不再出现白/灰矩形边框；快捷键录音、转写和上屏不回退。

