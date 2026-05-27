# VoxType 代码导读

本文面向 Rust 和 Tauri 新手，按“从用户点击按钮到 Rust 执行”的顺序解释当前代码。它不是完整 Rust 教程，只解释你读本仓库时马上会遇到的概念。

## 先看哪几个文件

建议按这个顺序读：

1. `src/App.tsx`：先看用户界面、按钮点击和快捷键事件后发生什么。
2. `src/VoiceOverlay.tsx` 和 `src/voiceOverlayModel.ts`：看 V2 语音状态和波纹如何渲染。
3. `src/tauriClient.ts`：看前端怎样调用 Rust、监听 Tauri 事件。
4. `src-tauri/src/lib.rs`：看 Rust 暴露了哪些 Tauri command，以及全局快捷键如何注册。
5. `src-tauri/src/state.rs`：看状态是怎么表示的。
6. `src-tauri/src/config.rs`：看默认配置是什么。
7. `src-tauri/src/recorder.rs`：看麦克风信息、录音 stream 和音频 buffer。
8. `src-tauri/src/asr/mod.rs`：看 mock ASR、whisper.cpp adapter 和简体中文 prompt。
9. `src-tauri/src/insertion/mod.rs`：看剪贴板上屏。
10. `src-tauri/src/hotkey.rs`：看全局按住说话的状态机。
11. `src-tauri/src/preferences.rs`：看输入设备选择如何持久化。
12. `src-tauri/src/tray.rs`：看系统托盘。

## 一次按钮点击怎么走

```mermaid
flowchart LR
  classDef ui fill:#dbeafe,stroke:#2563eb,color:#172554
  classDef bridge fill:#fef3c7,stroke:#d97706,color:#422006
  classDef rust fill:#dcfce7,stroke:#16a34a,color:#052e16
  classDef result fill:#fce7f3,stroke:#db2777,color:#500724

  A[用户点击按钮]:::ui --> B[App.tsx 事件函数]:::ui
  B --> C[tauriClient.ts]:::bridge
  C --> D[Tauri invoke]:::bridge
  D --> E[lib.rs command]:::rust
  E --> F[Rust 模块执行业务]:::rust
  F --> G[返回 JSON 结果]:::result
  G --> H[React 更新状态和诊断日志]:::ui
```

关键理解：前端不直接碰系统能力。凡是麦克风、剪贴板、托盘、whisper.cpp 这类能力，都要通过 Tauri 调 Rust。

## `src/App.tsx`

这是当前主界面。

它维护几类状态：

| 状态 | 类型 | 用途 |
|---|---|---|
| `config` | `AppConfig` | 显示快捷键、语言、ASR、上屏策略 |
| `status` | `AppStatus` | 显示当前阶段和最后一次转写文本 |
| `recorderInfo` | `RecorderInfo` | 显示默认麦克风信息 |
| `inputDevices` | `RecorderInfo[]` | 显示可选择的输入设备列表 |
| `runtimeMessage` | `string` | 告诉你当前是浏览器模式还是 Tauri 模式 |
| `diagnostics` | `DiagnosticEntry[]` | 显示每一步成功/失败和原因 |

V2 主界面还会根据 `status` 和录音状态生成 `voiceOverlayModel`，交给 `VoiceOverlay` 渲染深色语音状态面板和彩色频谱波纹。

当前关键按钮：

- `handleSimulateDictation`：调用 mock 听写闭环。
- `handleClipboardInsert`：调用剪贴板上屏。
- `handleStartRecording` / `handleStopRecording`：启动和停止真实录音采集。
- `handleTranscribeLastRecording`：把最近一次录音交给 whisper.cpp 转写。
- `handleTranscribeAndInsert`：先转写，再给维护者 3 秒切回目标输入框，然后上屏。
- `handleStopTranscribeAndInsertNow`：全局快捷键松开后使用；停止录音、转写并立即上屏，因为用户焦点理论上仍在目标输入框。
- `handleExportLastRecordingWav`：把最近一次 ASR 输入导出为 WAV，用来复盘识别质量问题。

