# VoxType Layered Testing Strategy

This document defines VoxType testing layers, automation boundaries, and maintainer handoff rules. The goal is not to replace real desktop validation. The goal is to catch major regressions with low-risk automation before they reach maintainer testing.

## Principles

- Automate frequent, repeatable, low-risk regression checks first.
- Do not chase 100 percent automation by running risky system operations.
- Test reports must separate automated checks, safe desktop smoke checks, and manual desktop validation.
- Real microphones, real global shortcuts, real target apps, and system-level audio effects remain manual or explicitly approved controlled desktop checks.
- Automated tests must not write secrets, commit audio files, leak clipboard contents, modify system settings, or operate on real user windows unless the maintainer explicitly approves a controlled target.

## Testing Layers

### 1. Unit Tests

Unit tests cover pure functions, config parsing, state machines, and local adapters. They are suitable for audio quality thresholds, text optimization rules, user preference persistence, transcript history serialization, SenseVoice stdout JSON parsing, and non-interactive insertion strategy dispatch.

They do not prove that real recording, real shortcuts, real target windows, or real ASR model execution passed.

### 2. Integration Tests

Integration tests cover frontend state flow, Tauri command boundaries, and Rust module composition. They should verify that settings persist, recognized text flows through text optimization, insertion metadata and history, ASR failures do not create fake successes, and metadata propagates correctly.

They do not prove real Windows desktop behavior passed.

### 3. Safe Desktop Regression Harness

This is a V12 supplemental direction. It should prioritize low-risk, controlled desktop automation instead of operating on the maintainer real environment.

Recommended order:

1. Fixed WAV ASR regression: use a non-private test WAV to exercise transcribeLastRecording -> text postprocess -> insertion metadata -> history. This catches empty SenseVoice text, stale readiness, and successful ASR results that fail to enter history.
2. Recorder injection test mode: under VOXTYPE_E2E=1 or an equivalent explicit test flag, inject test audio into the recorder last recording instead of opening the real microphone.
3. Controlled insertion target: verify clipboard, sendinput, and auto only against a test-owned window or temporary text field. Do not default to operating on VS Code, WeChat, browser sessions, or other user windows.
4. Tauri desktop smoke: start the desktop app and verify Settings, Diagnostics, model readiness, build metadata, and basic actions do not crash. This still does not mean real dictation passed.

Safety limits:

- Do not automatically send global shortcuts to the system.
- Do not automatically steal focus from the user real active window.
- Do not automatically operate on IM apps, personal browser pages, or elevated windows.
- Do not save real microphone recordings into the repository.
- Do not modify system configuration, drivers, registry, audio effects, or user environment variables without explicit maintainer approval.

### 4. Manual Desktop Validation

These checks require real maintainer validation, or explicitly approved controlled scripts: real microphone recording quality, push-to-talk and continuous-input global shortcuts, Dolby Voice or Windows Voice Focus behavior, real insertion into target apps, permission-boundary targets, long-running continuous input, and restart persistence.

Manual evidence should be recorded in docs/guide/desktop-compatibility-matrix.md, docs/harness/progress.md, or docs/harness/debugging-log.md.

## Minimum Checks Before Maintainer Handoff

When a change affects the dictation main path, run the applicable automated checks before handing it to the maintainer. Also state what remains unverified.

Baseline checks:

```bash
npm test -- --run
npm run typecheck
npm run build
cargo check --manifest-path src-tauri/Cargo.toml --lib
python -m json.tool docs/harness/feature_list.json
git diff --check
```

Add scope-specific checks for Rust ASR/audio/insertion logic, UI/settings/metadata, and future V13 safe desktop regression scripts.

If bash init.sh fails because of a known harness policy issue, report that exact reason. Do not present it as passed.

## Reporting Language

Use explicit labels:

- Automated checks passed: unit, integration, typecheck, build, or Rust compile checks passed.
- Safe desktop smoke passed: only a controlled desktop target or test-owned window passed.
- Manual desktop validation pending: real microphone, real shortcut, real target window, or system audio effect still needs maintainer validation.
- Not verified: the check was not run or cannot run in the current environment.

Do not use a vague tests passed statement to imply the real desktop dictation loop passed. If the real desktop loop was not validated, list the remaining manual steps.

## V12 Output And V13 Follow-up

V12 establishes the testing policy and safe automation boundaries. It does not claim that all desktop automation sentinels already exist.

V13 may implement fixed WAV, recorder injection, and controlled insertion-target checks if they reduce maintainer manual regression load without risky desktop automation.

Rules for future sentinels:

- Do not default to dangerous system automation.
- Every new regression sentinel must document its command, coverage, and non-coverage here, in docs/guide/release-checklist.md, or in V13 progress notes.
- If maintainers find the same main-path bug more than once, evaluate whether it can become a safe regression sentinel.
