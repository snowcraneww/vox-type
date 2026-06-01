# V7 Model Routing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebuild VoxType model selection so push-to-talk and toggle dictation each have their own persisted default model, while the main window readiness state and transcript history show the model actually used.

**Architecture:** Keep existing Tauri command boundaries, but replace the current cloud-provider-centered UI with mode-specific model routing. Store model choices in user preferences, keep model readiness derived from existing local ASR and Baidu config status, remove MiniMax from user-facing configuration, and leave Baidu realtime WebSocket as a V8 placeholder.

**Tech Stack:** Tauri 2, Rust, React, TypeScript, Vitest, existing VoxType preference/config modules.

---

## Scope

V7 includes:

- Main readiness panel: microphone, paste path, push-to-talk model, toggle-dictation model.
- Mode-specific default model persistence.
- Model settings page redesign with separate “default model per input mode” and “model configuration” sections.
- Remove MiniMax from user-facing model settings.
- Keep Baidu short speech API working.
- Add Baidu short speech optional `lm_id` configuration.
- Add Baidu realtime WebSocket placeholder configuration and disabled/not-implemented state.
- Add transcript record metadata for input mode, model, duration, and character count.

V7 excludes:

- Real Baidu realtime WebSocket streaming implementation.
- LLM-based correction.
- Full cleanup of historical MiniMax docs.

## File Map

- Modify `src/types.ts`: add model IDs, input mode IDs, transcript metadata, config fields.
- Modify `src-tauri/src/preferences.rs`: persist `push_to_talk_model` and `toggle_dictation_model` with defaults.
- Modify `src-tauri/src/cloud_asr_config.rs`: remove MiniMax from active user-facing config path, add Baidu `lm_id` and realtime placeholder fields.
- Modify `src-tauri/src/cloud_asr.rs`: include optional `lm_id` in Baidu short speech requests; keep realtime returning a not-implemented error.
- Modify `src-tauri/src/lib.rs`: route transcription by input mode model and return model metadata to frontend where needed.
- Modify `src/App.tsx`: load/save mode model choices, choose ASR path by input mode, enrich transcript records.
- Modify `src/MainWindow.tsx`: readiness panel and transcript row metadata.
- Modify `src/ModelSettingsView.tsx`: redesign page sections and remove MiniMax UI.
- Modify `src/styles.css`: compact segmented controls, model cards, metadata chips.
- Modify `src/App.test.tsx`: cover V7 UI, persistence calls, metadata display, MiniMax removal.
- Modify docs: `docs/harness/feature_list.json`, `docs/harness/progress.md`, `docs/harness/debugging-log.md` as implementation evidence changes.

## Task 1: TypeScript Model Routing Types

**Files:**
- Modify: `src/types.ts`
- Test: `src/App.test.tsx`

- [ ] **Step 1: Add failing UI expectations for mode-specific model labels**

Add a test that expects the main window readiness panel to show `按住说话` and `连续输入` with model names, and not show `本地识别` or `云端 API` as separate readiness labels.

```typescript
it('shows mode-specific model readiness instead of separate local and cloud readiness', async () => {
  render(<App />);

  expect(await screen.findByRole('region', { name: '准备状态' })).toBeInTheDocument();
  expect(screen.getByText('按住说话')).toBeInTheDocument();
  expect(screen.getByText('连续输入')).toBeInTheDocument();
  expect(screen.queryByText('本地识别')).not.toBeInTheDocument();
  expect(screen.queryByText('云端 API')).not.toBeInTheDocument();
});
```

Run: `npm test -- --run src/App.test.tsx`
Expected: FAIL because current readiness panel still shows local/cloud entries.

- [ ] **Step 2: Add shared frontend types**

In `src/types.ts`, add:

```typescript
export type InputModeId = 'push-to-talk' | 'toggle-dictation';
export type TranscriptionModelId = 'local-whisper' | 'baidu-short' | 'baidu-realtime';

export interface ModeModelPreferences {
  pushToTalkModel: TranscriptionModelId;
  toggleDictationModel: TranscriptionModelId;
}

export interface ModelReadiness {
  id: TranscriptionModelId;
  label: string;
  ready: boolean;
  message: string;
  availableInV7: boolean;
}
```

Extend `TranscriptRecord`:

```typescript
export interface TranscriptRecord {
  id: number;
  time: string;
  text: string;
  durationMs: number;
  inputMode: InputModeId;
  modelId: TranscriptionModelId;
  charCount: number;
}
```

If `TranscriptRecord` already exists, update its existing declaration instead of duplicating it.

Run: `npm run typecheck`
Expected: FAIL until record creation sites are updated in later tasks.

## Task 2: Persist Mode-Specific Model Preferences

