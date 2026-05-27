#!/usr/bin/env bash
set -euo pipefail

required_files=(
  "AGENTS.md"
  "README.md"
  "LICENSE"
  ".gitignore"
  ".editorconfig"
  ".gitattributes"
  "docs/harness/working-agreement.md"
  "docs/harness/progress.md"
  "docs/harness/feature_list.json"
  "docs/harness/quality.md"
  "docs/harness/debugging-log.md"
  "docs/harness/evaluator-rubric.md"
  "docs/harness/session-handoff.md"
  "docs/harness/clean-state-checklist.md"
  "docs/harness/research-log.md"
  "docs/harness/lesson-synthesis.md"
  "docs/research/README.md"
  "openspec/README.md"
)

product_files=(
  "package.json"
  "src/App.tsx"
  "src-tauri/Cargo.toml"
  "src-tauri/src/lib.rs"
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

python - <<'PY'
from pathlib import Path
import re
paths = [Path('AGENTS.md'), Path('README.md')]
for root in [Path('docs/harness'), Path('docs/research'), Path('openspec')]:
    paths.extend(path for path in root.rglob('*') if path.is_file())
bad = []
for path in paths:
    text = path.read_text(encoding='utf-8')
    if re.search(r'`n(?![A-Za-z0-9_-])', text):
        bad.append(str(path))
if bad:
    print('init failed: found literal PowerShell newline escape')
    for path in bad:
        print(f'bad: {path}')
    raise SystemExit(1)
PY

python -m json.tool docs/harness/feature_list.json >/dev/null

product_scaffold="files present"
for file in "${product_files[@]}"; do
  if [[ ! -f "$file" ]]; then
    product_scaffold="missing files"
  fi
done

harness_summary=$(python - <<'PY'
import json
from pathlib import Path

data = json.loads(Path('docs/harness/feature_list.json').read_text(encoding='utf-8'))
features = data.get('features', [])

scaffold = next((item for item in features if item.get('id') == 'scaffold-001'), None)
scaffold_status = scaffold.get('status', 'unknown') if scaffold else 'unknown'

unfinished = [item for item in features if item.get('status') != 'passing']
unfinished.sort(key=lambda item: item.get('priority', 9999))

if unfinished:
    next_item = f"{unfinished[0].get('id', 'unknown')} ({unfinished[0].get('status', 'unknown')})"
else:
    next_item = 'no unfinished harness item; choose next product improvement intentionally'

print(f"Product scaffold: {scaffold_status}")
print(f"Next: {next_item}")
PY
)

echo "VoxType harness baseline OK"
echo "License: Apache-2.0"
echo "Product files: $product_scaffold"
echo "$harness_summary"
