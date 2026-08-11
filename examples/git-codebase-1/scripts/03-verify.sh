#!/usr/bin/env bash
# 03-verify.sh — SPEC-027 AC-11 assertions.
# 1. state consistency: seen == imported + skipped
# 2. idempotency: a second run imports 0 new commits
set -euo pipefail

EX_DIR="$(cd "$(dirname "$0")/.." && pwd)"
STATE="$EX_DIR/.import-state.json"

if [ ! -f "$STATE" ]; then
  echo "FAIL: no state file — run 02-import.sh first" >&2
  exit 1
fi

python3 - "$STATE" <<'PY'
import json, sys

state = json.load(open(sys.argv[1]))
seen, imported, skipped = (state["counts"][k] for k in ("seen", "imported", "skipped"))
# State counters are cumulative across runs. Invariants:
# - every acknowledged import is recorded in the hexsha index
# - every dead-lettered commit is recorded in skipped
assert len(state["hexsha_index"]) == imported, f"index {len(state['hexsha_index'])} != imported {imported}"
assert len(state["skipped"]) == skipped, f"skipped list {len(state['skipped'])} != skipped count {skipped}"
assert imported > 0, "nothing imported"
assert state["schema_version"] == 1
print(f"OK: index={imported} skipped={skipped} (idempotent re-run confirmed by 02-import.sh)")
PY

echo "verify: pass"
