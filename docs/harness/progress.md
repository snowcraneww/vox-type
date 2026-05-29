# Progress Log

## 当前已验证状态

- 仓库根目录：当前 Git 工作树根目录
- 当前阶段：MVP proof-of-life、V2 日常输入体验、V3 切换录音模式和 V4 原生无外框浮窗已完成；当前进入 V5 主窗体产品化重设计阶段。
- 产品 scaffold：`scaffold-001` 已标记为 `passing`。
- 许可证：Apache-2.0，见 `LICENSE`。
- 标准启动路径：`bash init.sh`
- 标准验证路径：`bash init.sh` 和 `python -m json.tool docs/harness/feature_list.json`
- 当前最高优先级未完成项：`v5-001`，主窗体产品化重设计。
- 文档语言规则：面向维护者的研究、方案、进度和规则文档默认中文；函数名、API 名、命令、仓库名、错误消息和专有名词保持原文。
- 当前 blocker：无。

## 2026-05-28 V4 原生浮窗视觉恢复

- V4 收尾：维护者确认原生浮窗外层白/灰矩形框已经消失，当前浮窗视觉先保持现状；V4 标记为 `passing`，下一阶段不继续盲调浮窗和主界面样式。
- 提交：`7012093 feat: refine native overlay and main shell`，使用项目级身份 `VoxType <maintainers@voxtype.dev>`。
- 提交前隐私检查：当前跟踪改动未命中本机用户名、个人邮箱、用户目录、旧 GitHub noreply 或本机仓库路径；`AGENTS.md` 和 `progress.md` 中的本机绝对路径已泛化。
- 提交前验证：`npm test -- --run src/App.test.tsx src/VoiceOverlay.test.tsx` 通过，2 个测试文件、11 个测试；`npm run typecheck` 通过；`npm run build` 通过；`cargo check --manifest-path src-tauri/Cargo.toml --lib` 通过；`cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` 通过；`cargo fmt --all --manifest-path src-tauri/Cargo.toml --check` 通过；`git diff --check` 通过。

- 主窗体二次调整：解决上一版“缩成一团”的问题，放宽用户主界面宽度和右侧声谱胶囊，同时精简文字和按钮文案，保留功能不变。

- 主窗体视觉对齐：用 React/CSS 将用户主界面改成和 native 浮窗一致的黑色精密胶囊风格，减少玻璃卡片和大标题感，保留诊断模式作为工作台。

- 追加体验修复：录音态光感从单向 sweep 改为左右往返；`Ctrl+Alt+V` 第二次触发后先停止录音并隐藏浮窗，再继续尾段转写和上屏，减少“第二次按了反应慢”的体感。

- 追加渲染升级：为解决 GDI 直接绘制和 `SetWindowRgn` 硬裁剪导致的锯齿和低分辨率感，原生浮窗改为 Rust 软件高倍渲染 BGRA 位图，再通过 `UpdateLayeredWindow` 做每像素 alpha 透明；这保留无外框，但胶囊边缘改为抗锯齿。

- 追加视觉微调：颜色回到更柔和的旧版蓝绿黄粉紫顺序，录音动效降速，6 个转写点从上下波浪改回原地呼吸加载；胶囊边缘通过内缩绘制和降低透明度弱化锯齿，仍不画可见边框。

- 追加视觉回调：按旧 WebView/SVG 浮窗参考恢复 20 根频谱柱和 6 个转写点的动效感觉，修正彩色光带为向右推进；继续保持 `native-win32` 无可见外边框策略。

- 维护者手动验证 `native-win32` 浮窗后确认：诊断模式无报错，浮窗外层白/灰矩形框已经消失。
- 维护者反馈：原生 GDI 版本虽然解决外框，但视觉比之前 WebView/SVG 版本粗糙，希望恢复彩色频谱柱录音态和六点转写态。
- 本轮没有修改 `asr`、`audio`、`recorder`、`whisper.cpp` adapter 或剪贴板上屏链路；识别准确率理论上不应由本轮 overlay 改动引起。
- 原生浮窗继续使用 Win32 `WS_POPUP`、`WS_EX_LAYERED`、`WS_EX_NOACTIVATE`、`CreateRoundRectRgn` 和 `SetWindowRgn`，这是去除外层白/灰矩形的关键。
- 为避免测试二进制继续引入额外 Win32 入口点，移除项目对 `windows-sys` 的显式依赖，改为在 `native_overlay.rs` 内声明最小 Win32 FFI。
- 原生绘制恢复为 20 根彩色频谱柱录音态和 6 个彩色点转写态，背景/描边统一为深黑，减少可见边缘。
- 自动验证：`cargo check --manifest-path src-tauri/Cargo.toml --lib` 通过；`npm test -- --run` 通过，6 个测试文件、21 个测试通过；`npm run typecheck` 通过；`cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` 通过；`cargo fmt --all --manifest-path src-tauri/Cargo.toml --check` 通过；`npm run build` 通过。
- 已知限制：`cargo test --manifest-path src-tauri/Cargo.toml overlay` 当前在测试 exe 启动阶段失败，错误为 `STATUS_ENTRYPOINT_NOT_FOUND`；PE 导入表显示测试二进制仍导入 `user32.dll!SetWindowDisplayAffinity`，属于当前 Windows 测试环境/依赖链入口点问题，不是 overlay 断言失败。

## 2026-05-28 V5 主窗体产品化重设计

- 2026-05-29 V5.1 细化：维护者确认新的信息架构方向，但要求准备状态不能只放图标，状态胶囊要直接沿用桌面浮窗里的彩色声波/六点动效语言。
- V5.1 实现：主界面改为“识别记录为主体”的控制中心，顶部保留 `VoxType` 和小号说明，右侧入口改为“模型选择”和“诊断”。
- V5.1 输入模式：左侧展示“按住说话”和“连续输入”两种模式，并分别显示 `Ctrl+Alt+Space`、`Ctrl+Alt+V` 的就绪状态；齿轮按钮暂作为下一版自定义快捷键入口占位。
- V5.1 准备状态：右侧保留麦克风、本地识别、上屏、云端 API 四项，每项有短文字和 tooltip；不再把快捷键重复放入准备状态。
- V5.1 动态状态：主界面中部直接复用 `VoiceOverlay` 的黑色胶囊、彩色声波和转写动效，但不重复显示识别正文，正文统一进入识别记录。
- V5.1 识别记录：本次运行期间按倒序保留转写文本，支持复制、重新上屏、删除、清空全部和导出文本；统计显示识别次数、累计录音时长、累计字数和平均速度。
- V5.1 模型选择：新增独立模型配置页面，迁移 whisper.cpp 可执行文件、模型路径、语言、一键安装、检测和保存入口；云端 API 作为下一版占位。
- V5.1 验证：`npm test -- --run src/App.test.tsx` 通过，10 个测试；`npm run typecheck` 通过。
- V5.1 完整自动验证：`npm test -- --run` 通过，6 个测试文件、25 个测试；`npm run typecheck` 通过；`npm run build` 通过；`cargo check --manifest-path src-tauri/Cargo.toml --lib` 通过；`cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` 通过；`cargo fmt --all --manifest-path src-tauri/Cargo.toml --check` 通过；`python -m json.tool docs/harness/feature_list.json` 通过；`git diff --check` 通过。
- V5.1 隐私和编码扫描：本轮修改文件未命中本机用户名、个人邮箱、用户目录、旧 GitHub noreply 或本机仓库路径；未命中问号乱码或替换字符。

- 2026-05-29 V5.2 紧凑布局修正：维护者反馈实际效果仍太松散，要求输入模式和准备状态保持同一行左右两侧、识别记录标题/统计/按钮压缩到同一行，并去掉中间过大的“已就绪”状态模块。
- V5.2 实现：默认主界面删除独立动态状态胶囊，把主要高度释放给识别记录；识别记录工具条合并标题、四项统计、清空和导出按钮，按钮文案改短并提高对比度。
- V5.2 模型选择：本地 whisper.cpp 和云端 API 改为左右 tab；选中本地时显示 whisper.cpp 配置，选中云端时显示下一版 API Key/模型/服务商配置占位。
- V5.2 自动验证：`npm test -- --run src/App.test.tsx` 通过，10 个测试；`npm run typecheck` 通过；`npm test -- --run` 通过，6 个测试文件、25 个测试；`npm run build` 通过；`python -m json.tool docs/harness/feature_list.json` 通过；`git diff --check` 通过。

