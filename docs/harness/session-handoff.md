# 会话交接

## 2026-06-01 V6 收尾与 V7 交接

- 维护者已手动验证百度短语音识别真实调用可用：`http://vop.baidu.com/server_api` 配合已保存的 `BAIDU_ASR_API_KEY` 和 `BAIDU_ASR_SECRET_KEY` 可以准确识别。
- V6 可以先收尾提交：自定义快捷键、浅绿浮窗、停止后转写动效、连续输入记录合并、识别记录导出反馈、MiniMax Key 管理、百度 ASR 配置和真实调用均已进入当前实现。
- 下个版本建议聚焦模型选择/模型配置页面重构，不混入 V6 提交。
- V7 目标：模型选择需要清楚表达“当前选中的转写引擎”，并允许用户持久化切换。
- V7 交互方向：本地 whisper.cpp、百度 API、MiniMax 三个模型入口并列展示，而不是“云端 API”下面再嵌套 provider 按钮。
- V7 默认候选：百度 API 可作为默认选中项；MiniMax 暂时保留但标注为未接通真实 ASR；本地模型仍可被用户选中并记住。
- V7 状态要求：用户点击任一模型卡/按钮即代表切换当前转写引擎，保存到配置；重启后保持上次选择；主界面云端/本地状态必须跟随真实选择动态显示。
- V7 设计注意：需要重新设计模型配置页的信息架构，避免“当前配置完整”和“当前正在使用”混淆。

## 2026-05-30 V6 当前交接

- 当前唯一活跃任务：`v6-001`，状态仍为 `in_progress`，原因是代码和自动验证已完成，但还等待维护者统一手动验收真实桌面行为。
- 本轮已修复维护者反馈：`Ctrl+Alt+Space` 松开后显示六点转写动效；`Ctrl+Alt+V` 第二次停止后显示六点转写动效；连续输入的识别记录合并为一次录音一条记录；识别记录导出后主界面显示复制反馈；MiniMax API Key 使用密码输入框并写入用户环境变量 `MINIMAX_API_KEY`。
- MiniMax Key 写入实现：Windows 下直接写 `HKCU\\Environment` 并广播环境变更，不通过 `setx` 命令行参数传递密钥；诊断日志、项目配置和文档只记录变量名或脱敏状态，不记录真实 Key。
- MiniMax 真实 ASR 仍未启用：官方 ASR endpoint、上传字段和返回文本字段未确认，当前只有配置、状态检测和占位抽象。
- 自动验证已通过：`npm test -- --run`、`npm run typecheck`、`npm run build`、`cargo check --manifest-path src-tauri/Cargo.toml --lib`、`cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`、`cargo fmt --all --manifest-path src-tauri/Cargo.toml --check`、`python -m json.tool docs/harness/feature_list.json`、`git diff --check`。
- 下一步：让维护者运行 `npm run tauri -- dev`，按 V6 手动验收清单验证真实快捷键、原生浮窗动效、连续输入历史、导出反馈和 MiniMax Key 写入。通过后再把 `v6-001` 标记为 `passing`。

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
- V2 日常输入体验已按维护者验收意见收尾：`Ctrl+Alt+Space` 按住说话、桌面浮窗、主界面闭环和诊断日志已可用；残余透明浮窗浅色边框记录为平台风险。
- V3 已完成并由维护者验证核心功能：界面精修、`Ctrl+Alt+V` 切换录音、实验分段上屏和停止尾段补转写均可用。
- V4 已完成调研和计划，下一步只解决桌面浮窗外层白/灰框问题。
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
- 当前 feature：`v2-001`，状态 `passing`。
- UI 方向：主界面改为深色径向背景和半透明工具面板，不再使用夸张仿苹果窗口装饰；语音状态由彩色流光频谱波纹表达。
- 已接入：简体中文 prompt、输入设备持久化、`Ctrl+Alt+Space` 全局按住说话事件、按下开始录音、松开转写并上屏。
- 已接入：全局快捷键注册状态展示。主界面快速设置会显示 `Ctrl+Alt+Space` 或 `需处理`，诊断模式 `全局快捷键` 会显示注册成功或失败原因。
- 已接入：诊断模式 `刷新全局快捷键状态` 按钮；输入设备偏好恢复已改为状态驱动，减少异步竞态。
- 已接入：主界面主按钮日常闭环；再次点击停止录音后会转写、等待切回目标输入框并上屏。诊断模式仍保留纯录音采集按钮。
- 已接入：独立 Tauri 桌面浮窗 `dictation-overlay`，诊断模式可用 `测试桌面浮窗` / `隐藏桌面浮窗` 单独验证。
- 已接入：主窗口收到全局快捷键事件时记录 `收到全局快捷键按下` / `收到全局快捷键松开`，用于判断快捷键是否真的进入 VoxType。
- 已补强：overlay command 找不到窗口或显示失败时返回明确错误；快捷键闭环完成或失败后前端会隐藏桌面浮窗。
- 已修复：维护者反馈的“三条彩色线不动”问题，当前 `VoiceOverlay` 渲染 24 个动态频谱柱。
- 当前人工验证状态：维护者后续确认功能已实现，底部浮窗残余浅色边框暂不阻塞本版本。
- 计划审计：`docs/superpowers/plans/2026-05-27-v2-daily-dictation.md` 已按当前证据收尾。
- 重要边界：TSF、完整模型市场、云端 ASR 和 AI 改写不在 V2 默认范围内。

## V3 当前设计

