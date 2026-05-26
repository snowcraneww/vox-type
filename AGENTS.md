# AGENTS.md

VoxType 是一个早期开源项目，目标是探索和实现本地优先的语音输入法。本仓库也是 agent 工作的事实源：需求、决策、进度、验证证据、交接信息必须沉淀到仓库文件里，而不是只留在聊天记录里。

## 当前状态

- 阶段：项目 harness 初始化、产品发现和开源调研已完成；下一步是 MVP 技术方案 brainstorm / OpenSpec。
- 产品代码尚未 scaffold。
- 原始 AI 生成需求草稿在 `docs/plans/原始需求.md`，只能作为背景材料，不是已批准架构。
- Rust / Tauri / Whisper.cpp 是原始草稿候选方向。当前调研建议见 `docs/research/technical-options.md`，仍需经过维护者批准和 OpenSpec 记录后再定稿。

## 开工流程

每次开始工作前，从仓库根目录执行：

1. 确认当前目录是 `C:/grace_repos/open-source/vox-type` 或等价 Git Bash 路径。
2. 阅读 `docs/harness/progress.md`。
3. 阅读 `docs/harness/feature_list.json`，选择优先级最高的未完成项。
4. 阅读 `docs/harness/working-agreement.md`。
5. 运行 `bash init.sh`。
6. 如果基线检查失败，先修复基线或记录 blocker，不要在坏状态上继续叠新功能。

## 工作规则

- 按开源项目标准工作：不要写入密钥、私有 cookie、个人 token、内部机器路径依赖。
- WIP=1：一次只推进一个功能、spec 或 harness 任务。
- 仓库文件优先于聊天记忆。
- 改动必须限定在当前任务范围内。
- 不要为了宣称完成而降低验证标准。
- 功能状态只有在验证已运行并记录证据后，才能标为 `passing`。
- 不要重置、回滚或覆盖用户未要求处理的改动。
- 面向维护者的研究、方案、进度和规则文档默认使用中文；函数名、API 名、命令、仓库名、错误消息和专有名词保持原文。

## 技能使用

按任务选择最小必要技能集：

- `web-search`：当前资料、GitHub 项目、库/API 调研。
- `superpowers:brainstorming`：产品方案、功能行为、架构方向设计前使用。
- `superpowers:writing-plans`：spec 获批后生成可执行实现计划。
- `superpowers:test-driven-development`：实现代码或 bugfix 前使用。
- `superpowers:verification-before-completion`：声称完成前使用。
- `gstack` / `browse` / `qa`：存在可运行 UI 后用于端到端检查。
- `review`：非平凡实现变更落地前做结构性审查。
- `ship`：有 Git 仓库、测试、变更日志和 PR/发布流程后再用。

## OpenSpec 约定

- 产品行为和架构决策用 OpenSpec 管理。
- 新变更放在 `openspec/changes/<change-id>/`。
- 每个变更至少包含 `proposal.md` 和 `tasks.md`。
- 不从聊天想法直接实现产品功能；先形成 spec 或 plan。
- `docs/harness/` 管日常状态，`openspec/` 管长期产品/架构决策。

## Harness 文件

- `docs/harness/working-agreement.md`：协作和边界规则。
- `docs/harness/progress.md`：当前已验证状态与会话日志。
- `docs/harness/feature_list.json`：功能状态机和优先级。
- `docs/harness/quality.md`：质量快照。
- `docs/harness/evaluator-rubric.md`：验收评分表。
- `docs/harness/session-handoff.md`：长会话交接模板。
- `docs/harness/clean-state-checklist.md`：收尾检查。
- `docs/harness/research-log.md`：外部调研记录。
- `docs/research/`：长期需求、开源生态和技术选项研究。
- `docs/research/mvp-technical-proposal.md`：Rust + Tauri 2 + React/TS MVP 技术方案草案，未获维护者确认前不要当作正式 spec。
- `TMP/research/*.md`：本轮调研中间材料，可提交；`TMP/research/repos/` 是第三方仓库克隆，必须保持忽略。

## 完成定义

任务完成必须同时满足：

- 请求的行为、文档或配置已存在于仓库。
- 验证已运行，或无法运行的原因已记录。
- 证据记录在 `docs/harness/progress.md` 或 `docs/harness/feature_list.json`。
- 新假设、风险、后续决策已写入文档。
- 下一轮会话能只靠仓库文件继续。

## 收尾流程

结束重要会话前：

1. 更新 `docs/harness/progress.md`。
2. 如状态变化，更新 `docs/harness/feature_list.json`。
3. 未完成工作写入 `docs/harness/session-handoff.md`。
4. 运行当前最合适的验证命令。
5. 如实记录失败和未验证项。
6. 保持工作区可被下一轮安全检查。
