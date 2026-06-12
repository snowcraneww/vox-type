# V4 Native Overlay Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 消除 Windows 桌面语音浮窗黑色胶囊外侧的 Tauri/WebView2 白色或灰色矩形边框。

**Architecture:** 先用低风险 Tauri 配置和窗口 API 补丁验证 `WebviewWindow` 是否能达标；如果不能，再增加 Windows-only 原生 layered overlay backend。所有入口继续走 `overlay::show_dictation_overlay` / `overlay::hide_dictation_overlay` facade，失败时回退 WebView overlay。

**Tech Stack:** Rust、Tauri 2、Windows Win32 layered window、React、TypeScript、Vitest、Cargo tests。

---

## File Structure

- Modify: `src-tauri/tauri.conf.json`
  - 给 `dictation-overlay` 增加 `shadow: false`，保持现有 WebView overlay 作为第一层方案。
- Modify: `src-tauri/src/overlay.rs`
  - 保留统一 facade，增加 backend 状态结构和 WebView shadowless 显示逻辑。
- Create: `src-tauri/src/native_overlay.rs`
  - Windows-only 原生 overlay 原型；非 Windows 提供 no-op fallback。
- Modify: `src-tauri/src/lib.rs`
  - 注册新 command，例如 `get_overlay_backend_status`，让前端诊断模式可见。
- Modify: `src-tauri/src/overlay_tests.rs`
  - 锁定 overlay 尺寸、配置要求、backend fallback 行为。
- Modify: `src/tauriClient.ts`
  - 暴露 overlay backend 状态读取函数和类型。
- Modify: `src/App.tsx`
  - 诊断模式显示当前 overlay backend 和最近错误。
- Modify: `src/App.test.tsx`
  - 覆盖诊断模式读取 overlay backend 状态。
- Modify: `docs/harness/debugging-log.md`
  - 记录 Tauri shadowless 实验、原生 overlay 实验和人工验证结果。
- Modify: `docs/harness/progress.md`
  - 记录 V4 进度。
- Modify: `docs/harness/feature_list.json`
  - 新增或更新 `v4-001` 状态和证据。

---

### Task 1: Tauri Shadowless WebView Overlay

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/src/overlay_tests.rs`
- Modify: `docs/harness/debugging-log.md`

- [ ] **Step 1: Add a failing config test**

Add a Rust test that reads `src-tauri/tauri.conf.json` and asserts the `dictation-overlay` window has `shadow: false`.

```rust
#[test]
fn overlay_window_disables_shadow_in_tauri_config() {
    let config = std::fs::read_to_string("tauri.conf.json").expect("tauri config should exist");
    let json: serde_json::Value = serde_json::from_str(&config).expect("tauri config should be json");
    let windows = json["app"]["windows"].as_array().expect("windows should be an array");
    let overlay = windows
        .iter()
        .find(|window| window["label"] == "dictation-overlay")
        .expect("dictation-overlay window should exist");

    assert_eq!(overlay["transparent"], true);
    assert_eq!(overlay["decorations"], false);
    assert_eq!(overlay["shadow"], false);
}
```

- [ ] **Step 2: Run the failing test**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml overlay_window_disables_shadow_in_tauri_config
```

Expected: FAIL because `shadow` is missing.

- [ ] **Step 3: Add `shadow: false`**

Update only the `dictation-overlay` window in `src-tauri/tauri.conf.json`:

```json
"decorations": false,
"transparent": true,
"shadow": false,
"alwaysOnTop": true
```

- [ ] **Step 4: Verify the test passes**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml overlay_window_disables_shadow_in_tauri_config
```

Expected: PASS.

- [ ] **Step 5: Manual visual check**

Run:

```bash
npm run tauri -- dev
```

Then open diagnostic mode and click `测试桌面浮窗`.

Expected for success: bottom overlay only shows the black capsule and wave, with no white/gray rectangle outside.

If success, skip Task 3 native implementation and finish with docs/tests.

- [ ] **Step 6: Commit if shadowless solves it**

```bash
git add src-tauri/tauri.conf.json src-tauri/src/overlay_tests.rs docs/harness/debugging-log.md
git commit -m "fix: disable desktop overlay shadow"
```

---

### Task 2: Overlay Backend Status Facade

**Files:**
- Modify: `src-tauri/src/overlay.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/tauriClient.ts`
- Modify: `src/App.tsx`
- Modify: `src/App.test.tsx`

- [ ] **Step 1: Add Rust status type**

In `src-tauri/src/overlay.rs`, add:

```rust
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OverlayBackendStatus {
    pub backend: String,
    pub last_error: Option<String>,
}

