# SendInput Unicode / VK_PACKET repeated final character

日期：2026-06-05范围：V11 上屏策略，Windows SendInput / Auto

## 现象

维护者在 Notepad 里测试英文 `test i have a headache`。识别记录里文本正确，但目标输入框里显示为 `test eeeeeeeeeeeeeeeee`。中文测试时也出现过目标窗口重复最后一个字符的现象。

这个 bug 不是 ASR、SenseVoice、模型路由或识别记录持久化问题：记录文本一直正确，只有目标窗口中的上屏文本错了。

## 为什么复杂

这个问题连续三轮修复才收敛到真正原因：

1. 第一轮修复了 Win32 `INPUT` union 大小不对导致的 `SendInput failed: 0/16`。这只解决了 API 硬失败，没有解决目标窗口文本错误。
2. 第二轮把整段 Unicode 事件改成每个 UTF-16 code unit 单独发送，但维护者复测仍然重复最后的 `e`。这证伪了“整段批量提交”这个假设。
3. 第三轮查公开资料后确认：`KEYEVENTF_UNICODE` 走的是 `VK_PACKET` / `WM_CHAR` 路径，不等同于真实键盘虚拟键输入。Win11 Notepad 和一些目标控件对这类自动化文本输入存在兼容性问题。

## 根因

真正的根因不是提交批量大小，而是 VoxType 把英文 ASCII 文本也通过 `KEYEVENTF_UNICODE` 发送。官方文档说明，设置 `KEYEVENTF_UNICODE` 时 `wVk` 必须为 0，`wScan` 是 Unicode 字符，系统通过 `VK_PACKET` / `TranslateMessage` 生成 `WM_CHAR`。这条路径对某些目标控件不可靠，可能 API 返回成功但目标文本错误。

网上相关资料：

- Microsoft Learn `KEYBDINPUT`：`KEYEVENTF_UNICODE` 指定时 `wScan` 是 Unicode 字符，并通过 `TranslateMessage` 生成 `WM_CHAR`。
- Microsoft Learn `VkKeyScanW`：可将字符转成当前键盘布局下的 virtual-key code 和 shift state。
- AutoHotkey 社区多个帖子记录 Win11 Notepad 对 SendInput / automation 文本输入的兼容性问题，常见建议是改用其他发送模式或 clipboard。

## 修复

`src-tauri/src/insertion/mod.rs` 现在将 SendInput 分成两条路径：

- ASCII 文本：使用 `VkKeyScanW` 映射到 virtual-key code 和 Shift/Ctrl/Alt state，再发送普通 key down / key up 事件。这避开 `KEYEVENTF_UNICODE` / `VK_PACKET`。
- 非 ASCII 文本：在 `Auto` 中继续保守地走 clipboard。如果用户手动选择 `SendInput` 输入非 ASCII，则返回明确错误而不假装成功。

## 验证

已完成的自动验证：

```bash
cargo check --manifest-path src-tauri/Cargo.toml --lib
cargo test --manifest-path src-tauri/Cargo.toml insertion --no-run
```

需要维护者真实桌面复测：

1. Settings -> 输入 -> 上屏策略 -> `SendInput`。
2. 在 Notepad 中说 `test i have a headache`。
3. 期望目标文本与识别记录一致，不再重复最后的 `e`。
4. 同样用 `Auto` 测一次英文。
5. 中文在 `Auto` 下应继续显示 `auto -> clipboard`。

## 经验

- `SendInput` 返回成功只表示系统接收了事件，不表示目标控件中的文本一定正确。
- `KEYEVENTF_UNICODE` 不是“更稳的打字”，它是 `VK_PACKET` / `WM_CHAR` 路径，需要用真实目标窗口验证。
- 对于中文语音输入法，clipboard 仍是当前最稳的保守路径。
- 一个问题被修了两次还被用户复测证伪时，第三轮必须先查官方和社区资料，再改代码。


## 追加：修饰键状态导致英文仍不一致

维护者复测 ASCII virtual-key 路径后，重复 `e` 的问题消失，但目标英文文本仍与识别记录不完全一致。进一步判断为快捷键触发链路中的修饰键状态风险：`Ctrl+Alt+...` 触发后，直接输入可能在 Ctrl/Alt/Shift/Win 仍处于物理按下或目标线程逻辑按下状态时开始，导致普通 virtual-key 事件被目标解释为组合键或变体输入。

最终尝试是在 SendInput 打字前增加修饰键清理：用 `GetAsyncKeyState` 最多等待 400ms，让 Shift/Ctrl/Alt/Win 物理释放，然后发送这些修饰键的 key-up，再输入 ASCII virtual-key down/up。

如果这轮真实桌面复测仍不能做到目标文本与识别记录逐字一致，结论就是：当前 SendInput 方案不能作为 VoxType 的可靠自动策略。后续应把 `Auto` 降级为全量 clipboard，`SendInput` 只保留为显式实验策略，直到 TSF 或更深层 Windows 文本服务方案进入单独版本。


## 最终结论：Auto 全量 clipboard

最终修饰键清理后，维护者复测 `i have a headache`，识别记录正确，但 Notepad 上屏为 `I have a eaache`。这说明当前 SendInput 路径即使避开 `KEYEVENTF_UNICODE`、改用 virtual-key、并清理修饰键，也仍不能保证目标文本逐字等于识别记录。

产品结论：V11 不再继续修 SendInput。`Auto` 全量降级为 clipboard，并记录 `auto_clipboard_policy` metadata；`SendInput` 保留为 `SendInput 实验`，只用于显式兼容性测试，不进入默认可靠路径。

后续如果还要做直接文本上屏，应进入单独版本做 TSF 或 Windows 文本服务可行性研究，而不是继续在 SendInput 上叠补丁。
