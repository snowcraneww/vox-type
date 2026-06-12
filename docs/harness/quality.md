# Quality Snapshot

更新时间：2026-05-27

## 评级标准

- A：验证全部通过，边界清楚，下一轮 agent 可直接接手。
- B：验证通过，有少量未决问题但不阻塞下一步。
- C：部分可用，有明显缺口或未验证路径。
- D：不可用，或事实源缺失。

## 产品领域

| 领域 | 评级 | 验证状态 | Agent 可读性 | 测试稳定性 | 关键缺口 |
| --- | --- | --- | --- | --- | --- |
| Harness 基线 | A | `bash init.sh` 通过 | 高 | 当前稳定 | 后续随产品复杂度扩展验证 |
| 需求定义 | A | 已整理为中文 brief | 高 | 文档稳定 | 后续随真实用户反馈更新 |
| 外部调研 | A | 已完成第一轮 GitHub/开源项目调研 | 高 | 文档稳定 | 后续补充国内镜像和模型体验调研 |
| 技术方案 | A | Rust + Tauri 2 + React/TS + whisper.cpp 已确认并写入 OpenSpec | 高 | 文档稳定 | 后续细化全局快捷键和上屏策略 |
| 产品实现 | B | 第一版 MVP proof-of-life 已完成，核心单测通过，跨机器安装包已验证可安装 | 中 | Rust/TS 基线稳定 | 繁简输出标准化、识别质量、输入设备体验和上屏可靠性仍需增强 |

## 架构层

| 层级 | 评级 | 边界执行 | Agent 可读性 | 关键缺口 |
| --- | --- | --- | --- | --- |
| Repository Harness | A | 已建立 | 高 | 后续随项目复杂度定期简化 |
| OpenSpec | A | 已建立 MVP 技术方向 change | 高 | 后续每个较大产品行为继续走 change/plan |
| Application Shell | B | Tauri/React 桌面壳可运行 | 中 | 需要全局快捷键和更完整设置页 |
| Audio Pipeline | B | 真实录音、mono 化、16 kHz 重采样、音量摘要、诊断 WAV 导出已接入 | 中 | 需要设备选择持久化、VAD、降噪和更清晰音量提示 |
| Transcription Engine | B | whisper.cpp 一键安装、配置检测、最近录音转写已接入 | 中 | 当前默认 `ggml-base.bin`，需要模型选择和识别质量诊断 |
| Text Injection | C | 剪贴板 + `Ctrl+V` 已接入，已避免过早恢复旧剪贴板 | 中 | 后续评估 `SendInput(KEYEVENTF_UNICODE)` 或更深层输入法方案 |
| Packaging/Release | C | 已启用 Tauri bundle，debug NSIS/MSI 安装包可生成且跨机器可安装 | 中 | 尚未设计 release build、签名、版本发布和自动更新 |

## 变更历史

### 2026-05-26

- 初始化 harness 质量文档。
- Harness 基线已验证：`bash init.sh` 通过，`feature_list.json` JSON 与 UTF-8 内容检查通过。
- 记录当前所有产品层级仍未开始。
- 下一次更新应在 `discovery-001` 完成后进行。

### 2026-05-27

- MVP 真实闭环 proof-of-life 已由维护者手动验证。
- 新增输入设备选择、诊断 WAV 导出、剪贴板写入校验和避免过早恢复旧剪贴板。
- 将 44.1 kHz -> 16 kHz 重采样从跳点抽样改为线性插值，降低 ASR 输入失真。
- 当前主要风险从“链路能否跑通”转为“默认输入设备是否正确、录音质量是否足够、上屏策略是否足够可靠”。
- 启用 Tauri bundle，生成 NSIS/MSI debug 安装包；维护者确认安装包在另一台机器可安装。
- 另一台机器发现 whisper.cpp 可能返回繁体中文，记录为下一阶段文本标准化问题，不阻塞第一版 MVP 完成判断。