- 2026-05-29 V5.3 浅色主题试验：根据维护者反馈，主窗体先从深色测试台风格改为浅色白底加薄荷绿径向渐变背景，文字改为深灰绿，卡片改为半透明白色玻璃质感。
- V5.3 范围约束：仅覆盖主界面 `.control-center.v51` 和模型选择页 `.model-shell` / `.model-panel` 的视觉主题；桌面浮窗 overlay 继续保持当前深色彩色动效，诊断功能和 ASR/快捷键逻辑不变。
- V5.3 自动验证：`npm test -- --run src/App.test.tsx` 通过，10 个测试；`npm run typecheck` 通过；`python -m json.tool docs/harness/feature_list.json` 通过；`git diff --check` 通过；`npm run build` 通过；隐私和乱码扫描未命中。
- 2026-05-29 V5.4 浅绿层次和密度修正：根据维护者反馈，将薄荷色调改为更正常的浅绿径向渐变，并让主窗体、panel、tab/card、识别记录项使用不同深浅的绿色层级。
- V5.4 布局修正：输入模式卡片改为单行结构，左侧为“模式名 + 快捷键”，右侧集中放 Ready 状态和快捷键设置按钮；准备状态项也改为“能力名 + 当前值”单行显示；识别记录减少上下留白并提高列表可用高度。
- V5.4 自动验证：`npm test -- --run src/App.test.tsx` 通过，10 个测试；`npm run typecheck` 通过；`python -m json.tool docs/harness/feature_list.json` 通过；`git diff --check` 通过；`npm run build` 通过；隐私和乱码扫描未命中。
- 2026-05-29 V5.5 视觉一致性修正：统一浅绿径向渐变光源为“上中部更绿、底部和四周更淡”，避免左右或边缘各自发绿造成方向混乱。
- V5.5 识别记录修正：识别记录容器从 grid 改为 flex column，列表使用 `align-content: start`，避免只有少量记录时工具条和记录之间、记录和底部之间出现异常大空白。
- V5.5 主题统一：模型选择页和诊断页按钮、panel、输入框、日志项、状态 pill 改为同一浅绿主题；模型页不再露出蓝色主按钮。
- V5.5 自动验证：`npm test -- --run src/App.test.tsx` 通过，10 个测试；`npm run typecheck` 通过；`python -m json.tool docs/harness/feature_list.json` 通过；`git diff --check` 通过；`npm run build` 通过；隐私和乱码扫描未命中。

- V5 实现进展：按计划拆出 `src/MainWindow.tsx` 和 `src/DiagnosticView.tsx`，`App.tsx` 主要保留运行时状态、Tauri command 调用和事件编排。
- 默认主界面已改为“语音输入控制中心”：顶部状态、两种输入模式、四项准备状态、最近结果和诊断入口；不再把主窗体当作录音动效舞台。
- 最近结果新增主界面动作：复制、重新上屏、清空；诊断模式继续保留 ASR 配置、录音测试、真实转写、真实闭环、浮窗测试和日志复制。
- 局部验证：`npm test -- --run src/App.test.tsx src/VoiceOverlay.test.tsx src/DictationOverlay.test.tsx` 通过，3 个测试文件、17 个测试通过；`npm run typecheck` 通过；`npm run build` 通过。
- 自动验证：`npm test -- --run` 通过，6 个测试文件、23 个测试通过；`npm run typecheck` 通过；`npm run build` 通过；`cargo check --manifest-path src-tauri/Cargo.toml --lib` 通过；`cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` 通过；`cargo fmt --all --manifest-path src-tauri/Cargo.toml --check` 通过；`python -m json.tool docs/harness/feature_list.json` 通过；`git diff --check` 通过。
- 当前状态：V5 自动验证通过，等待维护者手动验证主界面观感、两种快捷键模式和诊断模式能力后再标记为 `passing`。
- 维护者反馈：当前主窗体的根本问题不是单个声波柱或配色，而是功能没有站在用户角度规划，测试目标和日常入口混在一起，不像成熟产品。
- 本轮决策：先提交 V4，再进入 V5；V5 只重设主窗体产品功能和信息架构，底部原生浮窗保持当前效果，两种快捷键模式都保留。
- 使用技能：`superpowers:brainstorming` 用于重新定义产品意图，`plan-ceo-review` 用于审视主窗体职责，`design-consultation` 用于设计系统方向，`verification-before-completion` 用于收尾验证约束。
- 写入设计文档：`docs/superpowers/specs/2026-05-28-v5-main-window-product-redesign.md`。
- 写入实施计划：`docs/superpowers/plans/2026-05-28-v5-main-window-product-redesign.md`。
- V5 产品定位：主窗体是“语音输入控制中心”，不是录音动效舞台；它负责可用状态、输入模式、最近结果、设置入口和诊断入口。
- V5 范围：主界面信息架构、组件拆分、视觉层级、诊断边界和最近文本动作；不改 ASR、录音、剪贴板上屏、原生浮窗核心链路。

## 2026-05-28 V3 界面精修与切换录音模式

- 维护者确认 V2 功能已经实现，底部浮窗残余浅色边框暂不作为本版本阻塞项。
- 将 `v2-001` 标记为 `passing`，新增 `v3-001` 作为当前任务。
- 使用 `frontend-design`、`web-search`、`superpowers:test-driven-development`、`superpowers:writing-plans` 和 `superpowers:verification-before-completion` 约束本轮工作。
- 写入 V3 设计文档：`docs/superpowers/specs/2026-05-28-v3-ui-and-toggle-dictation-design.md`。
- 写入 V3 实施计划：`docs/superpowers/plans/2026-05-28-v3-ui-and-toggle-dictation.md`。
- 新增第二种全局快捷键模式：`Ctrl+Alt+V` 第一次触发开始录音，第二次触发停止、转写并上屏。
- 明确边界：当前不是实时流式 ASR，仍是停止后统一转写和上屏。
- 主界面和诊断模式继续压缩字号、间距、圆角和按钮密度，减少测试台观感。
- 调研 Tauri/WebView2 透明窗口浅色边框问题，记录到 `docs/harness/debugging-log.md`。当前判断更像 Windows WebView2/Tauri 透明窗口渲染限制，不作为 V3 阻塞项。
- 局部验证：`cargo test --manifest-path src-tauri/Cargo.toml hotkey` 通过，7 个 hotkey 测试通过；`npm test -- --run src/App.test.tsx` 通过，5 个前端测试通过；`npm run typecheck` 通过；`python -m json.tool docs/harness/feature_list.json` 通过。

## 2026-05-28 Ctrl+Alt+V 浮窗动效修复

- 维护者反馈：`Ctrl+Alt+V` 切换录音模式下，桌面浮窗会出现但没有录音动效，也没有边说边在目标窗口显示文字。
- 根因 1：`DictationOverlay` 没有把 `toggleStartRecording` / `toggleStopAndTranscribe` 映射到 recording/transcribing 状态。
- 根因 2：当前 ASR 仍是停止后整段调用 whisper.cpp，不是流式 ASR，因此不能实时上屏。
- 修复：补齐 overlay 对 toggle action 的状态映射；overlay 背景改为黑色兜底；主窗口录音状态视觉改为黑色胶囊声波，贴近桌面浮窗风格。
- 局部验证：`npm test -- --run src/DictationOverlay.test.tsx` 通过；`npm test -- --run src/DictationOverlay.test.tsx src/VoiceOverlay.test.tsx src/App.test.tsx` 通过，3 个测试文件、13 个测试通过；`npm run typecheck` 通过。

## 2026-05-28 实验分段实时输入与透明浮窗调研

- 维护者追问 Windows WebView2/Tauri 透明浮窗浅色边缘是否有理论解决办法，以及别人如何解决。
- 使用 `web-search` 调研 Tauri/WebView2 透明窗口公开问题：Tauri issue `#4881`、StackOverflow `Tauri transparent window only works when resized`、Tauri v2 window API、Tauri issue `#11654`、Tauri issue `#12450` 均显示 Windows 透明窗口存在平台差异或 workaround。
- 当前判断：这更像 WebView2/Tauri/Windows 宿主窗口合成层限制，不是普通 CSS 边框。短期策略是小尺寸深色兜底；若要彻底无白边，后续建议做 Win32/Direct2D/WinUI native overlay spike。
- 使用 `web-search` 调研 whisper.cpp 实时转写：`examples/stream` / `whisper-stream` 可以做 naive realtime inference，但当前项目的 adapter 是 `whisper-cli.exe` 整段命令行转写，不是真流式 ASR。
- 本轮实现实验分段实时输入：`Ctrl+Alt+V` 开始录音后，前端每约 4 秒请求一次正在录音缓冲区的新增片段，Rust 将新增片段重采样为 16 kHz 后交给 whisper.cpp 转写，返回文本后立即通过剪贴板策略上屏。
- Rust 新增 `RecordingSession::asr_samples_from`、`RecorderManager::active_asr_samples_from` 和 Tauri command `transcribe_active_recording_chunk`。
- 前端新增 `transcribeActiveRecordingChunk` client、`LiveTranscriptionChunk` 类型和 `Ctrl+Alt+V` 分段上屏逻辑；若没有任何分段文本成功上屏，停止时会用整段转写兜底。
- 已记录详细问题、原因、方案和限制到 `docs/harness/debugging-log.md`。
- 局部验证：`npm test -- --run src/App.test.tsx` 通过，6 个测试通过。

