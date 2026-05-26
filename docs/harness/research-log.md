# Research Log

## Harness Engineering

### Learn Harness Engineering

- URL: https://walkinglabs.github.io/learn-harness-engineering/zh/
- GitHub: https://github.com/walkinglabs/learn-harness-engineering
- 学习状态：已读取 12 讲课程正文摘要、官方模板清单和关键模板内容。
- 本项目映射：见 `docs/harness/lesson-synthesis.md`。

课程主题：

1. 模型能力强，不等于执行可靠。
2. Harness 到底是什么。
3. 让代码仓库成为唯一的事实来源。
4. 把指令拆分到不同文件里。
5. 让跨会话的任务保持上下文连续。
6. 让 agent 每次工作前先初始化。
7. 给 agent 划清每次任务的边界。
8. 用功能清单约束 agent 该做什么。
9. 防止 agent 提前宣告完成。
10. 跑通完整流程才算真正验证。
11. 让 agent 的运行过程可观测。
12. 每次会话结束前都做好交接。

### OpenAI Harness Engineering

- URL: https://openai.com/index/harness-engineering
- 搜索结论：OpenAI 将 repository knowledge 作为 system of record，并强调 agent legibility、execution plans、review/verification loops。
- 本项目映射：把 `docs/harness/`、`docs/research/`、`openspec/` 作为长期事实源。

### Agent Harness 资料

- LangChain: https://www.langchain.com/blog/the-anatomy-of-an-agent-harness
- O'Reilly Radar: https://www.oreilly.com/radar/agent-harness-engineering
- Datadog: https://www.datadoghq.com/blog/ai/harness-first-agents
- 交叉结论：有效 harness 需要上下文管理、工具执行、沙箱/权限、验证反馈、可观测性和恢复路径。

## VoxType 后续调研待办

- Windows 语音输入/文本注入相关开源项目。
- Whisper.cpp、faster-whisper、ONNX Runtime、Vosk 等本地 ASR 方案。
- Tauri、native Rust、Electron、.NET/WPF 等桌面壳方案。
- Windows SendInput、UI Automation、IME TSF 相关实现和限制。
- 开源语音输入法的许可证和可借鉴边界。
