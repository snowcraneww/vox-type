# V10 Audio Preprocessing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an explicit, conservative audio preprocessing pipeline that can improve quiet, distant, noisy, or silence-heavy recordings before ASR.

**Architecture:** Rust owns audio preprocessing and settings persistence. The pipeline runs after recording produces 16 kHz ASR samples and before stop-then-transcribe ASR adapters consume them. React exposes a compact settings UI and before/after diagnostics. Realtime WebSocket preprocessing remains disabled until low-latency behavior is separately validated.

**Tech Stack:** Tauri 2, Rust DSP helpers, React, TypeScript, Vitest, existing recorder/ASR sample path, V9 audio diagnostics.

---

## File Map

- Create: `src-tauri/src/audio_preprocess.rs` for preprocessing settings, pipeline, DC removal, high-pass filter, normalization, VAD trim, and tests.
- Modify: `src-tauri/src/audio_quality.rs` to reuse diagnostics before and after preprocessing.
- Modify: `src-tauri/src/preferences.rs` or the existing config module to persist audio enhancement settings.
- Modify: `src-tauri/src/lib.rs` to register settings commands and route stop-then-transcribe samples through the optional pipeline.
- Modify: `src/tauriClient.ts` to add audio enhancement settings commands.
- Modify: `src/types.ts` to add audio enhancement config and before/after diagnostic types.
- Modify: `src/ModelSettingsView.tsx` to turn the old model selection/configuration page into a unified Settings page with four tabs: model selection, model configuration, audio enhancement, and text optimization.
- Modify: `src/TextOptimizationView.tsx` so text optimization can render as a panel inside Settings while preserving the standalone view if still needed.
- Modify: `src/App.tsx` to render preprocessing metadata in transcript rows when present and to pass text optimization content into Settings.
- Modify: `src/App.test.tsx` for settings and transcript metadata tests.
- Modify: `docs/harness/progress.md`, `docs/harness/debugging-log.md`, `docs/harness/feature_list.json`.

## Task 1: Audio Preprocessing Config

- [ ] Define `AudioPreprocessConfig` in `src-tauri/src/audio_preprocess.rs` with fields: `enabled`, `remove_dc_offset`, `high_pass_enabled`, `normalize_enabled`, `vad_trim_enabled`, `denoise_enabled`.
- [ ] Default to `enabled: false`, `remove_dc_offset: true`, `high_pass_enabled: false`, `normalize_enabled: true`, `vad_trim_enabled: true`, `denoise_enabled: false`.
- [ ] Add config persistence commands: `get_audio_preprocess_config`, `save_audio_preprocess_config`.
- [ ] Run: `cargo check --manifest-path src-tauri/Cargo.toml --lib`.

## Task 2: DC Offset Removal

- [ ] Write tests with synthetic samples whose mean is non-zero.
- [ ] Implement `remove_dc_offset(samples: &mut [f32])`.
- [ ] Assert the output mean is close to zero without changing duration.
- [ ] Run: `cargo test --manifest-path src-tauri/Cargo.toml audio_preprocess --no-run`.

## Task 3: High-Pass Filter

- [ ] Write tests for a low-frequency synthetic signal and a speech-band synthetic signal.
- [ ] Implement a simple first-order high-pass filter around 80 Hz.
- [ ] Assert low-frequency energy is reduced more than speech-band energy.
- [ ] Run: `cargo test --manifest-path src-tauri/Cargo.toml audio_preprocess --no-run`.

## Task 4: Normalization With Gain Cap

- [ ] Write tests for quiet samples, normal samples, and near-clipping samples.
- [ ] Implement RMS-target normalization with max gain cap and clipping prevention.
- [ ] Use target RMS `0.10` and max gain `8.0` as first constants.
- [ ] Assert quiet samples are boosted, normal samples are not over-boosted, and peaks remain below `1.0`.
- [ ] Run: `cargo test --manifest-path src-tauri/Cargo.toml audio_preprocess --no-run`.

