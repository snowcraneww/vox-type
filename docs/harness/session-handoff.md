# 会话交接

## 当前已验证

- `bash init.sh` 通过。
- `docs/harness/feature_list.json` 是有效 JSON。
- Apache-2.0 许可证已写入 `LICENSE`，README 已同步。
- `harness-001` 已标记为 `passing`。
- `discovery-001` 已标记为 `passing`。
- `research-001` 已标记为 `passing`。
- `spec-001` 已标记为 `passing`。
- `scaffold-001` 已完成 MVP 真实闭环验证，状态 `passing`。
- 第一版 MVP 已完成 proof-of-life：本机真实闭环通过，跨机器安装包可安装并能运行到转写链路。
- 产品代码已有最小 Tauri/React/Rust mock 骨架。
- 产品代码已有录音采集 proof-of-life：Tauri command 可启动/停止 `cpal` 输入 stream，并返回采样数和时长。
- 维护者手动验证录音采集成功：`782863` 个 mono 样本、`17751 ms`、`44100 Hz`。
- 维护者手动验证另一台机器安装包可安装；该机器转写返回繁体中文，已记录为下一阶段文本标准化问题。
- 2026-05-27 验证通过：`npm run typecheck`、`python -m json.tool docs/harness/feature_list.json`、`bash init.sh`、`cargo fmt --all --manifest-path src-tauri/Cargo.toml`、`npm test -- --run`、`cargo test --manifest-path src-tauri/Cargo.toml`、`cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`、`npm run build`、`npm run tauri -- build --debug`。
- 2026-05-27 录音采集验证通过：`cargo test --manifest-path src-tauri/Cargo.toml` 17 个测试通过，`npm run tauri -- build --debug` 通过。
- Tauri 本机调试 exe：`src-tauri/target/debug/vox-type.exe`。
- 跨机器验证请使用安装包：`src-tauri/target/debug/bundle/nsis/VoxType_0.1.0_x64-setup.exe` 或 `src-tauri/target/debug/bundle/msi/VoxType_0.1.0_x64_en-US.msi`。

## 本轮改动

- 建立项目级 agent 规则、Apache-2.0 许可证和 harness 文档。
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
- 已接入托盘入口、默认输入设备探测、音频 mono 标准化、whisper.cpp CLI adapter 和 Windows 剪贴板上屏 adapter。
- 已接入 UI 诊断日志、录音采集按钮、`start_recording`、`stop_recording`、`get_recording_status`。
- 已接入 16 kHz ASR 输入准备摘要：`asr_sample_rate`、`asr_sample_count`、`asr_duration_ms`。
- 已接入 `transcribe_last_recording` command 和 `转写最近录音` UI 按钮；通过 `VOXTYPE_WHISPER_CPP_BINARY`、`VOXTYPE_WHISPER_CPP_MODEL`、`VOXTYPE_ASR_LANGUAGE` 配置 whisper.cpp。
- 已接入应用内 ASR 配置体验：`ASR 配置` 面板可保存/检测 whisper.cpp binary、模型路径和语言；应用内配置优先，环境变量只做 fallback。
- 已修复结构化 Tauri error 的前端展示，诊断日志不应再出现 `[object Object]`。
- 已接入 `一键安装 whisper.cpp`：下载官方 Windows x64 CPU binary 和 `ggml-base.bin` 到应用数据目录，并自动保存 ASR 配置。
- 已接入 `转写并上屏最近录音`：使用最近一次录音做真实转写后，通过剪贴板 adapter 尝试上屏。
- 已接入诊断日志滚动和 `复制全部日志`，最多保留 100 条 UI 诊断日志。
- `转写并上屏最近录音` 现在先转写，再等待 3 秒让维护者切回目标输入框，然后发送剪贴板上屏请求。
- 录音结果新增 `peakAmplitude` 和 `rmsAmplitude` 音量摘要，用于排查 whisper.cpp 返回 `(音)` 的原因。
- 已接入输入设备列表和输入设备选择；如果默认设备显示 `Remote Audio`，优先切到真实麦克风再录音。
- 已接入 `导出最近录音 WAV`，会把最近一次 16 kHz ASR 输入导出到 Tauri app data 的 `diagnostics/last-asr-input.wav`，用于播放确认 whisper.cpp 实际收到的声音。
- 已修复剪贴板上屏粘贴旧内容的问题：当前策略写入后会读回校验，并且 MVP 暂时不恢复旧剪贴板，优先保证目标输入框读到本次文本。
- 已将 ASR 重采样改为线性插值，减少 44.1 kHz 到 16 kHz 这种非整数比例转换的失真。
- 已定位并修复录音变音根因：双声道输入在第一次回调后被错误当作单声道处理，导致后续样本错位；详见 `docs/harness/debugging-log.md`。
- 维护者再次手动验证 `真实闭环成功`，覆盖录音、whisper.cpp 转写、延迟切回目标输入框和剪贴板上屏链路。
- 默认 UI 已改为 macOS 风格用户主界面，原测试工作台保留在 `诊断模式`。
- UI 已继续二次打磨：主界面和诊断模式统一为更接近 macOS 原生工具的 translucent panel 风格。
- 已增强一键安装可靠性：下载器设置 20 秒连接超时和 300 秒单文件下载总超时，另一台机器网络不可达时应返回失败而不是无限卡住。
- 已启用 Tauri bundle，避免把 debug exe 直接复制到另一台机器后 WebView 找 `localhost:1420`。
- 已更新规则：验证、手动测试或用户反馈发现 bug 时必须记录到 `docs/harness/debugging-log.md`；影响状态时同步 progress、feature list 或 handoff。
- 已修正 Tauri identifier 为 `dev.voxtype.desktop`。
- 维护者确认暂不需要 `CONTRIBUTING.md`；`init.sh` 和 README 已不再把它作为必需入口。
- 中间调研资料保存在 `TMP/research/`；第三方克隆在 `TMP/research/repos/` 且被 Git 忽略。
- 维护者文档语言规则：默认中文；函数名、API 名、命令、仓库名、错误消息和专有名词保持原文。

