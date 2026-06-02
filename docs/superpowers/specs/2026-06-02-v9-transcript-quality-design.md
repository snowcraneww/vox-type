# V9 Transcript Quality And History Persistence Design

## Context

V8 is closed. Baidu Realtime WebSocket API continuous input has been manually verified on the real desktop app, and the capsule overlay border clipping follow-up has been accepted. The next product gap is daily-use quality: speech recognition can mishear technical terms, and transcript history currently behaves like session-local working memory instead of a durable user record.

V9 combines two related capabilities:

- Transcript quality: deterministic glossary, replacement, cleanup, and formatting rules.
- Transcript history persistence: recognized entries survive app restart and keep useful metadata.

These belong together because the stored history should preserve the final text users actually inserted, plus enough metadata to diagnose which input mode/model produced it.

## Product Goals

Users can configure local text optimization rules and can keep their recognition history across restarts.

The app should store transcript entries with:

- final inserted text after post-processing,
- input mode: push-to-talk or continuous input,
- model: local whisper.cpp, Baidu Short Speech API, or Baidu Realtime WebSocket API,
- duration, character count, timestamp,
- optional processing metadata such as applied rule count.

The first implementation should be local-only, deterministic, and explainable. It should not require a cloud LLM or any account beyond the already configured ASR providers.

## Non-Goals

- Do not implement SendInput or TSF in V9.
- Do not insert Baidu MID_TEXT partial results into the target app.
- Do not add LLM rewriting as a required dependency.
- Do not store raw audio in transcript history.
- Do not store API keys, access tokens, or secret environment values.
- Do not build cloud sync for transcript history.

## Recommended Approach

Use a Rust-owned durable storage layer for transcript history, and a Rust-owned deterministic post-processing module for text quality. React remains the UI/control layer, but persistence and validation live in Tauri commands so the behavior is consistent across all ASR paths.

A simple JSON file in the Tauri app data directory is enough for V9. Use bounded history retention to avoid unbounded file growth.

Default retention:

- keep latest 500 entries,
- provide clear-all action,
- no automatic cloud sync,
- future export can reuse the same stored model.

## Data Flow

```mermaid
flowchart LR
  classDef asr fill:#fce7f3,stroke:#db2777,color:#500724
  classDef quality fill:#dcfce7,stroke:#16a34a,color:#052e16
  classDef store fill:#dbeafe,stroke:#2563eb,color:#172554
  classDef output fill:#ede9fe,stroke:#7c3aed,color:#2e1065

  Whisper[local whisper.cpp final text]:::asr --> Quality[TranscriptPostProcessor]:::quality
  BaiduShort[Baidu Short Speech API text]:::asr --> Quality
  BaiduRT[Baidu Realtime FIN_TEXT]:::asr --> Quality
  Quality --> Insert[target app insertion]:::output
  Quality --> Entry[TranscriptHistoryEntry]:::store
  Entry --> Store[local history JSON in app data]:::store
  Store --> UI[history list after restart]:::output
```

## Transcript Quality Design

The post-processing module should support:

- enabled flag,
- replacement rules such as `scale => skill`,
- glossary terms with fixed spelling such as `WebSocket`, `whisper.cpp`, `VoxType`,
- noise cleanup for short caption artifacts,
- whitespace and punctuation normalization,
- preview command for the settings UI.

Processing order:

1. Normalize whitespace.
2. Drop known noise-only strings.
3. Apply enabled replacement rules, longest source text first.
4. Normalize glossary term spelling.
5. Trim final output.

The processor returns both processed text and a small metadata object, for example `rulesApplied: 2`. The diagnostics should not log full user text by default.

## Transcript History Persistence Design

Create a history store that loads on app startup and writes whenever a new transcript entry is created.

Suggested type shape:

```typescript
interface PersistedTranscriptEntry {
  id: string;
  text: string;
  inputMode: 'push-to-talk' | 'toggle-dictation';
  model: 'local-whisper' | 'baidu-short' | 'baidu-realtime';
  createdAtMs: number;
  durationMs: number;
  characterCount: number;
  postprocessRulesApplied: number;
}
```

The store should provide commands:

- `load_transcript_history() -> Vec<PersistedTranscriptEntry>`
- `save_transcript_history_entry(entry) -> PersistedTranscriptEntry`
- `delete_transcript_history_entry(id) -> Vec<PersistedTranscriptEntry>`
- `clear_transcript_history() -> Vec<PersistedTranscriptEntry>`

React can still manage the visible list, but mutations should go through these commands so disk and UI stay aligned.

## UI Design

Keep the main history UI visually close to the current V7/V8 record cards. V9 should add persistence without making the main screen heavier.

Required user-visible changes:

- history loads after restart,
- clear-all clears persistent storage too,
- deleting one row deletes it from storage too,
- export uses persisted + current visible entries consistently,
- settings page adds a text optimization panel with preview.

## Error Handling

- Corrupt history file: rename or ignore it, start with empty history, and show a diagnostic warning.
- Disk write failure: keep the UI entry but show a non-blocking warning that persistence failed.
- Post-processing config failure: fall back to disabled text optimization.
- Replacement rule creates empty text: keep original text and report a warning.

## Privacy

Transcript history contains user content. Keep it local in the Tauri app data directory. Do not put history entries in repository files, tests, logs, or diagnostics. Tests should use synthetic text only.

## Verification

Automated checks:

- Rust no-run tests for post-processing and history storage.
- React tests for loading persisted history, saving new entries, deleting entries, and clearing all.
- React tests for `scale => skill` preview and runtime application.
- Existing V8 realtime tests continue passing.

Manual checks:

- Add a replacement rule `scale => skill`, dictate a sentence, and confirm inserted text/history use `skill`.
- Restart the Tauri app and confirm history remains.
- Delete one history entry, restart, and confirm it stays deleted.
- Clear all, restart, and confirm history is empty.
