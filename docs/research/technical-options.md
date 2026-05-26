# 技术选项对比

更新时间：2026-05-26

## 推荐方向

主项目建议采用 Rust + Tauri 2 + React/TypeScript。Rust 负责快捷键、录音、ASR 集成、文本上屏、配置和状态机；React 负责设置页、状态展示和非关键 UI。

这个方向和原始草稿一致，也和 Handy、OpenLess、faster-whisper-dictation 的调研结论相符。

## MVP 主流程

```mermaid
flowchart LR
    A[按住全局快捷键] --> B[录制麦克风]
    B --> C[统一音频格式]
    C --> D[本地 ASR]
    D --> E[轻量文本后处理]
    E --> F[输入到当前应用]
    F --> G[显示成功或失败]
```

## 组件选择

| 组件 | MVP 建议 | 后续升级 | 理由 |
| --- | --- | --- | --- |
| 桌面壳 | Tauri 2 + React/TS | 继续沿用 | 体积小，Rust 调 Windows native API 方便。 |
| 音频采集 | Rust `cpal` | 增加设备管理和录音 watchdog | Rust/Tauri 同类项目已有实践。 |
| 音频格式 | 16 kHz mono PCM | 支持流式 chunk | ASR 常用边界，便于测试。 |
| VAD | 最小时长 + 静音裁剪 | Silero/ONNX VAD + 预录音缓冲 | 第一版先别被 VAD 模型复杂度卡住。 |
| ASR | 评估 whisper.cpp/transcribe-rs 和 sherpa-onnx | 通过 engine adapter 支持多引擎 | 不要过早把 `whisper-cli` 子进程当成长期架构。 |
| 文本上屏 | 剪贴板粘贴并恢复 | Unicode `SendInput`，再到 TSF | 先跑通 MVP，再逐步提高可靠性。 |
| UI | 托盘 + 设置页 + 不抢焦点的状态提示 | onboarding、模型管理 | 第一版要保持工具属性，不做大而全产品。 |

## 对比过的方案

- Rust/Tauri 原生 MVP：推荐。长期维护、Windows native 能力、体积和开源协作都更合适。
- Python/faster-whisper 原型：适合做一次性 ASR 质量实验，不建议做主应用。
- Electron 应用：适合复杂桌面产品，但对轻量输入工具偏重，而且 native 难点仍然存在。
- 一开始做完整 TSF IME：技术上有吸引力，但实现成本高、验证慢，不适合作为第一阶段。

## 下一步需要讨论

在 scaffold 产品代码前，需要先确认两个关键决策：第一版 ASR 引擎实验选什么，第一版文本上屏是否接受剪贴板方案。当前最务实的默认值是：上屏先用剪贴板粘贴并恢复；ASR 用清晰的 engine adapter，把 whisper.cpp 风格和 sherpa-onnx 风格的后端都留出评估空间。