## V2 当前设计

- V2 设计文档：`docs/superpowers/specs/2026-05-27-v2-daily-dictation-design.md`。
- V2 实施计划：`docs/superpowers/plans/2026-05-27-v2-daily-dictation.md`。
- 当前 feature：`v2-001`，状态 `in_progress`。
- UI 方向：主界面改为深色径向背景和半透明工具面板，不再使用夸张仿苹果窗口装饰；语音状态由彩色流光频谱波纹表达。
- 已接入：简体中文 prompt、输入设备持久化、`Ctrl+Alt+Space` 全局按住说话事件、按下开始录音、松开转写并上屏。
- 已修复：维护者反馈的“三条彩色线不动”问题，当前 `VoiceOverlay` 渲染 24 个动态频谱柱。
- 计划审计：`docs/superpowers/plans/2026-05-27-v2-daily-dictation.md` 已按当前证据勾选；唯一未勾选项是 Windows 桌面真实手动验证 `Ctrl+Alt+Space` 全局按住说话。
- 重要边界：TSF、完整模型市场、云端 ASR 和 AI 改写不在 V2 默认范围内。

## 仍可优化或待增强

- 识别质量仍可优化：模型选择、音量提示、VAD 和降噪都还没有做。
- 文本输出标准化已有第一步：whisper.cpp `zh` 调用会传简体中文 prompt；如果后续仍出现繁体，需要增加 OpenCC 类后处理或设置项。
- 上屏策略仍是剪贴板 + `Ctrl+V`，当前为了可靠性不恢复旧剪贴板；后续可评估延迟恢复、`SendInput(KEYEVENTF_UNICODE)` 或更深层输入法方案。
- 全局快捷键已接入代码，但还需要维护者在真实 Windows 桌面里验证是否和其他软件冲突、是否能在目标输入框上屏。
- 一键安装还缺模型选择器和国内镜像源。
- 一键安装已避免无限卡住，但仍缺进度条、取消按钮、镜像源和更详细的分阶段状态。
- 托盘入口已接入，但尚未手动确认系统托盘显示和菜单行为。
- CMake 已安装在 `C:/Program Files/CMake/bin/cmake.exe`，但当前 shell 的 PATH 尚未刷新。

## 下一步最佳动作

- 当前下一任务：手动验证 `v2-001` 第二版日常语音输入体验。
- 为什么它是下一步：主要代码已接入，必须确认真实 Windows 桌面里全局快捷键、录音、转写和上屏是否连贯。
- 建议优先级：先验证 `Ctrl+Alt+Space` 按住说话和动态波纹；通过后再做快捷键设置、下载进度、模型选择和音量/VAD。
- 不要动：不要把 TSF 当作 MVP 默认路线；不要提交模型文件、音频文件或密钥。

## 命令

- 启动/验证命令：`bash init.sh`
- JSON 检查：`python -m json.tool docs/harness/feature_list.json`
- 文件清单：`rg --files`
