#!/usr/bin/env bash
set -euo pipefail

ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$ROOT"

run_step() {
  local name="$1"
  shift
  echo "==> $name"
  "$@"
}

echo "VoxType V13 clipboard-first readiness checks"
echo "Scope: safe automated checks only. This script does not record audio, send global shortcuts, steal focus, or operate on real target windows."
echo

run_step "harness baseline" bash init.sh
run_step "frontend tests" npm test -- --run
run_step "typecheck" npm run typecheck
run_step "production build" npm run build
run_step "rust library check" cargo check --manifest-path src-tauri/Cargo.toml --lib
run_step "insertion compile check" cargo test --manifest-path src-tauri/Cargo.toml insertion --no-run
run_step "feature list JSON" python -m json.tool docs/harness/feature_list.json >/dev/null
run_step "diff whitespace check" git diff --check

echo
echo "Automated V13 checks passed."
echo "Manual desktop validation still required: real microphone capture, Ctrl+Alt+Space, Ctrl+Alt+V, Clipboard and Auto safe insertion in Notepad/VS Code/browser/daily target, and optional SendInput experimental observation."