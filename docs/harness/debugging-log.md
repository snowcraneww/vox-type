# 调试记录

本文记录 VoxType MVP 调试过程中遇到的真实问题、根因判断、修复方式和验证证据。它不是使用教程；使用步骤见 `docs/guide/run-and-understand.md`。

## 记录规则

- 每个问题都要写清楚现象、影响范围、根因、修复、验证和后续风险。
- 只记录已经观察到的事实和已经验证过的判断；不把猜测写成结论。
- 涉及代码、命令、API、文件名和错误消息时保留原文。
- 面向维护者的解释默认使用中文。

## 2026-06-01 百度 ASR 配置保存后跳回 MiniMax、检测无反馈

### 现象

维护者在模型配置页切到“百度短语音识别”后，保存百度 API Key 会跳回 MiniMax 相关 tab 或表单；保存百度设置后点击“检测百度配置”看起来没有任何反应。

### 影响

V6 当前目标是先让云端 ASR provider 配置链路可用。这个问题不会泄漏 API Key，但会让用户无法确认百度配置是否已经保存，也会误导用户以为百度 provider 没有被选中。

### 根因

`save_baidu_asr_api_key_to_user_env` 写入 `BAIDU_ASR_API_KEY` 后，继续读取磁盘里的 `cloud-asr-config.json` 生成返回状态。如果磁盘配置仍是 MiniMax，返回给前端的 `CloudAsrConfigStatus.config.provider` 仍是 `minimax`，前端再用 `applyCloudAsrConfigStatus` 应用状态时就会把当前 UI 切回 MiniMax。

“检测百度配置无反应”的根因是 `handleTestCloudAsrConfig` 只重新读取并应用后端状态；当返回消息和之前一致，或返回 provider 仍是旧 MiniMax 时，页面没有新的可见反馈。

### 修复

- `save_baidu_asr_api_key_to_user_env` 在写入 `BAIDU_ASR_API_KEY` 后，如果磁盘配置不是 Baidu，会用百度短语音识别默认字段生成并保存 Baidu provider 配置，再返回 Baidu 状态。
- 前端 `applyCloudAsrConfigStatus` 保留用户当前选择的云端 provider，避免后端旧 provider 覆盖正在编辑的 tab。
- 点击“检测百度配置”后会显示明确文案：`百度配置检测完成：...`；MiniMax 同理显示 `MiniMax 配置检测完成：...`。
- 新增前端回归测试覆盖：保存百度 API Key 后仍停留在“百度短语音识别配置”；点击“检测百度配置”后显示显式反馈。
- 新增 Rust 回归测试覆盖：从 MiniMax 磁盘配置切到 Baidu provider 时会使用百度默认短语音识别字段并返回 Baidu 状态。测试不调用真实用户环境变量写入路径。

### 验证

- `npm test -- --run src/App.test.tsx`：通过，1 个测试文件、15 个测试通过。
- `npm run typecheck`：通过。
- `cargo test --manifest-path src-tauri/Cargo.toml cloud_asr_config --no-run`：通过，测试二进制编译成功。
- `cargo check --manifest-path src-tauri/Cargo.toml --lib`：通过。

### 已知限制

本轮只修复百度 provider 配置保存和检测反馈。真实百度 ASR 转写调用还没有接入；当前日常转写链路仍然使用本地 whisper.cpp。

## 2026-06-01 百度 provider 按钮选中态、环境变量刷新和真实调用接入

### 现象

维护者反馈云端 API 页里 `MiniMax` 和 `百度短语音识别` 两个按钮的视觉选中态和实际 provider 不一致：MiniMax 一直显示绿色，点击百度后百度没有明显变色。维护者还反馈百度 API Key 和配置已经保存多次，但重新运行后仍显示没有 API Key。

### 影响

- 用户无法从界面判断当前究竟选中了 MiniMax 还是百度。
- 即使保存过 `BAIDU_ASR_API_KEY`，从 VS Code / 旧终端重新启动的 Tauri 进程也可能读不到新用户环境变量，导致百度配置一直显示未就绪。
- 若开始测试百度真实接入，必须避免“界面看起来选了百度，实际仍在用本地 whisper.cpp”的误导。

### 根因

- CSS 里 `.model-panel .button-row button:first-child` 会把每个按钮行的第一个按钮强制渲染为绿色主按钮；provider 切换行复用了 `.button-row`，所以 MiniMax 作为第一个按钮一直像选中态。
- Windows 用户环境变量写入 `HKCU\Environment` 后，当前 VS Code / terminal 进程的 environment block 不会自动刷新；子进程继承旧环境，因此 `std::env::var("BAIDU_ASR_API_KEY")` 仍可能返回 missing。

### 修复

- provider 切换行改为专用 `.cloud-provider-switch`，按钮使用 `aria-pressed` 和 `data-active` 表达选中态，不再受 `.button-row button:first-child` 全局主按钮样式影响。
- Windows 下读取 API Key 时，先读进程环境变量；如果缺失，再直接读取 `HKCU\Environment` 注册表值。这样保存后即使没有重启 VS Code 或 terminal，后续 Tauri 进程也能读到已经写入的用户环境变量。
- `transcribe_last_recording` 接入百度短语音识别：当云端 provider 是 `baidu` 时，要求百度配置 ready，然后用最近录音的 16 kHz mono PCM 构造 JSON 请求并调用百度 endpoint；如果百度配置未就绪，直接报百度配置错误，不静默回退到本地 whisper.cpp。
- 百度请求体使用 `Content-Type: application/json`，音频样本转 little-endian PCM bytes 后 base64 放入 `speech`，`len` 使用原始 PCM 字节数；返回解析取 `result[0]`。
- API Key 不写入 URL、日志、项目配置或测试快照。自动测试只使用假 key 验证构造逻辑。

### 验证

- `npm test -- --run src/App.test.tsx`：通过，1 个测试文件、15 个测试通过；覆盖 provider 按钮 `aria-pressed` 随选择变化。
- `npm run typecheck`：通过。
- `cargo test --manifest-path src-tauri/Cargo.toml cloud_asr --no-run`：通过，覆盖百度请求构造、base64 PCM 和响应解析。
- `cargo test --manifest-path src-tauri/Cargo.toml cloud_asr_config --no-run`：通过，覆盖云端配置逻辑。
- `cargo check --manifest-path src-tauri/Cargo.toml --lib`：通过。

### 已知限制

真实百度网络效果未写入自动测试，因为它依赖维护者本机真实 `BAIDU_ASR_API_KEY`、网络状态和一段实际录音。当前真实调用路径只接入整段 `transcribe_last_recording`；`Ctrl+Alt+V` 录音中的分段实时片段仍走本地 whisper.cpp 分段逻辑，后续需要单独设计云端分段/流式策略。

## 2026-06-01 百度 ASR 404 不可见、主界面云端状态写死

### 现象

维护者反馈：模型页已选择百度短语音识别，配置检测显示通过，但主界面准备状态仍显示“云端 API 未配置”；按 `Ctrl+Alt+Space` 说话后主界面没有识别记录、目标输入框没有上屏，也没有明显报错。进入诊断页复制日志后才能看到：`语音识别失败：百度 ASR HTTP 404 Not Found：404 page not found（asr_failed）`。

### 根因

- 主界面 `MainWindow` 的“云端 API”准备状态仍是 V5 占位文案，写死为 `未配置`，没有接入 `cloudAsrConfigStatus`。
- 快捷键闭环失败时只更新全局状态和诊断日志，没有把错误展示在识别记录区；用户在主界面会看到“没有任何反应”。
- 百度配置检测只检查 key、endpoint、dev_pid、cuid、format、sample rate 等字段是否存在，没有校验 endpoint 是否像短语音识别标准版的 `/server_api` 地址，因此错误 URL 也可能显示“配置完整”，直到真实请求才返回 HTTP 404。

### 修复

- `MainWindow` 新增 `cloudAsrConfigStatus` 入参，主界面“云端 API”状态会显示 `百度 ASR` / `MiniMax` 以及 ready 或未就绪状态，tooltip 使用实际配置消息。
- `Ctrl+Alt+Space` 快捷键闭环失败时，将错误写入识别记录区域的 `role=status` 消息，例如 `语音识别失败：百度 ASR HTTP 404 Not Found...`，不再只藏在诊断页。
- 百度配置状态新增 endpoint 形状检查：必须以 `/server_api` 结尾，否则配置不 ready，并提示 `百度 ASR Endpoint 应为短语音识别 server_api 地址。`
- 百度真实请求构造会 trim 并去掉 endpoint 尾部 `/`，减少用户输入尾斜杠导致的 URL 变体。

### 验证

- `npm test -- --run src/App.test.tsx`：通过，1 个测试文件、17 个测试通过；新增覆盖主界面显示百度云端 ready 状态，以及快捷键 ASR 失败时主界面显示错误。
- `npm run typecheck`：通过。
- `cargo test --manifest-path src-tauri/Cargo.toml cloud_asr_config --no-run`：通过；新增覆盖错误百度 endpoint 不应 ready。
- `cargo test --manifest-path src-tauri/Cargo.toml cloud_asr --no-run`：通过。
- `cargo check --manifest-path src-tauri/Cargo.toml --lib`：通过。

### 已知限制

配置检测仍不是一次真实网络探测，它只做本地字段和 endpoint 形状校验。真实调用如果因百度 token/API Key 权限、套餐、网络或服务端参数返回错误，主界面现在会显示错误，诊断日志仍保留详细信息。


## 2026-06-01 V7 主界面图标显示为问号、模型配置页过杂

### 现象

维护者反馈 V7 修改后，主界面原本正常的图标变成可见的问号；模型选择和模型配置页面信息太杂，新加和新改的局部样式显得粗糙。

### 影响

这是 V7 模型路由重构引入的可见 UI 回归。它不影响百度短语音识别真实调用和模型偏好保存逻辑，但会直接降低日常主界面的可用性和可信度。

### 根因

V7 实现时为了规避符号字体显示差异，把主界面快捷键设置、复制、重新上屏和删除按钮里的旧符号替换成了问号占位，导致用户界面直接显示问号。模型页同时把模式路由、本地配置、百度短语音密钥、短语音参数和百度实时 WebSocket 预留字段平铺到同一视觉层级，缺少清晰分组。

### 修复

- 主界面新增本地 inline SVG Icon 组件，替换问号占位；快捷键设置图标继续保持无外圈按钮，只显示放大的绿色图标。
- 识别记录动作按钮改为 SVG 图标，保留 aria-label 和 title 供无障碍和 tooltip 使用。
- 模型页把顶部路由改成两行“按住说话模型 / 连续输入模型”的 segmented controls，当前选中模型和 ready 圆点在同一行表达。
- 模型配置区按能力分组：本地 whisper.cpp、百度短语音、百度实时 WebSocket V8 预留；百度 API Key 和 Secret Key 收入密钥子区，短语音参数收入紧凑字段网格。
- 总统计里的 T/W/S 改成 时长 / 字数 / 速度，减少不明缩写。

### 验证结果

- npm test -- --run src/App.test.tsx：通过，16 个测试。
- npm run typecheck：通过。
- npm run build：通过。
- 可见问号扫描未发现主界面按钮仍包含问号占位；剩余双问号为 TypeScript nullish coalescing。

### 已知限制

本轮是 V7 UI polish，不把百度实时 WebSocket 真实流式接入提前到 V7。v7-001 仍等待维护者在 Tauri 桌面环境手动验收后再标记为 passing。


