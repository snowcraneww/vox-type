# Windows 原生 overlay 调研记录

## 背景

VoxType 当前桌面语音浮窗使用 Tauri `WebviewWindow` 渲染 React 页面。窗口已配置 `transparent: true`、`decorations: false`、`alwaysOnTop: true`、`skipTaskbar: true`，页面侧也已经把 `html`、`body`、`#root` 和 `.overlay-root` 处理为透明或黑色兜底。但维护者在 Windows 桌面上仍能看到黑色胶囊外侧的一层浅色或灰白色矩形边缘。

本调研只聚焦一个目标：判断下一版本怎样消除浮窗外层白/灰框。

## 已有本地证据

- `src-tauri/tauri.conf.json`：`dictation-overlay` 窗口已设置 `transparent: true` 和 `decorations: false`，但当前未显式设置 `shadow: false`。
- `src-tauri/src/overlay.rs`：当前 overlay 是 Tauri webview window，显示逻辑只负责定位、`show()` 和 `set_always_on_top(true)`。
- `src/DictationOverlay.tsx` + `src/styles.css`：视觉内容实际只有 `120 x 32` 的 SVG 胶囊和波形，CSS 已尽量减少外层背景。
- `index.html`：overlay 首帧已加 `overlay-document`，在 React 启动前设置透明背景，说明白/灰框不太像普通页面首帧白底。

## 网络调研摘要

### Tauri / WebView2 方向

- Tauri issue `#11654`：有人在 Windows 10 上遇到透明窗口边框显示异常，并通过在 `tauri.conf.json` 里设置 `shadow: false` 解决。
  - 来源：https://github.com/tauri-apps/tauri/issues/11654
- Tauri v2 配置文档：`WindowConfig` 支持窗口配置；Tauri JS API 暴露 `Window.setShadow`、`Window.setBackgroundColor`、`setDecorations`、`setSize` 等能力，但具体行为存在平台差异。
  - 来源：https://v2.tauri.app/reference/config
  - 来源：https://v2.tauri.app/reference/javascript/api/namespacewebviewwindow
- Tauri issue `#12450`：`WebviewWindowBuilder::transparent(true)` 在 Windows child window 场景下可能不按预期透明。
  - 来源：https://github.com/tauri-apps/tauri/issues/12450
- StackOverflow `Tauri transparent window only works when resized`：社区观察到透明窗口可能 resize 后才正常，说明透明合成时机和 WebView2/窗口状态有关。
  - 来源：https://stackoverflow.com/questions/77344488/tauri-transparent-window-only-works-when-resized
- Tauri issue `#1564`：早期讨论过 webview background color 和白色背景闪烁，支持“WebView 宿主层可能参与绘制”的判断。
  - 来源：https://github.com/tauri-apps/tauri/issues/1564

### Windows 原生方向

- Rust 用户论坛关于透明 overlay：如果要画透明 overlay 并可点击穿透，常见路径是 Win32 扩展样式 `WS_EX_LAYERED | WS_EX_TRANSPARENT`，再通过 `SetLayeredWindowAttributes` 或 per-pixel alpha 更新窗口。
  - 来源：https://users.rust-lang.org/t/possible-to-create-a-full-screen-overlay-that-catches-clicks-but-does-not-come-into-focus/75567
- Win32 原生 per-pixel alpha / Direct2D / Direct3D 方案能绕开 WebView2 背景层，但引入更多平台代码、消息循环、DPI、多显示器和崩溃风险。

## 方案判断

推荐把下一版本做成两阶段 spike，而不是马上替换现有 overlay：

1. **Tauri 层补丁先试**：增加 `shadow: false`，并在显示 overlay 时尝试 `set_shadow(false)`、窗口背景色/重绘/resize hack。成本低、风险小，如果能解决就是最划算方案。
2. **原生 Windows overlay 原型兜底**：如果 Tauri 层仍有白/灰框，新增 Windows-only 原生 overlay 模块，用 Win32 layered popup 绘制黑色胶囊和简化波形。先作为可开关实验，不直接删除 WebView overlay。

## 验收建议

- 自动测试只能覆盖配置、状态机和 fallback；白/灰框属于视觉合成问题，需要维护者在真实 Windows 桌面人工确认。
- 诊断模式应增加明确的 `overlay backend` 状态，显示当前使用 `webview` 还是 `native-win32`。
- 人工验收标准：按 `Ctrl+Alt+Space` 或点击测试浮窗时，屏幕底部只看到黑色胶囊和波形，看不到胶囊外侧浅色/灰白矩形。

