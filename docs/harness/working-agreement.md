# Working Agreement

本文定义 VoxType 项目中人类贡献者、Codex、Superpowers、gstack、OpenSpec 的协作方式。

## 角色分工

- 人类维护者：决定产品方向、许可证、发布策略和风险接受标准。
- Codex：执行调研、文档、spec、实现、验证和交接工作。
- Superpowers：用于结构化 brainstorm、spec、计划、TDD 和完成前验证。
- gstack / browse / qa：用于有可运行 UI 后的浏览器级 QA。桌面输入法的系统行为仍需要专门验证策略。
- OpenSpec：记录长期产品行为和架构决策。

## 工作阶段

1. Harness 初始化：配置本文件、进度、功能清单、质量、交接和验证入口。
2. 需求澄清：整理 `docs/plans/原始需求.md`，区分愿景、假设、已决策和待验证。
3. 外部调研：搜索 GitHub 开源语音输入、Whisper、Tauri/Rust、Windows text injection 相关项目。
4. 方案 brainstorm：用 Superpowers 产出设计，并由维护者批准。
5. OpenSpec：把获批方案转成 durable spec。
6. Scaffold：创建最小可运行项目和测试基线。
7. MVP：按 feature list 一项一项实现。

## WIP 规则

- 同一时间只允许一个 `in_progress` 项。
- 如果要切换任务，先更新当前项的状态、证据和交接说明。
- 不允许同时做“调研技术方案”和“实现产品代码”，除非 feature list 明确拆分且当前任务只覆盖其中一个。

## 状态转移

- `not_started` -> `in_progress`：开始本轮唯一任务。
- `in_progress` -> `blocked`：阻塞原因、已尝试动作和所需输入已记录。
- `in_progress` -> `passing`：验证已运行，证据已记录。
- `passing` -> `in_progress`：发现回归或需求变更，必须记录原因。

## 文档边界

- `README.md` 面向用户和贡献者，只写当前真实状态。
- `AGENTS.md` 面向 agent，只写入口、硬规则、路由。
- `docs/harness/` 面向协作状态和质量管理。
- `docs/research/` 面向外部调研材料。
- `openspec/` 面向批准后的产品和架构变更。

## 搜索和调研

- 优先使用官方文档、源仓库、活跃开源项目。
- 使用环境变量中的搜索密钥，不打印密钥值。
- 每条关键结论必须记录来源 URL。
- 如果资料冲突，记录冲突，不强行合并。

## 验证策略

当前阶段：

- `bash init.sh`
- `python -m json.tool docs/harness/feature_list.json`

产品 scaffold 后逐步增加：

- Rust: `cargo fmt --all --check`、`cargo clippy`、`cargo test`
- Frontend: package manager lint/typecheck/test
- Tauri: dev run and build smoke check
- UI: browser QA where applicable
- Desktop input: Windows-focused manual or scripted E2E checklist

## 完成前自查

每次声称完成前回答：

1. 当前任务是什么？
2. 范围内做了什么？
3. 哪些验证跑过？
4. 证据写在哪里？
5. 下一轮不用聊天上下文能否继续？