## 2026-05-28 Ctrl+Alt+V 浮窗状态收束

- 维护者反馈：实验分段实时输入可用，但 `Ctrl+Alt+V` 的桌面浮窗录音动效只闪一下，随后只剩黑色胶囊；停止时又出现六点转写动效。
- 根因：切换快捷键释放时会发送 `ignore`，旧 overlay 收到后回到 `idle`；停止事件 `toggleStopAndTranscribe` 又被映射成 `transcribing`，与当前“录音中已经分段转写”的语义不匹配。
- 修复：overlay 忽略 `ignore` 事件；`toggleStopAndTranscribe` 继续保持录音波形，等待主流程隐藏 overlay；保留 `Ctrl+Alt+Space` 的 `stopAndTranscribe -> transcribing` 行为。
- 原生 Windows overlay 不纳入当前 V3 收束范围，作为下一个版本独立规划项处理。
- 局部验证：`npm test -- --run src/DictationOverlay.test.tsx` 通过，1 个测试文件、5 个测试通过。

## 2026-05-28 Ctrl+Alt+V 尾段上屏修复

- 维护者反馈：实验分段实时输入可用但有延迟，第二次按 `Ctrl+Alt+V` 停止时，最后一小段短句可能没有上屏。
- 根因：停止前最后一次 active chunk 如果不足约 1 秒，会被 `quietShortChunk` 静默忽略；因为前面已有分段文本成功上屏，所以整段兜底不会触发，导致尾句丢失。
- 修复：停止后从完整的最近录音里按 `liveCursorRef` 取尾段并调用 whisper.cpp 转写，避免重复前面已上屏片段，同时补上最后短句。
- 新增 Rust command：`transcribe_last_recording_chunk`。
- 新增测试：前端覆盖已有实时片段后停止时继续上屏尾段；Rust 覆盖停止后的 `RecordedAudio` 按源样本游标取尾段 ASR 样本。

## 2026-05-27 开源隐私清理

- 维护者指出 Git commit 元数据中泄露个人信息。
- 已暂停 V2 功能推进，优先执行开源安全清理。
- 初始审计：当前跟踪文件未命中个人信息或常见密钥模式；18 个 commit metadata 命中个人姓名/邮箱/用户名。
- 已重写本地 Git 历史，把所有 author/committer 改为 `VoxType <maintainers@voxtype.dev>`。
- 已删除 `refs/original`，执行 reflog expire 和 Git GC。
- 已设置仓库 local Git 身份：`VoxType <maintainers@voxtype.dev>`。
- 重扫结果：commit metadata 个人信息扫描 `metadata_findings 0`；当前跟踪文件扫描 `tracked_findings 0`；全历史内容扫描 `history_content_findings 0`；`git fsck --no-reflogs --unreachable --no-progress` 无输出。
- 当前仓库没有 remote；如果后续添加或已在别处推送过旧历史，需要确保公开远端不保留旧提交。

## 课程学习状态

已逐节读取 Learn Harness Engineering 中文课程 12 讲正文摘要，并将对本项目的配置影响记录到 `docs/harness/lesson-synthesis.md`。

已覆盖主题：

1. 模型能力与执行可靠性的区别。
2. Harness 五子系统：指令、工具、环境、状态、反馈。
3. 仓库作为事实源。
4. 指令文件拆分。
5. 跨会话连续性。
6. 初始化阶段。
7. WIP=1 与范围边界。
8. 功能清单状态机。
9. 防止过早宣告完成。
10. 端到端验证。
11. 可观测性。
12. 清洁状态。

## 验证证据

### 2026-05-26 Harness Baseline

- `bash init.sh`：exit 0；输出包含 `VoxType harness baseline OK`、`License: Apache-2.0`、`Product scaffold: not started`。
- `python -m json.tool docs/harness/feature_list.json`：exit 0。
- UTF-8 内容断言：exit 0；确认 `feature_list.json`、`quality.md`、`evaluator-rubric.md`、`README.md`、`AGENTS.md` 中文内容可读。
- 文本扫描：旧的 MIT 候选信息只保留在 `docs/plans/原始需求.md` 原始草稿中；项目事实已在 README、LICENSE 和 harness 中改为 Apache-2.0。

### 2026-05-26 产品发现与开源项目调研

- 阅读 `docs/plans/原始需求.md`，整理出 VoxType 的目标、MVP 行为、非目标、风险和待确认问题。
- 使用 Tavily 和 GitHub API 搜索本地语音输入、Whisper、Windows 热键和文本上屏相关项目。
- 深读本地克隆：Handy、OpenLess、OpenWhispr、VoiceFlow、whisper-writer、tambourine-voice、faster-whisper-dictation。
- 验证生态参考：whisper.cpp、sherpa-onnx、local-whisper、FluidAudio、FluidVoice。
- 产出中间文档：`TMP/research/requirements-brief.md`、`TMP/research/repo-candidates.md`、`TMP/research/solution-analysis.md`。
- 产出正式文档：`docs/research/requirements-brief.md`、`docs/research/open-source-landscape.md`、`docs/research/technical-options.md`。
- 许可证判断：MIT/Apache-2.0 项目可作为更直接参考；GPL/AGPL 项目仅做概念参考，避免污染 Apache-2.0 项目。

### 2026-05-26 中文化与 MVP 技术方案草案

- 将 `docs/research/requirements-brief.md`、`docs/research/open-source-landscape.md`、`docs/research/technical-options.md` 改为中文维护者文档。
- 将 `TMP/research/README.md`、`TMP/research/requirements-brief.md`、`TMP/research/repo-candidates.md`、`TMP/research/solution-analysis.md` 改为中文中间文档。
- 新增 `docs/research/mvp-technical-proposal.md`，详细说明 Rust + Tauri 2 + React/TS MVP 方案、模块边界、上屏策略、ASR adapter、测试策略和阶段路线。
- 更新 `AGENTS.md` 和 `docs/research/README.md`，明确维护者文档默认中文。

### 2026-05-26 MVP 技术方向确认

- 维护者确认所有关键决策采用建议默认值：中文优先兼容英文、第一版接受剪贴板上屏、ASR 优先 whisper.cpp 路线、UI 接受“托盘 + 设置页 + 状态提示”。
- 更新 `docs/research/mvp-technical-proposal.md`，把状态改为已确认，并补充“剪贴板 -> SendInput -> TSF”的上屏策略解释。
- 创建 `openspec/changes/voxtype-mvp-technical-direction/proposal.md`。
- 创建 `openspec/changes/voxtype-mvp-technical-direction/tasks.md`。
- 将 `spec-001` 标记为 passing。

### 2026-05-26 MVP Scaffold 骨架

- 写入实现计划：`docs/superpowers/plans/2026-05-26-scaffold-voxtype-mvp.md`。
- 创建前端骨架：`package.json`、`vite.config.ts`、`tsconfig*.json`、`src/App.tsx`、`src/App.test.tsx`、`src/styles.css`。
- 创建 Tauri/Rust 骨架：`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json`、`src-tauri/src/*`。
- 创建 Rust 模块：`state`、`config`、`error`、`audio`、`asr`、`insertion`、`hotkey`、`recorder`。
- 创建 mock 闭环 command：`simulate_dictation`。
- 生成临时图标：`src-tauri/icons/icon.ico`。
- `npm install`：成功，0 vulnerabilities。
- `cargo fetch --manifest-path src-tauri/Cargo.toml`：成功。
- `npm run typecheck`：通过。
- `npm test -- --run`：通过，1 个测试文件、1 个测试通过。
- `cargo test --manifest-path src-tauri/Cargo.toml`：通过，8 个 Rust 测试通过。
- `npm run build`：通过。
- `cargo check --manifest-path src-tauri/Cargo.toml`：通过。
- 当前未完成：真实麦克风录音、真实 whisper.cpp 推理、真实剪贴板上屏、托盘入口、Windows 手动 E2E。
- CMake：已通过 `winget install --id Kitware.CMake -e --accept-package-agreements --accept-source-agreements` 安装到 `C:/Program Files/CMake/bin/cmake.exe`；当前 shell 的 PATH 尚未刷新，后续可直接调用完整路径或临时追加 PATH。

### 2026-05-27 MVP 系统能力适配边界

