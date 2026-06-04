# Desktop Compatibility Matrix

V12 uses this matrix to verify insertion behavior across real Windows targets after V11 adds strategy selection.

## How To Test

Run the desktop app with:

```bash
npm run tauri -- dev
```

For every target app below:

1. Open Settings -> Input -> Insertion strategy.
2. Select `Clipboard`, use `Ctrl+Alt+Space` and `Ctrl+Alt+V`, and confirm text appears in the focused target.
3. Select `SendInput`, repeat both shortcuts, and confirm text appears without depending on the clipboard paste path.
4. Select `Auto`, repeat both shortcuts, and check transcript metadata plus diagnostics for actual strategy and fallback.
5. Record failures exactly: no insertion, wrong target, mojibake, duplicate text, focus lost, permission boundary, or fallback used.

## Matrix

| Target | Target type | Clipboard | SendInput | Auto | Caveats | Evidence date |
| --- | --- | --- | --- | --- | --- | --- |
| VS Code editor | Electron text editor | Pending maintainer desktop test | Pending maintainer desktop test | Pending maintainer desktop test | Primary daily target. Check both normal editor and terminal input. | Pending |
| Notepad | Native Win32 text box | Pending maintainer desktop test | Pending maintainer desktop test | Pending maintainer desktop test | Baseline native target for SendInput Unicode. | Pending |
| Browser text field | Chromium/Edge web input | Pending maintainer desktop test | Pending maintainer desktop test | Pending maintainer desktop test | Test address bar and a normal web textarea separately if possible. | Pending |
| IME-heavy chat target | WeChat or similar chat input | Pending maintainer desktop test | Pending maintainer desktop test | Pending maintainer desktop test | Important for Chinese IME composition and focus behavior. | Pending |
| Elevated target | Admin app or permission-boundary window | Pending maintainer desktop test | Pending maintainer desktop test | Pending maintainer desktop test | Expected to expose Windows integrity-level limits; do not bypass UAC. | Pending |

## TSF Decision Gate

V12 does not implement TSF. After this matrix has real desktop evidence:

- If `SendInput` or `Auto` works reliably in the first four targets, keep TSF out of the next public release.
- If chat/IME or elevated targets fail in ways that clipboard fallback cannot cover, plan V13 as a TSF feasibility spike.
- If only elevated targets fail, document it as a Windows permission boundary rather than a product blocker.
