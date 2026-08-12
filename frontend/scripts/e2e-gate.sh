#!/usr/bin/env bash
# e2e-gate.sh — run the real-browser E2E suite BEFORE serving the app.
# Catches runtime crashes (CJS interop, G6 errors, API contract drift) that
# unit tests cannot: zero page/console errors is a hard gate.
#
# Usage: npm run e2e:gate   (or: bash scripts/e2e-gate.sh)
# Prerequisites: docker TerminusDB (:6363) + API server (:8080) + an imported
# dataset. The dev server is started automatically when not already running.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

if ! curl -s -o /dev/null --max-time 3 http://localhost:3100/; then
  echo "dev server not running — starting it..."
  (cd "$ROOT" && nohup bash scripts/dev-bg.sh > /tmp/dikwms-dev.log 2>&1 &)
  sleep 35
fi
if ! curl -s -o /dev/null --max-time 3 http://localhost:3100/; then
  echo "dev server failed to start — see /tmp/dikwms-dev.log" >&2
  exit 1
fi

echo "running E2E gate (chrome-headless-shell) against http://localhost:3100 ..."
(cd "$ROOT/e2e" && npx playwright test)
echo "e2e gate: PASS — the app is safe to serve"
