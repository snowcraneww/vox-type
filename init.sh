#!/usr/bin/env bash
set -euo pipefail

required_files=(
  "AGENTS.md"
  "README.md"
  "CONTRIBUTING.md"
  "LICENSE"
  ".gitignore"
  ".editorconfig"
  ".gitattributes"
  "docs/harness/working-agreement.md"
  "docs/harness/progress.md"
  "docs/harness/feature_list.json"
  "docs/harness/quality.md"
  "docs/harness/evaluator-rubric.md"
  "docs/harness/session-handoff.md"
  "docs/harness/clean-state-checklist.md"
  "docs/harness/research-log.md"
  "docs/harness/lesson-synthesis.md"
  "docs/research/README.md"
  "openspec/README.md"
)

missing=0
for file in "${required_files[@]}"; do
  if [[ ! -f "$file" ]]; then
    echo "missing: $file"
    missing=1
  fi
done

if [[ "$missing" -ne 0 ]]; then
  echo "init failed: required harness files are missing"
  exit 1
fi

if grep -R '\`n' AGENTS.md README.md CONTRIBUTING.md init.sh docs/harness docs/research openspec >/dev/null 2>&1; then
  echo "init failed: found literal PowerShell newline escape"
  exit 1
fi

python -m json.tool docs/harness/feature_list.json >/dev/null

echo "VoxType harness baseline OK"
echo "License: Apache-2.0"
echo "Product scaffold: not started"
echo "Next: read docs/harness/feature_list.json and pick the highest-priority not_started item"