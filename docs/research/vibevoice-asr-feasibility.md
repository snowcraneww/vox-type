# VibeVoice / VibeVoice-ASR 可行性调研

日期：2026-06-05
状态：当前路线放弃，不纳入 V13/V14 规划

## 结论

VibeVoice 不是一个适合 VoxType 当前路线的模型选项。维护者已决定直接放弃 VibeVoice，不把它放入 V13 或后续近期路线图。

原因很直接：

- `microsoft/VibeVoice-1.5B`、`microsoft/VibeVoice-Realtime-0.5B` 主要是 Text-to-Speech / 语音生成，不是 VoxType 需要的 ASR 主链路。
- `microsoft/VibeVoice-ASR` / `microsoft/VibeVoice-ASR-HF` 虽然是 ASR，但定位更偏长音频结构化转写，支持说话人、时间戳、热词、50+ 语言和最长约 60 分钟单段音频。
- VoxType 当前核心是短句、低延迟、桌面输入法式上屏。VibeVoice-ASR 的模型规模、依赖栈和硬件预期都不适合无 GPU 日常输入。

这份文档只保留为研究记录。除非未来产品方向明确扩展到“长音频转写 / 会议记录 / 播客结构化转写”，否则不重新规划 VibeVoice。

## 为什么不适合当前路线

V13 的目标是 clipboard-first release readiness：把已经稳定的语音输入闭环做成可发布、可验证、低风险版本。VibeVoice-ASR 是重型 Transformers/PyTorch 路线，集成风险明显高于当前 whisper.cpp / SenseVoice / 百度 ASR 路线。

VoxType 当前日常场景是短句、低延迟、桌面输入法式上屏。VibeVoice-ASR 的强项是长音频、一小时上下文、说话人分离和结构化转写，更像会议记录/播客转写模型。

## 机器配置判断

公开资料给出的硬件信号并不完全一致，但方向清楚：

- 标准 VibeVoice-ASR 约 9B 级别，社区反馈大约需要 18GB VRAM 才能较顺畅运行。
- FP16 / FP4 或 4-bit 量化版本可能降到约 12-16GB VRAM。
- CPU-only 理论上可能通过 C++/ggml 社区移植运行，但这不是官方主路径，速度和稳定性未知，不适合作为产品能力承诺。
- 没有独显或只有普通办公本 CPU 的机器，不应期待它作为实时语音输入模型可用。

对 VoxType 的保守判断：

- 日常实时/准实时输入：不采用 VibeVoice-ASR。
- 离线长音频转写实验：至少 16GB RAM；推荐 NVIDIA GPU 12GB+ VRAM；更稳是 16-24GB VRAM。
- CPU-only：不作为近期研究或产品承诺。

## 与当前模型的关系

| 模型 | 当前适配价值 | 适合场景 | 决策 |
| --- | --- | --- | --- |
| whisper.cpp | 已有本地路线 | 本地短句/通用识别 | 保留 |
| SenseVoice Small | 已接入 | 中文优先短句识别 | 保留 |
| 百度短语音 / 实时 WebSocket | 已接入 | 云端准确率/实时连续输入 | 保留 |
| VibeVoice TTS | 不适合主链路 | 文本转语音/播客生成 | 放弃 |
| VibeVoice-ASR | 与 ASR 相关但过重 | 长音频结构化转写、说话人/时间戳 | 当前路线放弃 |

## 后续触发条件

只有满足以下条件之一，才重新打开 VibeVoice 研究：

1. VoxType 明确增加长音频转写或会议记录方向。
2. VibeVoice-ASR 出现成熟、低成本、Windows 友好的 CPU/低显存 runtime。
3. 有真实目标机器证明它能在无 GPU 或普通办公本上达到可接受延迟。

否则不要把它加入模型选择、默认安装器、V13/V14 planning 或发布检查清单。

## 资料来源

- Microsoft VibeVoice 项目页：https://microsoft.github.io/VibeVoice
- Microsoft/VibeVoice GitHub：https://github.com/microsoft/VibeVoice
- Hugging Face `microsoft/VibeVoice-ASR`：https://huggingface.co/microsoft/VibeVoice-ASR
- Hugging Face `microsoft/VibeVoice-ASR-HF`：https://huggingface.co/microsoft/VibeVoice-ASR-HF
- Microsoft TechCommunity VibeVoice ASR 介绍：https://techcommunity.microsoft.com/blog/azure-ai-foundry-blog/introducing-vibevoice-asr-longform-structured-speech-recognition-at-scale/4501276
- Microsoft Foundry model catalog: https://ai.azure.com/catalog/models/microsoft-vibevoice-asr-hf
