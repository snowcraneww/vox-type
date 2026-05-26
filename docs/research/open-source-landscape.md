# Open Source Landscape

Last updated: 2026-05-26

Research method: read the original requirement, use Tavily/GitHub search, query GitHub metadata, and deep-read local clones under `TMP/research/repos/`. Intermediate notes are in `TMP/research/`.

## Strongest References

| Project | URL | License | Stack | Assessment |
| --- | --- | --- | --- | --- |
| Handy | https://github.com/cjpais/Handy | MIT | Rust, Tauri, React | Strong architecture reference for local dictation, audio capture, VAD, model settings, and paste strategy. |
| OpenLess | https://github.com/Open-Less/openless | MIT | Rust, Tauri 2, React/TS, Windows native helpers | Closest UX match. Valuable for Windows insertion strategy: TSF, Unicode `SendInput`, clipboard fallback, debounce. |
| faster-whisper-dictation | https://github.com/bhargavchippada/faster-whisper-dictation | MIT | Python daemon/CLI | Good low-level reference for VAD, pre-speech buffering, hotkey debounce, and clipboard locking. |
| VoiceFlow | https://github.com/infiniV/VoiceFlow | MIT | Python/Pyloid, React/Vite | Useful for fast local transcription and packaging tradeoffs, but less ideal as long-term architecture. |
| OpenWhispr | https://github.com/OpenWhispr/openwhispr | MIT | Electron, React, TypeScript | Product reference for broader voice workflows; too large for VoxType MVP. |
| whisper-writer | https://github.com/savbell/whisper-writer | GPL noted in README; verify before reuse | Python, PyQt5 | Minimal dictation UX reference. License and dependency age make it concept-only. |
| tambourine-voice | https://github.com/kstonekuan/tambourine-voice | AGPL-3.0 | Tauri, Rust, Python server | Useful provider/post-processing ideas, but AGPL blocks direct code reuse. |

## Engine And Platform References

| Project | URL | License | Notes |
| --- | --- | --- | --- |
| whisper.cpp | https://github.com/ggml-org/whisper.cpp | MIT | Mature local Whisper runtime; good proof and fallback engine. |
| sherpa-onnx | https://github.com/k2-fsa/sherpa-onnx | Apache-2.0 | Strong license-compatible ASR/VAD candidate with Windows support and multiple bindings. |
| local-whisper | https://github.com/t2o2/local-whisper | MIT | macOS offline dictation UX reference. |
| FluidAudio | https://github.com/FluidInference/FluidAudio | Apache-2.0 | Apple/CoreML audio model reference for future macOS direction. |
| FluidVoice | https://github.com/altic-dev/FluidVoice | GPL-3.0 | macOS offline dictation product reference; concept-only. |

## Lessons For VoxType

- The difficult part is not only ASR; it is the whole loop: hotkey, focus, recording, transcription, insertion, feedback, and failure recovery.
- Windows insertion should be staged. Start with clipboard restoration, then Unicode `SendInput`, then TSF only if needed.
- Debounce and pre-speech buffering are small details with large UX impact.
- Avoid focus-stealing overlays.
- Keep the first release narrow. The successful comparable projects become complex quickly when they add meetings, agents, provider marketplaces, and formatting workflows.

## License Notes

VoxType is Apache-2.0. MIT and Apache-2.0 projects can inform implementation more directly, subject to attribution and dependency license review. GPL and AGPL projects must remain concept-only unless the project intentionally changes licensing strategy.

