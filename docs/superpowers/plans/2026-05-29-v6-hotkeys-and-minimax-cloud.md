# VoxType V6 自定义快捷键与 MiniMax 云端识别实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让用户可以在界面里修改两种语音输入快捷键，并在模型选择页配置 MiniMax 云端语音识别，为后续本地/云端 ASR 切换打基础。

**Architecture:** V6 分两条主线：快捷键配置进入 `user-preferences.json` 并在 Tauri 启动时注册；云端模型配置新增独立 `cloud-asr-config.json`，前端先完成 MiniMax 配置、检测和保存，再接入 ASR 引擎抽象。MiniMax ASR 官方 endpoint 需要在实现前二次确认，不能把搜索结果里的非官方接口写死。

**Tech Stack:** React + TypeScript、Tauri command、Rust、serde JSON 配置、tauri-plugin-global-shortcut、MiniMax HTTP API。

---

## 当前状态

- 当前满意版已提交：`74aa469 style: align mode status indicators`。
- 工作区当前应保持干净；开始 V6 实现前先运行 `git status --short`。
- 当前两个快捷键是硬编码默认值：
  - 按住说话：`Ctrl+Alt+Space`
  - 连续输入：`Ctrl+Alt+V`
- 当前模型页已有 `本地 whisper.cpp / 云端 API` tab，但云端 API 仍是占位。
- 当前 ASR 运行路径只支持本地 `whisper.cpp`。

## V6 范围

V6 必做：

- 点击主界面两个设置图标，可以分别配置：
  - 按住说话快捷键。
  - 连续输入快捷键。
- 快捷键配置保存后持久化，下次启动继续生效。
- 快捷键冲突、格式错误、注册失败要在 UI 和诊断日志里可见。
- 模型选择页的 `云端 API` tab 支持 MiniMax 配置：
  - API Key。
  - Group ID 或官方要求的账号/组字段。
  - Base URL。
  - 模型名。
  - 语言。
  - 保存配置。
  - 检测配置。
- API Key 不能写入仓库、日志、文档、测试快照；UI 中默认用 password input。

V6 不做：

- 不做多厂商云端 API，只做 MiniMax。
- 不做流式实时云端识别，只做与当前录音后转写链路兼容的非流式接入。
- 不改桌面浮窗视觉风格。

V7 候选：

- 把桌面录音浮窗从当前深色彩色胶囊，调整为与主窗体一致的浅绿渐变版本。
- 需要保留当前浮窗的精致动效，只换色彩系统和背景质感。
- V7 实施前要先做视觉对比，因为当前浮窗虽然不统一，但动效质量较高。

---

## 文件结构

### 前端

- 修改 `src/MainWindow.tsx`
  - 设置图标从 disabled 状态改为可点击。
  - 点击后打开快捷键设置对话框。
- 新建 `src/HotkeySettingsDialog.tsx`
  - 展示两个快捷键输入项。
  - 支持录入/保存/取消。
  - 展示注册状态和错误信息。
- 修改 `src/ModelSettingsView.tsx`
  - 云端 API tab 改为 MiniMax 配置表单。
  - 本地和云端 tab 保持互斥选中状态。
- 修改 `src/App.tsx`
  - 加载、保存快捷键偏好。
  - 加载、保存云端 ASR 配置。
  - 刷新快捷键注册状态和云端配置状态。
- 修改 `src/tauriClient.ts`
  - 增加 `getHotkeyPreferences`、`saveHotkeyPreferences`、`getCloudAsrConfigStatus`、`saveCloudAsrConfig`、`testCloudAsrConfig`。
- 修改 `src/types.ts`
  - 增加快捷键偏好类型、云端 ASR 配置类型。
- 修改 `src/App.test.tsx`
  - 覆盖快捷键设置入口。
  - 覆盖 MiniMax 配置表单。
  - 覆盖空 API Key 不应显示为已就绪。

### Rust / Tauri

