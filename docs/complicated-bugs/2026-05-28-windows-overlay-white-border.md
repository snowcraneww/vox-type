# Windows 透明浮窗白/灰边框

日期：2026-05-28  
相关范围：Tauri WebView2 透明窗口、Windows 合成、Win32 native overlay

## 现象

底部语音浮窗希望只显示黑色胶囊和彩色声波，但在 Windows 桌面上，胶囊外侧经常出现白色、灰色或浅色矩形边框。多轮 CSS 调整只能缓解，不能稳定消除。

## 为什么复杂

这个问题横跨 CSS、WebView2、Tauri window、Windows 透明窗口合成和首帧背景。它不是普通 CSS border。

浏览器预览正常不代表 Tauri 透明窗口正常。WebView2 在透明窗口、阴影、默认背景、窗口尺寸和首帧渲染之间存在平台差异。

## 根因判断

多轮排查后判断：白/灰边不是组件内部边框，而是 Windows WebView2/Tauri 透明窗口合成限制或首帧/窗口背景参与合成导致。

曾尝试过：

- `html/body/#root` 透明化。
- overlay 首帧样式提前写入 `index.html`。
- 缩小窗口尺寸。
- 设置 `shadow: false`。
- 调整 SVG viewBox 和胶囊安全边距。

这些都不能彻底稳定解决。

## 修复

V4 改为 Windows-only native overlay：

- 新增 `src-tauri/src/native_overlay.rs`。
- 使用 Win32 `WS_POPUP`、`WS_EX_LAYERED`、`WS_EX_NOACTIVATE` 等创建独立浮窗。
- 后续改为软件高倍渲染 BGRA 位图，再通过 `UpdateLayeredWindow` 做 per-pixel alpha，降低 GDI 圆角/边缘锯齿。
- WebView overlay 保留为 fallback。

核心代码位置：

- `src-tauri/src/native_overlay.rs`
- `src-tauri/src/overlay.rs`
- `src/DictationOverlay.tsx`
- `src-tauri/tauri.conf.json`

## 验证

历史验证记录见 `docs/harness/debugging-log.md` 中 V4 native overlay 相关条目。

关键验证：

```bash
cargo check --manifest-path src-tauri/Cargo.toml --lib
npm test -- --run src/DictationOverlay.test.tsx
npm run typecheck
npm run build
```

最终必须由维护者在真实 Windows Tauri 桌面模式手动确认：浮窗外层白/灰矩形消失。

## 后续经验

- Windows WebView2 透明窗口问题不要无限 CSS 微调。若问题来自窗口合成层，应尽早切 native overlay spike。
- overlay 视觉问题和 ASR 准确率问题必须拆开。修改浮窗不应被误判为识别质量变化。
- WebView fallback 仍有价值，但 Windows 默认路径应优先 native overlay。
