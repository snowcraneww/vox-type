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

## MiniMax ASR 官方 endpoint 调研

### 会话 2026-05-29

目标：为 V6 Task 6 确认 MiniMax 官方 ASR / Speech-to-Text endpoint、认证头、音频上传字段、音频格式和返回文本字段。

检索查询：

- `MiniMax official speech to text ASR API documentation endpoint`
- `site:minimax.io ASR speech recognition API MiniMax`
- `site:platform.minimaxi.com speech to text API`

查到的官方来源：

- MiniMax API Overview: https://platform.minimax.io/docs/api-reference/api-overview
- Text to Speech (T2A) HTTP: https://platform.minimax.io/docs/api-reference/speech-t2a-http
- 中文开放平台异步语音合成: https://platform.minimaxi.com/docs/guides/speech-t2a-async
- 中文开放平台同步语音合成 HTTP: https://platform.minimaxi.com/docs/api-reference/speech-t2a-http
- 中文开放平台同步语音合成 WebSocket: https://platform.minimaxi.com/docs/api-reference/speech-t2a-websocket

结论：本轮只确认到官方 TTS / T2A 文档和通用 API overview，没有找到官方 ASR / Speech-to-Text 文档页，也没有可靠确认 ASR endpoint、上传字段、音频格式和返回文本字段。Tavily 第三方结果中出现的 ASR endpoint 描述不作为实现依据。

处理决定：V6 当前只保留 MiniMax 配置数据结构和 UI，不实现真实 MiniMax ASR 调用；Task 6 保持未完成，直到拿到官方 ASR 文档或维护者提供 endpoint 资料。

### 会话 2026-06-01 Token Plan / MiniMax CLI 复核

目标：复核维护者提供的 Token Plan 和 MiniMax CLI 官方入口，确认 Token Plan 中的 `Speech` 是否包含可用于 VoxType 听写的 ASR / Speech-to-Text 模型，以及是否存在官方调用方式。

维护者提供入口：

- Token Plan: https://platform.minimaxi.com/subscribe/token-plan
- MiniMax CLI: https://platform.minimaxi.com/docs/token-plan/minimax-cli

补充查到的官方来源：

- Token Plan Overview: https://platform.minimax.io/docs/token-plan/intro
- MiniMax CLI Guide: https://platform.minimax.io/docs/token-plan/minimax-cli
- MiniMax API Overview: https://platform.minimax.io/docs/api-reference/api-overview
- Text to Speech (T2A) HTTP: https://platform.minimax.io/docs/api-reference/speech-t2a-http
- 官方 CLI 仓库: https://github.com/MiniMax-AI/cli

官方证据：

- Token Plan Overview 说明 Token Plan Key 可调用非 LLM 能力 `Speech, Image, Video, Music`，并引导通过 MiniMax CLI 使用。
- MiniMax CLI 文档中的 speech 示例是 `mmx speech synthesize --text ...`，语义为语音合成。
- 官方 T2A 文档 endpoint 是 `https://api.minimax.io/v1/t2a_v2`，模型示例包括 `speech-2.8-hd`、`speech-2.8-turbo`、`speech-2.6-hd`、`speech-2.6-turbo`、`speech-02-hd`、`speech-02-turbo`、`speech-01-hd`、`speech-01-turbo`，均为 Text-to-Speech 路径。
- 官方 CLI 仓库中 speech 相关实现只确认到 `speech synthesize` 和 `speech voices`；源码路径包括 `src/commands/speech/synthesize.ts`、`src/commands/speech/voices.ts`、`src/sdk/speech/index.ts`。未找到 `asr`、`stt`、`transcribe`、`recognition` 等命令或 SDK 方法。

结论：Token Plan 当前官方文档确实包含 `Speech` 能力，但已确认的调用路径是 TTS / 语音合成，不是 ASR / Speech-to-Text。没有找到可作为 VoxType 实现依据的官方 ASR 模型名、endpoint、上传字段、音频格式或返回文本字段。

处理决定：V6 不切换到 MiniMax ASR；MiniMax 配置页继续只作为云端识别配置占位和 Key 管理入口。若后续要启用 MiniMax ASR，需要官方 ASR 文档，或维护者明确接受基于非官方接口做实验集成的风险。

## 百度 ASR 官方接入调研

### 会话 2026-06-01

目标：根据维护者提供的百度智能云语音技术文档，确认是否存在可用于 VoxType 听写的 ASR API，并确定本轮配置页面应支持哪些字段。

维护者提供入口：

- 免费测试资源: https://cloud.baidu.com/doc/SPEECH/s/Wl9mh4doe
- 短语音识别标准版 API: https://cloud.baidu.com/doc/SPEECH/s/Jlbxdezuf
- 实时语音识别 WebSocket API: https://cloud.baidu.com/doc/SPEECH/s/jlbxejt2i

官方证据：

- 短语音识别标准版可将 60 秒以内语音识别为文字，适用于语音输入、语音交互、语音指令和语音搜索等短语音场景。
- 短语音识别请求地址为 `POST http://vop.baidu.com/server_api`。
- 音频支持 `pcm`、`wav`、`amr`、`m4a`，推荐 `pcm`；采样率支持 `16000` 和 `8000`，声道固定单声道，编码为 16bit。
- JSON 上传字段包括 `format`、`rate`、`channel`、`cuid`、`token` 或 API Key 鉴权、`dev_pid`、`speech` 和 `len`。
- 成功返回 JSON 中的 `result` 数组，通常取第一个候选文本。
- 实时语音识别 WebSocket API 使用 `wss://vop.baidu.com/realtime_asr?sn=...`，适合后续真正流式识别，不适合作为当前停止后整段转写链路的第一步。

处理决定：V6 增量支持百度短语音识别 provider 的配置和选择，默认 endpoint 为 `http://vop.baidu.com/server_api`、`dev_pid` 为 `1537`、`cuid` 为 `voxtype-local`、format 为 `pcm`、sample rate 为 `16000`。真实百度 ASR 调用不在本轮启用，后续应在独立任务中实现请求构造、鉴权验证、响应解析和端到端上屏验证。

详细接入记录：`docs/integrations/baidu-asr.md`。
