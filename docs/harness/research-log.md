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

## VoxType 产品与开源项目调研

### 会话 002 - 2026-05-26

- 原始需求来源：`docs/plans/原始需求.md`。
- 中间调研资料：`TMP/research/requirements-brief.md`、`TMP/research/repo-candidates.md`、`TMP/research/solution-analysis.md`。
- 正式调研资料：`docs/research/requirements-brief.md`、`docs/research/open-source-landscape.md`、`docs/research/technical-options.md`。
- 本地深读仓库目录：`TMP/research/repos/`，该目录被 Git 忽略，只作为本机分析资料。

深读项目：

- Handy: https://github.com/cjpais/Handy
- OpenLess: https://github.com/Open-Less/openless
- OpenWhispr: https://github.com/OpenWhispr/openwhispr
- VoiceFlow: https://github.com/infiniV/VoiceFlow
- whisper-writer: https://github.com/savbell/whisper-writer
- tambourine-voice: https://github.com/kstonekuan/tambourine-voice
- faster-whisper-dictation: https://github.com/bhargavchippada/faster-whisper-dictation

生态参考：

- whisper.cpp: https://github.com/ggml-org/whisper.cpp
- sherpa-onnx: https://github.com/k2-fsa/sherpa-onnx
- local-whisper: https://github.com/t2o2/local-whisper
- FluidAudio: https://github.com/FluidInference/FluidAudio
- FluidVoice: https://github.com/altic-dev/FluidVoice

当前结论：Rust + Tauri 2 + React/TS 是推荐主线；MVP 优先验证按住说话、松开本地转写、上屏到当前光标。完整 TSF IME、会议转写和 AI agent 工作流后置。

## Windows 原生桌面浮窗调研

### 会话 2026-05-28

- 问题：当前 Tauri/WebView2 `dictation-overlay` 已经配置透明和无边框，但维护者在 Windows 桌面仍能看到黑色胶囊外侧的白色或灰色矩形边框。
- 中间调研资料：`TMP/research/native-overlay-spike.md`。
- 正式设计文档：`docs/superpowers/specs/2026-05-28-v4-native-overlay-design.md`。
- 实施计划：`docs/superpowers/plans/2026-05-28-v4-native-overlay.md`。

关键来源：

- Tauri issue `#11654`：Windows 透明窗口边框异常，有用户通过 `shadow: false` 解决。
- Tauri v2 config / WebviewWindow API：确认存在 `shadow`、`setShadow`、`setBackgroundColor` 等配置或 API，但平台行为有差异。
- Tauri issue `#12450`：`WebviewWindowBuilder::transparent(true)` 在 Windows child window 场景下可能不按预期透明。
- StackOverflow `Tauri transparent window only works when resized`：透明窗口可能 resize 后才正常，说明合成时机可能影响结果。
- Rust 用户论坛：Windows 透明 overlay 常见路线是 `WS_EX_LAYERED` 和 per-pixel alpha；点击穿透可评估 `WS_EX_TRANSPARENT`。

当前结论：下一版只解决 overlay 外层白/灰框。先做低风险 Tauri `shadow: false` 和窗口刷新验证；如果维护者仍能看到白/灰框，再做 Windows-only 原生 layered overlay 原型，并保留 WebView overlay fallback。