- 修改 `src-tauri/src/preferences.rs`
  - 在 `UserPreferences` 中增加 `push_to_talk_hotkey` 和 `toggle_dictation_hotkey`。
  - 兼容旧配置文件缺字段的情况。
- 修改 `src-tauri/src/hotkey.rs`
  - 增加快捷键格式校验函数。
  - 把默认快捷键和用户配置快捷键合并成运行时绑定。
- 修改 `src-tauri/src/lib.rs`
  - 启动时读取用户快捷键偏好并注册。
  - 增加保存快捷键偏好的 command。
  - 保存后重新注册快捷键，失败时保留旧快捷键或明确回滚。
- 新建 `src-tauri/src/cloud_asr_config.rs`
  - 负责 MiniMax 云端配置的读写、脱敏状态、ready 判断。
- 新建 `src-tauri/src/cloud_asr.rs`
  - 负责 MiniMax HTTP 调用。
  - 先把接口封装成 trait-friendly 单元，方便后续接入其它厂商。
- 修改 `src-tauri/src/asr.rs`
  - 增加云端 ASR 引擎抽象入口，但 V6 只在用户选择云端时调用。
- 修改 `src-tauri/Cargo.toml`
  - 如需要 HTTP client，优先使用 `reqwest` + `rustls-tls`。

### 文档

- 修改 `docs/harness/progress.md`
- 修改 `docs/harness/feature_list.json`
- 如实现中发现 MiniMax 官方 ASR endpoint 不明确，记录到 `docs/harness/debugging-log.md`。

---

## Task 1: 快捷键偏好数据结构

**Files:**

- Modify: `src-tauri/src/preferences.rs`
- Modify: `src/types.ts`
- Modify: `src/tauriClient.ts`

- [ ] **Step 1: 写 Rust 偏好测试**

在 `src-tauri/src/preferences.rs` 的 tests 中增加：

```rust
#[test]
fn saves_and_loads_hotkey_preferences() {
    let dir = tempfile::tempdir().unwrap();

    save_user_preferences(
        dir.path().to_path_buf(),
        UserPreferences {
            selected_input_device_name: None,
            push_to_talk_hotkey: Some("Ctrl+Alt+Space".to_string()),
            toggle_dictation_hotkey: Some("Ctrl+Alt+V".to_string()),
        },
    )
    .unwrap();

    let loaded = load_user_preferences(dir.path().to_path_buf());

    assert_eq!(loaded.push_to_talk_hotkey.as_deref(), Some("Ctrl+Alt+Space"));
    assert_eq!(loaded.toggle_dictation_hotkey.as_deref(), Some("Ctrl+Alt+V"));
}
```

- [ ] **Step 2: 跑测试确认失败**

```bash
cargo test --manifest-path src-tauri/Cargo.toml preferences
```

预期：因为 `UserPreferences` 还没有两个快捷键字段而失败。

- [ ] **Step 3: 增加字段和 normalize 逻辑**

在 `UserPreferences` 中增加：

```rust
pub push_to_talk_hotkey: Option<String>,
pub toggle_dictation_hotkey: Option<String>,
```

在 `normalize_preferences` 中 trim 空字符串为 `None`。

- [ ] **Step 4: 更新 TypeScript 类型**

在 `src/types.ts` 的 `UserPreferences` 中增加：

```ts
pushToTalkHotkey: string | null;
toggleDictationHotkey: string | null;
```

- [ ] **Step 5: 跑测试**

```bash
cargo test --manifest-path src-tauri/Cargo.toml preferences
npm run typecheck
```

- [ ] **Step 6: 提交**

```bash
git add src-tauri/src/preferences.rs src/types.ts src/tauriClient.ts
git -c user.name="VoxType" -c user.email="maintainers@voxtype.dev" commit -m "feat: store hotkey preferences"
```

---

## Task 2: 快捷键保存与重新注册

**Files:**

- Modify: `src-tauri/src/hotkey.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/tauriClient.ts`

