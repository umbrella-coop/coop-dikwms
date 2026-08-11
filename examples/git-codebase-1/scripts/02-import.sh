#!/usr/bin/env bash
# 02-import.sh — build + run the git-importer (SPEC-027 REQ-001..003).
# Architecture: git-import → data-graph API → terminusdb. Requires the API
# server: cargo run -p api (which talks to the docker TerminusDB).
# Deterministic window: --since 2025-08-11. Re-runs are idempotent
# (server-side dedupe by key).
# Env: API_BASE (default http://localhost:8080)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
EX_DIR="$ROOT/examples/git-codebase-1"
REPO_DIR="${REPO_DIR:-$EX_DIR/.cache/terminusdb}"
SINCE="${SINCE:-2025-08-11}"

if [ ! -d "$REPO_DIR/.git" ]; then
  echo "missing clone — run ./scripts/01-clone.sh first" >&2
  exit 1
fi

echo "importing $REPO_DIR since $SINCE (state: $EX_DIR/.import-state.json)"
(
  cd "$ROOT/backend"
  env RUSTUP_TOOLCHAIN=nightly \
    cargo run -p git-importer -- \
    --repo "$REPO_DIR" \
    --since "$SINCE" \
    --state "$EX_DIR/.import-state.json" \
    "${@}"
)