## Task 5: VAD Trim

- [ ] Write tests for leading silence, trailing silence, all-silence, and speech with short pauses.
- [ ] Implement frame-energy leading/trailing trim using V9-style 20 ms RMS frames.
- [ ] Keep padding around detected speech, for example 120 ms before and after.
- [ ] For all-silence input, return the original samples and a warning instead of empty samples.
- [ ] Run: `cargo test --manifest-path src-tauri/Cargo.toml audio_preprocess --no-run`.

## Task 6: Pipeline Integration For Stop-Then-Transcribe

- [ ] Add `preprocess_asr_samples(samples, sample_rate, config)` that returns processed samples plus before/after diagnostics.
- [ ] Route push-to-talk local whisper.cpp samples through the pipeline only when enabled.
- [ ] Route push-to-talk Baidu Short Speech API samples through the pipeline only when enabled.
- [ ] Do not route Baidu Realtime WebSocket API PCM through the pipeline in V10 initial implementation.
- [ ] Add Rust tests for disabled config returning raw samples and enabled config returning metadata.
- [ ] Run: `cargo check --manifest-path src-tauri/Cargo.toml --lib`.

## Task 7: Frontend Settings And Metadata

- [ ] Add `AudioPreprocessConfig` and `AudioPreprocessSummary` to `src/types.ts`.
- [ ] Add Tauri client wrappers in `src/tauriClient.ts`.
- [ ] Rename the main-window entry from model selection/configuration to Settings.
- [ ] Move the settings surface to five tabs: input, model, audio enhancement, text optimization, and diagnostics.
- [ ] Keep audio enhancement out of provider-specific model configuration; it belongs to the audio enhancement tab. Model routing and provider configuration now share the model tab.
- [ ] Embed V9 text optimization inside the text optimization tab instead of keeping it as a separate top-level main-window button.
- [ ] Add settings UI with a master toggle and conservative preset controls, reusing the existing UI style guide and classes.
- [ ] Render compact transcript metadata whenever the enabled preprocessing pipeline runs, including the no-material-change case.
- [ ] Add tests for loading settings, saving settings, the five settings tabs, hotkey and diagnostics inside Settings, text optimization inside Settings, and rendering/persisting preprocessing metadata.
- [ ] Run: `npm test -- --run src/App.test.tsx` and `npm run typecheck`.

## Task 8: Noise Suppression Spike

- [ ] Research WebRTC NS and RNNoise integration options for Rust/Tauri on Windows.
- [ ] Document license, binary size, CPU cost, maintenance risk, and Windows build impact in `docs/research/audio-preprocessing-options.md`.
- [ ] Do not enable denoise by default until the spike has evidence.
- [ ] If no suitable library is selected, keep `denoise_enabled` disabled and document the reason.

## Task 9: Docs And Verification

- [ ] Update `docs/harness/progress.md` with V10 implementation evidence.
- [ ] Update `docs/harness/debugging-log.md` for issues found.
- [ ] Update `docs/harness/feature_list.json` evidence/status.
- [ ] Run full verification:

```bash
npm test -- --run src/App.test.tsx
npm run typecheck
npm run build
cargo check --manifest-path src-tauri/Cargo.toml --lib
cargo test --manifest-path src-tauri/Cargo.toml audio_quality --no-run
cargo test --manifest-path src-tauri/Cargo.toml audio_preprocess --no-run
python -m json.tool docs/harness/feature_list.json
git diff --check
```

## Execution Notes

- V10 starts with stop-then-transcribe paths only.
- Keep the feature user-visible and disable-able.
- Do not store raw audio in transcript history.
- Do not silently alter Baidu Realtime WebSocket API streaming PCM in this initial V10 plan.
- Use V9 diagnostics to decide whether preprocessing improves quality; do not rely on subjective assumptions only.
