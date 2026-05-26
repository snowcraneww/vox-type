# VoxType

VoxType 是一个早期开源项目，目标是探索本地优先、隐私友好的语音输入法体验。

当前仓库已完成 harness 初始化、产品发现和第一轮开源调研。产品代码尚未创建；`docs/plans/原始需求.md` 是原始需求草稿，不代表最终技术方案。

## 目标方向

VoxType 计划优先验证这些能力：

- 按住说话、松开转写并输入到当前光标位置。
- 本地语音识别优先，尽量减少网络依赖。
- 面向 Windows 先做可用 MVP，再评估 macOS/Linux。
- 开源协作友好：需求、设计、验证和进度都沉淀在仓库内。

## 当前阶段

1. 项目 harness 配置。
2. 语音输入法需求分析和边界澄清。
3. GitHub 开源项目调研。已完成第一轮，见 `docs/research/open-source-landscape.md`。
4. 技术方案 brainstorm。下一步。
5. OpenSpec 记录架构决策。已创建 `openspec/changes/voxtype-mvp-technical-direction/`。
6. Scaffold 产品代码并建立测试基线。下一步。

## 快速开始

```bash
bash init.sh
```

当前 `init.sh` 只检查仓库文档和 harness 基线，因为产品代码尚未 scaffold。

## 重要文档

- `AGENTS.md`：agent 工作规则。
- `CONTRIBUTING.md`：开源贡献约定。
- `docs/harness/working-agreement.md`：人和 agent 的协作规则。
- `docs/harness/feature_list.json`：当前任务和功能状态。
- `docs/harness/progress.md`：已验证状态和会话日志。
- `docs/harness/research-log.md`：课程学习和后续调研记录。
- `docs/research/requirements-brief.md`：整理后的需求 brief。
- `docs/research/open-source-landscape.md`：相关开源项目调研。
- `docs/research/technical-options.md`：MVP 技术选项建议。
- `docs/research/mvp-technical-proposal.md`：Rust + Tauri 2 + React/TS MVP 技术方案草案。
- `TMP/research/`：调研中间材料；其中 `repos/` 是本地第三方克隆，不提交。
- `docs/plans/原始需求.md`：原始需求草稿。

## 开发状态

还没有可运行应用。请不要运行 `npm run tauri dev`、`cargo test` 或类似命令作为本项目的标准验证，直到 scaffold 完成并写入文档。

## 许可证

VoxType 使用 Apache-2.0 许可证。详见 LICENSE。
