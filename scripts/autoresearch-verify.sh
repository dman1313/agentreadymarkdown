#!/usr/bin/env bash
# AgentReady autoresearch verification harness.
# Usage: ./scripts/autoresearch-verify.sh [baseline|full]
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

MODE="${1:-full}"

run_lib() {
  local out
  out="$(cargo test --lib 2>&1)"
  echo "$out" >&2
  echo "$out" | rg -o 'test result: ok\. [0-9]+ passed' | rg -o '[0-9]+ passed' | rg -o '^[0-9]+' || echo 0
}

if [[ "$MODE" == "baseline" ]]; then
  UNIT_PASS="$(run_lib)"
  echo "unit_pass:${UNIT_PASS}"
  exit 0
fi

UNIT_PASS="$(run_lib)"
INT_OUT="$(cargo test -p agentready --test integration 2>&1)" || INTEGRATION_FAILED=1
INTEGRATION_FAILED="${INTEGRATION_FAILED:-0}"
echo "$INT_OUT" >&2

INTEGRATION_PASS="$(echo "$INT_OUT" | rg -o 'test result:.*' | tail -1 | rg -o '[0-9]+ passed' | rg -o '^[0-9]+' || echo 0)"
FAILED="$(echo "$INT_OUT" | rg -o 'test result:.*' | tail -1 | rg -o '[0-9]+ failed' | rg -o '^[0-9]+' || echo 0)"
IGNORED="$(echo "$INT_OUT" | rg -o 'test result:.*' | tail -1 | rg -o '[0-9]+ ignored' | rg -o '^[0-9]+' || echo 0)"
INTEGRATION_TOTAL=$((INTEGRATION_PASS + FAILED + IGNORED))

echo "unit_pass:${UNIT_PASS}"
echo "integration_pass:${INTEGRATION_PASS}"
echo "integration_total:${INTEGRATION_TOTAL}"

if [[ "$INTEGRATION_FAILED" -eq 1 ]]; then
  exit 1
fi