浏览器模式和 Tauri 模式的区别在这里判断：

```ts
isTauriRuntime()
```

如果不是 Tauri 模式，就不能测麦克风、托盘、剪贴板真实上屏。

## `src/VoiceOverlay.tsx` 和 `src/voiceOverlayModel.ts`

`voiceOverlayModel.ts` 把通用状态 `AppStatus` 转成适合界面展示的模型：标题、说明、模式、预览文本和诊断提示。

`VoiceOverlay.tsx` 负责渲染 V2 的语音状态面板。当前波纹不是三条固定线，而是 24 个 `.voice-wave-bar` 频谱柱：

- `idle`：低幅度呼吸，表示待命。
- `recording`：频谱柱错峰起伏，表示正在听。
- `transcribing` / `inserting`：保持活动状态，表示后台正在处理。
- `succeeded` / `failed`：降低或切换视觉状态，配合文字说明。

当前波纹是 CSS 动效，还没有绑定实时麦克风音量。后续如果要更接近真实语音输入法，需要从 Rust 录音链路持续发音量事件到前端。

## `src/tauriClient.ts`

这个文件是前端到 Rust 的唯一入口封装。

```ts
export async function simulateDictation(): Promise<AppStatus> {
  return invoke<AppStatus>('simulate_dictation');
}
```

这段代码的意思：请求 Tauri 调 Rust 里名叫 `simulate_dictation` 的 command，并期待它返回 `AppStatus`。

好处是 `App.tsx` 不需要到处写字符串形式的 command 名称。

`listenToPushToTalk` 使用 Tauri event API 监听 Rust 发出的 `voxtype-push-to-talk` 事件。Rust 只负责告诉前端“快捷键按下/松开”，真正的录音、转写和上屏仍复用 `App.tsx` 里的现有流程。

## `src-tauri/src/lib.rs`

这是 Rust 侧的 Tauri command 总入口。

当前 command：

| Command | 前端函数 | 当前作用 |
|---|---|---|
| `get_config` | `getConfig` | 返回默认配置 |
| `get_status` | `getStatus` | 返回空闲状态 |
| `simulate_dictation` | `simulateDictation` | mock ASR + mock insertion |
| `get_default_input_info` | `getDefaultInputInfo` | 读取默认麦克风信息 |
| `list_input_devices` | `listInputDevices` | 读取所有可用输入设备 |
| `set_input_device` | `setInputDevice` | 选择后续录音使用的输入设备 |
| `start_recording` | `startRecording` | 打开输入设备并开始采集 |
| `stop_recording` | `stopRecording` | 停止采集并生成 ASR 输入 |
| `get_user_preferences` | `getUserPreferences` | 读取已保存的输入设备偏好 |
| `transcribe_last_recording` | `transcribeLastRecording` | 用 whisper.cpp 转写最近录音 |
| `transcribe_last_recording_and_insert` | `transcribeLastRecordingAndInsert` | Rust 侧最近录音转写并上屏 command |
| `export_last_recording_wav` | `exportLastRecordingWav` | 导出最近录音的 16 kHz WAV |
| `insert_text_with_clipboard` | `insertTextWithClipboard` | 用剪贴板尝试上屏 |

Tauri 通过这段注册 command：

```rust
.invoke_handler(tauri::generate_handler![
    get_config,
    get_status,
    simulate_dictation,
    get_default_input_info,
    insert_text_with_clipboard
])
```

如果前端调用的名字没有在这里注册，就会调用失败。

`lib.rs` 还会在 `.setup()` 阶段调用 `setup_push_to_talk_hotkey`：

- 使用 `tauri-plugin-global-shortcut` 注册 `Ctrl+Alt+Space`。
- 把 OS 层 `Pressed` / `Released` 交给 `hotkey.rs` 的状态机。
- 通过 `app.emit("voxtype-push-to-talk", payload)` 发给前端。