- V3 设计文档：`docs/superpowers/specs/2026-05-28-v3-ui-and-toggle-dictation-design.md`。
- V3 实施计划：`docs/superpowers/plans/2026-05-28-v3-ui-and-toggle-dictation.md`。
- 当前 feature：`v3-001`，状态 `passing`。
- UI 方向：主界面和诊断模式继续走深色精致工具风格，降低字号、间距、边框和卡片重量，不做仿 macOS 红黄绿窗口按钮。
- 已接入：`Ctrl+Alt+V` 切换录音模式。第一次按下开始录音，录音中实验分段转写并上屏，第二次按下停止并补转写尾段。
- 重要边界：当前 `Ctrl+Alt+V` 不是边说边出字；whisper.cpp 当前仍是停止后统一转写。真正实时流式 ASR 需要后续单独设计。
- 平台风险：Tauri/WebView2 透明 overlay 在 Windows 上仍可能出现浅色边框或 ghost 背景，已经作为 V4 独立任务处理。

## V4 当前设计

- V4 设计文档：`docs/superpowers/specs/2026-05-28-v4-native-overlay-design.md`。
- V4 实施计划：`docs/superpowers/plans/2026-05-28-v4-native-overlay.md`。
- 当前 feature：`v4-001`，状态 `in_progress`。
- 唯一目标：消除桌面语音浮窗黑色胶囊外侧的 Tauri/WebView2 白色、灰色或浅色矩形边框。
- 推荐路径：先给 `dictation-overlay` 验证 `shadow: false` 和 WebView overlay 刷新补丁；如果仍不达标，再实现 Windows-only 原生 layered overlay。
- 回退要求：原生 overlay 失败时必须自动回退 WebView overlay，不影响录音、转写和上屏。

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

- 当前下一任务：按 `docs/superpowers/plans/2026-05-28-v4-native-overlay.md` 执行 Task 1，先验证 Tauri `shadow: false` 是否能去掉外层白/灰框。
- 为什么它是下一步：维护者已经确认 V3 核心功能可用，剩余最明显问题是 overlay 平台渲染边框。
- 建议优先级：先做低风险 WebView shadowless 补丁并让维护者人工验收；不通过再进入 Windows 原生 overlay。
- 不要动：不要把 TSF 当作 MVP 默认路线；不要提交模型文件、音频文件或密钥。

## 命令

- 启动/验证命令：`bash init.sh`
- JSON 检查：`python -m json.tool docs/harness/feature_list.json`
- 文件清单：`rg --files`

## 2026-06-01 V7 交接

- V7 代码实现、UI polish 和自动验证已完成，当前工作区包含未提交实现改动。
- 主界面准备状态已改为麦克风、上屏、按住说话模型、连续输入模型。
- 模型选择页已改为两块：输入模式默认模型、模型配置。
- 按住说话和连续输入可分别选择 `local-whisper`、`baidu-short`、`baidu-realtime`，默认都是 `baidu-short`，并通过 `save_mode_model_preferences` 持久化。
- MiniMax 已从用户配置界面移除；后端历史占位代码仍保留，后续可单独清理。
- 百度短语音配置增加可选 `lm_id`，请求 JSON 会在配置存在时发送 `lm_id`。
- 百度实时 WebSocket 只作为 V8 预留配置入口，当前选择后不能真实转写。
- 已修复快捷键监听 stale state：全局快捷键事件处理会读取最新 ASR/cloud/model 状态。
- 已修复 V7 UI 回归：主界面可见问号图标改为本地 inline SVG；模型选择页重新分层为输入模式默认模型、模型配置、密钥区和参数区；统计缩写改为中文标签。
- 已通过：`npm test -- --run src/App.test.tsx`、`npm run typecheck`、`npm run build`、`cargo fmt --all --manifest-path src-tauri/Cargo.toml --check`、`cargo check --manifest-path src-tauri/Cargo.toml --lib`、`cargo test --manifest-path src-tauri/Cargo.toml cloud_asr --no-run`、`cargo test --manifest-path src-tauri/Cargo.toml cloud_asr_config --no-run`、`python -m json.tool docs/harness/feature_list.json`、`git diff --check`。
- `v7-001` 暂保持 `in_progress`：还需要维护者运行 Tauri 桌面环境，手动验收准备状态、模型选择持久化、MiniMax UI 移除、百度短语音真实转写、百度实时 WebSocket V8 占位和识别记录 metadata。
- 下一步：提交 V7 当前改动；维护者之后执行桌面手动验收，通过后再把 `v7-001` 标记为 `passing`。
- V7.1 追加视觉调整已完成：统计区恢复彩色图标胶囊，识别记录 metadata 拆成小胶囊，记录动作按钮放到每条记录右下角。
- V7.2 追加视觉调整已完成：识别记录标题去掉重复条数，总统计胶囊去掉内部嵌套胶囊；单条记录时间和正文同一行显示，正文换行时继续对齐正文列并留出时间列。
- V7.3 追加视觉调整已完成：单条识别记录内时间和上屏文本改为垂直居中对齐，避免一个偏上一个偏下。
- V7.4 追加视觉调整已完成：每条识别记录最后一行 metadata 胶囊和动作按钮整体右对齐并缩小尺寸，降低短文本记录里工具区喧宾夺主的问题。
- 模型页已从单页重构为“模型选择”和“模型配置”两个页签；模型选择页只负责按住说话模型和连续输入模型默认值，模型配置页只显示本地 whisper.cpp、百度短语音、百度实时 WebSocket 的配置切换和当前配置面板。
- 最新已通过：`npm test -- --run src/App.test.tsx`、`npm run typecheck`、`npm run build`、`git diff --check`。
- 下一步：提交 V7.4 当前改动；维护者之后执行 Tauri 桌面手动验收，通过后再把 `v7-001` 标记为 `passing`。
