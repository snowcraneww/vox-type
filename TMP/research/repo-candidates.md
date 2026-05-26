# VoxType Repository Candidate Notes

Last updated: 2026-05-26

Local clones are stored under `TMP/research/repos/` and ignored by Git. They are used only for reading and comparison. Do not copy third-party source code into VoxType.

## Deep-Read Candidates

| Project | Upstream | Local path | License | Stack | Relevance | Strengths | Risks |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Handy | https://github.com/cjpais/Handy | `TMP/research/repos/Handy` | MIT | Rust, Tauri, React | Very high | Offline-first dictation, `cpal` audio, VAD, local ASR, shortcut/settings/UI patterns, multiple paste methods | Mature app scope is broader than VoxType MVP; do not inherit all settings complexity |
| OpenLess | https://github.com/Open-Less/openless | `TMP/research/repos/openless` | MIT | Rust, Tauri 2, React/TS, Windows native IME helper | Very high | Closest match to push-to-talk then insert; has Windows TSF, `SendInput(KEYEVENTF_UNICODE)`, clipboard fallback, debounce, streaming insertion edge handling | TSF path is complex for MVP; use as design reference, not first milestone |
| OpenWhispr | https://github.com/OpenWhispr/openwhispr | `TMP/research/repos/openwhispr` | MIT | Electron, React, TypeScript | Medium | Rich product surface: dictation, notes, local/cloud providers, agent-like workflows | Scope and runtime weight are too large for VoxType MVP |
| VoiceFlow | https://github.com/infiniV/VoiceFlow | `TMP/research/repos/VoiceFlow` | MIT | Python/Pyloid, React/Vite | Medium-high | Fast local implementation using `faster-whisper`, hold hotkey, paste at cursor, GPU fallback thinking | Python desktop packaging and runtime dependencies add maintenance risk |
| whisper-writer | https://github.com/savbell/whisper-writer | `TMP/research/repos/whisper-writer` | README says GPL; verify LICENSE before relying on it | Python, PyQt5 | Medium | Minimal dictation chain: hotkey, recording modes, faster-whisper/API, input simulation | License and dependency age make it unsuitable as a code source |
| tambourine-voice | https://github.com/kstonekuan/tambourine-voice | `TMP/research/repos/tambourine-voice` | AGPL-3.0 | Tauri, Rust, Python server, Pipecat | Medium | Strong provider architecture and post-processing ideas | AGPL is incompatible with the intended Apache-2.0 reuse path; concept-only reference |
| faster-whisper-dictation | https://github.com/bhargavchippada/faster-whisper-dictation | `TMP/research/repos/faster-whisper-dictation` | MIT | Python CLI/daemon | High for pipeline details | Good VAD/hotkey/clipboard engineering notes, Silero VAD, pre-speech buffer, debounce, tests | Not a desktop product shell; Python implementation is not the recommended long-term core |

## Ecosystem References Not Fully Cloned

| Project | URL | License | Stack | Why it matters |
| --- | --- | --- | --- | --- |
| whisper.cpp | https://github.com/ggml-org/whisper.cpp | MIT | C/C++ | Mature local Whisper runtime. Good baseline for local/offline ASR, though process-spawning `whisper-cli` should be treated as a prototype path. |
| sherpa-onnx | https://github.com/k2-fsa/sherpa-onnx | Apache-2.0 | C++/ONNX Runtime, multi-language bindings | License-compatible local ASR/VAD/TTS toolkit with Windows support. Strong candidate for later engine evaluation. |
| local-whisper | https://github.com/t2o2/local-whisper | MIT | Swift/macOS | Small offline dictation app with hotkey-to-any-app UX; useful for product pattern comparison, not Windows implementation. |
| FluidAudio | https://github.com/FluidInference/FluidAudio | Apache-2.0 | Swift/CoreML | Shows direction of local audio models, VAD, ASR, diarization on Apple platforms; mostly future macOS reference. |
| FluidVoice | https://github.com/altic-dev/FluidVoice | GPL-3.0 | Swift/macOS | Offline dictation product reference. GPL means concept-only for Apache-2.0 project. |
| Whispering | Search result discussed a local-first open source dictation app, but `epicenter-so/whispering` did not resolve via GitHub API on 2026-05-26 | Unknown | Unknown | Mention only as an unverified product signal; not used as evidence for architecture. |

## Cross-Candidate Patterns

- Push-to-talk needs debounce. OpenLess and faster-whisper-dictation both highlight key repeat or release handling.
- Audio should be normalized to 16 kHz mono PCM before ASR.
- A pre-speech ring buffer or silence trimming prevents clipping the first syllable.
- Clipboard insertion is easy to implement but must preserve clipboard content and serialize access.
- Windows Unicode input through `SendInput(KEYEVENTF_UNICODE)` is a useful step before TSF.
- Full TSF insertion gives better IME-native behavior but is too expensive for the first MVP.
- Overlay windows can steal focus; status UI must avoid breaking the target caret.

