# VoxType 代码导读

本文面向 Rust 和 Tauri 新手，按“从用户点击按钮到 Rust 执行”的顺序解释当前代码。它不是完整 Rust 教程，只解释你读本仓库时马上会遇到的概念。

## 先看哪几个文件

建议按这个顺序读：

1. `src/App.tsx`：先看用户界面和按钮点击后发生什么。
2. `src/tauriClient.ts`：看前端怎样调用 Rust。
3. `src-tauri/src/lib.rs`：看 Rust 暴露了哪些 Tauri command。
4. `src-tauri/src/state.rs`：看状态是怎么表示的。
5. `src-tauri/src/config.rs`：看默认配置是什么。
6. `src-tauri/src/recorder.rs`：看麦克风信息、录音 stream 和音频 buffer。
7. `src-tauri/src/asr/mod.rs`：看 mock ASR 和 whisper.cpp adapter。
8. `src-tauri/src/insertion/mod.rs`：看剪贴板上屏。
9. `src-tauri/src/tray.rs`：看系统托盘。

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
| `runtimeMessage` | `string` | 告诉你当前是浏览器模式还是 Tauri 模式 |
| `diagnostics` | `DiagnosticEntry[]` | 显示每一步成功/失败和原因 |

当前两个按钮：

- `handleSimulateDictation`：调用 mock 听写闭环。
- `handleClipboardInsert`：调用剪贴板上屏。

浏览器模式和 Tauri 模式的区别在这里判断：

```ts
isTauriRuntime()
```

如果不是 Tauri 模式，就不能测麦克风、托盘、剪贴板真实上屏。

## `src/tauriClient.ts`

这个文件是前端到 Rust 的唯一入口封装。

```ts
export async function simulateDictation(): Promise<AppStatus> {
  return invoke<AppStatus>('simulate_dictation');
}
```

这段代码的意思：请求 Tauri 调 Rust 里名叫 `simulate_dictation` 的 command，并期待它返回 `AppStatus`。

好处是 `App.tsx` 不需要到处写字符串形式的 command 名称。

## `src-tauri/src/lib.rs`

这是 Rust 侧的 Tauri command 总入口。

当前 command：

| Command | 前端函数 | 当前作用 |
|---|---|---|
| `get_config` | `getConfig` | 返回默认配置 |
| `get_status` | `getStatus` | 返回空闲状态 |
| `simulate_dictation` | `simulateDictation` | mock ASR + mock insertion |
| `get_default_input_info` | `getDefaultInputInfo` | 读取默认麦克风信息 |
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

现在它只是默认值，还没有接本地配置文件保存。后续设置页需要能修改这些值并持久化。

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

第三，管理真实录音 stream：

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

当前还没有把录音结果传给 whisper.cpp。下一步要做的是把 `asrSampleRate = 16000` 的 ASR 输入交给 `WhisperCppEngine`。

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

还缺的是：从真实录音 stream 取得音频，并把 binary/model 路径从配置传进来。

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
  E --> F[发送 Ctrl V]
  F --> G[恢复旧剪贴板]
  G --> H[返回成功]
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

## 当前代码和真实 MVP 的差距

```mermaid
flowchart TD
  classDef done fill:#bbf7d0,stroke:#16a34a,color:#052e16
  classDef missing fill:#fecaca,stroke:#dc2626,color:#450a0a
  classDef partial fill:#fde68a,stroke:#d97706,color:#451a03

  A[设置页和状态 UI]:::done --> B[Tauri command]:::done
  B --> C[默认麦克风探测]:::done
  B --> D[Mock 转写]:::done
  B --> E[剪贴板上屏 adapter]:::partial
  C --> F[持续录音 stream]:::done
  F --> G[重采样到 16 kHz mono]:::done
  G --> H[真实 whisper.cpp 调用]:::partial
  H --> E
  E --> I[目标软件 E2E 验证]:::missing
```

下一步真正应该实现的是把 `RecordedAudio` 的 16 kHz ASR 输入交给 whisper.cpp，然后增加“录音后转写”的 Tauri command。这样才会从“能录到样本”进入“真实语音输入 proof-of-life”。