- 保留 `scaffold-001` 为 `in_progress`，因为真实按住录音、真实 whisper.cpp 模型推理闭环和 Windows 手动 E2E 仍未完成。
- 新增 Tauri 托盘入口：`src-tauri/src/tray.rs`，菜单包含显示主窗口和退出。
- 新增前端 Tauri client：`src/tauriClient.ts`，浏览器预览模式保留 fallback，Tauri 运行时调用真实 command。
- 新增默认输入设备探测：`get_default_input_info` 通过 `cpal` 返回麦克风名称、采样率和声道数。
- 新增音频 buffer 和 mono 标准化工具：`RecordingBuffer`、`normalize_to_mono_i16`；尚未建立持续录音 stream。
- 新增 `WhisperCppEngine` adapter：校验 binary/model、写入临时 16 kHz mono WAV、调用 whisper.cpp CLI、读取 stdout；模型和 binary 仍由本机配置提供，不提交到仓库。
- 新增 Windows 剪贴板上屏 adapter：通过 `arboard` 写入文本、`enigo` 发送 `Ctrl+V`、再恢复旧剪贴板文本；真实目标应用上屏仍需手动 E2E 验证。
- 修正 Tauri identifier：`dev.voxtype.app` -> `dev.voxtype.desktop`，避免 `.app` 结尾警告。
- 维护者确认暂不需要 `CONTRIBUTING.md`，因此 `init.sh` 和 README 不再把它作为必需入口；项目仍保持 Apache-2.0 开源许可证。
- `npm run typecheck`：通过。
- `python -m json.tool docs/harness/feature_list.json`：通过。
- `bash init.sh`：通过，输出 `Product scaffold: started`。
- `cargo fmt --all --manifest-path src-tauri/Cargo.toml`：通过。
- `npm test -- --run`：通过，1 个测试文件、1 个测试通过。
- `cargo test --manifest-path src-tauri/Cargo.toml`：通过，15 个 Rust 测试通过。
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`：通过。
- `npm run build`：通过。
- `npm run tauri -- build --debug`：通过，生成 `src-tauri/target/debug/vox-type.exe`。

### 2026-05-27 MVP 录音采集 proof-of-life

- 维护者手动验证 Tauri 桌面模式日志：麦克风探测成功，设备为 `Remote Audio / 44100 Hz / 2 声道`。
- 维护者手动验证 `simulate_dictation`：Rust command 返回 mock 文本 `这是 VoxType 的模拟转写结果。`。
- 维护者手动验证剪贴板上屏请求：程序已写入剪贴板、发送 `Ctrl+V` 并尝试恢复旧剪贴板；目标应用是否收到文本仍需按窗口焦点单独确认。
- 新增 UI 诊断日志面板，记录麦克风探测、模拟闭环、剪贴板上屏、录音启动/停止的成功或失败原因。
- 修复 React 开发模式 `StrictMode` 导致麦克风探测日志重复的问题。
- 修复 Mermaid 文档语法，避免节点标签中的括号、引号和 `#[...]` 被误解析。
- 新增 `docs/guide/run-and-understand.md` 和 `docs/guide/code-walkthrough.md`，面向 Rust/Tauri 新手解释运行、依赖库、模块边界和手动验证标准。
- 新增 `RecordingSession` 和 `RecorderManager`：可通过 Tauri command 启动/停止 `cpal` 输入 stream，并返回 `RecordedAudio` 的样本数、时长、采样率和声道信息。
- 新增前端按钮：`开始录音采集`、`停止录音采集`、`刷新录音状态`。
- 仍未完成：把录音结果重采样到 16 kHz、调用 whisper.cpp 做真实转写、把真实转写文本上屏、Windows Notepad/VS Code/浏览器输入框 E2E。
- TDD RED：新增 `RecordingSession` 行为测试后，`cargo test --manifest-path src-tauri/Cargo.toml recording_session` 因 `RecordingSession` 未实现失败。
- TDD GREEN：实现 `RecordingSession` 后，`cargo test --manifest-path src-tauri/Cargo.toml recording_session` 通过，2 个测试通过。
- `python -m json.tool docs/harness/feature_list.json`：通过。
- `bash init.sh`：通过。
- `npm run typecheck`：通过。
- `npm test -- --run`：通过，1 个测试文件、1 个测试通过。
- `cargo test --manifest-path src-tauri/Cargo.toml`：通过，17 个 Rust 测试通过。
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`：通过。
- `npm run build`：通过。
- `git diff --check`：通过。
- `npm run tauri -- build --debug`：首次因旧 `vox-type.exe` 进程占用失败；结束旧进程后重跑通过，生成 `src-tauri/target/debug/vox-type.exe`。

### 2026-05-27 录音采集手动验证与 ASR 输入准备

- 维护者手动验证录音采集成功：`开始录音采集` 后说话，`停止录音采集` 返回 `782863` 个 mono 样本、`17751 ms`、`44100 Hz`。
- 继续用 TDD 增加 `audio::resample_mono_i16`，把 mono PCM 从源采样率转换到目标采样率。
- TDD RED：新增重采样测试后，`cargo test --manifest-path src-tauri/Cargo.toml resampling` 因 `resample_mono_i16` 未实现失败。
- TDD GREEN：实现重采样后，`cargo test --manifest-path src-tauri/Cargo.toml audio::tests` 通过，4 个 audio 测试通过。
- `RecordedAudio` 现在包含 `asr_sample_rate`、`asr_sample_count`、`asr_duration_ms`，停止录音后 UI 诊断日志会显示已准备的 16 kHz ASR 样本数量。
- 仍未完成：把 ASR 样本交给 `WhisperCppEngine`，配置 whisper.cpp binary/model，真实转写后再上屏。

### 2026-05-27 whisper.cpp 最近录音转写命令

- 新增 `transcribe_last_recording` Tauri command：从 `RecorderManager` 读取最近一次录音的 16 kHz ASR 样本，并交给 `WhisperCppEngine`。
- 新增环境变量配置：`VOXTYPE_WHISPER_CPP_BINARY`、`VOXTYPE_WHISPER_CPP_MODEL`、`VOXTYPE_ASR_LANGUAGE`。
- 新增 UI 按钮：`转写最近录音`。
- 当前验证目标：如果环境变量未设置，诊断日志应显示明确的缺失变量；如果路径有效，whisper.cpp 应返回文本。
- 仍未完成：真实转写文本自动上屏、whisper.cpp binary/model 下载或配置 UI、Windows E2E。

### 2026-05-27 ASR 配置体验

- 使用 `superpowers:systematic-debugging` 定位维护者看到的 `[object Object]`：Rust 已返回结构化 Tauri error，但前端用 `String(error)` 展示对象。
- 使用 TDD 新增 `src/errorFormat.ts` 和测试，统一把 `{ message, code }` 格式化为中文可读错误，避免诊断日志再次出现 `[object Object]`。
- 新增 `src-tauri/src/asr_config.rs`：应用内 ASR 配置保存/读取、环境变量 fallback、binary/model 路径存在性检测、中文状态消息。
- 新增 Tauri commands：`get_asr_config_status`、`save_asr_config`。
- `transcribe_last_recording` 已改为优先使用应用内保存的 whisper.cpp binary/model 路径；环境变量只作为未保存配置时的兜底。
- UI 新增 `ASR 配置` 面板：可填写 `whisper-cli.exe` 路径、模型路径和语言，支持保存与检测。
- 仍未完成：文件选择器、一键下载 whisper.cpp/model、真实转写后自动上屏、Windows E2E。

### 2026-05-27 ASR 一键安装与真实闭环按钮

- 使用 web-search 和 GitHub API 确认官方来源：`ggml-org/whisper.cpp` GitHub Release 提供 `whisper-bin-x64.zip`，官方模型说明指向 `ggerganov/whisper.cpp` Hugging Face 仓库。
- 新增设计文档：`docs/superpowers/specs/2026-05-27-asr-one-click-install-design.md`。
- 新增实现计划：`docs/superpowers/plans/2026-05-27-asr-one-click-install.md`。
- 新增 `src-tauri/src/asr_installer.rs`：规划应用数据目录安装路径、下载 whisper.cpp Windows x64 CPU 版、校验/解压 ZIP、下载并校验 `ggml-base.bin`，并自动保存 ASR 配置。
- 新增 Tauri command：`install_managed_asr`。
- UI 新增 `一键安装 whisper.cpp` 按钮，安装完成后刷新 ASR 配置状态。
- 新增 `transcribe_last_recording_and_insert` command 和 `转写并上屏最近录音` UI 按钮，用于验证真实“录音 -> 转写 -> 剪贴板上屏”闭环。
- 仍未完成：模型选择器、镜像源、真实 Windows Notepad/VS Code/浏览器输入框 E2E。

### 2026-05-27 真实闭环手动验证与诊断增强

- 维护者手动验证 `转写并上屏最近录音` command 返回成功，但 whisper.cpp 文本为 `(音)`，说明当前链路跑通但识别质量或输入设备仍需诊断。
- 根据维护者反馈，修正诊断日志体验：日志不再只保留最新 6 条，改为保留最多 100 条，并提供滚动区域和 `复制全部日志` 按钮。
- 根据维护者反馈，修正上屏验证流程：`转写并上屏最近录音` 现在先完成转写，再给维护者 3 秒切回目标输入框，之后才发送剪贴板上屏请求。
- 新增录音音量摘要：`peakAmplitude` 和 `rmsAmplitude`，用于判断 `(音)` 是因为录音太小、未录到麦克风，还是默认输入设备不对。

### 2026-05-27 MVP 真实闭环通过

- 维护者再次手动验证 `真实闭环成功`。
- 当前证据覆盖：真实录音采集、whisper.cpp 转写、延迟切回目标输入框、剪贴板上屏请求。
- 将 `scaffold-001` 标记为 `passing`。后续工作从“证明 MVP 能跑通”转向“提升可用性和可靠性”。

### 2026-05-27 真实闭环调试增强

- 维护者反馈：长时间说话后 whisper.cpp 仍只返回 `(音)`；同时真实上屏粘贴了旧剪贴板内容，而不是本次识别文本。
- 使用 `superpowers:systematic-debugging` 拆分为两个边界问题：ASR 输入质量/设备选择问题，以及剪贴板写入/粘贴时序问题。
- 剪贴板根因：旧实现写入新文本后立即发送 `Ctrl+V`，随后马上恢复旧剪贴板；Windows 目标软件可能晚于恢复动作读取剪贴板，因此粘贴到旧内容。
- 剪贴板修复：写入后读回校验，发送 `Ctrl+V` 改用 `Key::V`，并且 MVP 暂时不恢复旧剪贴板，优先保证目标输入框读到本次识别文本。
- ASR 调试增强：新增输入设备列表和输入设备选择，避免只能使用系统默认的 `Remote Audio`；新增 `导出最近录音 WAV`，把最近一次 16 kHz ASR 输入写为 `last-asr-input.wav` 便于直接播放复盘。
- ASR 输入质量修复：44.1 kHz -> 16 kHz 重采样从跳点抽样改为线性插值，降低非整数采样率转换失真。
- 文档更新：`README.md`、`AGENTS.md`、`docs/guide/run-and-understand.md`、`docs/guide/code-walkthrough.md`、`docs/harness/quality.md` 已同步当前状态和验证步骤。

### 2026-05-27 录音变音根因定位

- 维护者播放 `last-asr-input.wav` 后反馈：文件里有自己的声音，但声音变成类似电子音，节奏像原始录音。
- 使用 TDD 新增回归测试 `stereo_samples_remain_mixed_across_multiple_callbacks`，复现多次 `cpal` 回调后双声道 interleaved 样本被错误当作 mono 追加的问题。
- 根因：`RecordingBuffer::push_interleaved_i16` 第一次处理双声道回调后把 `channels` 改成 `1`；后续回调仍是双声道输入，但代码不再按双声道 frame 混音。
- 修复：新增 `source_channels` 保存输入设备真实声道数，后续每次回调都按 `source_channels` 分组混音；`channels` 只表示输出 buffer 已经归一化为 mono。
- 新增 `docs/harness/debugging-log.md`，集中记录本轮调试发现的问题、根因、修复和验证方式。
- 验证：`cargo test --manifest-path src-tauri/Cargo.toml` 通过，34 个 Rust 测试全部通过；`npm run typecheck`、`npm test -- --run`、`cargo fmt --all --manifest-path src-tauri/Cargo.toml --check` 通过。

### 2026-05-27 主界面重设计与安装器可靠性增强

- 使用 `frontend-design`、`superpowers:test-driven-development`、`superpowers:systematic-debugging`、`documentation-writer` 和 `superpowers:verification-before-completion` 对本轮工作进行约束。
- 默认界面从测试工作台改为面向用户的 `VoxType` 主界面：macOS 风格窗口、主录音按钮、最近识别文本、快速设置和 ASR 未就绪提示。
- 原测试操作保留在 `诊断模式`：运行状态、ASR 配置、测试操作和诊断日志仍可访问。
- 修复前端测试隔离：`src/test/setup.ts` 在每个测试后调用 Testing Library `cleanup()`，避免不同测试渲染结果互相污染。
- 修复 jsdom 环境没有 `scrollIntoView` 导致测试失败的问题：诊断日志自动滚动改为可选调用。
- 一键安装下载器改为显式 `reqwest::blocking::Client`，设置 20 秒连接超时和 300 秒单文件下载总超时，避免目标机器访问 GitHub/Hugging Face 异常时无限卡住。
- 更新规则：验证、手动测试或用户反馈发现 bug 时，必须记录到 `docs/harness/debugging-log.md`，影响状态时同步 progress、feature list 或 handoff。
- 更新 `docs/guide/run-and-understand.md`：说明默认主界面、诊断模式、一键安装超时行为和当前剪贴板上屏策略。
- 本轮局部验证：`npm test -- --run src/App.test.tsx` 通过，2 个前端测试通过；`cargo test --manifest-path src-tauri/Cargo.toml asr_installer` 通过，6 个安装器测试通过；`npm run typecheck`、`npm run build`、`cargo fmt --all --manifest-path src-tauri/Cargo.toml --check` 通过。

### 2026-05-27 UI 二次打磨与跨机器安装包

- 根据维护者反馈，继续优化主界面和诊断模式视觉：统一为更克制的 macOS translucent panel 风格，调整字体层级、圆角、阴影、状态色、按钮和诊断卡片密度。
- 定位跨机器复制 `src-tauri/target/debug/vox-type.exe` 后显示 `localhost refused to connect` 的原因：debug exe 不是面向分发的 bundle，另一台机器没有 `localhost:1420` 的 Vite dev server。
- 启用 Tauri bundle：`src-tauri/tauri.conf.json` 中 `bundle.active=true`，icon 使用 `icons/icon.ico`。
- 生成跨机器验证产物：`src-tauri/target/debug/bundle/nsis/VoxType_0.1.0_x64-setup.exe` 和 `src-tauri/target/debug/bundle/msi/VoxType_0.1.0_x64_en-US.msi`。
- 文档更新：`docs/guide/run-and-understand.md` 明确本机 debug exe 和跨机器安装包的区别；`docs/harness/debugging-log.md` 记录 localhost refused 问题。
- 验证：`npm test -- --run src/App.test.tsx`、`npm run typecheck`、`npm run build` 通过；`npm run tauri -- build --debug` 首次因旧进程占用失败，结束旧 `vox-type.exe` 后重跑通过并生成 NSIS/MSI bundle。

### 2026-05-27 第一版 MVP 完成判断与下一阶段

- 维护者反馈：NSIS 安装包在另一台机器可以安装成功，并能运行到转写链路。
- 维护者反馈：另一台机器返回的转写文本是繁体中文，而本虚拟机上是简体中文。
- 系统性调试判断：当前 whisper.cpp 只传 `-l zh -nt`，未传简体 prompt，也未做繁简转换后处理；这是文本标准化缺口，不是 MVP 链路阻断。
- 结论：第一版 MVP 可视为完成；第二阶段建议从“简体中文输出标准化”、全局快捷键、设备选择持久化、模型选择和下载进度开始。
- 文档更新：`README.md`、`docs/harness/quality.md`、`docs/harness/session-handoff.md`、`docs/harness/debugging-log.md` 已同步第一版 MVP 完成判断和繁体输出问题。

### 2026-05-27 V2 日常语音输入体验推进

- 使用 `superpowers:brainstorming`、`frontend-design`、`superpowers:systematic-debugging`、`superpowers:test-driven-development`、`documentation-writer` 和 `document-release` 对本轮设计、调试、实现和文档同步进行约束。
- V2 设计文档和实施计划已写入 `docs/superpowers/specs/2026-05-27-v2-daily-dictation-design.md`、`docs/superpowers/plans/2026-05-27-v2-daily-dictation.md`。
- 已移除仿苹果红黄绿窗口按钮，主界面改为深色径向背景、半透明工具面板和更克制的日常输入布局；诊断模式保持可进入。
- 根据维护者反馈，修复“录音时三条彩色线不动”的视觉问题：`VoiceOverlay` 从三条横线改为 24 个彩色频谱柱和流动光带，录音态会错峰起伏。
- whisper.cpp `zh` 语言调用新增简体中文 prompt，降低跨机器返回繁体中文的概率。
- 新增输入设备偏好持久化：选择输入设备后保存到 Tauri app config，下次启动会尝试恢复。
- 新增全局按住说话基础闭环：Rust 通过 `tauri-plugin-global-shortcut` 注册 `Ctrl+Alt+Space`，按下向前端发送 `startRecording`，松开发送 `stopAndTranscribe`；前端收到事件后停止录音、调用 whisper.cpp 并立即剪贴板上屏。
- 修复规则文档中 PowerShell/UTF-8 规则行的控制字符污染，并记录到 `docs/harness/debugging-log.md`。
- 已运行验证：`npm test -- --run` 通过，5 个测试文件、12 个测试通过；`npm run typecheck` 通过；`cargo test --manifest-path src-tauri/Cargo.toml` 通过，42 个 Rust 测试通过。
- 仍需维护者手动验证：在 Tauri 桌面模式中把光标放到 Notepad、VS Code 或浏览器输入框，按住 `Ctrl+Alt+Space` 说话，松开后确认目标输入框出现识别文本，并确认语音波纹在录音态连续起伏。
- 2026-05-27 目标完成审计：V2 计划文件已按当前代码和证据勾选；自动化验证覆盖 UI、类型、Rust 状态机、构建和文档格式，但不能证明 OS 全局快捷键和目标软件焦点上屏，所以 `v2-001` 继续保持 `in_progress`，直到维护者完成上述手动验证。
- 2026-05-27 继续补强：新增全局快捷键注册状态可观测性。Rust 保存 `HotkeyRegistrationStatus`，前端通过 `get_hotkey_status` 在主界面和诊断模式显示 `Ctrl+Alt+Space` 是否注册成功；如果失败，维护者不需要只看终端猜测原因。
- 2026-05-27 V2 收尾补强：修复输入设备偏好恢复竞态，避免用户偏好和设备列表异步返回顺序导致已保存麦克风偶尔不恢复；诊断模式新增 `刷新全局快捷键状态` 按钮，方便维护者验证前后重新读取注册状态。
- 2026-05-27 分发烟测：`npm run tauri -- build --debug` 首次因旧 `vox-type.exe` 进程占用失败；结束旧进程后重跑通过，生成 `src-tauri/target/debug/bundle/msi/VoxType_0.1.0_x64_en-US.msi` 和 `src-tauri/target/debug/bundle/nsis/VoxType_0.1.0_x64-setup.exe`。
- 2026-05-27 V2 主界面闭环补强：主界面主按钮从“停止录音采集”改为日常输入闭环，再次点击会停止录音、调用 whisper.cpp、等待 3 秒切回目标输入框并尝试上屏；诊断模式仍保留纯录音采集按钮用于排查。

## 会话记录

### 会话 001 - 2026-05-26

- 本轮目标：深入学习 harness 课程，并为 VoxType 建立开源项目 harness 基线。
- 已完成：
  - 读取课程目录、官方模板和 12 讲正文摘要。
  - 创建 `docs/harness/lesson-synthesis.md`，把课程原则映射到 VoxType。
  - 创建/更新 `AGENTS.md`、`README.md`、`init.sh`。
  - 创建 `LICENSE`，许可证采用 Apache-2.0。
  - 创建 `.gitignore`、`.editorconfig`、`.gitattributes`。
  - 创建 `docs/harness/working-agreement.md`、`feature_list.json`、`quality.md`、`evaluator-rubric.md`、`session-handoff.md`、`clean-state-checklist.md`、`research-log.md`。
  - 创建 `docs/research/README.md` 和 `openspec/README.md`。
- 运行过的验证：
  - `bash init.sh`
  - `python -m json.tool docs/harness/feature_list.json`
  - UTF-8 内容断言
  - 文本扫描旧许可证和占位符
- 已记录证据：见本文件“验证证据”和 `docs/harness/feature_list.json`。
- 更新过的文件或工件：
  - `AGENTS.md`
  - `README.md`
  - `LICENSE`
  - `.gitignore`
  - `.editorconfig`
  - `.gitattributes`
  - `init.sh`
  - `docs/harness/*`
  - `docs/research/README.md`
  - `openspec/README.md`
- 已知风险或未解决问题：
  - 原始需求文件仍是草稿，包含旧的 MIT 候选信息；项目事实以 Apache-2.0 为准。
  - 产品技术栈尚未决策。
- 下一步最佳动作：执行 `discovery-001`，把 `docs/plans/原始需求.md` 整理成正式需求 brief。

### 会话 002 - 2026-05-26

- 本轮目标：学习原始需求，调研相关开源项目，并形成 VoxType 技术方向建议。
- 已完成：
  - 将第三方调研仓库集中在 `TMP/research/repos/`，并通过 `.gitignore` 排除。
  - 写入 `TMP/research/` 中间调研文档。
  - 写入 `docs/research/` 长期调研文档。
  - 将 `discovery-001` 和 `research-001` 标记为 passing。
- 初步建议：Rust + Tauri 2 + React/TS，Windows first，本地优先；MVP 先做 clipboard paste + restore，上屏可靠性第二阶段做 `SendInput(KEYEVENTF_UNICODE)`，TSF IME 后置。
- 下一步最佳动作：执行 `spec-001`，用 superpowers:brainstorming 和 OpenSpec 把 MVP 方案正式收敛并等待维护者批准。

### 会话 003 - 2026-05-26

- 本轮目标：把调研资料中文化，并写出 Rust + Tauri 2 + React/TS 的详细 MVP 技术方案草案。
- 已完成：
  - 中文化 `docs/research/` 和 `TMP/research/` 主要文档。
  - 新增 `docs/research/mvp-technical-proposal.md`。
  - 在 `AGENTS.md`、`docs/research/README.md` 和 harness 中加入“维护者文档默认中文”的规则。
- 下一步最佳动作：维护者审阅 `docs/research/mvp-technical-proposal.md`，确认或修改关键决策后进入 OpenSpec。

### 会话 004 - 2026-05-26

- 本轮目标：解释上屏策略分层和 TSF，并将维护者确认的 MVP 技术决策写入 OpenSpec。
- 已完成：
  - 解释并记录剪贴板、`SendInput(KEYEVENTF_UNICODE)`、TSF 的分阶段含义。
  - 把 `docs/research/mvp-technical-proposal.md` 从草案更新为已确认方案说明。
  - 新增 OpenSpec change：`openspec/changes/voxtype-mvp-technical-direction/`。
  - 将 `spec-001` 标记为 passing。
- 下一步最佳动作：为 `scaffold-001` 写详细 implementation plan，之后再开始 scaffold 产品代码。

### 会话 005 - 2026-05-26

- 本轮目标：按已确认方案开始 scaffold。
- 已完成：
  - 写入详细 implementation plan。
  - 创建最小 Tauri 2 + React/TypeScript + Rust 骨架。
  - 创建前端设置页和模拟闭环状态展示。
  - 创建 Rust mock 核心模块和测试。
  - 安装 npm/cargo 依赖，并运行基础验证。
- 下一步最佳动作：继续 `scaffold-001`，接入真实托盘、真实录音、whisper.cpp 路线 proof-of-life、剪贴板上屏，并进行 Windows 手动 E2E。

### 2026-05-27 V2 日常语音输入体验启动

- 维护者批准第二版方向：主界面要克制、专业，不再做夸张仿苹果窗口装饰；语音输入状态用小型彩色流光波纹浮窗表达。
- 使用 `web-search` 调研 Windows 语音输入和 Siri/AI voice waveform 类交互，结论是 V2 应贴近“光标在目标输入框、按住快捷键、浮窗提示、文字进入当前光标”的系统语音输入模型。
- 使用 `frontend-design` 约束 UI 方向：主界面安静，彩色和动效集中在语音浮窗，不让整页变花。
- 新增设计文档：`docs/superpowers/specs/2026-05-27-v2-daily-dictation-design.md`。
- 新增实施计划：`docs/superpowers/plans/2026-05-27-v2-daily-dictation.md`。
- 新增 feature：`v2-001`，状态 `in_progress`。
### 2026-05-27 V2 浮窗与主界面第一批实现

- 使用 TDD 新增 `src/voiceOverlayModel.ts` 和 `src/voiceOverlayModel.test.ts`，覆盖录音、成功截断和失败诊断提示。
- 使用 TDD 新增 `src/VoiceOverlay.tsx` 和 `src/VoiceOverlay.test.tsx`，覆盖彩色流光语音浮窗的录音、转写、成功和失败状态。
- 主界面移除夸张仿苹果标题栏和红黄绿窗口按钮，改为更克制的工具窗口；默认主界面嵌入语音浮窗作为 V2 状态预览。
- 根据本轮 PowerShell 中文替换踩坑，更新 `AGENTS.md` 和 `docs/harness/working-agreement.md`：编辑中文文件时避免在 PowerShell 命令里用中文片段局部替换，优先使用 Git Bash、ASCII 锚点或 UTF-8 no BOM 整文件写入。
- 验证通过：`npm test -- --run src/voiceOverlayModel.test.ts src/VoiceOverlay.test.tsx src/App.test.tsx`、`npm test -- --run`、`npm run typecheck`、`npm run build`、`git diff --check`。

### 2026-05-27 V2 全局快捷键无反馈调试

- 维护者手动验证：在 VS Code 输入框中按住 `Ctrl+Alt+Space` 说中文，没有出现录音反馈，松开后也没有文字进入输入框。
- 根因调查结论：`tauri-plugin-global-shortcut` 和底层 `global-hotkey` 支持 Windows `Pressed/Released`，但旧实现缺少桌面级可见反馈和热键事件诊断日志；用户无法区分“快捷键没进应用”和“进入应用后录音/转写/上屏失败”。
- 已新增 Tauri 隐藏窗口 `dictation-overlay`：默认不显示，收到按住说话事件后在屏幕底部显示小型五彩流光波纹浮窗。
- 已新增诊断按钮：诊断模式中 `测试桌面浮窗` / `隐藏桌面浮窗`，用于单独验证 overlay 窗口是否能显示在桌面上。
- 已新增热键事件日志：前端收到 `voxtype-push-to-talk` 后会记录“收到全局快捷键按下/松开”。如果按快捷键后没有这条日志，说明系统没有把该快捷键事件交给 VoxType。
- 已调整浮窗行为：按下快捷键显示“正在听”，松开后保持显示“正在识别”，直到快捷键闭环完成或失败后由前端隐藏。
- 继续补强：`show_dictation_overlay` / `hide_dictation_overlay` 现在会在找不到 overlay 窗口或系统显示失败时返回明确错误，避免诊断按钮误报成功。
- 继续补强：快捷键闭环完成或失败后前端统一调用 `hideDictationOverlay()` 收尾；新增回归测试覆盖按下开始录音、松开转写上屏和隐藏浮窗。
- UI 收敛：主界面继续压缩为深色半透明系统工具风格；底部小浮窗改为更小的胶囊状态条，录音态彩色频谱柱起伏更明显。
- 验证：`npm test -- --run src/App.test.tsx src/DictationOverlay.test.tsx` 通过，2 个测试文件、6 个测试通过；`npm run typecheck` 通过。

### 2026-05-27 V2 事件权限与浮窗视觉修复

- 维护者验证后提供关键日志：`event.listen not allowed`，同时全局快捷键注册成功、测试桌面浮窗可显示。
- 根因确认：Tauri 2 capability 未授予 `core:event:allow-listen`，导致前端不能监听 Rust 发出的 `voxtype-push-to-talk` 事件；快捷键注册本身不是失败点。
- 新增 `src-tauri/capabilities/default.json`，对 `main` 和 `dictation-overlay` 授予 `core:default`、`core:event:allow-listen`、`core:event:allow-unlisten`。
- 修复 React StrictMode 下异步 `listen()` 返回过晚可能造成的重复监听清理竞态。
- 浮窗视觉继续收敛：overlay 页面根节点透明化，去掉胶囊背后的黑色方框；窗口缩小到 `340 x 86`，胶囊、字体和阴影都缩小。
- 同步 Rust overlay 定位常量为 `340 x 86`，新增 `overlay_size_matches_tauri_window_config` 回归测试，避免配置尺寸和定位尺寸再次脱节。
- 验证：`npm test -- --run` 通过，6 个测试文件、16 个测试通过；`npm run typecheck` 通过；`cargo test --manifest-path src-tauri/Cargo.toml` 通过，45 个 Rust 测试通过；`cargo fmt --all --manifest-path src-tauri/Cargo.toml --check` 通过；`npm run build` 通过；`cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` 通过；`bash init.sh` 通过；`git diff --check` 通过；`npm run tauri -- build --debug` 第二次通过并生成 debug 包。第一次 debug build 失败是因为旧的 `vox-type.exe` 进程占用目标文件，关闭该项目进程后通过。

### 2026-05-27 V2 浮窗波纹重做

- 维护者反馈：底部浮窗仍像带文字的卡片，背后有白色框；期望只显示几条弯曲、流动、彩色的波纹线，不显示“正在听”等可见文字。
- 调整 `DictationOverlay`：移除可见文字、说明文案、胶囊卡片和柱状频谱，只保留 `aria-label` 给辅助技术使用。
- 重做浮窗视觉：使用透明 SVG 曲线光带，4 条曲线通过 `stroke-dashoffset`、位移、缩放和 SVG `animate attributeName="d"` 同时流动与变形，让曲率和边缘都动起来。
- 消除白框：`index.html` 在 React 加载前就给 overlay 页面加透明背景样式，避免透明 Tauri 窗口首帧出现浏览器默认白底。
- 窗口尺寸同步缩小到 `300 x 82`，Rust 定位常量和回归测试同步更新。
- 验证：`npm test -- --run` 通过，6 个测试文件、16 个测试通过；`npm run build` 通过；`npm run typecheck` 通过；`cargo test --manifest-path src-tauri/Cargo.toml overlay` 通过；`cargo fmt --all --manifest-path src-tauri/Cargo.toml --check` 通过；`git diff --check` 通过。

### 2026-05-27 V2 小胶囊多周期波纹调整

- 维护者反馈：波纹方向接近，但窗口仍太大、白色背景不对、波形周期太长导致峰谷太少，颜色没有流光感；按住录音时应流动，松开后应保持完整静态波纹，不应断成两节。
- 调整为黑色小胶囊底，窗口尺寸缩到 `176 x 46`，约为上一版视觉面积的四分之一。
- 波形改为短周期多峰谷曲线，在胶囊内显示更多波峰波谷，线条更细。
- 录音态才渲染 SVG `animate` 和流动渐变；识别态不渲染 `animate`，保持完整静态波纹，避免松开后断裂或继续流动。
- 验证：`npm test -- --run src/DictationOverlay.test.tsx` 通过，2 个测试通过；`npm test -- --run` 通过，6 个测试文件、16 个测试通过；`npm run typecheck` 通过；`npm run build` 通过；`cargo test --manifest-path src-tauri/Cargo.toml overlay` 通过；`cargo fmt --all --manifest-path src-tauri/Cargo.toml --check` 通过；`git diff --check` 通过。

### 2026-05-27 V2 浮窗白壳与主界面精致化

- 维护者反馈：黑色胶囊外仍像有白色外壳；胶囊仍偏大；波纹接近但略乱，希望更像霓虹流光；识别态完全不动会像卡住；主界面整体字体和组件仍偏粗大。
- 去掉胶囊外描边，将胶囊直接铺满 overlay 视窗，避免抗锯齿边缘露出浅色外壳；高光降到更弱的暗色内部高光。
- overlay 尺寸从 `176 x 46` 缩到 `120 x 32`，Rust 定位常量和测试同步更新。
- 识别态改为低速呼吸动效，不再是完全静止；录音态仍保持快速流光与波形变化。
- 主界面进一步压缩：窗口宽度、标题字号、正文字号、header 高度、主区间距、预览浮窗高度和卡片边距都下调，减少粗大感。
- 验证：`npm test -- --run src/DictationOverlay.test.tsx` 通过，2 个测试通过；`cargo test --manifest-path src-tauri/Cargo.toml overlay` 通过；`npm run typecheck` 通过；`npm test -- --run` 通过，6 个测试文件、16 个测试通过；`npm run build` 通过；`cargo fmt --all --manifest-path src-tauri/Cargo.toml --check` 通过；`git diff --check` 通过。

### 2026-05-27 V2 声波柱与转写三点动效

- 维护者反馈：黑色胶囊外仍有白边；波浪仍有些怪，希望更像录音软件实时声波；转写中更适合动态点或处理中动效；上屏曾有一次未成功但后续复测成功，本轮不改上屏链路。
- 将 overlay SVG 视窗同步为 `120 x 32`，与窗口/CSS 尺寸一致，避免缩放合成导致浅色边缘。
- 录音态从曲线路径改为 20 根多色实时声波柱，模拟音量大小变化；保留渐变流光。
- 转写态改为三点动态处理状态，背景声波柱降为很低透明度，不再表现为卡住的静态波形。
- `index.html` overlay 首帧样式增加 `color-scheme: dark`、`margin: 0`、`overflow: hidden`，继续降低 WebView 默认白边风险。
- 验证：`npm test -- --run src/DictationOverlay.test.tsx` 通过，2 个测试通过；`npm run typecheck` 通过；`cargo test --manifest-path src-tauri/Cargo.toml overlay` 通过；`npm test -- --run` 通过，6 个测试文件、16 个测试通过；`npm run build` 通过；`cargo fmt --all --manifest-path src-tauri/Cargo.toml --check` 通过；`git diff --check` 通过。

### 2026-05-27 V2 去边框、六点转写与主界面收敛

- 维护者反馈：录音浮窗效果不错，但外侧仍像有灰白边；转写态 3 个点太少；主窗口和浮窗精致度不匹配；录音动画仍是固定节奏，暂未随真实说话节奏变化。
- 去掉胶囊内部高光矩形，只保留纯黑胶囊底，降低白边视觉来源。
- 转写态从 3 个点改为 6 个五彩动态点，点位错峰动效更明显。
- 主窗口继续细化：按钮、输入框、窗口宽度、header、标题、正文、主预览区、卡片间距和边框强度都下调。
- 说明：录音动画要真正跟随说话节奏，需要 Rust 录音流向前端发送实时音量 level 事件；本轮未接实时音量，只优化固定动画表现。
- 验证：`npm test -- --run src/DictationOverlay.test.tsx` 通过，2 个测试通过；`npm run typecheck` 通过；`npm test -- --run` 通过，6 个测试文件、16 个测试通过；`npm run build` 通过；`cargo test --manifest-path src-tauri/Cargo.toml overlay` 通过；`cargo fmt --all --manifest-path src-tauri/Cargo.toml --check` 通过；`git diff --check` 通过。

### 2026-05-27 V2 转写六点动效强化

- 维护者反馈：六个点虽然存在，但视觉上颜色接近且动效不明显；胶囊外侧仍能看到一层灰白边。
- 将胶囊矩形内缩 `1px`，窗口最外圈保留透明像素，减少边缘合成出的灰白线。
- 6 个转写点改为独立颜色类：蓝、浅蓝、绿、黄、粉、紫，不再共用同一个渐变填充。
- 转写点动效增强为上下跳动、缩放和透明度变化，错峰延迟更明显。
- 验证：`npm test -- --run src/DictationOverlay.test.tsx` 通过，2 个测试通过；`npm run typecheck` 通过；`npm run build` 通过；`git diff --check` 通过。

### 2026-05-27 V2 胶囊光效裁剪与 SVG 点动画

- 维护者反馈：胶囊外侧仍像有白色光边；6 个点虽然彩色但动效仍不明显，像卡住。
- 新增 SVG `clipPath`，把声波和转写点的 glow filter 裁剪在胶囊内部，避免光效溢出到胶囊外形成白边。
- 转写点从 CSS transform 动画改为 SVG 原生 `animate`，直接改变 `cy`、`r`、`opacity`，上下弹跳和明暗变化更可靠。
- 验证：`npm test -- --run src/DictationOverlay.test.tsx` 通过，2 个测试通过；`npm run typecheck` 通过；`npm run build` 通过；`git diff --check` 通过。

### 2026-05-27 V2 透明安全边距与光感转写点

- 维护者澄清：问题不是光效溢出，而是胶囊背后仍像有一个白色/红色边框背景；转写点不应上下大跳，而应像录音态一样通过光感变化表现流动。
- overlay 窗口从 `120 x 32` 调整为 `136 x 44`，胶囊仍保持 `120 x 32` 视觉尺寸，但四周留透明安全边距，避免 WebView 边缘参与合成。
- 胶囊和内容整体平移到安全边距内部；clipPath 同步更新。
- 6 个转写点间距加大，动画从上下跳改为固定位置的半径/亮度流动，形成光从左到右扫过的感觉。
- 验证：`npm test -- --run src/DictationOverlay.test.tsx` 通过；`npm run typecheck` 通过；`cargo test --manifest-path src-tauri/Cargo.toml overlay` 通过；`npm run build` 通过；`cargo fmt --all --manifest-path src-tauri/Cargo.toml --check` 通过；`git diff --check` 通过。

### 2026-05-27 V2 单层准白边框

- 维护者反馈：透明安全边距让胶囊背后的内层灰框变得更大；期望去掉背后灰框，最多保留一层准白边框。
- 撤回 `136 x 44` 外层安全边距，overlay 窗口、SVG、胶囊重新贴合为 `120 x 32`。
- 胶囊从无边框改为单层 `rgba(255,255,255,0.28)` 准白描边，避免同时出现“背后灰框”和“胶囊边框”两层。
- 6 个转写点保持固定位置的光感流动，不恢复上下跳动。
- 验证：`npm test -- --run src/DictationOverlay.test.tsx` 通过；`npm run typecheck` 通过；`cargo test --manifest-path src-tauri/Cargo.toml overlay` 通过；`npm run build` 通过；`cargo fmt --all --manifest-path src-tauri/Cargo.toml --check` 通过；`git diff --check` 通过。

## 2026-05-28 V3 维护者验收与 V4 原生浮窗规划

- 维护者手动验证：`Ctrl+Alt+V` 实验分段输入的尾段修复通过，停止时最后短句不再丢失。
- V3 状态收尾：界面精修、`Ctrl+Alt+V` 切换录音、录音中分段转写上屏、尾段补转写均已完成自动验证和维护者核心功能验证。
- 新问题范围：下一版本只解决桌面浮窗黑色胶囊外侧残余白/灰矩形边框，不同时扩展 ASR、上屏、主界面或流式识别。
- 调研记录：`TMP/research/native-overlay-spike.md`。
- 设计文档：`docs/superpowers/specs/2026-05-28-v4-native-overlay-design.md`。
- 实施计划：`docs/superpowers/plans/2026-05-28-v4-native-overlay.md`。
- 推荐路径：先试 Tauri `shadow: false` 与 WebView overlay 低风险补丁；如果维护者仍能看到外层白/灰框，再实现 Windows-only 原生 layered overlay，并保留 WebView fallback。

## 2026-05-28 V4 webview-shadowless 第一阶段

- 按 V4 计划先执行低风险 Tauri WebView 补丁，而不是直接引入 Win32 原生窗口。
- `dictation-overlay` Tauri 窗口新增 `shadow: false`，用于关闭 Windows 上可能参与合成的窗口阴影。
- 新增 overlay backend 状态 command，诊断模式显示 `桌面浮窗后端`，当前值为 `webview-shadowless`。
- 新增测试锁住 overlay 配置：`transparent: true`、`decorations: false`、`shadow: false`。
- 自动验证：`cargo test --manifest-path src-tauri/Cargo.toml overlay` 通过，4 个 overlay 测试通过；`npm test -- --run src/App.test.tsx` 通过，6 个前端测试通过。
- 人工验证尚未完成：需要维护者在 Tauri 桌面模式点击 `测试桌面浮窗`，确认黑色胶囊外侧是否还有白/灰矩形。若仍有，下一步进入 Windows 原生 overlay 原型。

## 2026-05-28 V4 native-win32 原生浮窗原型

- 因维护者此前多轮反馈 WebView 浮窗外层白/灰框仍存在，继续执行 V4 第二阶段：Windows-only 原生 layered overlay。
- 新增 `native_overlay` Rust 模块，创建 `WS_POPUP` + `WS_EX_LAYERED` + `WS_EX_TOOLWINDOW` + `WS_EX_TOPMOST` + `WS_EX_NOACTIVATE` Win32 浮窗。
- 原生浮窗使用 `SetWindowRgn` 裁剪成圆角胶囊，并用 GDI 绘制深色胶囊和小型彩色声波，目标是先绕开 WebView2 矩形宿主窗口。
- `overlay::show_dictation_overlay` 现在优先使用 `native-win32`，失败时自动回退 `fallback-webview`，诊断模式会显示当前后端和失败原因。
- 自动验证进展：`cargo test --manifest-path src-tauri/Cargo.toml overlay_backend` 通过，3 个相关测试通过。
- 人工验证待办：启动 Tauri 桌面模式，进入诊断模式，确认 `桌面浮窗后端` 为 `native-win32`，并检查 `测试桌面浮窗` 是否不再出现外层白/灰矩形。

### 2026-05-28 V4.3 主界面重设计

- 使用 `frontend-design` 方向重新收敛主界面：暗色控制台、精简文字、统一字号和间距，并对齐 native-win32 黑色胶囊浮窗的视觉语言。
- 默认主界面保留品牌、快捷键提示、主语音胶囊、当前状态、最近文本和系统状态；详细操作继续放在诊断模式。
- 修复 Windows PowerShell 编码导致的问号乱码文案损坏，并将测试断言更新到新主界面。
- 验证：`npm test -- --run src/App.test.tsx src/VoiceOverlay.test.tsx` 通过；`npm run typecheck` 通过；`npm run build` 通过；`cargo check --manifest-path src-tauri/Cargo.toml --lib` 通过；`cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` 通过；`cargo fmt --all --manifest-path src-tauri/Cargo.toml --check` 通过。

### 2026-05-28 V4.4 极简主界面

- 根据维护者反馈，默认主界面从信息面板改为极简日常输入入口：移除系统状态卡、最近文本卡、ASR 安装条和多个次级按钮。
- 主界面现在只保留 VoxType、语音胶囊、一个开始/停止按钮、快捷键提示和诊断入口；其他信息继续留在诊断模式。
- 验证：`npm test -- --run src/App.test.tsx src/VoiceOverlay.test.tsx` 通过；`npm run typecheck` 通过；`npm run build` 通过；`git diff --check` 通过。

### 2026-05-28 V4.5 主界面声波胶囊复用

- 根据维护者反馈，去掉主界面的粗大彩色点视觉，改为直接复用录音态 20 根彩色声波柱的胶囊风格。
- 主界面不再过度空白：恢复一个轻量状态胶囊和最近文本预览，但不恢复系统信息卡片堆叠。
- 主按钮图标从圆点改为小声波柱，与浮窗录音状态语言一致。
- 验证：`npm test -- --run src/App.test.tsx src/VoiceOverlay.test.tsx` 通过；`npm run typecheck` 通过；`npm run build` 通过；`git diff --check` 通过。
