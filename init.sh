#!/usr/bin/env bash
set -euo pipefail

required_files=(
  "README.md"
  "LICENSE"
  ".gitignore"
  ".editorconfig"
  ".gitattributes"
  "package.json"
  "src/App.tsx"
  "src-tauri/Cargo.toml"
  "src-tauri/src/lib.rs"
  "src-tauri/tauri.conf.json"
  "docs/README.md"
  "docs/guide/run-and-understand.md"
  "docs/guide/code-walkthrough.md"
  "docs/integrations/baidu-asr.md"
)

missing=0
for file in "${required_files[@]}"; do
  if [[ ! -f "$file" ]]; then
    echo "missing: $file"
    missing=1
  fi
done

if [[ "$missing" -ne 0 ]]; then
  echo "init failed: required project files are missing"
  exit 1
fi

python - <<'PY'
from pathlib import Path
import json

json_files = [Path('package.json'), Path('src-tauri/tauri.conf.json')]
for path in json_files:
    json.loads(path.read_text(encoding='utf-8'))
PY

tracked_private=$(git ls-files | grep -E '^(\.codex/|\.agents/|\.claude/|\.gstack/|TMP/research/repos/)' || true)
if [[ -n "$tracked_private" ]]; then
  echo "init failed: private agent/cache files are tracked"
  echo "$tracked_private"
  exit 1
fi

echo "VoxType project baseline OK"
echo "License: Apache-2.0"
echo "Frontend: React + TypeScript + Vite"
echo "Desktop: Tauri 2 + Rust"
echo "Run desktop app: npm run tauri -- dev"