- [ ] **Step 1: 写快捷键校验测试**

在 `src-tauri/src/hotkey.rs` tests 中增加：

```rust
#[test]
fn hotkey_validation_rejects_duplicate_bindings() {
    let error = validate_hotkey_pair("Ctrl+Alt+Space", "Ctrl+Alt+Space").unwrap_err();
    assert_eq!(error, "两个输入模式不能使用同一个快捷键");
}

#[test]
fn hotkey_validation_accepts_distinct_bindings() {
    validate_hotkey_pair("Ctrl+Alt+Space", "Ctrl+Alt+V").unwrap();
}
```

- [ ] **Step 2: 实现校验函数**

在 `src-tauri/src/hotkey.rs` 增加：

```rust
pub fn validate_hotkey_pair(push_to_talk: &str, toggle: &str) -> Result<(), String> {
    let push = push_to_talk.trim();
    let toggle = toggle.trim();
    if push.is_empty() || toggle.is_empty() {
        return Err("快捷键不能为空".to_string());
    }
    if push.eq_ignore_ascii_case(toggle) {
        return Err("两个输入模式不能使用同一个快捷键".to_string());
    }
    Ok(())
}
```

- [ ] **Step 3: 增加 Tauri command**

在 `src-tauri/src/lib.rs` 增加 `save_hotkey_preferences` command：

```rust
#[tauri::command]
fn save_hotkey_preferences(
    app: AppHandle,
    push_to_talk_hotkey: String,
    toggle_dictation_hotkey: String,
) -> Result<hotkey::HotkeyRegistrationStatus, error::VoxError> {
    hotkey::validate_hotkey_pair(&push_to_talk_hotkey, &toggle_dictation_hotkey)
        .map_err(error::VoxError::Config)?;
    let config_dir = app_config_dir(&app)?;
    let mut preferences = preferences::load_user_preferences(config_dir.clone());
    preferences.push_to_talk_hotkey = Some(push_to_talk_hotkey.trim().to_string());
    preferences.toggle_dictation_hotkey = Some(toggle_dictation_hotkey.trim().to_string());
    preferences::save_user_preferences(config_dir, preferences)?;
    Ok(hotkey::HotkeyRegistrationStatus::registered(push_to_talk_hotkey))
}
```

说明：这一步先保存和返回状态；真正运行时重新注册要在同一 task 后续补上，避免 command 名先缺失。

- [ ] **Step 4: 前端 client 增加调用**

在 `src/tauriClient.ts` 增加：

```ts
export async function saveHotkeyPreferences(pushToTalkHotkey: string, toggleDictationHotkey: string): Promise<HotkeyRegistrationStatus> {
  return invoke<HotkeyRegistrationStatus>('save_hotkey_preferences', { pushToTalkHotkey, toggleDictationHotkey });
}
```

- [ ] **Step 5: 验证**

```bash
cargo test --manifest-path src-tauri/Cargo.toml hotkey preferences
npm run typecheck
```

- [ ] **Step 6: 提交**

```bash
git add src-tauri/src/hotkey.rs src-tauri/src/lib.rs src/tauriClient.ts
git -c user.name="VoxType" -c user.email="maintainers@voxtype.dev" commit -m "feat: save custom hotkeys"
```

---

## Task 3: 快捷键设置 UI

**Files:**

- Create: `src/HotkeySettingsDialog.tsx`
- Modify: `src/MainWindow.tsx`
- Modify: `src/App.tsx`
- Modify: `src/App.test.tsx`
- Modify: `src/styles.css`

- [ ] **Step 1: 写前端测试**

在 `src/App.test.tsx` 增加测试：

```ts
it('opens hotkey settings from the input mode gear buttons', async () => {
  render(<App />);

  fireEvent.click(screen.getByRole('button', { name: '修改按住说话快捷键' }));

  expect(await screen.findByRole('dialog', { name: '快捷键设置' })).toBeInTheDocument();
  expect(screen.getByLabelText('按住说话快捷键')).toHaveValue('Ctrl+Alt+Space');
  expect(screen.getByLabelText('连续输入快捷键')).toHaveValue('Ctrl+Alt+V');
});
```