## 2026-05-28 native-win32 浮窗去框成功但视觉降级

### 追加：主窗体缩成一团、文字太多

维护者反馈上一版主窗体看起来缩成一团，文字过多，录音声谱不像浮窗那种精致胶囊。本轮将用户主界面宽度放宽到 920px，右侧语音状态改为更大的黑色胶囊声谱，标题收敛为 VoxType，快捷键说明和最近文本占位文案精简，诊断模式保留详细信息。

### 追加：主窗体风格与浮窗不匹配

维护者认可 native 浮窗的黑色胶囊和彩色精细光感是目标风格，但主窗体仍然显得像粗糙测试界面。本轮不把 `UpdateLayeredWindow` 用到主窗体，而是用 CSS/SVG 能力对齐视觉语言：黑色底、胶囊形状、细彩色频谱、低边框、低文字密度。诊断模式保留工具台属性，暂不做大规模改版。

### 追加：`Ctrl+Alt+V` 第二次触发后体感慢

维护者反馈第二次 `Ctrl+Alt+V` 后反应有点慢。根因是旧流程在 `finally` 里隐藏浮窗，因此需等待尾段转写、可能的整段兜底转写和上屏完成后才收起浮窗。本轮将停止录音、状态反馈和隐藏浮窗前置到 `stopRecording()` 成功后，后续尾段转写继续执行。新增前端回归测试覆盖“浮窗隐藏不等待尾段转写完成”。

### 追加：从 GDI 硬裁剪升级到每像素 alpha

维护者继续反馈胶囊边缘有明显锯齿，整体像低分辨率绘制，不够高级。根因是旧实现依赖 GDI `RoundRect` 和 `SetWindowRgn` 硬裁剪，边缘几乎没有真正抗锯齿。本轮改为软件 3x 超采样渲染一张 BGRA 位图，再用 `UpdateLayeredWindow` 提交到分层窗口。这使胶囊外部仍是每像素透明，不恢复白灰外框，同时让胶囊边缘和彩色元素都有更细的过渡。

### 追加：降速和六点呼吸加载

维护者反馈新版颜色不如旧版好看，胶囊边缘不够光滑，录音动效闪得太快，6 个转写点不应该像波浪。本轮将计时器改为 72ms，颜色相位大幅降速，录音柱高起伏收敛，转写点改为原地半径和亮度呼吸。胶囊本体内缩 1px 绘制，整体 alpha 从 244 降到 236，用更暗的边缘降低 GDI 锯齿感，不增加可见描边。

### 追加：恢复旧动效但不恢复边框

维护者要求恢复之前 WebView/SVG 浮窗动效的感觉，但不能恢复可见边框。本轮参考旧版 120x32 胶囊、20 根频谱柱和 6 个转写点，提升原生绘制刷新率，修正光带方向为向右推进，并让转写点恢复上下弹性。边框策略不变：窗口仍通过 `SetWindowRgn` 裁成胶囊，绘制层不画可见外描边。

### 现象

维护者验证 `native-win32` 原生浮窗后确认：诊断模式没有报错，点击浮窗后只显示黑色胶囊，原先 WebView/Tauri 外层白色或灰色矩形框已经消失。但新浮窗的 GDI 绘制比之前 WebView/SVG 版本粗糙，分辨率感较差，缺少之前彩色频谱柱和六点转写态的视觉记忆。

维护者同时反馈识别准确率似乎没有以前高。

### 影响

- 外层白/灰框问题已经被原生窗口路线绕开，这是 V4 的核心目标。
- 视觉质量仍未达到之前 WebView/SVG 版本，需要在原生绘制层继续恢复。
- 准确率反馈需要和 overlay 改动拆开看，避免把 UI 改动误判成 ASR 质量改动。

### 根因

外层白/灰框来自 WebView2/Tauri 透明窗口宿主层，而不是页面 CSS。`native-win32` 成功的关键是不用 WebView 承载透明背景，而是创建独立 Win32 popup window，并用 `CreateRoundRectRgn` + `SetWindowRgn` 把窗口本身裁剪成胶囊。这样窗口矩形区域本身不再存在，自然不会露出 WebView 宿主白底。

视觉降级的根因是渲染栈从浏览器 SVG/CSS 动画切到了 Win32 GDI。GDI 的圆角、柔光、抗锯齿和渐变能力都比 SVG/CSS 弱；当前只能用多层深色/彩色 `RoundRect` 近似之前的频谱柱和光感。若要达到 WebView/SVG 级别的细腻度，后续应评估 GDI+、Direct2D 或 WinUI/Composition overlay。

准确率方面，本轮 overlay 改动没有修改录音采集、重采样、whisper.cpp 参数、模型路径、ASR prompt 或上屏策略。因此如果继续出现准确率变化，优先排查输入音频、模型、语言参数和 `last-asr-input.wav`，而不是 overlay 绘制。

### 修复

- 保留 `native-win32` 作为优先 overlay backend，保留 WebView overlay 作为 fallback。
- 原生浮窗继续使用 `WS_POPUP`、`WS_EX_LAYERED`、`WS_EX_TOOLWINDOW`、`WS_EX_TOPMOST`、`WS_EX_NOACTIVATE`。
- 原生浮窗使用 `CreateRoundRectRgn` + `SetWindowRgn` 裁剪窗口形状，避免 WebView 外层矩形。
- 录音态从低保真线条恢复为 20 根彩色频谱柱，并加入暗色柔光近似旧 SVG 的流光感。
- 转写态恢复为 6 个彩色点，并保留暗色频谱底纹，避免看起来像黑胶囊卡住。
- 背景和描边统一为深黑，减少边缘可见感。
- 删除 `windows-sys` 显式依赖，改为在 `native_overlay.rs` 声明项目实际使用的最小 Win32 FFI，降低测试二进制导入额外系统入口点的概率。

### 验证

- `cargo check --manifest-path src-tauri/Cargo.toml --lib`：通过。
- `npm test -- --run`：通过，6 个测试文件、21 个测试通过。
- `npm run typecheck`：通过。
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`：通过。
- `cargo fmt --all --manifest-path src-tauri/Cargo.toml --check`：通过。
- `npm run build`：通过。

### 已知限制

`cargo test --manifest-path src-tauri/Cargo.toml overlay` 当前在 Windows 测试 exe 启动阶段失败，错误为 `STATUS_ENTRYPOINT_NOT_FOUND`。PE 导入表确认测试二进制仍包含 `user32.dll!SetWindowDisplayAffinity`。这发生在测试 harness 启动前，不是某个 overlay 单元测试断言失败。后续需要单独处理 Windows 测试环境或 Tauri/窗口依赖链导入问题。

## 2026-05-28 Windows WebView2/Tauri 透明浮窗残留白边调研补充

### 问题

维护者希望底部语音浮窗只显示黑色胶囊和波形，不显示胶囊外面的浅色或灰白色窗口底层。项目内已经把 overlay 页面根节点、`html`、`body`、`#root` 和胶囊外层容器都设为透明，并且 Tauri overlay 窗口使用无边框透明窗口，但 Windows 上仍可能看到浅色边缘。

### 调研结论

这不是 VoxType 单独遇到的问题。公开资料里常见到 Windows + WebView2 + Tauri 透明窗口出现白底、白边、resize 后才透明、ghost titlebar background 等现象。可参考：

- Tauri issue `#4881`：Windows 上 `transparent: true` 仍可能先出现白底，需要 resize/crop 后才消失。
- StackOverflow `Tauri transparent window only works when resized`：社区 workaround 包括先使用 `decorations: false`，再通过 JS API 调整窗口状态。
- Tauri v2 `Window` / `WebviewWindow` API：提供 `setBackgroundColor`、`setShadow`、`setDecorations`、`setSize` 等接口，但这些接口在 Windows/WebView2 下有平台差异。
- Tauri issue `#11654`：有人通过在 `tauri.conf.json` 里设置 `shadow: false` 修复窗口边框显示问题。
- Tauri issue `#12450`：`transparent(true)` 在 Windows child window 场景下仍可能不按预期透明。

### 为什么纯 CSS 很难彻底解决

CSS 只能控制 WebView 里页面内容的背景和边框。当前看到的浅色层更像 WebView2 或宿主窗口的默认合成背景、阴影、非客户区残留或透明合成时机问题。也就是说，即使页面内部是透明的，外层 Windows 窗口或 WebView controller 仍可能先绘制一层背景。

### 别人通常怎么处理

- 继续使用 WebView overlay，但把窗口缩到尽量小，减少残留背景面积。
- 关闭窗口装饰和阴影：`decorations: false`、`shadow: false`。
- 在显示后强制 resize/repaint，规避“resize 后才透明”的 WebView2 现象。
- 使用深色背景兜底，让残留背景和黑色胶囊视觉上接近。
- 对要求极高的悬浮窗，放弃 WebView 渲染，改用原生 Win32/Direct2D/WinUI 轻量窗口绘制波形。

### 当前项目判断

VoxType 已将该问题从普通 CSS 调整范围升级为 Windows native overlay 技术 spike。WebView overlay 仍保留为回退方案；默认路径改为优先创建 Windows 原生 layered popup window，以绕开 WebView2 宿主窗口透明合成带来的外层浅色框。

## 2026-05-28 V4 webview-shadowless 浮窗实验

### 现象

维护者确认 V3 `Ctrl+Alt+V` 功能可用后，下一阶段只聚焦桌面语音浮窗黑色胶囊外侧的白色、灰色或浅色矩形边框。

### 根因判断

既有 CSS 和 Tauri 基础透明配置已经多次收敛，但浅色外框仍存在，因此它更可能来自 Windows + WebView2 + Tauri 宿主窗口的阴影、非客户区或透明合成层，而不是 React 组件内部的普通 border。

### 本轮修复

- `src-tauri/tauri.conf.json` 的 `dictation-overlay` 窗口新增 `shadow: false`。
- 新增 Rust 回归测试 `overlay_window_disables_shadow_in_tauri_config`，锁住 `transparent: true`、`decorations: false` 和 `shadow: false`。
- 新增 overlay backend 状态：当前返回 `webview-shadowless`。
- 诊断模式 `运行状态` 增加 `桌面浮窗后端`，并在启动日志里记录当前后端，便于人工验证时明确当前方案。

### 验证

- TDD 红灯：新增 `overlay_window_disables_shadow_in_tauri_config` 后，测试失败，原因是 `overlay["shadow"]` 为 `Null`。
- 绿灯：加入 `shadow: false` 后，`cargo test --manifest-path src-tauri/Cargo.toml overlay_window_disables_shadow_in_tauri_config` 通过。
- `cargo test --manifest-path src-tauri/Cargo.toml overlay` 通过，4 个 overlay 测试通过。
- `npm test -- --run src/App.test.tsx` 通过，6 个前端测试通过。

### 人工验证待办

需要维护者在 Tauri 桌面模式里进入诊断模式，确认 `桌面浮窗后端` 显示 `webview-shadowless`，点击 `测试桌面浮窗` 后观察底部黑色胶囊外侧是否仍有白/灰矩形。若仍有，按 V4 计划进入 Windows 原生 overlay 原型；若没有，则不需要做原生 overlay。

## 2026-05-28 V4 native-win32 浮窗原型

### 现象

维护者此前多次确认 Tauri/WebView 浮窗外侧仍能看到白色、灰色或浅色矩形区域，因此 V4 继续推进 Windows 原生浮窗原型，不再只等待 `shadow: false` 的人工验证结果。

