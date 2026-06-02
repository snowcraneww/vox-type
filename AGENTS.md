# AGENTS.md

VoxType 是一个早期开源项目，目标是探索和实现本地优先的语音输入法。本仓库也是 agent 工作的事实源：需求、决策、进度、验证证据、交接信息必须沉淀到仓库文件里，而不是只留在聊天记录里。

## 当前状态

- 阶段：项目 harness 初始化、产品发现、开源调研、MVP 技术方向确认、最小真实闭环、主界面产品化、模型选择配置和 V8 百度实时 WebSocket API 自动验证均已完成；当前重点是维护者真实桌面验证 V8 连续输入，以及继续提升语音输入可靠性。
- 产品代码已有 Tauri 2 + React/TypeScript + Rust 桌面应用，已接入录音采集、16 kHz ASR 输入、本地 whisper.cpp、百度短语音 API、百度实时 WebSocket API、模型按输入模式路由、剪贴板上屏、全局快捷键、原生 Windows 浮窗、诊断日志、输入设备选择和诊断 WAV 导出。
- 当前最高优先级未完成项见 `docs/harness/feature_list.json`；截至 2026-06-02，`v8-001` 仍保持 `in_progress`，因为真实百度实时 WebSocket 桌面流式验证需要维护者在本机执行。
- 原始 AI 生成需求草稿在 `docs/plans/原始需求.md`，只能作为背景材料，不是已批准架构。
- Rust + Tauri 2 + React/TypeScript + whisper.cpp 是已确认 MVP 路线，详见 `docs/research/mvp-technical-proposal.md` 和 `openspec/changes/voxtype-mvp-technical-direction/`。

## 常用启动命令

从仓库根目录运行：

```bash
npm run tauri -- dev
```

这是验证 VoxType 真实桌面能力的首选命令。它会启动 Vite 开发服务器并打开 Tauri 桌面窗口；麦克风、全局快捷键、托盘、剪贴板上屏、原生浮窗和百度实时 WebSocket 连续输入都必须在这个模式下验证。

只看前端 UI 时可以运行：

```bash
npm run dev
```

`npm run dev` 只启动浏览器预览，不能验证麦克风、全局快捷键、托盘、剪贴板上屏和 Tauri command 的真实系统行为。不要用浏览器预览结果判断桌面输入法链路是否通过。

每次开工前仍先运行：

```bash
bash init.sh
```

它只检查 harness 和仓库基线，不启动应用。

## 开工流程

每次开始工作前，从仓库根目录执行：

1. 确认当前目录是 仓库根目录或等价 Git Bash 路径。
2. 阅读 `docs/harness/progress.md`。
3. 阅读 `docs/harness/feature_list.json`，选择优先级最高的未完成项。
4. 阅读 `docs/harness/working-agreement.md`。
5. 运行 `bash init.sh`。
6. 如果基线检查失败，先修复基线或记录 blocker，不要在坏状态上继续叠新功能。

## 工作规则

- 按开源项目标准工作：不要写入密钥、私有 cookie、个人 token、内部机器路径依赖。
- Git 提交必须使用项目级身份 `VoxType <maintainers@voxtype.dev>`；不要使用个人姓名、公司邮箱或本机用户名作为 author/committer。
- WIP=1：一次只推进一个功能、spec 或 harness 任务。
- 仓库文件优先于聊天记忆。
- 改动必须限定在当前任务范围内。
- 不要为了宣称完成而降低验证标准。
- 真实桌面行为以 `npm run tauri -- dev` 验证为准；浏览器预览只能作为 UI 快速检查。
- 功能状态只有在验证已运行并记录证据后，才能标为 `passing`。
- 验证、手动测试或用户反馈发现 bug 时，必须记录到 `docs/harness/debugging-log.md`；如果影响功能状态或后续优先级，同时同步 `docs/harness/progress.md`、`docs/harness/feature_list.json` 或 `docs/harness/session-handoff.md`。
- API Key、Secret Key、access token、cookie、个人路径和本机环境变量值不得进入仓库、日志、测试快照或提交信息；文档只能写变量名和脱敏状态。
- 不要重置、回滚或覆盖用户未要求处理的改动。
- 面向维护者的研究、方案、进度和规则文档默认使用中文；函数名、API 名、命令、仓库名、错误消息和专有名词保持原文。
- Windows PowerShell 5.1 容易把 UTF-8 中文命令片段按本地代码页解释。涉及中文文件的查看、搜索、diff、局部替换和批量编辑时，默认使用 Git Bash 的 `bash -lc`、`rg`、`sed` 或脚本化 UTF-8 读写；不要在 PowerShell 命令里嵌入中文片段做替换。PowerShell 只用于 Windows 专用配置、系统 API、进程管理或必须使用 PowerShell cmdlet 的场景。 若必须通过 Python/脚本在 PowerShell 管道中修改中文，脚本源码必须保持 ASCII，并用 `\uXXXX`、`chr()` 或从 UTF-8 文件读取中文；不要把中文直接写进 PowerShell here-string 或命令参数。

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
- 不从聊天想法直接实现新的产品功能或架构变化；先形成 spec 或 plan。小范围 bugfix、文档同步、样式微调和验证补充可以直接执行，但必须同步 harness 记录。
- `docs/harness/` 管日常状态，`openspec/` 管长期产品/架构决策。

## Harness 文件

- `docs/harness/working-agreement.md`：协作和边界规则。
- `docs/harness/progress.md`：当前已验证状态与会话日志。
- `docs/harness/feature_list.json`：功能状态机和优先级。
- `docs/harness/quality.md`：质量快照。
- `docs/harness/debugging-log.md`：真实调试问题、根因、修复和验证记录。
- `docs/harness/evaluator-rubric.md`：验收评分表。
- `docs/harness/session-handoff.md`：长会话交接模板。
- `docs/harness/clean-state-checklist.md`：收尾检查。
- `docs/harness/research-log.md`：外部调研记录。
- `docs/guide/run-and-understand.md`：运行、调试和命令速查。
- `docs/guide/interview-project-brief.md`：面试讲解项目用的整体介绍、技术栈、架构、亮点、难点和后续规划。
- `docs/integrations/baidu-asr.md`：百度短语音 API 与百度实时 WebSocket API 接入记录。
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