pub fn backend_status() -> OverlayBackendStatus {
    OverlayBackendStatus {
        backend: "webview-shadowless".to_string(),
        last_error: None,
    }
}
```

- [ ] **Step 2: Add Tauri command**

In `src-tauri/src/lib.rs`, add:

```rust
#[tauri::command]
fn get_overlay_backend_status() -> overlay::OverlayBackendStatus {
    overlay::backend_status()
}
```

Register it in `tauri::generate_handler!`.

- [ ] **Step 3: Add frontend client**

In `src/tauriClient.ts`, add:

```typescript
export interface OverlayBackendStatus {
  backend: string;
  lastError: string | null;
}

export async function getOverlayBackendStatus(): Promise<OverlayBackendStatus> {
  return invoke<OverlayBackendStatus>('get_overlay_backend_status');
}
```

- [ ] **Step 4: Show backend in diagnostics**

In `src/App.tsx`, load the status with other startup diagnostics and display it in the diagnostic runtime panel as `桌面浮窗后端`.

Expected visible values:

- `webview-shadowless`
- `native-win32`
- `fallback-webview`

- [ ] **Step 5: Test frontend status display**

In `src/App.test.tsx`, mock `getOverlayBackendStatus`:

```typescript
getOverlayBackendStatus: vi.fn().mockResolvedValue({ backend: 'webview-shadowless', lastError: null }),
```

Add assertion in the existing render test:

```typescript
expect(await screen.findByText('webview-shadowless')).toBeInTheDocument();
```

- [ ] **Step 6: Run tests**

```bash
npm test -- --run src/App.test.tsx
cargo test --manifest-path src-tauri/Cargo.toml overlay
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/overlay.rs src-tauri/src/lib.rs src/tauriClient.ts src/App.tsx src/App.test.tsx
git commit -m "feat: expose overlay backend status"
```

---

### Task 3: Windows Native Overlay Prototype

Only do this task if Task 1 manual visual check still shows a white/gray frame.

**Files:**
- Create: `src-tauri/src/native_overlay.rs`
- Modify: `src-tauri/src/overlay.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/overlay_tests.rs`

- [ ] **Step 1: Add Windows dependency features**

In `src-tauri/Cargo.toml`, add explicit `windows` dependency under `target.'cfg(windows)'.dependencies` if the existing transitive dependency is not enough:

```toml
windows = { version = "0.61", features = [
  "Win32_Foundation",
  "Win32_Graphics_Gdi",
  "Win32_UI_WindowsAndMessaging",
] }
```

- [ ] **Step 2: Create native overlay module skeleton**

Create `src-tauri/src/native_overlay.rs`:

```rust
use crate::error::VoxError;

#[cfg(windows)]
pub fn show_native_overlay() -> Result<(), VoxError> {
    Err(VoxError::Config("native overlay is disabled until the Windows renderer is enabled".to_string()))
}

#[cfg(windows)]
pub fn hide_native_overlay() -> Result<(), VoxError> {
    Ok(())
}

#[cfg(not(windows))]
pub fn show_native_overlay() -> Result<(), VoxError> {
    Err(VoxError::Config("native overlay is only available on Windows".to_string()))
}

