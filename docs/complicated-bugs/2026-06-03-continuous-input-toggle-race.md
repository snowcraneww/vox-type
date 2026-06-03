# Ctrl+Alt+V 连续输入间歇失败

日期：2026-06-03  
相关范围：全局快捷键、Tauri event、React 状态机、连续输入、桌面浮窗

## 现象

维护者反馈 `Ctrl+Alt+V` 连续输入有时成功，有时失败。失败时常见表现是：

- 快捷键确实生效，桌面浮窗动效出现。
- 但没有文字上屏。
- 识别记录中也没有新增记录。
- 按住说话 `Ctrl+Alt+Space` 有时仍能正常工作。

这个问题是 2026-06-03 调整连续输入状态机后出现的回归；之前连续输入没有这种间歇性失败。

## 为什么复杂

这个问题同时跨了三层状态：

- Rust 全局快捷键状态机判断 `Ctrl+Alt+V` 是开始还是停止。
- React 前端用 `isRecordingRef.current` 判断当前是否真实录音。
- Rust overlay 会在收到热键后预先显示浮窗，即使前端还没有真正开始录音。

因此用户看到“浮窗出现”并不等于“前端录音链路已经启动”。这会制造假阳性视觉反馈，让问题看起来像 ASR 或上屏失败，实际是状态机和事件时序错位。

## 错误假设

排查过程中曾经尝试只在前端做自愈：收到 `toggleStopAndTranscribe` 但前端 idle 时，改为启动录音；收到 `toggleStartRecording` 但前端 recording 时，改为停止录音。

这个修法只能缓解部分错位，不能消除根因，因为 Rust 仍然维护另一份 toggle 状态，并且仍会提前显示 overlay。

## 根因

根因分两层。

第一层：Rust 和前端都试图拥有 `Ctrl+Alt+V` 的 toggle 权威状态。只要某次前端 start 被忽略、失败或延迟，Rust 的下一次按键语义就可能与前端实际录音状态相反。

第二层：把 toggle 权威移到前端后，又引入了异步竞态。`startRecording()` 和 stop/transcribe 都是异步操作。如果连续按键、重复 pressed 事件或 OS 快捷键事件过快到达，前端可能在上一轮 start/stop 尚未 settle 时再次处理 toggle。

此外，Rust 仍然预先显示 overlay，导致即使前端没有进入录音链路，用户也能看到浮窗。

## 修复

最终修复采用三条约束：

1. Rust 不再维护 `Ctrl+Alt+V` 开始/停止状态。`ToggleDictationState` 改为无状态，每次按下只发一个前端 toggle 信号。
2. React 前端成为连续输入 start/stop 的唯一权威，基于真实 `isRecordingRef.current` 判断本次按键是开始还是停止。
3. 连续输入 overlay 由前端在真实 start 成功后显示。Rust 不再为 `Ctrl+Alt+V` 预先显示录音或转写浮窗。

核心代码位置：

- `src-tauri/src/hotkey.rs`：`ToggleDictationState` 无状态化。
- `src-tauri/src/lib.rs`：Rust overlay 预显示只保留给 `Ctrl+Alt+Space`，不再处理 toggle overlay。
- `src/App.tsx`：前端根据 `isRecordingRef.current` 执行连续输入 start/stop；新增 `toggleActionInFlightRef` 串行化连续输入 toggle。
- `src/App.test.tsx`：新增重复 toggle、前端拥有 toggle 状态、连续输入 overlay 由前端启动的回归测试。

## 验证

已运行：

```bash
npm test -- --run src/App.test.tsx -t "toggle|realtime"
npm test -- --run src/App.test.tsx
npm run typecheck
cargo check --manifest-path src-tauri/Cargo.toml --lib
cargo test --manifest-path src-tauri/Cargo.toml hotkey --no-run
python -m json.tool docs/harness/feature_list.json
git diff --check
```

其中 `App.test.tsx` 通过 30 个测试。`git diff --check` 只有既有 CRLF/LF 提示。

## 后续经验

- 同一个用户动作只能有一个权威状态机。`Ctrl+Alt+V` 的真实录音状态应由前端拥有，因为前端知道录音 start/stop 是否真的成功。
- overlay 不应只表示“收到热键”，应表示“实际进入对应运行状态”。否则用户会被假动效误导。
- 对异步 start/stop 操作必须有 in-flight guard，避免重复快捷键事件打乱状态。
- 回归测试要模拟重复 pressed 事件和 start promise 未 resolve 的情况，不能只测理想 start/stop 顺序。