**Files:**
- Modify: `src-tauri/src/preferences.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/tauriClient.ts`
- Modify: `src/App.tsx`
- Test: Rust preference tests if present, plus `npm run typecheck`

- [ ] **Step 1: Add Rust preference fields with defaults**

In `src-tauri/src/preferences.rs`, add enum/string fields to the persisted preferences structure:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TranscriptionModelId {
    LocalWhisper,
    BaiduShort,
    BaiduRealtime,
}

fn default_transcription_model() -> TranscriptionModelId {
    TranscriptionModelId::BaiduShort
}
```

Add fields:

```rust
#[serde(default = "default_transcription_model")]
pub push_to_talk_model: TranscriptionModelId,
#[serde(default = "default_transcription_model")]
pub toggle_dictation_model: TranscriptionModelId,
```

Update default preference construction so both modes default to `BaiduShort`.

- [ ] **Step 2: Add Tauri commands for mode model preferences**

In `src-tauri/src/lib.rs`, expose commands:

```rust
#[tauri::command]
fn save_mode_model_preferences(
    push_to_talk_model: preferences::TranscriptionModelId,
    toggle_dictation_model: preferences::TranscriptionModelId,
) -> Result<preferences::UserPreferences, error::VoxError> {
    let mut prefs = preferences::load_user_preferences()?;
    prefs.push_to_talk_model = push_to_talk_model;
    prefs.toggle_dictation_model = toggle_dictation_model;
    preferences::save_user_preferences(&prefs)?;
    Ok(prefs)
}
```

Also ensure existing `get_user_preferences` returns the new fields.

- [ ] **Step 3: Add frontend client methods**

In `src/tauriClient.ts`, add:

```typescript
export async function saveModeModelPreferences(pushToTalkModel: TranscriptionModelId, toggleDictationModel: TranscriptionModelId) {
  return invoke<UserPreferences>('save_mode_model_preferences', { pushToTalkModel, toggleDictationModel });
}
```

Import `TranscriptionModelId` and update `UserPreferences` type if it exists in `src/types.ts`.

Run: `npm run typecheck`
Expected: PASS after TypeScript call sites compile.

## Task 3: Model Readiness Derivation

**Files:**
- Modify: `src/App.tsx`
- Modify: `src/MainWindow.tsx`
- Test: `src/App.test.tsx`

- [ ] **Step 1: Add readiness derivation helper in `src/App.tsx` or a small new module**

Use this mapping:

```typescript
function getModelReadiness(modelId: TranscriptionModelId, asr: AsrConfigStatus, cloud: CloudAsrConfigStatus): ModelReadiness {
  if (modelId === 'local-whisper') {
    return { id: modelId, label: 'whisper.cpp', ready: asr.ready, message: asr.message, availableInV7: true };
  }
  if (modelId === 'baidu-short') {
    return { id: modelId, label: '百度短语音', ready: cloud.config.provider === 'baidu' && cloud.ready, message: cloud.message, availableInV7: true };
  }
  return { id: modelId, label: '百度实时 WebSocket', ready: false, message: 'V8 接入，当前版本不可用。', availableInV7: false };
}
```

- [ ] **Step 2: Pass mode model readiness into `MainWindow`**

Add props:

```typescript
pushToTalkModelReadiness: ModelReadiness;
toggleDictationModelReadiness: ModelReadiness;
```

- [ ] **Step 3: Replace readiness items in `src/MainWindow.tsx`**

Use exactly four readiness items:

```typescript
const readinessItems = [
  { label: '麦克风', value: recorderInfo?.deviceName ?? '等待设备', state: recorderInfo ? 'ready' : 'warning', title: recorderInfo ? `${recorderInfo.sampleRate} Hz / ${recorderInfo.channels} 声道` : '等待 Tauri 读取默认输入设备。' },
  { label: '上屏', value: 'Clipboard', state: status.phase === 'failed' ? 'error' : 'ready', title: '使用剪贴板写入文本并发送 Ctrl+V。' },
  { label: '按住说话', value: pushToTalkModelReadiness.label, state: pushToTalkModelReadiness.ready ? 'ready' : 'warning', title: pushToTalkModelReadiness.message },
  { label: '连续输入', value: toggleDictationModelReadiness.label, state: toggleDictationModelReadiness.ready ? 'ready' : 'warning', title: toggleDictationModelReadiness.message },
];
```

Run: `npm test -- --run src/App.test.tsx`
Expected: PASS for the readiness test from Task 1.

## Task 4: Redesign Model Settings Page

**Files:**
- Modify: `src/ModelSettingsView.tsx`
- Modify: `src/App.tsx`
- Modify: `src/styles.css`
- Test: `src/App.test.tsx`

- [ ] **Step 1: Add failing tests for V7 page structure**

Add tests asserting:

```typescript
expect(screen.getByText('输入模式默认模型')).toBeInTheDocument();
expect(screen.getByText('按住说话')).toBeInTheDocument();
expect(screen.getByText('连续输入')).toBeInTheDocument();
expect(screen.getAllByText('百度短语音').length).toBeGreaterThan(0);
expect(screen.getAllByText('百度实时 WebSocket').length).toBeGreaterThan(0);
expect(screen.queryByText('MiniMax')).not.toBeInTheDocument();
```

Run: `npm test -- --run src/App.test.tsx`
Expected: FAIL until page is redesigned.

- [ ] **Step 2: Replace old tab structure**

In `src/ModelSettingsView.tsx`, remove `activeModel: 'local' | 'cloud'` and replace it with independent sections:

```tsx
<section className="model-routing-section" aria-label="输入模式默认模型">
  <div className="section-heading"><span>输入模式默认模型</span><strong>分别选择</strong></div>
  <ModeModelSelector label="按住说话" value={props.pushToTalkModel} onChange={props.onPushToTalkModelChange} readiness={props.modelReadiness} />
  <ModeModelSelector label="连续输入" value={props.toggleDictationModel} onChange={props.onToggleDictationModelChange} readiness={props.modelReadiness} />
  <div className="button-row"><button type="button" onClick={props.onSaveModeModelPreferences}>保存默认模型</button></div>