### 本轮修复

- 新增 `src-tauri/src/native_overlay.rs`，使用 Windows Win32 创建独立 `WS_POPUP` 浮窗。
- 原生浮窗使用 `WS_EX_LAYERED`、`WS_EX_TOOLWINDOW`、`WS_EX_TOPMOST`、`WS_EX_NOACTIVATE`，避免抢焦点、避免任务栏入口，并保持置顶。
- 使用 `CreateRoundRectRgn` + `SetWindowRgn` 将窗口本身裁剪成胶囊形，而不是让矩形 WebView 窗口承载透明背景。
- 使用 GDI 绘制深色胶囊和小型彩色声波；当前目标是先验证“无外层白/灰矩形”，视觉精修可以后续迭代。
- `overlay::show_dictation_overlay` 优先调用原生浮窗；如果原生创建失败，自动回退现有 WebView 浮窗，并在 `桌面浮窗后端` 显示 `fallback-webview / 错误原因`。

### 验证

- TDD 红灯：新增 `overlay_backend_result_reports_native_success` 和 `overlay_backend_result_reports_webview_fallback_reason` 后，Rust 测试因缺少 `OverlayBackendResult` 与 `resolve_overlay_backend` 失败。
- 绿灯：实现后端决策结构和原生浮窗入口后，`cargo test --manifest-path src-tauri/Cargo.toml overlay_backend` 通过，3 个相关测试通过。

### 人工验证待办

需要在 Tauri 桌面模式进入诊断模式，确认 `桌面浮窗后端` 显示 `native-win32`，点击 `测试桌面浮窗` 后观察底部是否只剩黑色胶囊和声波。如果显示 `fallback-webview`，需要复制诊断日志里的失败原因继续定位。

## 2026-05-28 Ctrl+Alt+V 实验分段实时输入

### 问题

维护者希望 `Ctrl+Alt+V` 模式下按一次开始录音、再按一次停止，并且录音过程中尽量“边说边在光标处显示文字”。

### 调研结论

whisper.cpp 有 `examples/stream` / `whisper-stream` 示例，说明可以做近实时麦克风转写；但官方示例也描述为 naive realtime inference，本质是不断采集短音频片段再推理。更成熟的真正流式体验通常还需要 partial/final 文本合并、上下文窗口、VAD、去重和延迟控制。当前 VoxType 使用的是 `whisper-cli.exe` 整段命令行转写 adapter，不是 `whisper-stream`，所以不能直接声称是真流式 ASR。

### 当前实现

本轮实现的是实验分段实时输入：

- Rust `RecordingSession` 新增 `asr_samples_from(from_sample_index)`，可以读取正在录音缓冲区的新增片段。
- Rust `RecorderManager` 新增 `active_asr_samples_from`，返回新增片段和当前源样本游标。
- Tauri command 新增 `transcribe_active_recording_chunk`，把正在录音中的新增片段重采样为 16 kHz 后交给 whisper.cpp 转写。
- 前端 `Ctrl+Alt+V` 开始录音后，每隔约 4 秒请求一次新增片段转写；如果返回文本，就用现有剪贴板上屏策略粘贴到当前焦点应用。
- 再按一次 `Ctrl+Alt+V` 会停止定时器、尝试处理最后一段并停止录音；如果期间没有任何实时片段成功上屏，会兜底走整段转写。

### 限制和风险

- 这不是完整流式 ASR，仍然是“定时切片 -> whisper-cli.exe -> 粘贴”。
- 每段都单独识别，可能因为上下文不足导致漏字、重复或标点不稳定。
- whisper.cpp 推理耗时如果超过 4 秒，前端会跳过重叠请求，避免并发推理，但实时感会下降。
- 每次上屏仍使用剪贴板粘贴，可能影响用户剪贴板；后续更可靠路线是 SendInput 文本输入或 TSF。
- 当前没有增量文本 diff 和替换机制，因此不会像成熟输入法那样先显示临时 partial 再修正 final。

### 验证

- `npm test -- --run src/App.test.tsx`：覆盖 `Ctrl+Alt+V` 录音中分段转写并上屏，以及有实时片段后停止时不再走整段兜底。
- `cargo test --manifest-path src-tauri/Cargo.toml`：覆盖录音缓冲区按源样本游标读取增量片段。

## 2026-05-28 Ctrl+Alt+V 浮窗动效只闪一下

### 现象

维护者验证实验分段实时输入时，`Ctrl+Alt+V` 第一次按下后底部浮窗只短暂闪过录音动效，随后只剩黑色胶囊背景；第二次按下停止时又出现六个彩色点的转写动效。

### 影响

实际分段上屏已经可用，但浮窗反馈会误导用户：录音中看起来像卡住或没有录音，停止时又像进入了一个长时间转写阶段。对于当前实验分段实时输入模式，这个视觉语义不对。

### 根因

`Ctrl+Alt+V` 是切换录音快捷键。第一次按下会发送 `toggleStartRecording`，释放按键时会发送 `ignore`。旧的 `DictationOverlay` 收到 `ignore` 后把 mode 算成 `idle`，所以录音动效只闪一下就停了。

第二次按下会发送 `toggleStopAndTranscribe`。旧逻辑把它和 `stopAndTranscribe` 一样映射为 `transcribing`，所以浮窗进入六点转写态。但当前 `Ctrl+Alt+V` 已经是录音中分段转写，不再需要一个明显的“停止后长转写”动效。

### 修复

- overlay 监听到 `ignore` 事件时不更新当前 payload，避免释放按键把录音态打回 idle。
- `toggleStopAndTranscribe` 在 overlay 中继续保持 `recording` 视觉态，等待主流程停止录音后隐藏 overlay。
- 保留 `Ctrl+Alt+Space` 的 `stopAndTranscribe -> transcribing` 映射，因为按住说话模式仍然是松开后整段转写。

### 验证

新增/更新 `src/DictationOverlay.test.tsx`：

- `toggleStartRecording` 显示录音波形。
- `ignore` 释放事件不改变录音波形。
- `toggleStopAndTranscribe` 不再显示六点转写态。
- `stopAndTranscribe` 仍显示六点转写态，保证按住说话模式不被破坏。

局部验证：

```bash
npm test -- --run src/DictationOverlay.test.tsx
```

结果：1 个测试文件、5 个测试通过。

## 2026-05-28 Ctrl+Alt+V 停止时最后短句可能丢失

### 现象

维护者验证实验分段实时输入时，录音过程中分段上屏可用，但第二次按 `Ctrl+Alt+V` 停止后，最后一小段短句没有出现在目标输入框里。

### 根因

旧停止流程是在 `stopRecording()` 之前调用 `transcribeActiveRecordingChunk(liveCursorRef.current)` 尝试处理最后新增片段。这个 command 为了避免过短片段频繁调用 whisper.cpp，少于约 1 秒 ASR 样本时会返回 `录音片段太短，暂不转写`。停止流程传入了 `quietShortChunk: true`，所以这个错误被静默忽略。

如果录音过程中已经有前面片段成功上屏，`liveInsertedTextRef.current` 为 `true`，停止后不会再走整段兜底转写。因此最后不足 1 秒但有语义的尾句就可能被丢掉。

### 修复

- Rust `RecordedAudio` 新增 `asr_samples_from_source_index`，可以在录音停止后从完整的最近录音里按源样本游标取尾段。
- Rust `RecorderManager` 新增 `last_asr_samples_from`。
- Tauri 新增 `transcribe_last_recording_chunk` command：停止录音后转写从上次实时游标到完整录音结尾的尾段。
- 前端 `Ctrl+Alt+V` 停止流程改为先 `stopRecording()` 固化完整录音，再调用 `transcribeLastRecordingChunk(liveCursorRef.current)` 补尾段。
- 前端保留“没有任何分段成功上屏时整段兜底”的行为。

### 验证

- `src/App.test.tsx` 覆盖：已有实时片段上屏后，停止时会用 `toSampleIndex` 作为游标转写尾段，并继续上屏尾段文本。
- `src-tauri/src/recorder.rs` 覆盖：停止后的 `RecordedAudio` 可以按源样本游标取尾段 ASR 样本，并拒绝越界游标。

## 2026-05-28 Tauri 透明浮窗仍有浅色边框

### 现象

维护者多次验证底部 `dictation-overlay` 浮窗时，黑色胶囊外侧仍能看到一层浅色或灰白色背景边缘。即使前端页面根节点、`html`、`body` 和 `#root` 都设置透明背景，Tauri 窗口也设置了 `transparent: true`、`decorations: false`、`alwaysOnTop: true`，浅色边缘仍可能存在。

### 影响

功能不受影响，但视觉上不像完全独立的精致悬浮胶囊，影响高级感。

### 调研来源

- Tauri issue `#12450`：`WebviewWindowBuilder::transparent(true)` 在 Windows child window 场景下仍可能显示黑色或不透明背景。
- Tauri issue `#14764`：Windows 上 `transparent: true` 的窗口在拖动或焦点变化时可能出现 ghost titlebar background，报告者怀疑与 WebView2 默认背景层有关。
- Tauri issue `#1564`：早期讨论过 webview background color 和白色背景闪烁问题。
- Tauri v2 window API 文档说明窗口透明、窗口效果和 Windows WebView2 行为存在平台差异。

### 根因判断

当前更像 Tauri/Wry/WebView2 透明窗口在 Windows 上的底层渲染限制，而不是普通 CSS 边框。项目内已排查并处理过这些应用层因素：

- overlay Tauri window 使用 `transparent: true`。
- overlay Tauri window 使用 `decorations: false`。
- overlay 页面根节点、`html`、`body`、`#root` 设置为透明。
- overlay 胶囊外层容器取消背景、边框、阴影。
- overlay 窗口尺寸已缩小到胶囊附近，避免大片背景露出。

### 当前处理

V2/V3 暂不继续把该问题作为阻塞项。当前策略是把窗口尽量缩小，并让胶囊自身边缘使用低透明白色描边，降低平台残余浅色边缘的违和感。

### 后续方向

如果继续攻关，需要单独做 Windows 原生实验：

- 研究 `shadow(false)`、window background color、WebView2 controller default background color 是否能从 Rust/Tauri 层设置。
- 评估不用 WebView 做 overlay，改用 Windows 原生轻量窗口绘制音频波形。
- 评估是否等待 Tauri/Wry/WebView2 上游修复。

### 验证

当前已验证 overlay 功能可用、动画可显示、快捷键链路可触发。视觉残余边框属于已知风险。

## 2026-05-28 Ctrl+Alt+V 模式下浮窗显示但没有动效

### 现象

维护者验证 `Ctrl+Alt+V` 切换录音模式时，桌面浮窗可以出现，但没有进入录音动效；同时期望的“边说边在光标处显示文字”也没有发生。

### 影响

- `Ctrl+Alt+V` 模式的桌面反馈不可信，用户不知道是否真的在录音。
- 用户可能误以为当前版本已经支持实时流式输入。

### 根因

浮窗动效问题是代码缺口：`DictationOverlay` 只把 `startRecording` 映射为 `recording`，只把 `stopAndTranscribe` 映射为 `transcribing`。V3 新增的 `toggleStartRecording` 和 `toggleStopAndTranscribe` 没有被识别，所以 overlay 一直保持 `idle`。