- [ ] **Step 2: 创建对话框组件**

`src/HotkeySettingsDialog.tsx`：

```tsx
interface HotkeySettingsDialogProps {
  pushToTalkHotkey: string;
  toggleDictationHotkey: string;
  isSaving: boolean;
  message: string | null;
  onPushToTalkHotkeyChange: (value: string) => void;
  onToggleDictationHotkeyChange: (value: string) => void;
  onSave: () => void;
  onClose: () => void;
}

export function HotkeySettingsDialog(props: HotkeySettingsDialogProps) {
  return (
    <div className="modal-backdrop" role="presentation">
      <section className="hotkey-dialog" role="dialog" aria-modal="true" aria-label="快捷键设置">
        <header><span>输入模式</span><strong>快捷键设置</strong></header>
        <label className="field"><span>按住说话快捷键</span><input value={props.pushToTalkHotkey} onChange={(event) => props.onPushToTalkHotkeyChange(event.target.value)} /></label>
        <label className="field"><span>连续输入快捷键</span><input value={props.toggleDictationHotkey} onChange={(event) => props.onToggleDictationHotkeyChange(event.target.value)} /></label>
        {props.message ? <p className="runtime-message">{props.message}</p> : null}
        <div className="button-row"><button type="button" onClick={props.onSave} disabled={props.isSaving}>{props.isSaving ? '保存中' : '保存快捷键'}</button><button type="button" onClick={props.onClose}>取消</button></div>
      </section>
    </div>
  );
}
```

- [ ] **Step 3: 接入 MainWindow 设置按钮**

给 `MainWindow` 增加 `onOpenHotkeySettings` prop，两颗设置按钮都调用它。

- [ ] **Step 4: App 管理对话框状态**

在 `App.tsx` 增加：

```ts
const [isHotkeyDialogOpen, setIsHotkeyDialogOpen] = useState(false);
const [pushToTalkHotkey, setPushToTalkHotkey] = useState('Ctrl+Alt+Space');
const [toggleDictationHotkeyInput, setToggleDictationHotkeyInput] = useState('Ctrl+Alt+V');
const [isSavingHotkeys, setIsSavingHotkeys] = useState(false);
const [hotkeySaveMessage, setHotkeySaveMessage] = useState<string | null>(null);
```

- [ ] **Step 5: 保存逻辑**

调用 `saveHotkeyPreferences`，成功后刷新 `getHotkeyStatus`，并写诊断日志。浏览器预览模式下显示“需要 Tauri 桌面环境保存快捷键”。

- [ ] **Step 6: 验证**

```bash
npm test -- --run src/App.test.tsx
npm run typecheck
```

- [ ] **Step 7: 提交**

```bash
git add src/HotkeySettingsDialog.tsx src/MainWindow.tsx src/App.tsx src/App.test.tsx src/styles.css
git -c user.name="VoxType" -c user.email="maintainers@voxtype.dev" commit -m "feat: add hotkey settings dialog"
```

---

## Task 4: MiniMax 云端配置数据结构

**Files:**

- Create: `src-tauri/src/cloud_asr_config.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/types.ts`
- Modify: `src/tauriClient.ts`

- [ ] **Step 1: Rust 配置测试**

新建 `src-tauri/src/cloud_asr_config.rs`，先写 tests：

```rust
#[test]
fn saves_status_without_exposing_api_key() {
    let dir = tempfile::tempdir().unwrap();
    let status = save_cloud_asr_config(
        dir.path().to_path_buf(),
        CloudAsrConfig {
            provider: "minimax".to_string(),
            api_key: Some("secret-key".to_string()),
            group_id: Some("group-1".to_string()),
            base_url: Some("https://api.minimax.io".to_string()),
            model: Some("speech-to-text".to_string()),
            language: Some("zh".to_string()),
        },
    )
    .unwrap();

    assert!(status.ready);
    assert_eq!(status.api_key_configured, true);
    assert_eq!(status.api_key_preview, Some("se***ey".to_string()));
}
```

