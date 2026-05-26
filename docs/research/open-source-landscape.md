# 开源项目调研

更新时间：2026-05-26

调研方法：阅读原始需求，使用 Tavily 和 GitHub API 搜索相关项目，查询 GitHub 元数据，并深读 `TMP/research/repos/` 下的本地克隆。中间记录保存在 `TMP/research/`。

## 最重要的参考项目

| 项目 | URL | 许可证 | 技术栈 | 对 VoxType 的价值 |
| --- | --- | --- | --- | --- |
| Handy | https://github.com/cjpais/Handy | MIT | Rust、Tauri、React | 很适合作为本地语音输入应用的架构参考，覆盖录音、VAD、本地模型、设置页和多种粘贴策略。 |
| OpenLess | https://github.com/Open-Less/openless | MIT | Rust、Tauri 2、React/TS、Windows native helper | 和 VoxType 的核心交互最接近。重点参考 Windows 上屏策略：TSF、`SendInput(KEYEVENTF_UNICODE)`、剪贴板 fallback、快捷键防抖。 |
| faster-whisper-dictation | https://github.com/bhargavchippada/faster-whisper-dictation | MIT | Python daemon/CLI | 适合学习底层链路：VAD、预录音缓冲、快捷键防抖、剪贴板锁和测试覆盖。 |
| VoiceFlow | https://github.com/infiniV/VoiceFlow | MIT | Python/Pyloid、React/Vite | 适合参考快速做出本地转写体验和 GPU fallback，但不适合作为长期主架构。 |
| OpenWhispr | https://github.com/OpenWhispr/openwhispr | MIT | Electron、React、TypeScript | 可参考更完整的语音产品形态，但范围明显大于 VoxType MVP。 |
| whisper-writer | https://github.com/savbell/whisper-writer | README 提到 GPL，复用前需核实 | Python、PyQt5 | 极简语音输入链路参考。许可证和依赖老化风险较高，只做概念参考。 |
| tambourine-voice | https://github.com/kstonekuan/tambourine-voice | AGPL-3.0 | Tauri、Rust、Python server | Provider 架构和后处理思路有参考价值，但 AGPL 不适合直接复用到 Apache-2.0 项目。 |

## 引擎和平台参考

| 项目 | URL | 许可证 | 说明 |
| --- | --- | --- | --- |
| whisper.cpp | https://github.com/ggml-org/whisper.cpp | MIT | 成熟的本地 Whisper C/C++ 推理实现。可作为本地 ASR 基线或 fallback。 |
| sherpa-onnx | https://github.com/k2-fsa/sherpa-onnx | Apache-2.0 | 许可证兼容，支持 ASR/VAD/TTS，覆盖 Windows 和多语言绑定，是值得重点评估的本地引擎候选。 |
| local-whisper | https://github.com/t2o2/local-whisper | MIT | macOS 离线语音输入产品，可参考“热键输入到任意应用”的产品体验。 |
| FluidAudio | https://github.com/FluidInference/FluidAudio | Apache-2.0 | Apple/CoreML 音频模型库，对未来 macOS 方向有参考意义。 |
| FluidVoice | https://github.com/altic-dev/FluidVoice | GPL-3.0 | macOS 离线语音输入产品参考。GPL 项目只做概念参考。 |

## 对 VoxType 的结论

- 难点不只是语音识别，而是完整闭环：快捷键、焦点、录音、识别、上屏、反馈、失败恢复。
- Windows 上屏要分阶段做。第一阶段用剪贴板粘贴并恢复，第二阶段做 Unicode `SendInput`，第三阶段再评估 TSF。
- 快捷键防抖、预录音缓冲、静音裁剪这些细节会显著影响体验。
- 状态 UI 必须避免抢焦点，否则会破坏“输入到当前光标”的核心能力。
- 第一版要收窄范围。很多语音产品很快会膨胀到会议、agent、格式化、provider 市场，VoxType 先不要走这个方向。

## 许可证边界

VoxType 使用 Apache-2.0。MIT 和 Apache-2.0 项目可以作为更直接的实现参考，但仍要保留依赖和归属记录。GPL/AGPL 项目只能作为产品和架构思路参考，不能复制源码或形成派生实现。

