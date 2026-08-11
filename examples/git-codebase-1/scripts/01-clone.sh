#!/usr/bin/env bash
# 01-clone.sh — blobless clone of terminusdb/terminusdb (SPEC-027 REQ-011).
# Blobless: full commit+tree history without blob content — the importer only
# needs trees for touched-path diffs. Reuses an existing clone when present.
set -euo pipefail

REPO_URL="${REPO_URL:-https://github.com/terminusdb/terminusdb.git}"
CACHE_DIR="$(cd "$(dirname "$0")/.." && pwd)/.cache"
DEST="$CACHE_DIR/terminusdb"

mkdir -p "$CACHE_DIR"
if [ -d "$DEST/.git" ]; then
  echo "clone exists at $DEST — fetching latest main"
  git -C "$DEST" fetch --filter=blob:none origin main
else
  echo "cloning $REPO_URL (blobless) into $DEST"
  git clone --filter=blob:none --no-checkout "$REPO_URL" "$DEST"
fi
echo "done: $DEST"