实时输入问题是架构边界：当前 whisper.cpp adapter 在停止录音后把整段 16 kHz PCM 交给 `whisper-cli.exe`，拿到完整文本后再通过剪贴板上屏。它不是流式 ASR，也没有增量文本 diff 和持续上屏机制，因此不能边说边把文字打到光标处。

### 修复

- `DictationOverlay` 现在把 `toggleStartRecording` 视为 `recording`。
- `DictationOverlay` 现在把 `toggleStopAndTranscribe` 视为 `transcribing`。
- overlay 窗口透明兜底从透明背景改为黑色背景；如果 WebView2 仍暴露窗口底层背景，视觉上也更接近黑色胶囊。
- 主窗口里的录音状态视觉改为更接近桌面浮窗的黑色胶囊声波。

### 验证

新增并通过回归测试：

```bash
npm test -- --run src/DictationOverlay.test.tsx
```

局部 UI 验证通过：

```bash
npm test -- --run src/DictationOverlay.test.tsx src/VoiceOverlay.test.tsx src/App.test.tsx
npm run typecheck
```

### 后续方向

真正实时输入需要单独设计，至少包含：分块录音、流式或近实时 ASR、稳定的 partial/final 文本合并、增量上屏策略、撤回/替换策略和目标应用焦点保护。不能只靠当前 `whisper-cli.exe` 整段转写接口实现。

## 2026-05-27 Mermaid 图无法渲染

### 现象

维护者打开文档时，Mermaid 图报错：

```text
Error: Parse error on line 7:
...:ts  B --> C[invoke('simulate_dictation
----------------------^
```

### 影响

运行指南和代码导读里的图不能渲染，影响 Rust/Tauri 新手理解调用链路。

### 根因

Mermaid 节点标签里包含括号、引号、`#[...]` 等字符，解析器把它们当成语法结构而不是普通文本。

### 修复

将 Mermaid 节点文本改成更简单的中文标签，避免在节点标签内直接写复杂代码片段、引号和括号。

### 验证

文档中的 Mermaid 图改为简单节点标签后，不再触发该解析错误。

## 2026-05-27 Vite dependency scan 提示依赖无法解析

### 现象

运行开发模式时出现：

```text
Failed to run dependency scan. Skipping dependency pre-bundling.
The following dependencies are imported but could not be resolved
```

### 影响

维护者不清楚这是致命错误还是开发服务器提示。

### 根因

前端引入 Tauri API 后，Vite 预扫描阶段可能没有正确处理 Tauri runtime 相关依赖。

### 修复

显式安装并使用 `@tauri-apps/api`，并在文档中说明：如果窗口能打开、按钮能用，这类 dependency scan 提示通常不是应用编译失败；真正需要处理的是明确依赖无法解析导致白屏或启动失败。

### 验证

后续 `npm run typecheck`、`npm test -- --run`、`npm run build` 和 `npm run tauri -- build --debug` 均能通过。

## 2026-05-27 Tauri error 显示为 `[object Object]`

### 现象

真实转写失败时，诊断日志只显示：

```text
[object Object]
```

### 影响

维护者无法知道是 ASR 配置缺失、模型不存在、whisper.cpp 执行失败，还是其他错误。

### 根因

Rust command 返回的是结构化 Tauri error，前端直接用 `String(error)` 展示对象，导致对象被转成 `[object Object]`。

### 修复

新增 `src/errorFormat.ts`，统一处理字符串、`Error`、`{ message, code }` 等错误结构，把它们格式化成中文可读文本。

### 验证

新增 `src/errorFormat.test.ts`，覆盖结构化错误格式化。后续前端测试通过：

```bash
npm test -- --run
```

## 2026-05-27 不知道如何配置 whisper.cpp

### 现象

维护者不知道 `whisper.cpp`、`whisper-cli.exe` 和模型文件应该从哪里下载，也不希望普通用户手动配置环境变量。

### 影响

真实转写门槛过高，MVP 只能被熟悉命令行和环境变量的人验证。

### 根因

早期实现只支持通过下面的环境变量配置：

```text
VOXTYPE_WHISPER_CPP_BINARY
VOXTYPE_WHISPER_CPP_MODEL
VOXTYPE_ASR_LANGUAGE
```

这对普通用户不友好，也不符合桌面工具预期。

### 修复

新增应用内 ASR 配置和一键安装：

- `src-tauri/src/asr_config.rs`：保存和检测 ASR 配置。
- `src-tauri/src/asr_installer.rs`：下载官方 Windows x64 CPU 版 `whisper-bin-x64.zip` 和 `ggml-base.bin`。
- 前端 `ASR 配置` 面板：支持保存、检测和一键安装。

安装位置在 Tauri app data 目录，不写入仓库。

### 验证

维护者已完成一键安装，界面能够调用 `whisper-cli.exe` 并返回文本。后续验证命令通过：

```bash
cargo test --manifest-path src-tauri/Cargo.toml
npm run typecheck
npm run tauri -- build --debug
```

## 2026-05-27 真实闭环返回 `(音)`

### 现象

维护者长时间说话后，whisper.cpp 仍只返回：

```text
(音)
```

录音日志显示有样本、有时长、有音量：

```text
采集到 1203569 个 mono 样本，时长 27291 ms，采样率 44100 Hz；峰值音量 4755，RMS 音量 573
```

### 影响

录音到转写链路虽然跑通，但结果不可用。

### 已排除

- 不是完全没有录到声音：导出的 `last-asr-input.wav` 中能听到维护者说话。
- 不是 whisper.cpp 没安装：`whisper-cli.exe` 和 `ggml-base.bin` 已存在，命令能返回结果。
- 不是 ASR 配置路径缺失：应用内配置状态为 ready。

### 根因

第一轮判断是输入设备可疑，因为界面只看到 `Remote Audio`。后来维护者播放导出的 `last-asr-input.wav`，确认里面有自己的声音，但声音像电子音、变音，节奏类似原始录音。

进一步用 TDD 复现后确认真正的代码级根因：`RecordingBuffer::push_interleaved_i16` 第一次处理双声道回调后，把 `channels` 改成了 `1`。后续 `cpal` 回调仍然传入双声道 interleaved 数据，但代码已经把它当作单声道数据直接追加，导致左右声道样本错位进入 mono buffer，最终导出的 WAV 听起来像变音。

失败测试：

```rust
fn stereo_samples_remain_mixed_across_multiple_callbacks()
```

失败时实际结果：

```text
left: [200, -200, 200, 400, -200, -400]
right: [200, -200, 300, -300]
```

### 修复

在 `RecordingBuffer` 中保留原始输入声道数：

- `source_channels`：始终表示输入设备的真实声道数。
- `channels`：表示当前输出 buffer 已经归一化后的声道数，停止录音后可以显示为 `1`。

后续每次音频回调都按 `source_channels` 做 frame 分组和双声道平均，不再受 `channels` 被改成 `1` 的影响。

### 验证

回归测试从失败变为通过：

```bash
cargo test --manifest-path src-tauri/Cargo.toml stereo_samples_remain_mixed_across_multiple_callbacks
```

全量 Rust 测试通过：

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

结果：34 个 Rust 测试全部通过。

### 后续验证

维护者需要重新录音并导出 `last-asr-input.wav`：

1. 启动 `npm run tauri -- dev`。
2. 录一段中文。
3. 点击 `导出最近录音 WAV`。
4. 播放 `last-asr-input.wav`。

如果修复有效，WAV 应该不再有明显电子音和变速感。之后再点击 `转写最近录音` 验证 whisper.cpp 输出。

## 2026-05-27 默认输入设备只有 `Remote Audio`

### 现象

输入设备下拉框里只有：

```text
Remote Audio
```

维护者说明当前机器是虚拟机，但虚拟机内其他语音输入法可以工作。

### 影响

不能简单认为“虚拟机完全没有麦克风”。系统可能通过远程音频设备暴露了录音能力，但设备名称不是物理麦克风名称。

### 根因判断

当前 VoxType 使用 `cpal` 枚举 Windows 暴露的输入设备。它能看到 `Remote Audio`，说明系统确实提供了一个输入设备。结合导出 WAV 有维护者声音，说明该设备可以录到语音。

真正影响识别的主要问题不是设备不存在，而是前一节记录的双声道回调处理错误。

### 修复

新增输入设备列表和选择 UI，保留对 `Remote Audio` 的支持；不再把 `Remote Audio` 直接视为不可用。

### 后续风险

如果以后在真实机器上出现多个输入设备，需要增加：

- 设备选择持久化。
- 输入音量实时提示。
- 录音前的设备测试按钮。

## 2026-05-27 上屏粘贴了旧剪贴板内容

### 现象

维护者点击真实闭环后，目标输入框粘贴的不是本次识别文本，而是之前手动右键复制过的文本。

### 影响

即使 ASR 返回文本，上屏链路也不可信。

### 根因

早期实现做了以下步骤：

1. 读取旧剪贴板。
2. 写入本次识别文本。
3. 发送 `Ctrl+V`。
4. 立即恢复旧剪贴板。

Windows 目标应用处理 `Ctrl+V` 可能晚于第 4 步，所以它实际读取到的是恢复后的旧剪贴板。

### 修复

当前 MVP 改为：

- 写入剪贴板后读回校验。
- 发送 `Ctrl+V` 使用 `Key::V`。
- 发送前做短暂等待。
- 暂时不恢复旧剪贴板，优先保证目标输入框读到本次文本。

### 验证

新增/更新剪贴板计划测试：

```rust
fn clipboard_plan_uses_ctrl_v_without_restoring_old_clipboard()
```

全量 Rust 测试通过。

### 取舍

当前策略会让系统剪贴板保留本次识别文本。这是 MVP 为可靠性做的取舍。后续可以评估延迟恢复、条件等待、`SendInput(KEYEVENTF_UNICODE)` 或 TSF/IME 方案。

## 2026-05-27 诊断日志只能看到最新几条

### 现象

维护者复制日志时，只能看到最新几条，前面的录音、转写、失败信息已经消失。

### 影响

无法复盘长链路问题。

### 根因

前端诊断日志保留数量太少，并且没有便捷复制所有日志的入口。

### 修复

- 诊断日志最多保留 100 条。
- 日志区域可滚动。
- 新增 `复制全部日志` 按钮。

### 验证

前端测试通过：

```bash
npm test -- --run
```

## 2026-05-27 Tauri debug build 无法覆盖 exe

### 现象

运行：

```bash
npm run tauri -- build --debug
```

失败：

```text
failed to remove file `src-tauri/target/debug/vox-type.exe`
Access is denied. (os error 5)
```

### 影响

无法生成新的 debug exe。

### 根因

旧的 `vox-type.exe` 进程还在运行，Windows 不允许覆盖正在运行的 exe 文件。

### 修复

结束旧进程后重跑 build：

```powershell
Get-Process | Where-Object { $_.ProcessName -eq 'vox-type' }
Stop-Process -Id <pid> -Force
npm run tauri -- build --debug
```

### 验证

结束旧进程后，`npm run tauri -- build --debug` 通过，并生成：

```text
src-tauri/target/debug/vox-type.exe
```

## 2026-05-27 另一台机器一键安装 whisper.cpp 卡住

### 现象

维护者把本机生成的 `vox-type.exe` 复制到另一台机器后，应用可以启动，但点击 `一键安装 whisper.cpp` 后长时间没有返回，看起来像卡住。

### 影响

普通用户无法判断是 exe 拷贝不完整、安装器失败、网络不可达，还是正在下载大模型。这个问题会直接影响第一次使用体验。

