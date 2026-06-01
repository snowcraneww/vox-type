# V8 Baidu Realtime WebSocket API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement real Baidu Realtime WebSocket API transcription for continuous input mode.

**Architecture:** Add a Rust realtime ASR session manager that owns the WebSocket lifecycle and reads 16 kHz PCM chunks from the existing recorder. React starts/stops the session through Tauri commands and listens for partial/final result events.

**Tech Stack:** Tauri 2, Rust, React, TypeScript, Vitest, `reqwest` for existing HTTP auth, a Rust WebSocket crate selected during implementation, existing `RecorderManager` and Baidu cloud config.

---

## File Map

- Create: `src-tauri/src/baidu_realtime.rs` for protocol frames, response parser, session state, and worker boundary.
- Modify: `src-tauri/Cargo.toml` to add a WebSocket client dependency compatible with blocking/background-thread use and rustls.
- Modify: `src-tauri/src/lib.rs` to register realtime session state and Tauri commands.
- Modify: `src-tauri/src/recorder.rs` only if current chunk polling is insufficient for realtime streaming.
- Modify: `src/tauriClient.ts` to add commands and event listener types.
- Modify: `src/types.ts` to add realtime event/status/summary types.
- Modify: `src/App.tsx` to route continuous input with `baidu-realtime` through the new session commands.
- Modify: `src/MainWindow.tsx` only if partial text/status needs display refinements.
- Modify: `src/App.test.tsx` for continuous input realtime routing and transcript metadata.
- Modify: `docs/integrations/baidu-asr.md`, `docs/harness/progress.md`, `docs/harness/debugging-log.md`, `docs/harness/feature_list.json`.

## Task 1: Protocol Types And Parsers

- [x] Add `src-tauri/src/baidu_realtime.rs` with `BaiduRealtimeStartFrame`, `BaiduRealtimeControlFrame`, and `BaiduRealtimeEvent`.
- [x] Write unit tests for official START JSON serialization. Expected top-level field: `type: "START"`. Expected nested `data` fields: `appid`, `appkey`, `dev_pid`, optional `lm_id`, `cuid`, optional `user`, `format`, and `sample`.
- [x] Write unit tests for FINISH, CANCEL, and HEARTBEAT frames.
- [x] Write parser tests for success partial/final JSON using official frame names and sanitized response shapes.
- [x] Write parser tests for Baidu error JSON and malformed JSON.
- [x] Run `cargo test --manifest-path src-tauri/Cargo.toml baidu_realtime --no-run` and keep it compiling.

## Task 2: PCM Frame Chunking

- [x] Add PCM chunking helpers via `PcmFrameBuffer` and `pcm_i16_le_bytes` for 160 ms frames.
- [x] Test that 2560 i16 samples produce one 5120-byte little-endian frame.
- [x] Test that partial tails are buffered instead of sent too early.
- [x] Add a small `PcmFrameBuffer` that accepts new samples and drains full 160 ms frames.
- [x] Run Rust no-run tests for `baidu_realtime`.

## Task 3: Realtime Session Manager

- [x] Add `BaiduRealtimeSessionManager` with session status strings for idle, streaming, finished, and failed states.
- [x] Store one active session only; duplicate start returns a runtime error.
- [x] Read Baidu realtime config from `CloudAsrConfigStatus`; require configured AppID and `BAIDU_ASR_API_KEY`.
- [x] Ensure no config error string includes API Key or Secret Key.
- [x] Add no-network tests for realtime config requirements; live session start/finish remains covered by frontend command mocks and Rust compile checks.

## Task 4: WebSocket Worker

- [x] Add `tungstenite` WebSocket dependency to `src-tauri/Cargo.toml`.
- [x] Implement worker startup: open `wss://vop.baidu.com/realtime_asr`, send the official START JSON frame, then stream 5120-byte PCM data frames as specified by the official Sending Data Frame section.
- [x] Poll `RecorderManager::active_asr_samples_from` from the last source cursor and feed `PcmFrameBuffer`.
- [x] Parse server messages and emit `voxtype-baidu-realtime-result` events.
- [x] Implement bounded finish: drain available complete PCM frames, send FINISH, drain final response, close socket, stop recording, return summary.
- [x] Implement cancel: send CANCEL if possible, close socket, stop recording.

## Task 5: Frontend Client And Types

- [x] Add TypeScript types: `BaiduRealtimeResultEvent`, `BaiduRealtimeSessionStatus`, `BaiduRealtimeSessionSummary`.
- [x] Add `startBaiduRealtimeSession`, `finishBaiduRealtimeSession`, `cancelBaiduRealtimeSession`, and `listenToBaiduRealtimeResults` in `src/tauriClient.ts`.
- [x] Add Vitest mocks for the new commands and listener.
- [x] Run `npm run typecheck`.

## Task 6: Continuous Input Routing

- [x] In `App.tsx`, if `toggleDictationModel === 'baidu-realtime'`, use realtime start/finish commands instead of local chunk transcription or short speech final transcription.
- [x] On realtime partial events, update transient status only.
- [x] On realtime final events, insert stable text with the existing clipboard insertion path.
- [x] On finish, add one transcript history record with input mode `toggle-dictation`, model `baidu-realtime`, total duration, and final character count.
- [x] Preserve existing behavior for `local-whisper` and `baidu-short`.

## Task 7: Tests And Regression Coverage

- [x] Add React test: selecting continuous input Baidu Realtime WebSocket API starts realtime session on first toggle hotkey press.
- [x] Add React test: realtime final event inserts text and final summary creates one merged record.
- [x] Add React test: push-to-talk with Baidu Realtime WebSocket API still shows a clear unsupported error.
- [x] Run `npm test -- --run src/App.test.tsx`.
- [x] Run `npm run typecheck` and `npm run build`.

## Task 8: Docs, Harness, And Verification

- [x] Update `docs/integrations/baidu-asr.md` with implemented realtime details and residual risks.
- [x] Update `docs/harness/debugging-log.md` for bugs found during implementation.
- [x] Update `docs/harness/progress.md` and `docs/harness/feature_list.json` with verification evidence.
- [x] Run full V8 automated verification commands from the design doc.
- [x] Keep `v8-001` in progress until maintainer manual streaming verification is complete; automated verification is recorded below.

## Automated Verification Evidence

- 2026-06-01: `npm test -- --run src/App.test.tsx` passed, 20 tests.
- 2026-06-01: `npm run typecheck` passed.
- 2026-06-01: `npm run build` passed.
- 2026-06-01: `cargo fmt --all --manifest-path src-tauri/Cargo.toml --check` passed after formatting.
- 2026-06-01: `cargo check --manifest-path src-tauri/Cargo.toml --lib` passed.
- 2026-06-01: `cargo test --manifest-path src-tauri/Cargo.toml baidu_realtime --no-run` passed.
- 2026-06-01: `cargo test --manifest-path src-tauri/Cargo.toml cloud_asr_config --no-run` passed.

## Manual Verification Status

Maintainer manual streaming verification is still required in the Tauri desktop app with real Baidu credentials. Until that is done, `v8-001` remains `in_progress` rather than `passing`.
