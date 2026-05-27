# 调试记录

本文记录 VoxType MVP 调试过程中遇到的真实问题、根因判断、修复方式和验证证据。它不是使用教程；使用步骤见 `docs/guide/run-and-understand.md`。

## 记录规则

- 每个问题都要写清楚现象、影响范围、根因、修复、验证和后续风险。
- 只记录已经观察到的事实和已经验证过的判断；不把猜测写成结论。
- 涉及代码、命令、API、文件名和错误消息时保留原文。
- 面向维护者的解释默认使用中文。

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