这样 Rust 不在快捷键回调里做长耗时 ASR，避免全局快捷键线程被转写阻塞。

## `state.rs`

`state.rs` 定义应用状态。

```mermaid
stateDiagram-v2
  [*] --> Idle
  Idle --> Recording: 后续按住快捷键
  Recording --> Transcribing: 后续松开快捷键
  Transcribing --> Inserting: 后续识别完成
  Inserting --> Succeeded: 上屏成功
  Inserting --> Failed: 上屏失败
  Transcribing --> Failed: 识别失败
  Recording --> Failed: 录音失败
```

当前真实代码里已经有这些状态枚举，但 UI 主要用到了 `idle`、`succeeded`、`failed`。

## `config.rs`

`config.rs` 定义默认配置：

```rust
hotkey: "Ctrl+Alt+Space"
language: "zh-CN"
asr_engine: "whisper.cpp"
insertion_strategy: "clipboard"
```

现在它仍是默认运行配置；输入设备选择已经通过 `preferences.rs` 单独持久化，快捷键、ASR 引擎和上屏策略后续再做完整设置页。

## `recorder.rs`

这个模块目前做三件事。

第一，读取默认麦克风信息：

```rust
pub fn default_input_info() -> Result<RecorderInfo, VoxError>
```

它使用 `cpal`：

- 找默认输入设备。
- 读取默认输入配置。
- 返回设备名、采样率、声道数。

第二，处理音频 buffer：

```rust
pub fn normalize_to_mono_i16(input: &[i16], channels: u16, sample_rate: u32) -> RecordingBuffer
```

如果输入是双声道，它会把左右声道平均成单声道。whisper.cpp MVP 通常希望我们给它稳定的 mono PCM。

第三，列出和选择输入设备：

```rust
pub fn list_input_devices() -> Result<Vec<RecorderInfo>, VoxError>
RecorderManager::set_input_device(...)
```

如果系统默认设备是 `Remote Audio` 这类远程音频设备，识别结果可能只返回 `(音)`。所以 UI 现在允许先切换到真实麦克风，再开始录音。

第四，管理真实录音 stream：

```rust
RecorderManager::start()
RecorderManager::stop()
RecorderManager::status()
```

`start()` 会用 `cpal` 打开默认输入设备，创建输入 stream，并把收到的音频帧写入 `RecordingSession`。

`stop()` 会停止 stream，并返回 `RecordedAudio`：

- `samples`：采集到的 mono PCM 样本。
- `sampleRate`：实际输入采样率，例如 `44100`。
- `channels`：停止后 buffer 的声道数，当前会归一成 `1`。
- `sampleCount`：样本数量。
- `durationMs`：按采样率计算出的录音时长。
- `asrSampleRate`：准备给 ASR 的目标采样率，当前是 `16000`。
- `asrSampleCount`：重采样后的 ASR 样本数量。
- `asrDurationMs`：按 `16000 Hz` 计算的 ASR 输入时长。

当前已经可以通过 `transcribe_last_recording` 把最近一次录音的 `16 kHz` ASR 输入交给 `WhisperCppEngine`。它优先使用应用内保存的 ASR 配置；如果应用内没有保存路径，再用环境变量作为开发者兜底。

`export_last_recording_wav` 会把最近一次 `16 kHz` ASR 输入写成 `last-asr-input.wav`。这个文件不提交到仓库，只用于调试：如果文件里听不到人声，问题在输入设备或录音；如果听得到人声但识别差，问题更可能在模型或 ASR 参数。

## `asr_config.rs`

这个模块专门管理 whisper.cpp 的本机配置。它不负责下载模型，也不负责转写，只回答三个问题：

1. 当前 `whisper-cli.exe` 路径是什么。
2. 当前模型文件路径是什么。
3. 这两个路径是否存在，是否已经可以调用真实 ASR。

