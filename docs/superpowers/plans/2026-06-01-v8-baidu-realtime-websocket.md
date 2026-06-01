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

- [ ] Add `src-tauri/src/baidu_realtime.rs` with `BaiduRealtimeStartFrame`, `BaiduRealtimeControlFrame`, and `BaiduRealtimeEvent`.
- [ ] Write unit tests for official START JSON serialization. Expected top-level field: `type: "START"`. Expected nested `data` fields: `appid`, `appkey`, `dev_pid`, optional `lm_id`, `cuid`, optional `user`, `format`, and `sample`.
- [ ] Write unit tests for FINISH, CANCEL, and HEARTBEAT frames.
- [ ] Write parser tests for success partial/final JSON using fixture strings copied from official docs or sanitized captured responses.
- [ ] Write parser tests for Baidu error JSON and malformed JSON.
- [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml baidu_realtime --no-run` and keep it compiling.

## Task 2: PCM Frame Chunking

- [ ] Add a pure function `pcm_frames_160ms(samples: &[i16]) -> Vec<Vec<u8>>`.
- [ ] Test that 2560 i16 samples produce one 5120-byte little-endian frame.
- [ ] Test that partial tails are buffered instead of sent too early.
- [ ] Add a small `PcmFrameBuffer` that accepts new samples and drains full 160 ms frames.
- [ ] Run Rust no-run tests for `baidu_realtime`.

## Task 3: Realtime Session Manager

- [ ] Add `BaiduRealtimeSessionManager` with states `Idle`, `Starting`, `Streaming`, `Finishing`, `Failed`.
- [ ] Store one active session only; duplicate start returns a config/runtime error.
- [ ] Read Baidu realtime config from `CloudAsrConfigStatus`; require configured AppID and `BAIDU_ASR_API_KEY`.
- [ ] Ensure no error string includes API Key or Secret Key.
- [ ] Add command-level tests for missing key, duplicate start, and cancel cleanup where possible without network.

## Task 4: WebSocket Worker

- [ ] Add the selected WebSocket dependency to `src-tauri/Cargo.toml`.
- [ ] Implement worker startup: open `wss://vop.baidu.com/realtime_asr`, send the official START JSON frame, then stream 5120-byte PCM data frames exactly as specified by the official Sending Data Frame section.
- [ ] Poll `RecorderManager::active_asr_samples_from` from the last source cursor and feed `PcmFrameBuffer`.
- [ ] Parse server messages and emit `voxtype-baidu-realtime-result` events.
- [ ] Implement bounded finish: send FINISH, drain final response, close socket, stop recording, return summary.
- [ ] Implement cancel: send CANCEL if possible, close socket, stop recording.

## Task 5: Frontend Client And Types

- [ ] Add TypeScript types: `BaiduRealtimeResultEvent`, `BaiduRealtimeSessionStatus`, `BaiduRealtimeSessionSummary`.
- [ ] Add `startBaiduRealtimeSession`, `finishBaiduRealtimeSession`, `cancelBaiduRealtimeSession`, and `listenToBaiduRealtimeResults` in `src/tauriClient.ts`.
- [ ] Add Vitest mocks for the new commands and listener.
- [ ] Run `npm run typecheck`.

## Task 6: Continuous Input Routing

- [ ] In `App.tsx`, if `toggleDictationModel === 'baidu-realtime'`, use realtime start/finish commands instead of local chunk transcription or short speech final transcription.
- [ ] On realtime partial events, update transient status only.
- [ ] On realtime final events, insert stable text with the existing clipboard insertion path.
- [ ] On finish, add one transcript history record with input mode `toggle-dictation`, model `baidu-realtime`, total duration, and final character count.
- [ ] Preserve existing behavior for `local-whisper` and `baidu-short`.

## Task 7: Tests And Regression Coverage

- [ ] Add React test: selecting continuous input Baidu Realtime WebSocket API starts realtime session on first toggle hotkey press.
- [ ] Add React test: realtime final event inserts text and final summary creates one merged record.
- [ ] Add React test: push-to-talk with Baidu Realtime WebSocket API still shows a clear unsupported error.
- [ ] Run `npm test -- --run src/App.test.tsx`.
- [ ] Run `npm run typecheck` and `npm run build`.

## Task 8: Docs, Harness, And Verification

- [ ] Update `docs/integrations/baidu-asr.md` with implemented realtime details and residual risks.
- [ ] Update `docs/harness/debugging-log.md` for any bugs found during implementation.
- [ ] Update `docs/harness/progress.md` and `docs/harness/feature_list.json` with verification evidence.
- [ ] Run full V8 verification commands from the design doc.
- [ ] Do not mark `v8-001` passing until automated verification is recorded and maintainer manual streaming test is either complete or explicitly deferred with evidence.