</section>
```

Add a small local component in the same file:

```tsx
function ModeModelSelector({ label, value, onChange, readiness }: { label: string; value: TranscriptionModelId; onChange: (value: TranscriptionModelId) => void; readiness: Record<TranscriptionModelId, ModelReadiness> }) {
  const options: TranscriptionModelId[] = ['local-whisper', 'baidu-short', 'baidu-realtime'];
  return (
    <div className="mode-model-row">
      <span>{label}</span>
      <div className="segmented-models" role="group" aria-label={`${label}默认模型`}>
        {options.map((id) => (
          <button key={id} type="button" data-active={value === id} onClick={() => onChange(id)} title={readiness[id].message}>
            <span className="ready-dot" data-ready={readiness[id].ready} />
            {readiness[id].label}
          </button>
        ))}
      </div>
    </div>
  );
}
```

- [ ] **Step 3: Add three model configuration panels**

Keep local whisper config fields. Replace cloud provider switch with:

- `本地 whisper.cpp`
- `百度短语音`
- `百度实时 WebSocket`

The realtime panel must display `V8 接入，当前版本不可用。` and no button that claims to test real realtime recognition.

- [ ] **Step 4: Remove MiniMax user-facing controls**

Remove these labels from `ModelSettingsView.tsx`:

- `MiniMax`
- `MiniMax API Key`
- `MiniMax Group ID`
- `MiniMax Base URL`
- `MiniMax 模型`

Run: `npm test -- --run src/App.test.tsx`
Expected: PASS for the new structure tests.

## Task 5: Baidu Short Speech `lm_id`

**Files:**
- Modify: `src/types.ts`
- Modify: `src-tauri/src/cloud_asr_config.rs`
- Modify: `src-tauri/src/cloud_asr.rs`
- Modify: `src/ModelSettingsView.tsx`
- Test: Rust `cloud_asr` / `cloud_asr_config` `--no-run`, frontend typecheck

- [ ] **Step 1: Add config field**

Add optional `baiduLmId` to TypeScript and Rust cloud config structures.

TypeScript:

```typescript
baiduLmId: string | null;
```

Rust:

```rust
#[serde(default)]
pub baidu_lm_id: Option<String>,
```

- [ ] **Step 2: Add UI field**

In Baidu short speech config panel:

```tsx
<label className="field"><span>百度 ASR lm_id（可选）</span><input aria-label="百度 ASR lm_id" value={props.cloudBaiduLmId} onChange={(event) => props.onCloudBaiduLmIdChange(event.target.value)} placeholder="自训练平台模型 ID" /></label>
```

- [ ] **Step 3: Include `lm_id` in short speech requests when present**

In `src-tauri/src/cloud_asr.rs`, include `lm_id` in the request payload/query only when non-empty. Do not send an empty string.

- [ ] **Step 4: Verify**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml cloud_asr --no-run
cargo test --manifest-path src-tauri/Cargo.toml cloud_asr_config --no-run
npm run typecheck
```

Expected: all commands exit 0.

## Task 6: Route Transcription by Input Mode

**Files:**
- Modify: `src/App.tsx`
- Modify: `src-tauri/src/lib.rs` if routing is backend-side
- Test: `src/App.test.tsx`

- [ ] **Step 1: Add tests for no fallback behavior**

Add a test where push-to-talk selects `baidu-realtime` and triggering transcription shows an error that realtime is V8-only, instead of falling back to local or short speech.

Expected message:

```text
百度实时 WebSocket 将在 V8 接入，当前版本不可用于转写。
```

- [ ] **Step 2: Implement frontend routing helper**

In `src/App.tsx`, create:

```typescript
function assertModelUsable(model: ModelReadiness) {
  if (!model.availableInV7) {
    throw new Error('百度实时 WebSocket 将在 V8 接入，当前版本不可用于转写。');
  }
  if (!model.ready) {
    throw new Error(`${model.label} 未就绪：${model.message}`);
  }
}
```

Before push-to-talk final transcription, validate `pushToTalkModelReadiness`. Before toggle final transcription, validate `toggleDictationModelReadiness`.

- [ ] **Step 3: Preserve existing Baidu short speech path**

When selected model is `baidu-short`, use the current backend Baidu short speech path. When selected model is `local-whisper`, use the local whisper path. If the backend currently selects provider globally, add a command parameter or temporary preference write so the requested model is explicit.

Run: `npm test -- --run src/App.test.tsx`
Expected: PASS.

## Task 7: Transcript Record Metadata

**Files:**
- Modify: `src/App.tsx`
- Modify: `src/MainWindow.tsx`
- Modify: `src/styles.css`
- Test: `src/App.test.tsx`

- [ ] **Step 1: Update record creation**

Where transcript records are created, populate:

```typescript
{
  inputMode: 'push-to-talk',
  modelId: pushToTalkModel,
  durationMs,
  charCount: text.length,
}
```

For toggle dictation final merged record:

```typescript
{
  inputMode: 'toggle-dictation',
  modelId: toggleDictationModel,
  durationMs,
  charCount: mergedText.length,
}
```

- [ ] **Step 2: Render metadata line**

In `src/MainWindow.tsx`, under record text:

```tsx
<div className="record-meta">
  <span>{record.inputMode === 'push-to-talk' ? '按住说话' : '连续输入'}</span>
  <span>{formatModelLabel(record.modelId)}</span>
  <span>{formatDuration(record.durationMs)}</span>
  <span>{record.charCount} 字</span>
</div>
```

Add `formatModelLabel`:

```typescript
function formatModelLabel(modelId: TranscriptionModelId) {
  if (modelId === 'local-whisper') return 'whisper.cpp';
  if (modelId === 'baidu-short') return '百度短语音';
  return '百度实时 WebSocket';
}
```

- [ ] **Step 3: Add CSS**

In `src/styles.css`:

```css
.record-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  color: rgba(31, 61, 45, 0.68);
  font-size: 11px;
  line-height: 1.3;
}

.record-meta span {
  border: 1px solid rgba(52, 199, 89, 0.2);
  border-radius: 999px;
  padding: 2px 7px;
  background: rgba(255, 255, 255, 0.58);
}
```

Run: `npm test -- --run src/App.test.tsx`
Expected: PASS and metadata appears in tests.

## Task 8: Documentation and Harness Updates

**Files:**
- Modify: `docs/harness/feature_list.json`
- Modify: `docs/harness/progress.md`
- Modify: `docs/harness/debugging-log.md` if any bug is found
- Modify: `docs/integrations/baidu-asr.md`

- [ ] **Step 1: Update feature status to in_progress when implementation starts**

Set `v7-001.status` to `in_progress` and update its user-visible behavior to remove MiniMax and mention mode-specific default models.

- [ ] **Step 2: Update Baidu integration doc**

Add a section explaining:

- short speech endpoint remains `http://vop.baidu.com/server_api`
- realtime WebSocket endpoint is `wss://vop.baidu.com/realtime_asr`
- realtime is V8 scope
- optional `lm_id` is for Baidu self-trained short speech model customization

- [ ] **Step 3: Final verification**

Run:

```bash
npm test -- --run src/App.test.tsx
npm run typecheck
cargo check --manifest-path src-tauri/Cargo.toml --lib
cargo test --manifest-path src-tauri/Cargo.toml cloud_asr --no-run
cargo test --manifest-path src-tauri/Cargo.toml cloud_asr_config --no-run
python -m json.tool docs/harness/feature_list.json
git diff --check
```

Expected: all commands exit 0, except full Rust test execution remains subject to the existing Windows `STATUS_ENTRYPOINT_NOT_FOUND` issue if run without `--no-run`.

## Self-Review

- Spec coverage: The plan covers mode-specific model selection, readiness panel changes, MiniMax removal, Baidu short speech continuity, Baidu realtime V8 placeholder, transcript metadata, and `lm_id`.
- Placeholder scan: No TBD/TODO placeholders remain; V8 scope is explicitly excluded from V7 implementation.
- Type consistency: `TranscriptionModelId`, `InputModeId`, `ModelReadiness`, `pushToTalkModel`, and `toggleDictationModel` are used consistently across tasks.