### 根因判断

只复制 debug exe 本身并不是主要问题：一键安装会把 whisper.cpp binary 和模型下载到目标机器的 Tauri app data 目录，不依赖仓库目录里的模型文件。

更可疑的根因是下载链路没有超时边界。旧实现使用 `reqwest::blocking::get`，没有显式连接超时或总下载超时；如果目标机器访问 GitHub Release 或 Hugging Face 很慢、被代理拦截或网络不可达，请求可能长时间阻塞，界面只能显示正在安装。

### 修复

`src-tauri/src/asr_installer.rs` 改为通过 `reqwest::blocking::Client::builder()` 创建下载客户端，并配置：

- 连接超时：20 秒。
- 单个下载总超时：300 秒。

这样网络不可达时应返回明确失败，而不是无限等待。

### 验证

新增测试覆盖超时配置：

```bash
cargo test --manifest-path src-tauri/Cargo.toml asr_installer
```

结果：6 个 `asr_installer` 测试通过。

### 后续风险

当前修复只能避免无限卡住，还没有下载进度条、取消按钮、国内镜像源或模型选择。后续应把下载拆成带阶段状态的流程，并在 UI 上显示当前阶段、目标文件和失败建议。

## 2026-05-27 前端测试之间互相污染

### 现象

新增默认主界面和诊断模式测试后，第二个测试点击 `诊断模式` 失败，Testing Library 报告找到多个同名按钮。

### 影响

测试结果不稳定，并且会误导为 UI 里真的渲染了多个主窗口。

### 根因

`src/test/setup.ts` 只加载了 `@testing-library/jest-dom/vitest`，没有在每个测试后调用 `cleanup()`。因此第一个测试渲染的 DOM 没有卸载，第二个测试又渲染了一份 App，导致页面里出现两个 `诊断模式` 按钮。

### 修复

在 `src/test/setup.ts` 中增加：

```ts
afterEach(() => {
  cleanup();
});
```

同时把诊断日志自动滚动改为兼容 jsdom 的可选调用，避免测试环境没有 `scrollIntoView` 时抛错。

### 验证

```bash
npm test -- --run src/App.test.tsx
```

结果：2 个前端测试通过。

## 2026-05-27 跨机器复制 debug exe 后显示 localhost refused

### 现象

维护者把 `src-tauri/target/debug/vox-type.exe` 复制到另一台机器运行后，窗口没有加载 VoxType UI，而是显示：

```text
Hmmm… can't reach this page
localhost refused to connect
ERR_CONNECTION_REFUSED
```

### 影响

另一台机器无法验证应用，也容易误判为前端资源、网络或系统防火墙问题。

### 根因

`src-tauri/target/debug/vox-type.exe` 是开发机本地调试产物，不是面向跨机器分发的安装包。它可能仍携带开发模式资源加载假设，另一台机器没有 `localhost:1420` 的 Vite dev server，因此 WebView 加载失败。

### 修复

启用 Tauri bundle：

```json
"bundle": {
  "active": true,
  "targets": "all",
  "icon": ["icons/icon.ico"]
}
```

之后运行：

```bash
npm run tauri -- build --debug
```

生成可复制到另一台机器的安装包：

```text
src-tauri/target/debug/bundle/nsis/VoxType_0.1.0_x64-setup.exe
src-tauri/target/debug/bundle/msi/VoxType_0.1.0_x64_en-US.msi
```

### 验证

`npm run tauri -- build --debug` 在结束旧 `vox-type.exe` 进程后通过，并生成 NSIS setup 和 MSI 两个 bundle。

### 后续风险

当前 bundle 仍是 debug bundle，适合测试分发链路。后续正式发布应使用 release build、签名、版本号和发布说明。

## 2026-05-27 跨机器转写返回繁体中文

### 现象

维护者在另一台机器安装 `VoxType_0.1.0_x64-setup.exe` 后，应用可以安装并运行到转写链路，但 whisper.cpp 返回的中文文本是繁体；本虚拟机上同类流程返回的是简体中文。

### 影响

第一版 MVP 的录音、转写、上屏链路仍然成立，但普通中文用户预期默认输出简体中文。跨机器输出不一致会影响可用性和信任感。

### 根因判断

当前 `WhisperCppEngine` 调用 whisper.cpp 时只传：

```text
-l zh -nt
```

没有传入简体中文 prompt，也没有做 OpenCC 类后处理。因此不同机器、模型文件、whisper.cpp 版本、音频内容或解码差异都可能导致繁简输出不一致。

### 当前结论

这不是第一版 MVP 的阻断问题，因为跨机器安装包已经能安装并运行到真实转写链路；但它是第二阶段必须处理的文本标准化问题。

### 后续修复方向

- 在 whisper.cpp 调用中评估 `--prompt` 或等价参数，提示“请使用简体中文输出”。
- 增加输出后处理，把繁体统一转换为简体。
- 在设置页暴露“输出文字：简体/繁体/跟随模型”选项。
- 为文本标准化增加单元测试，避免把专有名词、英文、标点处理坏。

## 2026-05-27 V2 语音波纹看起来不动

### 现象

维护者反馈：点击录音后，界面上的三条彩色线没有流动起来，看起来只是固定不动的三条横线，没有语音波纹感。

### 影响

主界面无法让用户直观看出“正在听”。即使录音链路在工作，视觉反馈也像静态装饰，影响日常输入体验。

### 根因

旧实现只渲染三条水平线，通过移动渐变背景表达动效。线条本身的形状、高度和节奏变化很弱，因此录音态看起来仍然是固定横线。

### 修复

将 `VoiceOverlay` 改为渲染 24 个彩色频谱柱，并叠加流动光带：

- 待命态：低幅度呼吸。
- 录音态：频谱柱错峰起伏，速度更快。
- 转写/上屏态：保持较高活动感。
- 失败态：保留错误状态色。

同时把主界面改成深色径向背景和半透明面板，使彩色流光效果更清楚。

### 验证

前端组件测试确认 `VoiceOverlay` 渲染 24 个 `.voice-wave-bar`：

    npm test -- --run VoiceOverlay.test.tsx

全量前端测试通过：

    npm test -- --run

### 后续风险

当前波纹是 CSS 动效，并没有实时绑定麦克风音量。后续如果要更像语音输入法，应把 `RecorderRuntimeStatus` 或实时 RMS 音量事件接到前端，让波形幅度随真实声音变化。

## 2026-05-27 规则文档出现控制字符

### 现象

在更新 PowerShell/UTF-8 规则后，`AGENTS.md` 和 `docs/harness/working-agreement.md` 中的 `bash -lc`、`rg` 被写坏，出现不可见控制字符，渲染为异常文本。

### 影响

规则文档本身变得不可信，后续 agent 可能继续复制错误规则。

### 根因

在 Windows 默认 PowerShell 外层或 shell 嵌套中使用反引号和中文片段时，反引号被解释为命令替换或转义相关语义，导致写入内容被破坏。

### 修复

使用 Git Bash 下的 UTF-8 脚本按整行重写，并用 `chr(96)` 生成 Markdown 反引号，避免 shell 解释。随后扫描控制字符，确认数量为 0。

### 验证

    grep -n 'PowerShell' AGENTS.md docs/harness/working-agreement.md

结果：两个文件的控制字符数量均为 0。

## 当前调试结论

## 2026-05-27 Git 提交元数据泄露个人信息

### 现象

维护者指出最近多个 commit 里包含个人姓名、公司邮箱或本机用户名等个人信息。进一步检查发现本地历史里的全部 18 个提交都存在个人 author/committer 元数据。

### 影响

VoxType 是开源项目，提交元数据会随仓库历史公开。个人姓名、公司邮箱和本机用户名不应作为项目默认提交身份进入公开历史。

### 根因

仓库没有设置项目级 Git author/committer，提交时沿用了本机或临时命令里的个人身份。后续临时修正提交时也没有统一使用项目身份。

### 修复

- 使用 Git 历史重写把所有提交的 author/committer 统一改为 `VoxType <maintainers@voxtype.dev>`。
- 删除 `filter-branch` 产生的 `refs/original` 备份引用。
- 执行 reflog 过期和 Git GC，清理旧提交对象引用。
- 设置仓库 local Git 身份为 `VoxType <maintainers@voxtype.dev>`。
- 更新 `AGENTS.md` 和 `docs/harness/working-agreement.md`，要求后续提交使用项目级身份。

### 验证

已完成以下扫描，均未发现残留：

- commit metadata 个人信息扫描：`metadata_findings 0`。
- 当前跟踪文件个人信息和常见密钥模式扫描：`tracked_findings 0`。
- 全历史文件内容个人信息和常见密钥模式扫描：`history_content_findings 0`。
- `git fsck --no-reflogs --unreachable --no-progress`：无输出。

### 后续风险

如果后续已经把旧历史推送到远端，必须强制更新远端并通知所有协作者重新克隆或重置；否则远端仍可能保留旧提交。当前检查到本地仓库没有 remote。

截至 2026-05-27，最重要的根因是录音双声道回调处理错误。这个问题会直接造成导出的 ASR WAV 变音，并解释 whisper.cpp 只返回 `(音)` 的现象。

下一次手动验证应优先确认：

1. 重新录音后导出的 `last-asr-input.wav` 是否恢复为正常人声。
2. `转写最近录音` 是否从 `(音)` 变成可读中文。
3. `转写并上屏最近录音` 是否粘贴本次识别文本，而不是旧剪贴板。

## 2026-05-27 Ctrl+Alt+Space 无反应

### 现象

维护者在 VS Code 输入框中聚焦后，按住 `Ctrl+Alt+Space` 说一句中文，没有看到任何浮窗或状态变化；松开后文字也没有进入输入框。

### 初步根因

代码层面已经注册 `tauri-plugin-global-shortcut`，底层 `global-hotkey` 在 Windows 上通过 `WM_HOTKEY` 发出 `Pressed`，再轮询主键释放发出 `Released`。但旧实现有两个可观测性缺口：

1. 没有独立桌面浮窗；主窗口不在前台时，即使热键事件进入应用，用户也看不到“正在录音”。
2. 前端收到 `voxtype-push-to-talk` 后没有单独写诊断日志；用户无法判断事件是否进入前端。

因此本轮不能直接断言是插件、VS Code、RDP 或组合键冲突导致，必须先补观测点。

### 修复

- 新增 `dictation-overlay` Tauri 窗口，默认隐藏、无边框、透明、always-on-top、skip taskbar。
- 新增 `src/DictationOverlay.tsx`，渲染底部小型五彩流光波纹浮窗。
- Rust 热键 handler 收到 `StartRecording` 时显示 overlay，收到 `StopAndTranscribe` 时保持 overlay 进入识别态。
- 前端快捷键闭环完成或失败后隐藏 overlay。
- 主窗口收到热键事件时写入诊断日志：`收到全局快捷键按下` / `收到全局快捷键松开`。
- 诊断模式新增 `测试桌面浮窗` 和 `隐藏桌面浮窗`，用于把 overlay 显示问题和热键事件问题拆开排查。
- `show_dictation_overlay` / `hide_dictation_overlay` 改为找不到窗口或显示失败时返回明确错误，避免诊断按钮出现“请求已发送但实际窗口不存在”的假阳性。
- 快捷键闭环结束或失败后，前端统一调用 `hideDictationOverlay()` 收尾，避免底部浮窗卡在识别态。
- 主界面和底部浮窗进一步收敛为深色半透明系统工具风格，底部浮窗的彩色频谱柱录音态动效更明显。

