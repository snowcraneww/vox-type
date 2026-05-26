# 会话交接

## 当前已验证

- `bash init.sh` 通过。
- `docs/harness/feature_list.json` 是有效 JSON。
- Apache-2.0 许可证已写入 `LICENSE`，README 已同步。
- `harness-001` 已标记为 `passing`。
- `discovery-001` 已标记为 `passing`。
- `research-001` 已标记为 `passing`。
- `spec-001` 已标记为 `passing`。
- `scaffold-001` 已开始，状态 `in_progress`。
- 产品代码已有最小 Tauri/React/Rust mock 骨架。

## 本轮改动

- 建立项目级 agent 规则和开源协作文档。
- 建立 harness 状态文件和课程学习映射。
- 建立 `feature_list.json` 作为任务事实源。
- 建立 research 和 OpenSpec 入口目录。
- 完成原始需求整理，见 `docs/research/requirements-brief.md`。
- 完成开源项目调研，见 `docs/research/open-source-landscape.md`。
- 完成技术选项建议，见 `docs/research/technical-options.md`。
- 完成详细 MVP 技术方案草案，见 `docs/research/mvp-technical-proposal.md`。
- 已确认 MVP 技术方向，并写入 `openspec/changes/voxtype-mvp-technical-direction/`。
- 已创建实现计划：`docs/superpowers/plans/2026-05-26-scaffold-voxtype-mvp.md`。
- 已创建最小产品骨架：`src/`、`src-tauri/`、`package.json`。
- 中间调研资料保存在 `TMP/research/`；第三方克隆在 `TMP/research/repos/` 且被 Git 忽略。
- 维护者文档语言规则：默认中文；函数名、API 名、命令、仓库名、错误消息和专有名词保持原文。

## 仍损坏或未验证

- 真实麦克风录音尚未接入。
- 真实 whisper.cpp 推理尚未接入。
- 真实剪贴板上屏尚未接入。
- 托盘入口尚未接入。
- Windows 手动 E2E 尚未执行。
- CMake 已安装在 `C:/Program Files/CMake/bin/cmake.exe`，但当前 shell 的 PATH 尚未刷新。

## 下一步最佳动作

- 当前下一任务：`scaffold-001`。
- 为什么它是下一步：mock 骨架已完成，需要继续接入真实系统能力并验证 Windows 闭环。
- passing 标准：创建最小可运行项目骨架，运行格式化、lint/typecheck 和测试基线，并更新 README 的真实启动与验证命令。
- 不要动：不要把 TSF 当作 MVP 默认路线；不要提交模型文件、音频文件或密钥。

## 命令

- 启动/验证命令：`bash init.sh`
- JSON 检查：`python -m json.tool docs/harness/feature_list.json`
- 文件清单：`rg --files`