- [ ] **Step 2: 实现配置结构**

结构字段：

```rust
pub struct CloudAsrConfig {
    pub provider: String,
    pub api_key: Option<String>,
    pub group_id: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub language: Option<String>,
}
```

状态字段不返回完整 API Key，只返回 `api_key_configured` 和 `api_key_preview`。

- [ ] **Step 3: command 接入**

在 `src-tauri/src/lib.rs` 增加：

```rust
#[tauri::command]
fn get_cloud_asr_config_status(app: AppHandle) -> Result<cloud_asr_config::CloudAsrConfigStatus, error::VoxError> {
    Ok(cloud_asr_config::get_cloud_asr_config_status(app_config_dir(&app)?))
}

#[tauri::command]
fn save_cloud_asr_config(app: AppHandle, config: cloud_asr_config::CloudAsrConfig) -> Result<cloud_asr_config::CloudAsrConfigStatus, error::VoxError> {
    cloud_asr_config::save_cloud_asr_config(app_config_dir(&app)?, config)
}
```

- [ ] **Step 4: TypeScript 类型和 client**

`src/types.ts` 增加 `CloudAsrConfig` 和 `CloudAsrConfigStatus`；`src/tauriClient.ts` 增加对应 invoke。

- [ ] **Step 5: 验证**

```bash
cargo test --manifest-path src-tauri/Cargo.toml cloud_asr_config
npm run typecheck
```

- [ ] **Step 6: 提交**

```bash
git add src-tauri/src/cloud_asr_config.rs src-tauri/src/lib.rs src/types.ts src/tauriClient.ts
git -c user.name="VoxType" -c user.email="maintainers@voxtype.dev" commit -m "feat: add cloud asr config"
```

---

## Task 5: MiniMax 配置 UI

**Files:**

- Modify: `src/ModelSettingsView.tsx`
- Modify: `src/App.tsx`
- Modify: `src/App.test.tsx`
- Modify: `src/styles.css`

- [ ] **Step 1: 前端测试**

增加测试：

```ts
it('shows MiniMax cloud API settings in model selection', async () => {
  render(<App />);
  fireEvent.click(screen.getByRole('button', { name: '模型选择' }));
  fireEvent.click(screen.getByRole('tab', { name: /云端 API/ }));

  expect(screen.getByLabelText('MiniMax API Key')).toBeInTheDocument();
  expect(screen.getByLabelText('MiniMax Group ID')).toBeInTheDocument();
  expect(screen.getByLabelText('MiniMax Base URL')).toBeInTheDocument();
  expect(screen.getByLabelText('MiniMax 模型')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: '保存 MiniMax 配置' })).toBeInTheDocument();
});
```

- [ ] **Step 2: 表单字段**

在云端 tab 中添加：

- `MiniMax API Key`：`type="password"`
- `MiniMax Group ID`
- `MiniMax Base URL`：默认 `https://api.minimax.io`
- `MiniMax 模型`
- `识别语言`：默认 `zh`

- [ ] **Step 3: 保存按钮**

调用 `saveCloudAsrConfig`，保存后显示脱敏状态，不在 UI 日志里打印完整 API Key。

- [ ] **Step 4: 检测按钮**

先实现“本地配置完整性检测”：API Key、Base URL、模型都存在即可显示“配置完整，等待真实 API 验证”。真实调用在 Task 6。

- [ ] **Step 5: 验证**

```bash
npm test -- --run src/App.test.tsx
npm run typecheck
```

- [ ] **Step 6: 提交**