### 自动化验证

- `npm test -- --run src/App.test.tsx src/DictationOverlay.test.tsx` 通过，2 个测试文件、6 个测试通过。
- `npm run typecheck` 通过。
- 新增前端回归测试覆盖：收到快捷键按下会启动录音；收到松开会停止录音、转写、上屏，并调用 `hideDictationOverlay()`。

### 下次验证方法

1. 运行 `npm run tauri -- dev`。
2. 进入诊断模式，点击 `测试桌面浮窗`。如果底部没有出现小型彩色浮窗，并且诊断日志显示“桌面浮窗显示失败”，优先排查 Tauri overlay 窗口；如果日志显示“请求已发送”但看不见，优先排查窗口层级、透明窗口或多屏位置。
3. 若浮窗能显示，回到 VS Code 输入框，按住 `Ctrl+Alt+Space`。
4. 如果没有浮窗，也没有诊断日志“收到全局快捷键按下”，说明快捷键没有进入 VoxType，可能被系统、VS Code、RDP 或其他软件占用。
5. 如果有日志但录音/转写/上屏失败，复制全部诊断日志继续排查后续链路。

## 2026-05-27 全局快捷键事件监听权限失败

### 现象

维护者再次验证后，诊断日志显示：

`event.listen not allowed. Permissions associated with this command: core:event:allow-listen, core:event:default`

同时 `全局快捷键已注册：Ctrl+Alt+Space`，并且 `测试桌面浮窗` 可以显示底部浮窗。

### 根因

Rust 侧全局快捷键注册成功，overlay command 也能显示窗口；失败点在前端事件监听权限。Tauri 2 默认 capability 没有显式授予 `core:event:allow-listen`，所以 `listen('voxtype-push-to-talk')` 被 ACL 拦截。结果是 Rust 发出的快捷键事件没有进入 React，录音、转写和上屏流程都不会启动。

开发模式下日志出现两次，是因为 React `StrictMode` 会触发 effect 挂载/卸载/再挂载；异步 `listen()` 如果在卸载后才返回，也可能留下重复监听。

### 修复

- 新增 `src-tauri/capabilities/default.json`，授予 `main` 和 `dictation-overlay` 窗口 `core:default`、`core:event:allow-listen` 和 `core:event:allow-unlisten`。
- 修复 `App.tsx` 和 `DictationOverlay.tsx` 中异步监听的清理竞态：如果组件已经卸载，`listen()` 返回后立即调用 `unlisten()`。
- 缩小 overlay 窗口尺寸到 `340 x 86`，并让 overlay 页面根节点完全透明，去掉胶囊背后的黑色方框感。
- 同步 `src-tauri/src/overlay.rs` 中的定位尺寸常量为 `340 x 86`，新增 Rust 回归测试锁住窗口配置和定位逻辑的一致性。
- 缩小底部胶囊和字体，降低阴影和黑色底色强度，让浮窗更像小型系统状态条。

### 验证结果

