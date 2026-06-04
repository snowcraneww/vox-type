# VoxType

VoxType is a Windows desktop voice input tool for local-first dictation experiments. It lets you speak with global hotkeys, transcribe speech with local `whisper.cpp` or Baidu speech APIs, and insert the final text into the app that currently has focus.

The project is still early. The current focus is a usable desktop MVP: reliable recording, visible dictation state, model selection, cloud/local ASR routing, and privacy-conscious configuration.

## Features

- Global push-to-talk: hold `Ctrl+Alt+Space`, speak, release to transcribe and insert text.
- Global continuous input: press `Ctrl+Alt+V` to start, press it again to stop.
- Local ASR through `whisper.cpp` with in-app configuration and one-click setup support.
- Baidu Short Speech API support for stop-then-transcribe input.
- Baidu Realtime WebSocket API support for continuous input.
- Per-mode model selection: push-to-talk and continuous input can use different default models.
- Native Windows dictation overlay with recording and transcribing states.
- Recognition history in the main window with copy, re-insert, delete, clear, export, duration, model, and input-mode metadata.
- Diagnostics for microphone, model configuration, recording, ASR, overlay, and insertion flow.
- Conservative audio enhancement for stop-then-transcribe recordings with persisted settings and transcript metadata.
- Unified Settings tabs for input, models, audio enhancement, text optimization, and diagnostics.

## Current Limitations

- Windows is the primary supported platform.
- Text insertion currently uses a clipboard-based strategy.
- Baidu Realtime WebSocket API inserts final `FIN_TEXT` results; partial `MID_TEXT` results are status-only.
- Recognition quality depends heavily on microphone distance, noise, model choice, and provider behavior.
- Audio enhancement is conservative and currently applies to stop-then-transcribe recordings; Baidu Realtime WebSocket audio remains raw.
- Text insertion currently uses a clipboard-based strategy; the next planned milestone is Windows `SendInput(KEYEVENTF_UNICODE)` with clipboard fallback.

## Tech Stack

- Tauri 2 desktop shell
- Rust backend for recording, ASR adapters, hotkeys, overlay, config, and Windows integration
- React + TypeScript frontend
- Vite and Vitest
- `whisper.cpp` for local transcription
- Baidu Speech APIs for cloud transcription

## Requirements

- Windows 10/11
- Node.js and npm
- Rust stable toolchain
- A working microphone
- Optional: Baidu Speech API credentials for cloud ASR
- Optional: local `whisper.cpp` binary and model, or use the app's setup flow

## Quick Start

Install dependencies:

```bash
npm install
```

Run the desktop app:

```bash
npm run tauri -- dev
```

Browser-only UI preview is available with:

```bash
npm run dev
```

Use the Tauri desktop command when testing microphone capture, global hotkeys, tray behavior, native overlay, clipboard insertion, or Tauri commands. The browser preview cannot validate those desktop features.

## Development Checks

```bash
npm test -- --run
npm run typecheck
npm run build
cargo check --manifest-path src-tauri/Cargo.toml --lib
```

Some Rust test binaries can be affected by Windows environment differences around native system entry points. `cargo check` and targeted `cargo test --no-run` checks are useful when full execution is blocked by the local OS environment.

## Model Configuration

### Local whisper.cpp

Open the model configuration page in the app and configure:

- `whisper.cpp` executable path
- model file path
- language

Large model files are local user assets and should not be committed to Git.

### Baidu Short Speech API

The short speech API uses:

- endpoint: `http://vop.baidu.com/server_api`
- `BAIDU_ASR_API_KEY`
- `BAIDU_ASR_SECRET_KEY`

### Baidu Realtime WebSocket API

The realtime API uses Baidu's WebSocket protocol and is intended for continuous input mode. It shares the Baidu API key/secret setup with the short speech API and also needs an AppID in the app configuration.

See [docs/integrations/baidu-asr.md](docs/integrations/baidu-asr.md) for implementation notes.

## Privacy

VoxType is designed to keep sensitive values out of the repository.

- Do not commit API keys, access tokens, cookies, local app data, recordings, models, or generated diagnostic media.
- Baidu credentials should be configured through environment variables or the app's secure configuration flow.
- Local model files and exported WAV diagnostics stay on the user's machine.
- Cloud ASR modes send audio to the selected provider. Use local `whisper.cpp` if the audio must stay local.

## Public Documentation

- [Run and understand](docs/guide/run-and-understand.md)
- [Code walkthrough](docs/guide/code-walkthrough.md)
- [Baidu ASR integration notes](docs/integrations/baidu-asr.md)

Internal planning, research, agent harness, and AI coding records are intentionally excluded from the public repository.

## License

VoxType is licensed under the Apache License 2.0. See [LICENSE](LICENSE).
