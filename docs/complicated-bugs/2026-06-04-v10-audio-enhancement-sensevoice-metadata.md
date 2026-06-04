# V10 音频增强与 SenseVoice 空文本、metadata 丢失、配置不持久

日期：2026-06-04

范围：V10 audio preprocessing、SenseVoice Small stop-then-transcribe、React Settings、transcript history persistence。

## 现象

V10 音频增强连续暴露了几个相关问题：开启增强后 SenseVoice Small 可能执行成功但返回空 JSON text；识别记录最初看不到增强 metadata；重启后增强开关没有恢复用户选择；保存识别记录后 audioPreprocess metadata 会丢失；增强入口放在模型配置里也让验证路径不清晰。

## 为什么复杂

这个问题跨 Rust audio preprocessing、sherpa-onnx CLI、React Settings 和 transcript history persistence。单看进程 exit code、设置保存结果或一次 ASR 返回值，都不能说明端到端行为正确。

## 错误假设

- 以为 pipeline 接入 ASR 后，用户自然能从识别效果感知增强；实际必须有可见 metadata。
- 以为 saveTranscriptHistoryEntry 只追加记录；实际 schema 缺字段时会剥掉 audioPreprocess。
- 以为 SenseVoice 空 text 是普通识别失败；实际可以针对增强后空文本安全重试 raw audio。
- 以为保存开关即可；实际启动时还必须读取 getAudioPreprocessConfig。

## 根因

根因不是单个 bug，而是新链路缺少端到端可观察性和模型兼容兜底：增强样本改变了 SenseVoice 输入分布；history schema 没有同步 audio_preprocess；前端启动没有加载已保存配置；UI 信息架构把增强藏在模型配置里。

## 修复

- SenseVoice processed audio 先试；命中增强后 empty-text failure 时，用原始 raw samples 重试一次。
- AudioPreprocessSummary 增加 fallback_to_raw / fallbackToRaw，UI 显示“增强 / 已回退”。
- PersistedTranscriptEntry 增加 audio_preprocess，save/load 都保留 metadata。
- 前端启动调用 getAudioPreprocessConfig，同步用户上次选择。
- 设置页改为 输入 / 模型 / 音频增强 / 文本优化 / 诊断；主页面只保留设置入口，快捷键胶囊打开快捷键设置。

## 验证

自动验证覆盖：npm test -- --run src/App.test.tsx；npm test -- --run；npm run typecheck；npm run build；cargo check --manifest-path src-tauri/Cargo.toml --lib；cargo test --manifest-path src-tauri/Cargo.toml saves_and_loads_audio_preprocess_metadata --no-run；cargo test --manifest-path src-tauri/Cargo.toml sensevoice_empty_preprocessed_result_retries_raw_audio --no-run；python -m json.tool docs/harness/feature_list.json。

维护者真实桌面反馈确认：开启增强后识别记录能看到“增强 / 裁剪”或“增强 / 无明显变化”。仍需继续用 npm run tauri -- dev 做真实 before/after 质量验证。

## 后续经验

- 任何影响音频输入样本的功能都必须提供用户可见 metadata。
- 新 metadata 字段必须同步 Rust result type、Tauri DTO、TypeScript type、history persistence schema 和 UI render。
- 第三方 ASR 模型失败不能只看 exit code；结构化 stdout 为空 text 也要进入兼容策略。
- 配置持久化必须同时覆盖 save command 和 app startup load。
- 反复修三轮仍不稳定的问题，必须进入 docs/complicated-bugs。