核心类型：

| 类型 | 作用 |
|---|---|
| `AsrConfig` | 用户保存的配置：binary 路径、model 路径、语言 |
| `AsrConfigStatus` | 给前端展示的检测结果：是否配置、文件是否存在、是否 ready、中文提示 |

读取优先级：

1. 应用内 JSON 配置。
2. 环境变量 `VOXTYPE_WHISPER_CPP_BINARY`、`VOXTYPE_WHISPER_CPP_MODEL`、`VOXTYPE_ASR_LANGUAGE`。
3. 默认语言 `zh`。

这样做是为了让普通用户在界面里保存路径，同时保留开发者用环境变量快速实验的能力。

## `preferences.rs`

这个模块保存用户偏好，目前只保存一个字段：

```rust
selected_input_device_name: Option<String>
```

当用户在界面里切换输入设备时，`set_input_device` 会同时更新 `RecorderManager` 和本地偏好文件。下次启动时，前端会先读取 `get_user_preferences`，再读取设备列表，如果保存过的设备仍然存在，就自动切回该设备。

这和 `asr_config.rs` 分开，是为了避免把 ASR 路径、模型配置和普通 UI 偏好混在同一个模块里。

## `asr_installer.rs`

这个模块负责“一键安装 whisper.cpp”。它不参与转写，只负责把外部依赖准备好，并把结果写回 `asr_config.rs`。

当前默认安装：

| 文件 | 来源 | 安装到 |
|---|---|---|
| `whisper-bin-x64.zip` | `ggml-org/whisper.cpp` GitHub Release | 应用数据目录的 `managed-asr/whisper.cpp/downloads/` |
| `whisper-cli.exe` | 从 ZIP 解压 | 应用数据目录的 `managed-asr/whisper.cpp/bin/` |
| `ggml-base.bin` | `ggerganov/whisper.cpp` Hugging Face 仓库 | 应用数据目录的 `managed-asr/whisper.cpp/models/` |

## `insertion/mod.rs`

这个模块负责把文字送进目标输入框。当前 MVP 使用剪贴板策略：

1. 用 `arboard` 打开系统剪贴板。
2. 写入本次识别文本。
3. 读回剪贴板确认写入成功。
4. 等待一个很短的时间，让系统剪贴板状态稳定。
5. 用 `enigo` 发送 `Ctrl+V`。

早期版本会在发送 `Ctrl+V` 后马上恢复旧剪贴板。维护者实测发现目标软件可能还没读取新剪贴板，旧剪贴板就被恢复了，导致实际粘贴的是上一次手动复制的内容。因此当前 MVP 暂时不自动恢复旧剪贴板，优先保证上屏可靠性。

安装完成后，模块会调用 `save_asr_config`，所以前端不需要知道真实文件保存在哪里。

## `asr/mod.rs`

ASR 是 Automatic Speech Recognition，也就是语音识别。

这个模块有一个接口：

```rust
pub trait AsrEngine {
    fn transcribe(&self, pcm_16khz_mono: &[i16]) -> Result<Transcript, VoxError>;
}
```

当前两个实现：

| 实现 | 作用 |
|---|---|
| `MockAsrEngine` | 不识别真实音频，直接返回固定中文文本 |
| `WhisperCppEngine` | 准备调用 whisper.cpp CLI 做真实识别 |

`WhisperCppEngine` 当前做了这些边界工作：

1. 检查 whisper.cpp 可执行文件是否存在。
2. 检查模型文件是否存在。
3. 把 PCM 写成临时 WAV。
4. 调用 whisper.cpp CLI。
5. 从 stdout 读取转写文本。

当语言以 `zh` 开头时，它还会给 whisper.cpp 增加简体中文 prompt，提示模型优先输出简体中文。这不能 100% 保证永远不出繁体，但能降低跨机器输出不一致的概率。后续如果仍出现繁体，需要加 OpenCC 类后处理。

