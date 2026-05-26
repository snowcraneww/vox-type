# VoxType 候选仓库调研记录

更新时间：2026-05-26

本地克隆放在 `TMP/research/repos/`，该目录被 Git 忽略。这些仓库只用于阅读和比较，不能把第三方源码复制进 VoxType。

## 深读项目

| 项目 | 上游地址 | 本地路径 | 许可证 | 技术栈 | 相关性 | 可借鉴点 | 风险 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Handy | https://github.com/cjpais/Handy | `TMP/research/repos/Handy` | MIT | Rust、Tauri、React | 很高 | 离线优先、`cpal` 录音、VAD、本地 ASR、快捷键设置、多种粘贴策略 | 功能较成熟，设置复杂度高，不能照搬全部范围 |
| OpenLess | https://github.com/Open-Less/openless | `TMP/research/repos/openless` | MIT | Rust、Tauri 2、React/TS、Windows native helper | 很高 | 最接近按住说话再上屏；有 TSF、`SendInput(KEYEVENTF_UNICODE)`、剪贴板 fallback、防抖、流式插入边界处理 | TSF 路径复杂，不适合作为第一阶段默认实现 |
| OpenWhispr | https://github.com/OpenWhispr/openwhispr | `TMP/research/repos/openwhispr` | MIT | Electron、React、TypeScript | 中 | 可参考更完整语音产品、local/cloud provider 和 agent 工作流 | 产品范围太大，运行时太重，容易偏离 VoxType MVP |
| VoiceFlow | https://github.com/infiniV/VoiceFlow | `TMP/research/repos/VoiceFlow` | MIT | Python/Pyloid、React/Vite | 中高 | `faster-whisper`、长按热键、光标处粘贴、GPU fallback | Python 桌面分发和依赖维护风险较高 |
| whisper-writer | https://github.com/savbell/whisper-writer | `TMP/research/repos/whisper-writer` | README 提到 GPL，复用前需核实 LICENSE | Python、PyQt5 | 中 | 最小语音输入链路：快捷键、录音模式、faster-whisper/API、输入模拟 | 许可证和依赖老化，不适合作为代码来源 |
| tambourine-voice | https://github.com/kstonekuan/tambourine-voice | `TMP/research/repos/tambourine-voice` | AGPL-3.0 | Tauri、Rust、Python server、Pipecat | 中 | provider 架构和 LLM 后处理思路 | AGPL 不适合 Apache-2.0 项目直接复用，只能概念参考 |
| faster-whisper-dictation | https://github.com/bhargavchippada/faster-whisper-dictation | `TMP/research/repos/faster-whisper-dictation` | MIT | Python CLI/daemon | 高 | VAD、防抖、预录音缓冲、剪贴板锁、测试覆盖 | 不是桌面产品壳，Python 不适合作为长期核心 |

## 未完整克隆但已验证的生态参考

| 项目 | URL | 许可证 | 技术栈 | 为什么重要 |
| --- | --- | --- | --- | --- |
| whisper.cpp | https://github.com/ggml-org/whisper.cpp | MIT | C/C++ | 成熟本地 Whisper 运行时。适合作为本地 ASR 基线，但长期不建议只靠 `whisper-cli` 子进程边界。 |
| sherpa-onnx | https://github.com/k2-fsa/sherpa-onnx | Apache-2.0 | C++/ONNX Runtime，多语言绑定 | 许可证兼容，支持本地 ASR/VAD/TTS 和 Windows，是重点候选引擎。 |
| local-whisper | https://github.com/t2o2/local-whisper | MIT | Swift/macOS | 小型离线听写应用，适合参考产品体验，不适合直接参考 Windows 实现。 |
| FluidAudio | https://github.com/FluidInference/FluidAudio | Apache-2.0 | Swift/CoreML | 展示本地音频模型、VAD、ASR、说话人能力方向，主要用于未来 macOS 参考。 |
| FluidVoice | https://github.com/altic-dev/FluidVoice | GPL-3.0 | Swift/macOS | 离线语音输入产品参考。GPL 项目只做概念参考。 |
| Whispering | 搜索结果提到 local-first 开源听写应用，但 `epicenter-so/whispering` 在 2026-05-26 不能通过 GitHub API 解析 | 未确认 | 未确认 | 只作为未验证产品信号，不作为架构证据。 |

## 跨项目共性

- Push-to-talk 必须做防抖。OpenLess 和 faster-whisper-dictation 都处理了按键重复或释放抖动问题。
- 音频最好统一成 16 kHz mono PCM 再交给 ASR。
- 预录音 ring buffer 或静音裁剪能避免开头音节被截断。
- 剪贴板上屏实现快，但必须恢复原剪贴板，并避免并发破坏剪贴板。
- Windows Unicode `SendInput(KEYEVENTF_UNICODE)` 是 TSF 之前很有价值的阶段。
- 完整 TSF 上屏更接近真正输入法，但不适合作为第一版门槛。
- 浮窗可能抢焦点，状态 UI 必须谨慎。