- `npm test -- --run` 通过：6 个测试文件、16 个测试。
- `npm run typecheck` 通过。
- `cargo test --manifest-path src-tauri/Cargo.toml` 通过：45 个 Rust 测试。
- `cargo fmt --all --manifest-path src-tauri/Cargo.toml --check` 通过。
- `npm run build` 通过。
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` 通过。
- `bash init.sh` 通过。
- `git diff --check` 通过。
- `npm run tauri -- build --debug` 第一次失败原因是旧 `vox-type.exe` 进程占用目标文件；关闭本项目进程后重跑通过，并生成 debug 版 exe/MSI/NSIS 安装包。

### 下次验证方法

1. 重新运行 `npm run tauri -- dev`。
2. 启动后诊断日志不应再出现 `event.listen not allowed`。
3. 点 `测试桌面浮窗`，底部应只看到一个小胶囊，不应有黑色方形背景。
4. 聚焦 VS Code、Notepad 或浏览器输入框，按住 `Ctrl+Alt+Space`。
5. 诊断日志应出现 `收到全局快捷键按下`；松开后应出现 `收到全局快捷键松开`，然后进入录音停止、转写和上屏日志。

## 2026-05-27 浮窗白框与柱状波形问题

### 现象

维护者验证后反馈：底部浮窗背后仍有白色框，视觉像粗糙卡片；浮窗里不应显示“正在听”等文字；当前柱状频谱不像波纹，边缘没有形成流动的弯曲线。

### 根因

旧实现仍然是“胶囊卡片 + 频谱柱 + 文字”的结构。即使窗口透明，卡片背景、阴影、文字布局和柱状条都会让它看起来像一个小面板，而不是系统语音输入那种轻量波纹。

白色框可能来自 overlay 页面在 React/CSS 接管前的浏览器默认白底首帧，也可能来自窗口尺寸中多余透明区域被系统合成成浅色背景。需要在 HTML 最早阶段就把 overlay 文档背景设为透明，并把窗口尺寸压到只包住波纹。

### 修复

- `DictationOverlay` 删除可见文案、胶囊卡片和柱状频谱，仅保留 `role="status"` 与 `aria-label`。
- 用 SVG 绘制 4 条彩色贝塞尔曲线；通过 `stroke-dashoffset`、水平位移、纵向缩放和 SVG `animate attributeName="d"` 让曲线持续流动并改变弯曲形状。
- `index.html` 在 React 启动前识别 `?view=overlay`，立即给 `html` 加 `overlay-document` 并写入透明背景样式，避免首帧白底。
- overlay 窗口从 `340 x 86` 缩小为 `300 x 82`，Rust 定位常量和测试同步更新。

### 验证结果

- `npm test -- --run` 通过：6 个测试文件、16 个测试。
- `npm run build` 通过。
- `npm run typecheck` 通过。
- `cargo test --manifest-path src-tauri/Cargo.toml overlay` 通过。
- `cargo fmt --all --manifest-path src-tauri/Cargo.toml --check` 通过。
- `git diff --check` 通过。

### 下次验证方法

1. 重新运行 `npm run tauri -- dev`。
2. 进入诊断模式，点击 `测试桌面浮窗`。
3. 底部应只看到几条彩色、弯曲、流动的波纹线；不应看到白色方框、胶囊卡片、文字或柱状频谱。
4. 按住 `Ctrl+Alt+Space` 时波纹应更活跃；松开后进入识别态，再由闭环结束隐藏。

## 2026-05-27 浮窗过大、周期过长与松开断裂问题

### 现象

维护者再次验证后反馈：波纹方向有进步，但仍有几个问题：背景应是黑色胶囊而不是白底；窗口太大，应缩小到当前约四分之一；波浪周期太长，只看到一两个波峰波谷；颜色没有明显流光感；按住录音时波浪应流动，松开后应保持完整静态波浪，而不是断成两节。

### 根因

上一版 SVG 使用大视窗 `300 x 82` 和长贝塞尔曲线，视觉上波长过长，导致单位宽度内峰谷太少。识别态仍使用部分动画与虚线相关样式，容易让曲线看起来像分段。纯透明背景也不符合维护者这次明确要求的“黑色胶囊状背景”。

### 修复

- overlay 窗口缩小到 `176 x 46`，Rust 定位常量和测试同步更新。
- SVG 视窗同步改为 `176 x 46`，新增黑色半透明胶囊底和轻微高光。
- 曲线改为短周期多峰谷路径，在小胶囊内显示多组波峰波谷。
- 录音态渲染 SVG `animate` 和流动渐变，让波纹和颜色流动。
- 识别态不渲染 `animate`，也不使用虚线分段，保持完整静态波浪。
- 回归测试新增断言：识别态 `animate` 数量为 `0`。

### 验证结果

- `npm test -- --run src/DictationOverlay.test.tsx` 通过：2 个测试。
- `npm test -- --run` 通过：6 个测试文件、16 个测试。
- `npm run typecheck` 通过。
- `npm run build` 通过。
- `cargo test --manifest-path src-tauri/Cargo.toml overlay` 通过。
- `cargo fmt --all --manifest-path src-tauri/Cargo.toml --check` 通过。
- `git diff --check` 通过。

### 下次验证方法

1. 重新运行 `npm run tauri -- dev`。
2. 进入诊断模式，点击 `测试桌面浮窗`。
3. 预期看到一个很小的黑色胶囊，内部是多峰谷细波纹。
4. 按住 `Ctrl+Alt+Space`：波纹和颜色应流动。
5. 松开后：波纹应保持完整但停止流动，不能断成两节。

## 2026-05-27 浮窗白色外壳与识别态卡住感

### 现象

维护者反馈：黑色胶囊外侧仍像有一圈白色外壳；胶囊还是太大；波纹接近目标但略乱，期望更像霓虹流光；松开进入识别态后完全不动，看起来像系统卡住；主界面整体 UI 字号、间距和组件仍偏粗大。

### 根因

上一版胶囊使用浅色描边和较强高光，透明窗口又正好贴合内容边界，边缘抗锯齿容易被看成白色外壳。识别态为了避免断裂停掉全部动画，但用户需要一个明确的“处理中”状态，所以完全静止会产生卡住感。主界面虽然已经压缩过，但标题、工具窗口、主预览区和卡片间距仍偏大。

### 修复

- 移除胶囊浅色描边，胶囊铺满 SVG 视窗，降低内部高光强度。
- overlay 窗口从 `176 x 46` 缩小到 `120 x 32`，Rust 定位常量和回归测试同步更新。
- 录音态继续保留快速流光和波形变化。
- 识别态新增低速呼吸动画，只做轻微透明度和纵向缩放变化，不使用 SVG `animate` 路径变形，避免断裂但能表达“处理中”。
- 主界面进一步收敛：窗口更窄，标题、正文、预览区、header、边距和阴影都减小。

### 验证结果

- `npm test -- --run src/DictationOverlay.test.tsx` 通过：2 个测试。
- `cargo test --manifest-path src-tauri/Cargo.toml overlay` 通过。
- `npm run typecheck` 通过。
- `npm test -- --run` 通过：6 个测试文件、16 个测试。
- `npm run build` 通过。
- `cargo fmt --all --manifest-path src-tauri/Cargo.toml --check` 通过。
- `git diff --check` 通过。

### 下次验证方法

1. 重新运行 `npm run tauri -- dev`。
2. 点击 `测试桌面浮窗`：应看到更小的黑色胶囊，外侧不应有白色壳。
3. 按住 `Ctrl+Alt+Space`：波形和颜色应快速流动。
4. 松开后：波纹应保持完整，并以较慢节奏呼吸，表示正在识别而不是卡住。
5. 主界面应比上一版更轻、更窄、字号更小。

## 2026-05-27 浮窗声波表现与转写处理中动效

### 现象

维护者反馈：黑色胶囊外仍像有一圈白色边；波浪形状仍有些怪，希望更接近录音软件中随声音大小变化的实时声波，但保留五颜六色的效果；转写中更适合三个点或类似“处理中”的动效。上屏曾有一次未成功，但后续多次复测可以成功，本轮不改上屏链路。

### 根因

上一版 CSS/窗口尺寸为 `120 x 32`，但 SVG `viewBox` 仍是 `176 x 46`，内容被缩放后更容易出现边缘合成异常。曲线路径适合表达抽象波纹，但不像录音软件的实时声波。转写态使用低速呼吸波纹，仍容易被理解为“波形卡住”。

### 修复

- SVG `viewBox` 改为 `120 x 32`，与 Tauri 窗口和 CSS 尺寸完全一致。
- 保留黑色胶囊背景，但不使用浅色外描边。
- 录音态改为 20 根多色声波柱，每根柱子用错峰动画模拟音量变化。
- 转写态改为 3 个动态点，背景声波降到低透明度，明确表达“处理中”。
- `index.html` overlay 首帧样式增加 `color-scheme: dark`、`margin: 0`、`overflow: hidden`，降低默认白底/白边参与合成的风险。

### 验证结果

- `npm test -- --run src/DictationOverlay.test.tsx` 通过：2 个测试。
- `npm run typecheck` 通过。
- `cargo test --manifest-path src-tauri/Cargo.toml overlay` 通过。
- `npm test -- --run` 通过：6 个测试文件、16 个测试。
- `npm run build` 通过。
- `cargo fmt --all --manifest-path src-tauri/Cargo.toml --check` 通过。
- `git diff --check` 通过。

### 下次验证方法

1. 重新运行 `npm run tauri -- dev`。
2. 点击 `测试桌面浮窗`：黑色胶囊外侧不应出现白边。
3. 按住 `Ctrl+Alt+Space`：看到彩色声波柱上下变化。
4. 松开进入转写：看到 3 个动态点，表示处理中。
5. 若上屏偶现失败，复制诊断日志和目标输入框场景，再单独排查上屏链路。

## 2026-05-27 浮窗灰白边、转写点数与主界面粗糙感

### 现象

维护者反馈：录音浮窗效果已经不错，但胶囊外侧仍像有灰白边；录音动画目前是固定节奏，没有跟真实说话节奏完全对上；转写态 3 个点太少；主窗口整体仍比浮窗粗糙，字体、颜色、间距、边框都需要继续收敛。

### 根因

胶囊内仍有高光矩形，透明窗口合成时容易被看作灰白边。转写态 3 点在小浮窗里信息量偏弱。录音动画目前只在前端按固定 keyframe 播放，前端还没有接收 Rust 录音流实时音量，所以无法真正随说话强弱变化。

### 修复

- 删除胶囊内部高光矩形，只保留纯黑底。
- 转写态改为 6 个彩色点，使用错峰缩放和透明度变化。
- 主窗口进一步细化：降低按钮/输入框边框强度，减小窗口宽度、标题、正文、header、预览区、卡片边距和圆角。
- 保留当前固定声波动画；后续若要跟随真实说话节奏，需要新增 Rust -> 前端实时音量事件。

### 验证结果

- `npm test -- --run src/DictationOverlay.test.tsx` 通过：2 个测试。
- `npm run typecheck` 通过。
- `npm test -- --run` 通过：6 个测试文件、16 个测试。
- `npm run build` 通过。
- `cargo test --manifest-path src-tauri/Cargo.toml overlay` 通过。
- `cargo fmt --all --manifest-path src-tauri/Cargo.toml --check` 通过。
- `git diff --check` 通过。

### 后续建议

下一步如果继续优化浮窗体验，应实现实时音量事件：录音线程定期计算 RMS/peak level，Rust 发出 lightweight event，前端根据 level 调整声波柱高度，而不是纯 CSS 固定动画。

## 2026-05-27 转写六点颜色和动效不明显

### 现象

维护者反馈：转写态虽然已经有 6 个点，但点的颜色看起来接近，动效也不明显；胶囊外侧仍能看到灰白边。

### 根因

6 个点共用同一个 SVG 渐变填充，点本身较小，动画只有缩放和透明度变化，视觉差异不够强。胶囊矩形铺满整个 `120 x 32` 视窗，边缘抗锯齿仍可能与透明窗口合成出浅色线。

### 修复

- 胶囊矩形从 `0,0,120,32` 改为 `1,1,118,30`，让窗口最外圈保持透明像素。
- 6 个点改为独立颜色类：蓝、浅蓝、绿、黄、粉、紫。
- 转写点动画改为上下跳动、缩放和透明度变化，错峰延迟为 `--dot-index * 95ms`。

### 验证结果

- `npm test -- --run src/DictationOverlay.test.tsx` 通过。
- `npm run typecheck` 通过。
- `npm run build` 通过。
- `git diff --check` 通过。

## 2026-05-27 胶囊外侧白光与转写点卡住感

### 现象

维护者反馈：胶囊外侧仍能看到一层白色光；6 个点颜色有了，但动效仍不明显，像卡住。

### 根因

SVG glow filter 应用在声波和点组上，滤镜模糊区域会超出黑色胶囊，透明窗口合成时就像胶囊外侧的白色光边。转写点使用 CSS transform 动画，在 SVG `circle` 上的表现不如直接动画 SVG 属性稳定。

### 修复

- 新增 `clipPath`，把声波和转写点都裁剪到胶囊矩形内部。
- 转写点继续保留 6 个独立颜色，但动效改为 SVG 原生 `animate`：改变 `cy`、`r`、`opacity`。
- 删除转写点的 CSS transform keyframes，避免 SVG/CSS transform 表现不明显。

### 验证结果

- `npm test -- --run src/DictationOverlay.test.tsx` 通过。
- `npm run typecheck` 通过。
- `npm run build` 通过。
- `git diff --check` 通过。

## 2026-05-27 胶囊背后白色背景与转写点动效方向

### 现象

维护者澄清：看到的问题不是胶囊内部光效溢出，而是胶囊背后仍有白色/红色边框背景；转写点不希望上下大跳，而希望像录音态一样通过颜色和光感变化表现流动。

### 根因

上一版 overlay 窗口尺寸与胶囊视觉尺寸完全贴合，WebView/窗口边缘合成时若底层默认背景参与，就会表现为胶囊背后一层浅色背景。转写点使用 `cy` 大幅变化，虽然动了，但运动语言与录音态声波光感不一致。

### 修复

- overlay 窗口增大到 `136 x 44`，胶囊视觉保持 `120 x 32`，在四周留透明安全边距。
- 胶囊、clipPath、声波柱和转写点整体平移到安全边距内，避免内容贴到窗口边缘。
- 转写点固定 `cy`，改为半径和透明度错峰变化，点距加大，形成光感扫描效果。

### 验证结果

- `npm test -- --run src/DictationOverlay.test.tsx` 通过。
- `npm run typecheck` 通过。
- `cargo test --manifest-path src-tauri/Cargo.toml overlay` 通过。
- `npm run build` 通过。
- `cargo fmt --all --manifest-path src-tauri/Cargo.toml --check` 通过。
- `git diff --check` 通过。

## 2026-05-27 透明安全边距导致背后灰框变大

### 现象

维护者反馈：上一版为了透明安全边距把窗口放大到 `136 x 44`，结果胶囊背后的灰色内层框更明显。期望不是大透明边距，而是去掉背后灰框，最多保留胶囊自身一层准白边框。

### 根因

在当前 WebView2 合成环境里，窗口尺寸大于胶囊内容时，透明安全边距区域仍可能被系统或 WebView 背景合成为可见底框。该策略不适合当前环境。

### 修复

- overlay 窗口、SVG 和胶囊视觉尺寸全部恢复为 `120 x 32`。
- clipPath、声波柱和转写点坐标同步恢复。
- 胶囊使用单层浅白描边 `rgba(255,255,255,0.28)`，让可见边界只有一层。

### 验证结果

- `npm test -- --run src/DictationOverlay.test.tsx` 通过。
- `npm run typecheck` 通过。
- `cargo test --manifest-path src-tauri/Cargo.toml overlay` 通过。
- `npm run build` 通过。
- `cargo fmt --all --manifest-path src-tauri/Cargo.toml --check` 通过。
- `git diff --check` 通过。


## 2026-05-28 主界面字号失衡与中文编码损坏

### 现象

维护者反馈主界面整体仍然很丑，字号大小不协调；检查时发现 `src/App.tsx` 的新主界面文案被写成 连续问号乱码，测试仍断言旧标题。

### 根因

Windows PowerShell 命令里直接嵌入中文做替换时，命令文本会被本地代码页破坏。主界面 CSS 也有多段历史覆盖，导致窗口、胶囊、卡片和字号层级互相打架。

### 修复

- 恢复主界面 JSX 中的 UTF-8 中文文案，并用 Python `unicode_escape` 抽样确认没有 `???` 残留。
- 将 `styles.css` 末尾 V4 主界面覆盖收束为 V4.3 样式：暗色控制台、紧凑状态条和与浮窗一致的黑色语音胶囊。
- 更新 `src/App.test.tsx` 默认界面断言，锁定新标题 `说话，上屏`、`开始录音`、`诊断模式` 和 `安装 whisper.cpp`。
- 规则补充：中文替换不再直接写进 PowerShell here-string，必要时用 `\uXXXX`、`chr()` 或从 UTF-8 文件读取中文。

### 验证结果

- `npm test -- --run src/App.test.tsx src/VoiceOverlay.test.tsx` 通过，2 个测试文件、11 个测试通过。
- `npm run typecheck` 通过。
- `npm run build` 通过。
- `cargo check --manifest-path src-tauri/Cargo.toml --lib` 通过。
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` 通过。
- `cargo fmt --all --manifest-path src-tauri/Cargo.toml --check` 通过。


## 2026-05-28 主界面过杂不够高级

### 现象

维护者反馈 V4.3 主界面仍然很丑，东西太杂，不够简洁、高级和高雅。

### 根因

V4.3 虽然统一了字号和色彩，但默认界面仍然承载了设备、ASR、快捷键、最近文本、安装等维护者信息，它的信息架构还是诊断面板，而不是日常输入工具。

### 修复

- 默认主界面改为极简结构：品牌、小标题、语音胶囊、一个主按钮、一行状态脚注和诊断入口。
- 移除主界面上的系统信息卡、最近文本卡、转写/上屏次级按钮和 ASR 安装条，这些仍可在诊断模式找到。
- 视觉上去掉大卡片框感，用中心悬浮黑色胶囊和弱光背景承载主体。

### 验证结果

- `npm test -- --run src/App.test.tsx src/VoiceOverlay.test.tsx` 通过，2 个文件、11 个测试。
- `npm run typecheck` 通过。
- `npm run build` 通过。
- `git diff --check` 通过。


## 2026-05-28 主界面过简且粗点视觉丑

### 现象

维护者反馈 V4.4 过于简单，并且主界面的彩色点太粗大、不如录音状态浮窗的声波胶囊好看。

### 根因

上一版把信息量减得太狠，但没有把浮窗中最有辨识度的录音态声波语言完整迁移过来。点状加载视觉放到主界面会显得粗糙。

### 修复

- 主界面 `VoiceOverlay` 在非错误态下强制使用 `recording` 声波视觉，保持 20 根彩色声波柱和流光。
- 增加轻量状态胶囊，显示阶段、状态文案和最近识别文本，避免主界面过空。
- 开始/停止按钮的图形改为三根小声波柱，不再使用粗大圆点。

### 验证结果

- `npm test -- --run src/App.test.tsx src/VoiceOverlay.test.tsx` 通过，2 个文件、11 个测试。
- `npm run typecheck` 通过。
- `npm run build` 通过。
- `git diff --check` 通过。
# 2026-05-29 Rust 单测二进制启动失败：STATUS_ENTRYPOINT_NOT_FOUND

## 现象

