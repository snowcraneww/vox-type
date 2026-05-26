# Progress Log

## 当前已验证状态

- 仓库根目录：`C:/grace_repos/open-source/vox-type`
- 当前阶段：MVP scaffold 进行中，已完成最小 Tauri/React/Rust mock 闭环骨架。
- 产品 scaffold：进行中。
- 许可证：Apache-2.0，见 `LICENSE`。
- 标准启动路径：`bash init.sh`
- 标准验证路径：`bash init.sh` 和 `python -m json.tool docs/harness/feature_list.json`
- 当前最高优先级未完成项：`scaffold-001`，状态 `in_progress`
- 文档语言规则：面向维护者的研究、方案、进度和规则文档默认中文；函数名、API 名、命令、仓库名、错误消息和专有名词保持原文。
- 当前 blocker：无。

## 课程学习状态

已逐节读取 Learn Harness Engineering 中文课程 12 讲正文摘要，并将对本项目的配置影响记录到 `docs/harness/lesson-synthesis.md`。

已覆盖主题：

1. 模型能力与执行可靠性的区别。
2. Harness 五子系统：指令、工具、环境、状态、反馈。
3. 仓库作为事实源。
4. 指令文件拆分。
5. 跨会话连续性。
6. 初始化阶段。
7. WIP=1 与范围边界。
8. 功能清单状态机。
9. 防止过早宣告完成。
10. 端到端验证。
11. 可观测性。
12. 清洁状态。

## 验证证据

### 2026-05-26 Harness Baseline

- `bash init.sh`：exit 0；输出包含 `VoxType harness baseline OK`、`License: Apache-2.0`、`Product scaffold: not started`。
- `python -m json.tool docs/harness/feature_list.json`：exit 0。
- UTF-8 内容断言：exit 0；确认 `feature_list.json`、`quality.md`、`evaluator-rubric.md`、`README.md`、`AGENTS.md` 中文内容可读。
- 文本扫描：旧的 MIT 候选信息只保留在 `docs/plans/原始需求.md` 原始草稿中；项目事实已在 README、LICENSE 和 harness 中改为 Apache-2.0。

### 2026-05-26 产品发现与开源项目调研

- 阅读 `docs/plans/原始需求.md`，整理出 VoxType 的目标、MVP 行为、非目标、风险和待确认问题。
- 使用 Tavily 和 GitHub API 搜索本地语音输入、Whisper、Windows 热键和文本上屏相关项目。
- 深读本地克隆：Handy、OpenLess、OpenWhispr、VoiceFlow、whisper-writer、tambourine-voice、faster-whisper-dictation。
- 验证生态参考：whisper.cpp、sherpa-onnx、local-whisper、FluidAudio、FluidVoice。
- 产出中间文档：`TMP/research/requirements-brief.md`、`TMP/research/repo-candidates.md`、`TMP/research/solution-analysis.md`。
- 产出正式文档：`docs/research/requirements-brief.md`、`docs/research/open-source-landscape.md`、`docs/research/technical-options.md`。
- 许可证判断：MIT/Apache-2.0 项目可作为更直接参考；GPL/AGPL 项目仅做概念参考，避免污染 Apache-2.0 项目。

### 2026-05-26 中文化与 MVP 技术方案草案

- 将 `docs/research/requirements-brief.md`、`docs/research/open-source-landscape.md`、`docs/research/technical-options.md` 改为中文维护者文档。
- 将 `TMP/research/README.md`、`TMP/research/requirements-brief.md`、`TMP/research/repo-candidates.md`、`TMP/research/solution-analysis.md` 改为中文中间文档。
- 新增 `docs/research/mvp-technical-proposal.md`，详细说明 Rust + Tauri 2 + React/TS MVP 方案、模块边界、上屏策略、ASR adapter、测试策略和阶段路线。
- 更新 `AGENTS.md` 和 `docs/research/README.md`，明确维护者文档默认中文。

### 2026-05-26 MVP 技术方向确认

- 维护者确认所有关键决策采用建议默认值：中文优先兼容英文、第一版接受剪贴板上屏、ASR 优先 whisper.cpp 路线、UI 接受“托盘 + 设置页 + 状态提示”。
- 更新 `docs/research/mvp-technical-proposal.md`，把状态改为已确认，并补充“剪贴板 -> SendInput -> TSF”的上屏策略解释。
- 创建 `openspec/changes/voxtype-mvp-technical-direction/proposal.md`。
- 创建 `openspec/changes/voxtype-mvp-technical-direction/tasks.md`。
- 将 `spec-001` 标记为 passing。

### 2026-05-26 MVP Scaffold 骨架

- 写入实现计划：`docs/superpowers/plans/2026-05-26-scaffold-voxtype-mvp.md`。
- 创建前端骨架：`package.json`、`vite.config.ts`、`tsconfig*.json`、`src/App.tsx`、`src/App.test.tsx`、`src/styles.css`。
- 创建 Tauri/Rust 骨架：`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json`、`src-tauri/src/*`。
- 创建 Rust 模块：`state`、`config`、`error`、`audio`、`asr`、`insertion`、`hotkey`、`recorder`。
- 创建 mock 闭环 command：`simulate_dictation`。
- 生成临时图标：`src-tauri/icons/icon.ico`。
- `npm install`：成功，0 vulnerabilities。
- `cargo fetch --manifest-path src-tauri/Cargo.toml`：成功。
- `npm run typecheck`：通过。
- `npm test -- --run`：通过，1 个测试文件、1 个测试通过。
- `cargo test --manifest-path src-tauri/Cargo.toml`：通过，8 个 Rust 测试通过。
- `npm run build`：通过。
- `cargo check --manifest-path src-tauri/Cargo.toml`：通过。
- 当前未完成：真实麦克风录音、真实 whisper.cpp 推理、真实剪贴板上屏、托盘入口、Windows 手动 E2E。

