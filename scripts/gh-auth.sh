#!/usr/bin/env bash
# AgentReady — GitHub CLI auth helper (safe: never stores tokens in this repo)
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()  { echo -e "${GREEN}→${NC} $*"; }
warn()  { echo -e "${YELLOW}!${NC} $*"; }
fail()  { echo -e "${RED}✗${NC} $*" >&2; exit 1; }

if ! command -v gh >/dev/null 2>&1; then
  fail "GitHub CLI (gh) is not installed. Install: brew install gh"
fi

info "Repository: $REPO_ROOT"
if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  info "Git remote(s):"
  git remote -v 2>/dev/null | sed 's/^/    /' || true
  info "Branch: $(git branch --show-current 2>/dev/null || echo '(unknown)')"
else
  warn "Not a git repository."
fi

echo ""
info "Current GitHub auth status:"
if gh auth status 2>&1; then
  echo ""
  info "Already authenticated. You can push with:"
  echo "    git push origin \$(git branch --show-current)"
  exit 0
fi

echo ""
warn "Not logged in or token is invalid (common after token expiry or revoke)."
echo ""
echo "  This script will run:  gh auth login -h github.com"
echo "  Choose one of:"
echo "    • Login with a web browser (recommended)"
echo "    • Paste a token only in the terminal prompt (never in chat or in repo files)"
echo ""
read -r -p "Continue with gh auth login? [y/N] " ans
case "${ans:-n}" in
  y|Y|yes|Yes) ;;
  *) info "Cancelled."; exit 0 ;;
esac

gh auth login -h github.com

echo ""
info "Verifying auth..."
gh auth status

echo ""
info "Testing access to this repo's GitHub remote (if configured)..."
if url="$(git remote get-url origin 2>/dev/null)"; then
  # https://github.com/owner/repo.git or git@github.com:owner/repo.git
  owner_repo=""
  if [[ "$url" =~ github\.com[:/]([^/]+)/([^/.]+) ]]; then
    owner_repo="${BASH_REMATCH[1]}/${BASH_REMATCH[2]}"
    if gh repo view "$owner_repo" --json name,url -q '"\(.name) — \(.url)"' 2>/dev/null; then
      info "Can access $owner_repo on GitHub."
    else
      warn "Authenticated, but could not view $owner_repo (check repo name or permissions)."
    fi
  fi
else
  warn "No origin remote; skip repo check."
fi

echo ""
info "Done. Push when ready:"
echo "    git add -A && git commit -m \"your message\""
echo "    git push -u origin \$(git branch --show-current)"
