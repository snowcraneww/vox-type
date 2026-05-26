# Contributing to VoxType

VoxType 目前处于项目启动阶段。欢迎围绕需求、竞品调研、技术方案、验证策略和文档提出 issue 或 PR。

## 贡献原则

- 公开透明：项目事实写入仓库，不依赖私聊上下文。
- 小步提交：每个 PR 聚焦一个问题或一个功能。
- 可验证：行为变更必须附带测试、手工验证步骤或明确的未验证说明。
- 隐私优先：不要提交语音样本、模型文件、个人配置、token 或 cookie。

## 当前优先级

请先查看：

- `docs/harness/feature_list.json`
- `docs/harness/progress.md`
- `docs/harness/research-log.md`

如果你想做产品代码，请先确认对应 spec 或 plan 已存在。

## 本地检查

```bash
bash init.sh
```

产品 scaffold 前，此命令只检查文档和 harness 文件是否齐全。

## PR 要求

PR 描述应包含：

- 改了什么。
- 为什么改。
- 运行过哪些验证。
- 哪些路径尚未验证。
- 是否影响 `docs/harness/feature_list.json` 或 `docs/harness/progress.md`。

## 文档语言

项目当前默认中文文档。代码、命令、API 名称和错误信息保持原文。

## 许可证

除非明确说明，提交到本项目的贡献按 Apache-2.0 许可证授权。