```bash
git add src/ModelSettingsView.tsx src/App.tsx src/App.test.tsx src/styles.css
git -c user.name="VoxType" -c user.email="maintainers@voxtype.dev" commit -m "feat: configure minimax cloud asr"
```

---

## Task 6: MiniMax ASR 调用适配

**Files:**

- Create: `src-tauri/src/cloud_asr.rs`
- Modify: `src-tauri/src/asr.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: 官方 endpoint 二次确认**

实现前必须查 MiniMax 官方文档。当前搜索只确认 MiniMax 有语音 API 和官方 TTS 文档；ASR endpoint 没有直接命中官方页面。实现者需要记录：

- 官方 ASR endpoint URL。
- 认证头格式。
- 音频上传字段名。
- 支持的音频格式。
- 返回文本字段路径。

记录到 `docs/harness/research-log.md`。

- [ ] **Step 2: HTTP client 测试边界**

为 `cloud_asr.rs` 写纯函数测试：

- 构造 request 时不把 API Key 写入错误信息。
- 解析 MiniMax response 时能提取文本。
- response 缺文本时返回清晰错误。

- [ ] **Step 3: 接入非流式转写**

将当前 `RecordedAudio.asrSamples` 写成临时 WAV 或 multipart bytes，调用 MiniMax ASR endpoint，返回统一 `Transcript { text, engine: "minimax" }`。

- [ ] **Step 4: 引擎选择**

增加当前 ASR engine 配置：

- 默认 `local-whisper-cpp`。
- 用户选择云端 tab 并保存 MiniMax ready 后，可选择 `cloud-minimax`。

- [ ] **Step 5: 验证**

```bash
cargo test --manifest-path src-tauri/Cargo.toml cloud_asr
cargo check --manifest-path src-tauri/Cargo.toml --lib
npm run typecheck
```

- [ ] **Step 6: 提交**

```bash
git add src-tauri/src/cloud_asr.rs src-tauri/src/asr.rs src-tauri/src/lib.rs src-tauri/Cargo.toml docs/harness/research-log.md
git -c user.name="VoxType" -c user.email="maintainers@voxtype.dev" commit -m "feat: add minimax asr engine"
```

---

## Task 7: 文档、验证和收尾

**Files:**

- Modify: `docs/harness/progress.md`
- Modify: `docs/harness/feature_list.json`
- Modify: `docs/guide/run-and-understand.md`

- [ ] **Step 1: 更新用户文档**

在 `docs/guide/run-and-understand.md` 增加：

- 如何设置快捷键。
- 快捷键保存后需要重启还是立即生效。
- 如何配置 MiniMax。
- API Key 不会出现在日志中的说明。

- [ ] **Step 2: 更新 harness**

记录 V6 已完成项、验证命令、残余风险。

- [ ] **Step 3: 全量验证**

```bash
npm test -- --run
npm run typecheck
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo fmt --all --manifest-path src-tauri/Cargo.toml --check
python -m json.tool docs/harness/feature_list.json
git diff --check
```

- [ ] **Step 4: 隐私扫描**

扫描内容：

- 个人用户名。
- 本机路径。
- API Key 形态文本。
- `MINIMAX_API_KEY` 的真实值不能出现。

- [ ] **Step 5: 提交**

```bash
git add docs/harness/progress.md docs/harness/feature_list.json docs/guide/run-and-understand.md
git -c user.name="VoxType" -c user.email="maintainers@voxtype.dev" commit -m "docs: update v6 hotkeys and cloud asr guidance"
```

---

## V6 完成定义

- 主界面两个设置图标可点击，并能打开快捷键设置对话框。
- 两个快捷键可保存、可持久化、可在启动后用于实际全局快捷键注册。
- 重复快捷键和非法快捷键有明确错误。
- 模型选择页可配置 MiniMax 云端 API，并且不会泄露 API Key。
- 如果 MiniMax 官方 ASR endpoint 未确认，V6 至少完成配置和抽象，真实转写任务不得假装完成。
- 文档、harness、测试全部对齐。