- 执行 `cargo test --manifest-path src-tauri/Cargo.toml preferences -- --no-capture` 时，代码可以编译到 test profile，但运行测试二进制失败。
- 同样执行既有 `hotkey` 测试也失败，说明不是 V6 Task 1 新增断言导致。
- 错误：`STATUS_ENTRYPOINT_NOT_FOUND` / exit code `0xc0000139`。

## 当前证据

- `cargo check --manifest-path src-tauri/Cargo.toml --lib` 通过。
- `npm run typecheck` 通过。
- `where.exe VCRUNTIME140.dll` 首个命中路径是 JDK 目录下的 `vcruntime140.dll`，然后才是 `C:\Windows\System32\vcruntime140.dll`，可能存在 Windows DLL 优先级/运行时入口点不匹配。

## 处理策略

- V6 实现期间不把 `cargo test` 失败误报为代码测试失败，也不声称 Rust 单测通过。
- Rust 代码阶段先以 `cargo check --manifest-path src-tauri/Cargo.toml --lib` 验证可编译。
- 后续需要调整 PATH 或运行环境，确保系统 VC runtime 优先，再恢复完整 `cargo test`。

# 2026-05-29 V6 自定义快捷键保存后不生效

## 现象

维护者在快捷键设置对话框中修改快捷键后，主界面显示的新快捷键会更新，并提示全局快捷键注册成功；但实际按新的快捷键不会触发录音和桌面浮窗。

同时维护者反馈快捷键设置入口在两个模式卡片内各放一个，信息架构不合理；设置对话框缺少明显关闭按钮。

## 根因

- `setup_push_to_talk_hotkey` 启动注册时始终使用 `HotkeyBinding::default()` 和 `ToggleHotkeyBinding::default()`，没有读取 `user-preferences.json` 中保存的快捷键偏好。
- `save_hotkey_preferences` 只保存偏好并返回 `HotkeyRegistrationStatus::registered(...)`，没有调用 `tauri-plugin-global-shortcut` 重新注册全局快捷键，因此 UI 出现假成功状态。
- 前端入口沿用了 V5.10 每个模式卡片一个齿轮的占位设计，但 V6 对话框一次配置两个快捷键，入口应提升到“快捷键状态”标题行。

## 修复

- 新增 `hotkey::RuntimeHotkeyBindings` 和 `runtime_bindings_from_preferences`，将保存偏好与默认值合并成运行时快捷键绑定，并复用重复/空值校验。
- 启动时从用户偏好读取快捷键并注册；保存快捷键后调用 `global_shortcut().unregister_all()`，再用新绑定重新注册，只有实际注册成功才返回成功状态。
- 主界面输入模式区只保留一个上级设置按钮，`title` / `aria-label` 为“自定义快捷键”。
- 快捷键设置对话框新增右上角关闭按钮，`aria-label` 为“关闭快捷键设置”。

## 验证结果

- `npm test -- --run src/App.test.tsx` 通过，1 个测试文件、11 个测试。
- `npm run typecheck` 通过。
- `cargo test --manifest-path src-tauri/Cargo.toml hotkey::tests::runtime_bindings_use_saved_preferences_with_defaults --no-run` 通过。
- `cargo check --manifest-path src-tauri/Cargo.toml --lib` 通过。

## 残余风险

本机完整 Rust 测试二进制运行仍受既有 `STATUS_ENTRYPOINT_NOT_FOUND` 环境问题影响；本轮 Rust 验证使用 `--no-run` 和 `cargo check --lib`。

# 2026-05-30 V6 MiniMax ASR 官方 endpoint 未确认

## 现象

V6 需要 MiniMax 云端识别配置，但调研阶段没有找到 MiniMax 官方 ASR / Speech-to-Text endpoint、音频上传字段、支持格式和返回文本字段。

## 处理

- 保留 MiniMax 非密钥配置 UI 和状态检测。
- 新增 `src-tauri/src/cloud_asr.rs` 占位抽象，构造请求配置时不包含 API key。
- 真实转写函数明确返回“MiniMax 云端识别尚未启用”，避免把第三方猜测接口写进产品代码。

## 验证

- `cargo test --manifest-path src-tauri/Cargo.toml cloud_asr --no-run` 通过。
- `cargo check --manifest-path src-tauri/Cargo.toml --lib` 通过。

## 残余风险

后续只有拿到官方 ASR endpoint、认证头格式、音频上传字段、支持格式和返回文本字段后，才能启用真实 MiniMax 云端转写。

# 2026-05-30 V6 停止后缺少转写动效、连续输入历史污染和导出无反馈

## 现象

维护者手动验证发现：

- `Ctrl+Alt+Space` 按住说话松开后会转写，但没有显示六点转写动效。
- `Ctrl+Alt+V` 连续输入第二次按下停止后也没有显示六点转写动效。
- 连续输入模式会把每个实验实时片段都写入识别记录，导致一次录音拆成很多条记录，还包含 `(字幕:J Chong)`、`(不知)`、`(無法 寶寶)` 等短噪声片段。
- 识别记录的“导出”按钮点击后没有主界面反馈，用户无法判断是否生效。

## 根因

- `handleStopTranscribeAndInsertNow` 只更新 React 状态为 `transcribing`，没有显式调用 Tauri `show_transcribing_overlay`，依赖热键事件层的 overlay 状态不够可靠。
- `handleStopLiveToggleRecording` 停止录音后立刻隐藏 overlay，再处理尾段转写，因此用户看不到停止后的六点转写状态。
- `processLiveTranscriptionChunk` 和 `processFinalLiveTranscriptionChunk` 每次成功都调用 `addTranscriptRecord`，把内部切片实现细节暴露到用户历史里。
- `handleExportRecords` 只写剪贴板和诊断日志，没有面向主界面的 `role="status"` 反馈。

## 修复

- `tauriClient` 新增 `showTranscribingOverlay`，Rust `invoke_handler` 暴露 `show_transcribing_overlay` command。
- 按住说话和连续输入停止录音后都显式调用 `showTranscribingOverlay`，直到转写/上屏处理完成后再隐藏 overlay。
- `DictationOverlay` 将 `toggleStopAndTranscribe` 映射为 `transcribing`，WebView fallback 也显示 6 个转写点。
- 连续输入的 live chunk 仍即时上屏，但只进入 session buffer；停止后合并为一条识别记录，并过滤常见短括号噪声。
- 识别记录导出后在主界面显示“已复制 N 条记录到剪贴板”；失败时显示错误。
- MiniMax API Key 输入改为 password field，保存时写入 Windows 用户环境变量 `MINIMAX_API_KEY`。实现直接写 `HKCU\\Environment`，不通过 `setx` 命令行参数传递密钥。

## 验证

- `npm test -- --run src/App.test.tsx src/DictationOverlay.test.tsx` 通过，2 个测试文件、18 个测试。
- `npm test -- --run` 通过，6 个测试文件、28 个测试。
- `npm run typecheck` 通过。
- `npm run build` 通过。
- `cargo check --manifest-path src-tauri/Cargo.toml --lib` 通过。
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` 通过。
- `cargo fmt --all --manifest-path src-tauri/Cargo.toml --check` 通过。
- `python -m json.tool docs/harness/feature_list.json` 通过。
- `git diff --check` 通过。

## 残余风险

本轮自动验证覆盖前端状态、command 可编译和 Rust lint；真实全局快捷键、原生浮窗视觉、剪贴板导出和 Windows 用户环境变量写入仍需要维护者在 `npm run tauri -- dev` 下手动验收。

# 2026-06-01 Baidu ASR 404 and auth mismatch

## Symptom

Maintainer selected Baidu ASR and local config check passed, but `Ctrl+Alt+Space` produced no visible transcript. Diagnostics showed `Baidu ASR HTTP 404 Not Found: 404 page not found`.

## Root Cause

Official short speech API docs confirm the endpoint is `http://vop.baidu.com/server_api`. The auth docs at `https://cloud.baidu.com/doc/SPEECH/s/cm8sn2bii` show the access token flow: API Key plus Secret Key are exchanged at `https://aip.baidubce.com/oauth/2.0/token`, then the returned `access_token` is used as the ASR JSON `token` field. VoxType previously treated `BAIDU_ASR_API_KEY` as that `token`, so the runtime auth model was incomplete and could not be trusted as a valid Baidu request.

## Fix

- Added `BAIDU_ASR_SECRET_KEY` user environment support.
- Added Baidu Secret Key password input and save command.
- Baidu readiness now requires both API Key and Secret Key.
- Runtime transcription fetches OAuth access token first, then sends that access token as JSON `token` to `http://vop.baidu.com/server_api`.
- ASR request tests verify raw API Key and Secret Key are not put in the ASR request URL/body.

## Verification

- `npm test -- --run src/App.test.tsx` passed, 17 tests.
- `npm run typecheck` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml cloud_asr_config --no-run` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml cloud_asr --no-run` passed.
- `cargo check --manifest-path src-tauri/Cargo.toml --lib` passed.

## Residual Risk

Real Baidu network transcription still needs maintainer validation in `npm run tauri -- dev` with valid `BAIDU_ASR_API_KEY` and `BAIDU_ASR_SECRET_KEY`. Full Rust test execution remains subject to the existing Windows `STATUS_ENTRYPOINT_NOT_FOUND` test-binary environment issue, so Rust verification used `--no-run` plus `cargo check`.
# 2026-06-01 V7.1 record rows and model page visual polish

## 现象

维护者复核 V7 UI 后反馈：

- 主界面统计区的时长、字数、速度被文字标签替代，原来的小图标更清楚也更有颜色层次。
- 识别记录每一条把输入方式、模型、时长和字数拼接成一行大文字，视觉粗糙且不好扫读。
- 模型选择与配置页面仍显得杂乱，三个模型按钮没有形成清晰的选中/未选中层级，配置内容平铺太重。

## 根因

- V7 首轮 UI polish 为了修复问号图标，过度退回了文字标签，丢掉了统计图标语言。
- 识别记录 metadata 使用普通文本流而不是结构化 chip，导致记录正文、统计和动作按钮之间缺少视觉边界。
- 模型配置页同时展示本地、百度短语音和百度实时 WebSocket 三块配置，信息密度超过单页可读范围。

## 修复

- 主界面统计区恢复为本地 inline SVG 图标 + 数值的 `StatPill`，并给次数、时长、字数、速度配置不同图标颜色。
- 每条识别记录改成正文区域 + 底部 footer，输入方式、模型、时长和字数分别显示为小胶囊；复制、重新上屏、删除三个 icon button 收敛到右下角。
- 模型选择页保持单页，但模型配置区改为三个大按钮 tab：本地 whisper.cpp、百度短语音、百度实时 WebSocket；只显示当前选中的配置面板。
- 使用浅绿色径向渐变和更深/更浅的绿色按钮状态区分选中与未选中，百度实时 WebSocket 继续明确标记为 V8 预留。

## 验证

- `bash init.sh` 通过。
- `npm test -- --run src/App.test.tsx` 通过，1 个测试文件、16 个测试。
- `npm run typecheck` 通过。
- `npm run build` 通过。
- `python -m json.tool docs/harness/feature_list.json` 通过。
- `git diff --check` 通过，仅有既有 CRLF/LF 提示，无 whitespace error。

## 残余风险

本轮自动验证覆盖 React 行为、TypeScript 和生产构建；真实 Tauri 桌面窗口里的视觉比例、滚动高度和百度短语音真实转写仍需要维护者手动验收后再把 `v7-001` 标记为 `passing`。