#[cfg(not(windows))]
pub fn hide_native_overlay() -> Result<(), VoxError> {
    Ok(())
}
```

Add `pub mod native_overlay;` in `src-tauri/src/lib.rs`.

- [ ] **Step 3: Route through fallback**

In `overlay.rs`, make `show_dictation_overlay` try native first only when a local constant is enabled:

```rust
const PREFER_NATIVE_OVERLAY: bool = true;
```

If `native_overlay::show_native_overlay()` returns error, call the existing WebView show logic and record `fallback-webview` in status.

- [ ] **Step 4: Replace skeleton with minimal Win32 implementation**

Implement a tiny `WS_POPUP` + `WS_EX_LAYERED` + `WS_EX_TOOLWINDOW` + `WS_EX_TOPMOST` + `WS_EX_NOACTIVATE` window. Draw a black rounded rectangle and simplified colored bars using GDI first; Direct2D can wait until after the white-frame problem is solved.

The first implementation may use a static visual; animation is not required until the no-frame window is verified.

- [ ] **Step 5: Manual native visual check**

Run:

```bash
npm run tauri -- dev
```

Expected:

- Diagnostic status shows `native-win32`.
- `测试桌面浮窗` shows the native capsule with no white/gray outer rectangle.
- Recording hotkeys still show/hide overlay and do not steal focus.

- [ ] **Step 6: Add fallback test**

Add a testable helper in `overlay.rs` that maps backend result to status:

```rust
pub fn resolve_overlay_backend(native_result: Result<(), String>) -> OverlayBackendStatus {
    match native_result {
        Ok(()) => OverlayBackendStatus { backend: "native-win32".to_string(), last_error: None },
        Err(error) => OverlayBackendStatus { backend: "fallback-webview".to_string(), last_error: Some(error) },
    }
}
```

Test both branches.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/native_overlay.rs src-tauri/src/overlay.rs src-tauri/src/lib.rs src-tauri/src/overlay_tests.rs
git commit -m "feat: prototype native windows overlay"
```

---

### Task 4: Documentation And Harness Alignment

**Files:**
- Modify: `docs/harness/debugging-log.md`
- Modify: `docs/harness/progress.md`
- Modify: `docs/harness/feature_list.json`
- Modify: `docs/harness/session-handoff.md`
- Modify: `docs/guide/run-and-understand.md`

- [ ] **Step 1: Update debugging log**

Add a section with:

- observed white/gray frame behavior,
- whether `shadow: false` solved it,
- whether native overlay was required,
- manual verification result,
- remaining platform risks.

- [ ] **Step 2: Update feature list**

Set `v4-001` to:

- `in_progress` while implementing,
- `passing` only after automatic verification and maintainer visual check.

- [ ] **Step 3: Update run guide**

Add a small diagnostic subsection:

```markdown
### 验证桌面浮窗后端

1. 启动 `npm run tauri -- dev`。
2. 进入诊断模式。
3. 查看 `桌面浮窗后端`。
4. 点击 `测试桌面浮窗`。
5. 通过标准：底部只看到黑色胶囊和五彩声波，看不到外层白/灰矩形。
```

- [ ] **Step 4: Verify docs**

```bash
python -m json.tool docs/harness/feature_list.json
git diff --check
```

Expected: both pass.

- [ ] **Step 5: Commit**

```bash
git add docs/harness/debugging-log.md docs/harness/progress.md docs/harness/feature_list.json docs/harness/session-handoff.md docs/guide/run-and-understand.md
git commit -m "docs: align native overlay harness"
```

---

### Task 5: Final Verification

**Files:**
- No new edits unless verification exposes bugs.

- [ ] **Step 1: Run full automatic verification**

```bash
npm test -- --run
npm run typecheck
cargo test --manifest-path src-tauri/Cargo.toml
cargo fmt --all --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
npm run build
python -m json.tool docs/harness/feature_list.json
git diff --check
```

Expected: all commands exit 0.

- [ ] **Step 2: Run privacy scan**

Use the project privacy scan checklist from `AGENTS.md` and `docs/harness/working-agreement.md`.

Expected: no tracked-file matches for secret assignments, maintainer names, account names, email addresses, phone numbers, local usernames, absolute home-directory paths, or secret values. Do not write private strings into this plan while documenting the scan.

- [ ] **Step 3: Manual acceptance request**

Ask maintainer to verify:

- `Ctrl+Alt+Space` overlay appears without white/gray frame.
- `Ctrl+Alt+V` overlay appears without white/gray frame.
- Text still transcribes and inserts.

- [ ] **Step 4: Mark `v4-001` passing after manual acceptance**

Only after maintainer confirms visual success, update `docs/harness/feature_list.json` and `docs/harness/progress.md`, then commit:

```bash
git add docs/harness/feature_list.json docs/harness/progress.md docs/harness/session-handoff.md
git commit -m "docs: mark native overlay accepted"
```
