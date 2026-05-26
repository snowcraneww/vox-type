# Session Handoff

## 当前已验证

- `bash init.sh` 通过。
- `docs/harness/feature_list.json` 是有效 JSON。
- Apache-2.0 许可证已写入 `LICENSE`，README 已同步。
- `harness-001` 已标记为 `passing`。
- 产品代码尚未 scaffold。

## 本轮改动

- 建立项目级 agent 规则和开源协作文档。
- 建立 harness 状态文件和课程学习映射。
- 建立 `feature_list.json` 作为任务事实源。
- 建立 research 和 OpenSpec 入口目录。

## 仍损坏或未验证

- 仓库不是 Git 仓库，尚未形成提交历史。
- 原始需求仍是草稿，尚未转成正式需求 brief。
- 产品技术栈尚未调研和决策。

## 下一步最佳动作

- 当前下一任务：`discovery-001`。
- 为什么它是下一步：需要把原始需求拆成事实、假设、风险、MVP 边界和待确认问题，才能进行外部调研和方案 brainstorm。
- passing 标准：产出 `docs/research/requirements-brief.md`，并更新 `progress.md` 与 `feature_list.json`。
- 不要动：不要 scaffold 产品代码，不要定稿技术栈，不要引入 npm/cargo 依赖。

## 命令

- 启动/验证命令：`bash init.sh`
- JSON 检查：`python -m json.tool docs/harness/feature_list.json`
- 文件清单：`rg --files`