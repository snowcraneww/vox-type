# 桌面兼容性矩阵

本矩阵由 V12 建立模板和测试规则，真实发布闸门放到 V13。V13 的基线是 clipboard-first：`Clipboard` 和 `Auto 安全` 必须稳定；`SendInput 实验` 只保留为可选兼容性证据，不作为发布通过条件。

## 如何测试

运行桌面应用：

```bash
npm run tauri -- dev
```

对每个目标应用执行：

1. 打开 `Settings -> 输入 -> 上屏策略`。
2. 选择 `Clipboard`，分别测试 `Ctrl+Alt+Space` 和 `Ctrl+Alt+V`，确认目标窗口文本与识别记录一致。
3. 选择 `Auto 安全`，重复两种快捷键，确认 metadata 显示 clipboard policy，目标窗口文本与识别记录一致。
4. 可选：选择 `SendInput 实验` 做兼容性观察。若目标文本与识别记录不一致，记录为 SendInput 不兼容，不阻塞 clipboard-first 发布。
5. 记录失败类型：未上屏、上错目标、乱码、重复字符、焦点丢失、权限边界、实验策略不兼容。

## 兼容性矩阵

| 目标应用 | 目标类型 | Clipboard | Auto 安全 | SendInput 实验 | 注意事项 | 证据日期 |
| --- | --- | --- | --- | --- | --- | --- |
| VS Code 编辑器 | Electron 文本编辑器 | 待 V13 桌面验证 | 待 V13 桌面验证 | 可选观察 | 主要日常目标。建议分别测试普通编辑器和终端输入。 | 待验证 |
| Notepad | 原生 Win32 文本框 | 维护者已反馈 Auto 安全通过 clipboard 稳定上屏 | 维护者已反馈 Auto 安全通过 clipboard 稳定上屏 | 已知不可靠：V11 中目标文本曾与识别记录不一致 | 原生文本框基线。SendInput 在这里失败，是 Auto 改为全量 clipboard 的原因。 | 2026-06-05 |
| 浏览器输入框 | Chromium / Edge Web 输入框 | 待 V13 桌面验证 | 待 V13 桌面验证 | 可选观察 | 如有时间，分别测试地址栏和普通 textarea。 | 待验证 |
| IME 重度聊天目标 | 微信或类似聊天输入框 | 待 V13 桌面验证 | 待 V13 桌面验证 | 可选观察 | 重点观察中文输入法组合、焦点和粘贴行为。 | 待验证 |
| 提权或权限边界目标 | 管理员应用或权限边界窗口 | 待 V13 桌面验证 | 待 V13 桌面验证 | 可选观察 | 预期可能暴露 Windows integrity level 限制；不要绕过 UAC。 | 待验证 |

## TSF 决策门槛

V12 不实现 TSF，也不再要求 SendInput 成为可靠路径。V13 根据真实桌面证据做决策：

- 如果 `Clipboard` 和 `Auto 安全` 覆盖维护者主要目标，V13 可以保持 clipboard-first 发布范围。
- 如果 clipboard-first 在关键目标里无法接受，再规划 TSF / Windows Text Services 独立 spike。
- 如果只有 `SendInput 实验` 失败，记录为实验策略限制，不阻塞 V13。
- 如果只有提权目标失败，记录为 Windows permission boundary，而不是默认产品 blocker。