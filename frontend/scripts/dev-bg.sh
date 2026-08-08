#!/usr/bin/env bash
# Keep one dikwms app dev server in the background (DX: persistent dev loop).
# Usage: npm run dev:bg   (from frontend/)
#   - kills anything on :3100, clears stale vite cache, starts `npm run dev`
#   - log: /tmp/dikwms-dev.log ; UI: http://localhost:3100
#   - stop: pkill -f "bit run app -p 3100"
set -euo pipefail
cd "$(dirname "$0")/.."

lsof -ti :3100 | xargs kill 2>/dev/null || true
sleep 1
rm -rf node_modules/.vite

nohup npm run dev > /tmp/dikwms-dev.log 2>&1 &
DEV_PID=$!
echo "dev server starting (pid $DEV_PID) — log: /tmp/dikwms-dev.log"
echo "UI: http://localhost:3100  (stop: pkill -f 'bit run app -p 3100')"
for i in $(seq 1 30); do
  if curl -sf -o /dev/null http://localhost:3100/ 2>/dev/null; then
    echo "up after ${i}s — http://localhost:3100"
    exit 0
  fi
  sleep 1
done
echo "ERROR: not responding on :3100 after 30s — check /tmp/dikwms-dev.log" >&2
exit 1
