# 复杂 Bug 归档

本目录归档 VoxType 开发中出现过的复杂问题。这里记录的不是普通 changelog，而是后续维护者容易误判、容易复发、跨前后端或跨系统边界的故障。

每篇文档固定覆盖：现象、为什么复杂、根因、修复、验证、后续经验。

## 已归档问题

- [V10 音频增强与 SenseVoice 空文本、metadata 丢失、配置不持久](2026-06-04-v10-audio-enhancement-sensevoice-metadata.md)
- [Ctrl+Alt+V 连续输入间歇失败](2026-06-03-continuous-input-toggle-race.md)
- [双声道录音被错误当作 mono 导致变音](2026-05-27-stereo-recording-channel-collapse.md)
- [剪贴板上屏粘贴旧内容](2026-05-27-clipboard-restore-race.md)
- [全局快捷键有事件但前端监听不到](2026-05-27-tauri-event-listen-capability.md)
- [Windows 透明浮窗白/灰边框](2026-05-28-windows-overlay-white-border.md)

## 使用规则

- 新增复杂 bug 时，不要只写最终修复，要写清楚排除过的错误假设。
- 如果问题跨 Rust/Tauri/React/Windows/第三方 API 边界，必须记录边界处的数据或状态如何流动。
- 如果修复依赖真实桌面验证，也要写明自动测试覆盖不到的部分。
- 同一问题反复修复三轮仍未稳定时，必须在这里留下文档；不要只把经验留在聊天记录或临时日志里。