## 会话记录

### 会话 001 - 2026-05-26

- 本轮目标：深入学习 harness 课程，并为 VoxType 建立开源项目 harness 基线。
- 已完成：
  - 读取课程目录、官方模板和 12 讲正文摘要。
  - 创建 `docs/harness/lesson-synthesis.md`，把课程原则映射到 VoxType。
  - 创建/更新 `AGENTS.md`、`README.md`、`CONTRIBUTING.md`、`init.sh`。
  - 创建 `LICENSE`，许可证采用 Apache-2.0。
  - 创建 `.gitignore`、`.editorconfig`、`.gitattributes`。
  - 创建 `docs/harness/working-agreement.md`、`feature_list.json`、`quality.md`、`evaluator-rubric.md`、`session-handoff.md`、`clean-state-checklist.md`、`research-log.md`。
  - 创建 `docs/research/README.md` 和 `openspec/README.md`。
- 运行过的验证：
  - `bash init.sh`
  - `python -m json.tool docs/harness/feature_list.json`
  - UTF-8 内容断言
  - 文本扫描旧许可证和占位符
- 已记录证据：见本文件“验证证据”和 `docs/harness/feature_list.json`。
- 更新过的文件或工件：
  - `AGENTS.md`
  - `README.md`
  - `CONTRIBUTING.md`
  - `LICENSE`
  - `.gitignore`
  - `.editorconfig`
  - `.gitattributes`
  - `init.sh`
  - `docs/harness/*`
  - `docs/research/README.md`
  - `openspec/README.md`
- 已知风险或未解决问题：
  - 原始需求文件仍是草稿，包含旧的 MIT 候选信息；项目事实以 Apache-2.0 为准。
  - 产品技术栈尚未决策。
- 下一步最佳动作：执行 `discovery-001`，把 `docs/plans/原始需求.md` 整理成正式需求 brief。

### 会话 002 - 2026-05-26

- 本轮目标：学习原始需求，调研相关开源项目，并形成 VoxType 技术方向建议。
- 已完成：
  - 将第三方调研仓库集中在 `TMP/research/repos/`，并通过 `.gitignore` 排除。
  - 写入 `TMP/research/` 中间调研文档。
  - 写入 `docs/research/` 长期调研文档。
  - 将 `discovery-001` 和 `research-001` 标记为 passing。
- 初步建议：Rust + Tauri 2 + React/TS，Windows first，本地优先；MVP 先做 clipboard paste + restore，上屏可靠性第二阶段做 `SendInput(KEYEVENTF_UNICODE)`，TSF IME 后置。
- 下一步最佳动作：执行 `spec-001`，用 superpowers:brainstorming 和 OpenSpec 把 MVP 方案正式收敛并等待维护者批准。

### 会话 003 - 2026-05-26

- 本轮目标：把调研资料中文化，并写出 Rust + Tauri 2 + React/TS 的详细 MVP 技术方案草案。
- 已完成：
  - 中文化 `docs/research/` 和 `TMP/research/` 主要文档。
  - 新增 `docs/research/mvp-technical-proposal.md`。
  - 在 `AGENTS.md`、`docs/research/README.md` 和 harness 中加入“维护者文档默认中文”的规则。
- 下一步最佳动作：维护者审阅 `docs/research/mvp-technical-proposal.md`，确认或修改关键决策后进入 OpenSpec。

### 会话 004 - 2026-05-26

- 本轮目标：解释上屏策略分层和 TSF，并将维护者确认的 MVP 技术决策写入 OpenSpec。
- 已完成：
  - 解释并记录剪贴板、`SendInput(KEYEVENTF_UNICODE)`、TSF 的分阶段含义。
  - 把 `docs/research/mvp-technical-proposal.md` 从草案更新为已确认方案说明。
  - 新增 OpenSpec change：`openspec/changes/voxtype-mvp-technical-direction/`。
  - 将 `spec-001` 标记为 passing。
- 下一步最佳动作：为 `scaffold-001` 写详细 implementation plan，之后再开始 scaffold 产品代码。

### 会话 005 - 2026-05-26

- 本轮目标：按已确认方案开始 scaffold。
- 已完成：
  - 写入详细 implementation plan。
  - 创建最小 Tauri 2 + React/TypeScript + Rust 骨架。
  - 创建前端设置页和模拟闭环状态展示。
  - 创建 Rust mock 核心模块和测试。
  - 安装 npm/cargo 依赖，并运行基础验证。
- 下一步最佳动作：继续 `scaffold-001`，接入真实托盘、真实录音、whisper.cpp 路线 proof-of-life、剪贴板上屏，并进行 Windows 手动 E2E。
