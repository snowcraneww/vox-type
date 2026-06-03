# 双声道录音被错误当作 mono 导致变音

日期：2026-05-27  
相关范围：录音采集、声道归一化、重采样、whisper.cpp 输入

## 现象

维护者录音后，导出的 `last-asr-input.wav` 能听到自己的声音，但声音像电子音、变音或节奏异常。whisper.cpp 转写结果不可用，经常只返回类似 `(音)` 的内容。

## 为什么复杂

初看像是麦克风、远程音频设备、whisper.cpp 模型或中文 prompt 的问题。实际录音链路有多个边界：

- Windows 输入设备可能暴露为双声道或远程音频设备。
- Rust 录音回调收到 interleaved PCM。
- 项目内部要归一化为 mono。
- ASR 前还要重采样到 16 kHz。

任何一层错位都会表现为“ASR 质量差”。

## 根因

`RecordingBuffer::push_interleaved_i16` 在第一次处理双声道回调后，把 `channels` 改成了 `1`。后续回调继续到来时，代码按 `channels = 1` 解释 interleaved 双声道数据，导致 frame 分组错误。

结果是后续双声道样本不再按真实左右声道配对平均，而是被当作 mono 顺序样本处理，造成变音和节奏异常。

## 修复

在录音 buffer 中区分两个概念：

- `source_channels`：输入设备真实声道数，整个录音 session 保持不变。
- `channels`：输出 buffer 归一化后的声道数，可以是 mono。

后续每次音频回调都按 `source_channels` 分组和平均，不再受输出 `channels` 影响。

核心代码位置：

- `src-tauri/src/recorder.rs`
- `src-tauri/src/audio.rs`

## 验证

历史验证记录见 `docs/harness/debugging-log.md` 中 “录音变音 / 双声道回调处理错误” 相关条目。

关键验证方式：

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

手动验证：重新录音，导出 `last-asr-input.wav`，确认人声不再变速/电子化，再执行最近录音转写。

## 后续经验

- 不要用输出格式字段覆盖输入设备真实格式字段。
- 音频问题不要先归因于 ASR 模型。先导出 ASR 输入 WAV，确认送给模型的音频正常。
- 对 interleaved 多声道 PCM，要用测试覆盖连续多次回调，而不是只测第一次回调。
