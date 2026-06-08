#!/usr/bin/env bash
# Start AgentReady local UI — builds into ./target and opens the browser.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

export CARGO_TARGET_DIR="$ROOT/target"
HOST="${HOST:-127.0.0.1}"
PORT="${PORT:-3000}"
OPEN_BROWSER="${OPEN_BROWSER:-1}"

usage() {
  echo "Usage: ./scripts/serve.sh [--port 3000] [--host 127.0.0.1] [--no-open]"
  exit 0
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -p|--port) PORT="$2"; shift 2 ;;
    --host) HOST="$2"; shift 2 ;;
    --no-open) OPEN_BROWSER=0; shift ;;
    -h|--help) usage ;;
    *) echo "Unknown option: $1" >&2; usage ;;
  esac
done

port_busy() {
  lsof -nP -iTCP:"$1" -sTCP:LISTEN >/dev/null 2>&1
}

pick_port() {
  local p="$1"
  local tries=0
  while port_busy "$p" && [[ $tries -lt 10 ]]; do
    echo "  Port $p is busy, trying $((p + 1))..." >&2
    p=$((p + 1))
    tries=$((tries + 1))
  done
  if port_busy "$p"; then
    echo "✗ Could not find a free port near $PORT." >&2
    echo "  Stop other servers: lsof -i :$PORT" >&2
    exit 1
  fi
  echo "$p"
}

PORT="$(pick_port "$PORT")"

echo "→ Building agentready..."
if ! cargo build -p agentready -q; then
  echo "✗ Build failed. Run: cargo build -p agentready" >&2
  exit 1
fi

BIN="$CARGO_TARGET_DIR/debug/agentready"
if [[ ! -x "$BIN" ]]; then
  echo "✗ Binary not found: $BIN" >&2
  exit 1
fi

if ! "$BIN" serve --help >/dev/null 2>&1; then
  echo "→ Stale binary detected, rebuilding..."
  cargo clean -p agentready -q
  cargo build -p agentready -q
fi

URL="http://${HOST}:${PORT}"
echo "→ Starting UI at $URL"
echo "  Leave this terminal open. Press Ctrl+C to stop."

if [[ "$OPEN_BROWSER" == "1" ]] && command -v open >/dev/null 2>&1; then
  (sleep 1.5 && open "$URL") &
fi

exec "$BIN" serve --host "$HOST" --port "$PORT"
