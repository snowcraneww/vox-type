# Release Checklist

This checklist is the V12 release-readiness gate. Run it from the repository root.

## Automated Checks

```bash
bash init.sh
npm test -- --run
npm run typecheck
npm run build
cargo check --manifest-path src-tauri/Cargo.toml --lib
cargo test --manifest-path src-tauri/Cargo.toml insertion --no-run
cargo test --manifest-path src-tauri/Cargo.toml transcript_history --no-run
python -m json.tool docs/harness/feature_list.json
git diff --check
```

`bash init.sh` currently may fail if harness/planning files are intentionally tracked. Treat that as a harness-policy issue to resolve or document before a public release; do not ignore unrelated failures.

## Release Build

```bash
npm run tauri -- build
```

Expected release artifacts are produced under `src-tauri/target/release/bundle/`. Installer binaries, model files, diagnostic WAV files, logs, and local config files must not be committed.

## Privacy Scan

Before committing or tagging a release, check that the diff does not contain:

- API keys, secret keys, access tokens, cookies, or full environment variable values.
- Local user profile paths or machine-specific absolute paths.
- Audio files, exported diagnostic WAV files, model binaries, or downloaded ASR runtimes.
- Raw clipboard contents outside intentional tests.
- Personal author identity in Git metadata; use `VoxType <maintainers@voxtype.dev>`.

A quick staged diff scan can use:

```bash
git diff --cached | grep -Ei 'C:/Users|C:\\Users|API_KEY=|SECRET_KEY=|access_token|cookie' || true
```

Variable names are allowed in documentation; raw secret values are not.

## Manual Desktop Gate

Before marking V12 passing, complete `docs/guide/desktop-compatibility-matrix.md` with real results for Clipboard, SendInput, and Auto.
