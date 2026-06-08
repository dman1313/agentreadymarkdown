#!/usr/bin/env bash
# Show AgentReady before/after: source bytes vs lean agent Markdown export.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
BIN="$CARGO_TARGET_DIR/debug/agentready"
FIXTURES="$ROOT/examples/sample-input/ebooks"
OUT="${1:-/tmp/agentready-demo}"

if [[ ! -x "$BIN" ]]; then
  echo "→ Building agentready..."
  cargo build -p agentready -q
fi

mkdir -p "$OUT"
rm -rf "$OUT"/* 2>/dev/null || true

echo "AgentReady — before → after (agent Markdown)"
echo "Output dir: $OUT"
echo

run_demo() {
  local file="$1"
  local base
  base=$(basename "$file")
  local stem="${base%.*}"
  local ext="${base##*.}"
  local subout="$OUT/${stem}-${ext}"
  local bytes
  bytes=$(wc -c <"$file" | tr -d ' ')

  echo "━━ $base ━━"
  echo "  Before: ${bytes} bytes (binary / opaque format)"
  if ! "$BIN" convert "$file" --output "$subout" >/dev/null 2>&1; then
    echo "  After:  conversion failed"
    echo
    return
  fi

  local md
  md=$(find "$subout/documents" -name '*.md' 2>/dev/null | head -1)
  if [[ -z "$md" || ! -f "$md" ]]; then
    echo "  After:  conversion failed"
    echo
    return
  fi

  local md_bytes body_chars
  md_bytes=$(wc -c <"$md" | tr -d ' ')
  body_chars=$(awk 'BEGIN{p=0} /^---$/{p++; next} p>=2{print}' "$md" | wc -c | tr -d ' ')

  echo "  After:  $(basename "$md") — ${md_bytes} bytes export (${body_chars} bytes content + frontmatter)"
  echo "  Preview:"
  awk 'BEGIN{p=0} /^---$/{p++; next} p>=2{print}' "$md" | head -8 | sed 's/^/    /'
  echo
}

for f in minimal.pdf minimal.docx minimal.epub; do
  if [[ -f "$FIXTURES/$f" ]]; then
    run_demo "$FIXTURES/$f"
  fi
done

echo "Done. Full exports in: $OUT"
echo "Claim: lean Markdown for agents — not exact token savings."
echo "Spec: docs/09_MARKDOWN_STANDARD.md"
