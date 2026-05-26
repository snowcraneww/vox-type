# Session Handoff

## 当前已验证

- `bash init.sh` 通过。
- `docs/harness/feature_list.json` 是有效 JSON。
- Apache-2.0 许可证已写入 `LICENSE`，README 已同步。
- `harness-001` 已标记为 `passing`。
- `discovery-001` 已标记为 `passing`。
- `research-001` 已标记为 `passing`。
- 产品代码尚未 scaffold。

## 本轮改动

- 建立项目级 agent 规则和开源协作文档。
- 建立 harness 状态文件和课程学习映射。
- 建立 `feature_list.json` 作为任务事实源。
- 建立 research 和 OpenSpec 入口目录。
- 完成原始需求整理，见 `docs/research/requirements-brief.md`。
- 完成开源项目调研，见 `docs/research/open-source-landscape.md`。
- 完成技术选项建议，见 `docs/research/technical-options.md`。
- 中间调研资料保存在 `TMP/research/`；第三方克隆在 `TMP/research/repos/` 且被 Git 忽略。

## 仍损坏或未验证

- 产品技术方案尚未经过维护者 brainstorm 批准。
- 尚未创建 OpenSpec change。
- 产品代码尚未 scaffold。

## 下一步最佳动作

- 当前下一任务：`spec-001`。
- 为什么它是下一步：需求整理和外部调研已经完成，需要维护者参与方案 brainstorm，并用 OpenSpec 记录 MVP 决策。
- passing 标准：使用 superpowers:brainstorming 形成设计，维护者批准后在 `openspec/changes/` 下记录 proposal 和 tasks。
- 不要动：不要 scaffold 产品代码，不要引入 npm/cargo 依赖，不要把 TSF 当作 MVP 默认路线。

## 命令

- 启动/验证命令：`bash init.sh`
- JSON 检查：`python -m json.tool docs/harness/feature_list.json`
- 文件清单：`rg --files`