现在 `transcribe_last_recording` 已经会从 `asr_config.rs` 取得 binary/model 路径。`transcribe_last_recording_and_insert` 会在真实转写完成后调用剪贴板上屏 adapter，形成“最近录音 -> 转写 -> 上屏”的 proof-of-life。

## `hotkey.rs`

这个模块现在不再只是占位。它包含全局按住说话的纯状态机：

| 输入事件 | 当前状态 | 输出动作 |
|---|---|---|
| `Pressed` | 未按下 | `StartRecording` |
| `Pressed` | 已按下 | `Ignore` |
| `Released` | 已按下 | `StopAndTranscribe` |
| `Released` | 未按下 | `Ignore` |

这样做的原因是：OS 全局快捷键可能重复发送按下事件，如果没有状态机，可能会重复启动录音或重复转写。

`hotkey.rs` 不直接碰麦克风，也不调用 whisper.cpp。它只生成 `PushToTalkPayload`，由 `lib.rs` 发给前端。

## `insertion/mod.rs`

Insertion 指“文本上屏”，也就是把识别出来的文字输入到当前光标位置。

当前有两种实现：

| 实现 | 作用 |
|---|---|
| `MockInsertion` | 测试用，只检查文本不是空 |
| `ClipboardInsertion` | Windows 上用剪贴板 + `Ctrl+V` 尝试上屏 |

`ClipboardInsertion` 的流程：

```mermaid
flowchart TD
  A[收到文本] --> B{文本为空吗}
  B -->|是| C[返回错误]
  B -->|否| D[保存旧剪贴板]
  D --> E[写入新文本]
  E --> F[读回校验]
  F --> G[发送 Ctrl V]
  G --> H[保留本次剪贴板]
  H --> I[返回成功]
```

注意：返回成功只说明程序发送了粘贴动作，不保证目标软件一定收到文字。目标软件是否收到，取决于当时焦点在哪里。

## `tray.rs`

托盘模块创建两个菜单：

- `打开设置`
- `退出`

点击 `打开设置` 会显示并聚焦主窗口。点击 `退出` 会关闭应用。

## `error.rs`

这个模块把 Rust 内部错误统一成前端能理解的错误。

例如：

```rust
VoxError::Recorder("没有找到默认输入设备".to_string())
```

会变成前端能显示的错误信息。这样 UI 不需要理解 Rust 的内部错误类型。

## 当前代码和第一版 MVP 状态

```mermaid
flowchart TD
  classDef done fill:#bbf7d0,stroke:#16a34a,color:#052e16
  classDef missing fill:#fecaca,stroke:#dc2626,color:#450a0a
  classDef partial fill:#fde68a,stroke:#d97706,color:#451a03

  A[设置页和状态 UI]:::done --> B[Tauri command]:::done
  B --> C[默认麦克风探测]:::done
  B --> D[Mock 转写]:::done
  B --> E[剪贴板上屏 adapter]:::done
  C --> F[持续录音 stream]:::done
  F --> G[重采样到 16 kHz mono]:::done
  G --> H[ASR 配置检测]:::done
  H --> J[一键安装 whisper.cpp 和模型]:::done
  J --> K[真实 whisper.cpp 调用]:::done
  K --> E
  E --> I[目标软件 E2E 验证]:::done
```

第一版 MVP proof-of-life 已经跑通：录音、16 kHz ASR 输入、whisper.cpp 转写、延迟切回目标输入框后的剪贴板上屏都已经被维护者手动验证。V2 已经开始把它做得更像日常工具：深色主界面、彩色语音波纹、输出默认简体中文、全局按住说话和持久化输入设备选择都已接入代码。下一步需要维护者手动验证全局快捷键真实闭环，然后继续做快捷键设置、模型选择/下载进度、实时音量和更可靠的上屏策略。
