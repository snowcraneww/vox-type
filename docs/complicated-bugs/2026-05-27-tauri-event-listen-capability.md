# 全局快捷键有事件但前端监听不到

日期：2026-05-27  
相关范围：Tauri 2 capability、全局快捷键、前端 event.listen、桌面 overlay

## 现象

维护者按 `Ctrl+Alt+Space` 后，Rust 全局快捷键注册和 overlay command 看起来正常，但前端没有收到热键事件，录音/转写/上屏闭环没有执行。

## 为什么复杂

这是 Tauri 2 权限模型问题。Rust 侧可以 emit event，但前端窗口是否允许 `event.listen` 取决于 capability 配置。

如果只看 Rust 日志，会误以为事件已经发出；如果只看前端，会误以为快捷键没有触发。

## 根因

Tauri 2 默认 capability 没有显式授予窗口 `core:event:allow-listen` 和 `core:event:allow-unlisten`。前端 `listen()` 调用被权限拒绝。

同时 React StrictMode / 异步 listen 清理也可能造成组件卸载后 listener 注册才返回，需要立即清理。

## 修复

- 更新 `src-tauri/capabilities/default.json`，给主窗口和 overlay 窗口授予必要 event listen/unlisten 权限。
- 修复 `App.tsx` 和 `DictationOverlay.tsx` 中异步 `listen()` 的清理竞态：如果组件已经 disposed，`listen()` 返回后立即调用 unlisten。
- 诊断模式保留“测试桌面浮窗 / 隐藏桌面浮窗”，用于拆分 overlay 显示问题和热键事件问题。

## 验证

历史验证记录见 `docs/harness/debugging-log.md` 中 “event.listen not allowed” 和 `Ctrl+Alt+Space` 快捷键闭环条目。

关键验证：

```bash
npm test -- --run src/App.test.tsx src/DictationOverlay.test.tsx
npm run tauri -- dev
```

真实桌面验证要看到：

- 诊断日志出现收到全局快捷键按下/松开。
- 按下时显示 overlay。
- 松开后停止录音、转写并上屏。

## 后续经验

- Tauri 2 下事件监听失败时，优先查 capability，不要只查 Rust emit。
- 对异步 listener 注册，要处理组件已卸载但 promise 后返回的情况。
- 桌面能力问题必须用 `npm run tauri -- dev` 验证，浏览器预览不能说明系统能力可用。
