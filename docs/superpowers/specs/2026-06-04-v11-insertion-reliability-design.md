# V11 Insertion Reliability Design

## Context

V10 is closed: audio enhancement, metadata, SenseVoice fallback, and the unified Settings surface are accepted. The next repeated product risk is insertion reliability. VoxType still uses clipboard paste as the main insertion path. That path is simple and verified, but it can overwrite the user clipboard, miss the target when focus changes, and behave differently across applications.

Existing project docs already define the insertion roadmap as Clipboard -> SendInput Unicode -> TSF. V11 implements the middle step only. TSF remains out of scope because it requires deeper Windows text-service integration and installation behavior.

## Goal

Add a Windows-first direct Unicode insertion path using `SendInput(KEYEVENTF_UNICODE)` while keeping clipboard insertion as a fallback. Users should be able to select the insertion strategy, understand which strategy was used for each insertion, and diagnose failures without losing the existing reliable clipboard path.

## Non-Goals

- Do not implement TSF or a full IME.
- Do not remove clipboard insertion.
- Do not change ASR model routing, audio preprocessing, or transcript post-processing.
- Do not add cloud sync or cross-device state.

## Product Behavior

V11 adds an insertion strategy setting with conservative options:

- `Auto`: try SendInput first on Windows; fallback to clipboard when direct insertion reports an unsupported or failed path.
- `SendInput`: use direct Unicode insertion and show a clear error if it fails.
- `Clipboard`: keep the current behavior.

The default should stay `Clipboard` until real desktop validation proves Auto is safe enough. This avoids surprising users who already rely on the existing path.

Each successful transcript record should preserve insertion metadata: requested strategy, actual strategy, fallback status, and failure reason when applicable. Diagnostics should log the same data.

## Architecture

Rust owns the insertion implementation because `SendInput` is a Windows API. The existing clipboard insertion command remains. A new insertion adapter should expose a strategy-aware command such as `insert_text(text, strategy)` or equivalent local naming that fits the existing `tauriClient` wrapper. Frontend insertion callers should route through one helper so push-to-talk, continuous input final chunks, reinsert actions, and diagnostics use the same fallback policy.

## Error Handling

Direct insertion should return structured errors, not raw Win32 messages. Minimum categories:

- `unsupported_platform` for non-Windows or unavailable API.
- `empty_text` for empty input.
- `sendinput_failed` when the Windows call reports failure.
- `fallback_used` metadata when Auto succeeds through clipboard after SendInput fails.

Diagnostics must not log private transcript text beyond the existing transcript behavior. They may log text length, strategy, and error category.

## Verification

Automated verification should cover strategy selection, fallback metadata, empty text, and Windows adapter command wiring. Real desktop verification must include VS Code, Notepad, browser text fields, and one non-standard or elevated target if available.

## Open Questions For Implementation

- Whether Auto should become default in a later V11.x after manual evidence.
- Whether clipboard restoration should be revisited after SendInput exists.
- Which apps reject SendInput due to privilege or text input stack behavior.
