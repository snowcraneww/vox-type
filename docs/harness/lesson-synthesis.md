# Harness 课程学习笔记

来源：Learn Harness Engineering 中文课程 12 讲与官方模板。本文记录对 VoxType 项目配置的直接影响。

## 课程结论

### 1. 模型能力不等于执行可靠

失败优先归因到 harness，而不是先换模型。VoxType 需要让 agent 在任务、上下文、环境、验证、交接五个层面都有明确入口。

项目落点：`AGENTS.md`、`init.sh`、`docs/harness/progress.md`、`docs/harness/feature_list.json`。

### 2. Harness 是五个子系统

Harness = 指令 + 工具 + 环境 + 状态 + 反馈。缺任何一层，agent 都可能看起来很聪明但执行不稳定。

项目落点：

- 指令：`AGENTS.md` 和 `docs/harness/working-agreement.md`
- 工具：`init.sh`、后续测试/QA 命令
- 环境：`README.md`、后续 scaffold 文档
- 状态：`progress.md`、`feature_list.json`、`session-handoff.md`
- 反馈：`evaluator-rubric.md`、`quality.md`、验证证据

### 3. 仓库是唯一事实源

不在仓库里的知识，对下一轮 agent 等于不存在。聊天记录只能辅助，不能作为长期状态。

项目落点：所有产品决策、技术调研、任务状态都写入 `docs/harness/`、`docs/research/` 或 `openspec/`。

### 4. 入口文件是路由器，不是百科全书

`AGENTS.md` 应短而硬，只包含规则、入口和链接。细节放专题文档。

项目落点：根目录 `AGENTS.md` 控制在可快速阅读的范围，详细协作约定放 `docs/harness/working-agreement.md`。

### 5. 长任务必须持久化状态

语音输入法项目会跨多轮：需求澄清、竞品调研、架构验证、MVP 实现、端到端测试。不能依赖上下文窗口。

项目落点：每轮更新 `progress.md` 和必要时的 `session-handoff.md`。

### 6. 初始化是独立阶段

初始化的目标不是写业务代码，而是确保能启动、能测试、能看进度、能接手下一步。

项目落点：当前阶段只配置 harness，不 scaffold 产品代码；`init.sh` 先验证文档基线。

### 7. WIP=1

Agent 容易 overreach 和 under-finish。必须一次只做一个清晰任务，并有外部化完成证据。

项目落点：`feature_list.json` 使用单一 active item，状态转移需要证据。

### 8. 功能清单是原语

功能项必须包含行为描述、验证方式、状态。它不是备忘录，而是调度和交接的基础设施。

项目落点：`docs/harness/feature_list.json` 按 discovery / harness / implementation 分区，并记录证据。

### 9. 防止过早宣告完成

单元测试通过不等于任务完成；文档写完也不等于配置完成。完成需要语法、行为、系统三层检查。

项目落点：当前至少运行 `bash init.sh` 和 JSON 校验；产品 scaffold 后补充 lint/test/e2e。

### 10. 端到端验证会改变结果

一旦有 UI 或输入法行为，必须建立端到端路径。语音输入法尤其不能只测函数：录音、转写、文本注入、焦点窗口都要整体验证。

项目落点：`feature_list.json` 预留 E2E 验证任务；后续用 gstack/browse 只测可浏览 UI，用系统级手工/自动脚本测桌面输入法路径。

### 11. 可观测性属于 harness

运行时信号解释发生了什么；过程工件解释为什么这么做。两者都要有。

项目落点：`quality.md` 跟踪质量快照，`research-log.md` 记录来源和决策，后续产品代码要内置日志和诊断导出。

### 12. 每次会话留下干净状态

清洁状态包括构建、测试、进度、工件、启动路径。不是“代码能编译”那么简单。

项目落点：`clean-state-checklist.md` 和 `session-handoff.md` 是每轮收尾要求。

## 对 VoxType 的配置原则

1. 先 harness，后产品代码。
2. 先研究和 spec，后技术选型定稿。
3. 默认开源协作：文档可公开、命令可复现、无私密依赖。
4. 产品任务必须从 `feature_list.json` 进入，不从聊天直接开写。
5. 技术调研必须记录来源、结论、未决问题。
6. 每次完成都必须留下验证证据。
7. 每轮结束要让新 agent 在 3 分钟内恢复上下文。
