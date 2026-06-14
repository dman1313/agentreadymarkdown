# AgentReady autoresearch

Autonomous improvement loop for the AgentReady Rust conversion engine. Modeled on [Karpathy autoresearch](https://github.com/karpathy/autoresearch); adapted for a coding project where the metric is **tests pass + fixture quality**.

## Setup

To start a new experiment run, work with the user to:

1. **Agree on a run tag**: e.g. `jun14` from today's date. Branch `autoresearch/<tag>` must not already exist.
2. **Pick a feature branch** to iterate on (or create one from `master`):
   - `feature/html` — HTML converter (text-only)
   - `feature/rtf` — RTF converter
   - `feature/kpi-drop-in` — KPI eval harness (Phase 5, spec TBD)
3. **Read context** (repo is small — read all of these once):
   - `CLAUDE.md` — build commands and architecture
   - `docs/SUMMARY.md` — spec index; `docs/08_CONVERSION_PIPELINE.md`, `docs/09_MARKDOWN_STANDARD.md`, `docs/17_TESTING_STRATEGY.md`
   - `crates/agentready-core/src/converters/mod.rs` — converter registry
   - `crates/agentready-cli/tests/integration.rs` — fixture integration patterns
   - `examples/sample-input/` — committed fixtures
4. **Establish baseline**: run the verify script (see below), record pass counts in `results.tsv`.
5. **Confirm and go**: kick off the loop.

**Git note:** if `git commit` fails with `unknown option 'trailer'`, use `/opt/homebrew/bin/git`.

## Experimentation

Each experiment should finish in **under 5 minutes** (fast unit tests + one fixture convert).

### EDITABLE surface (you may modify)

| Area | Files |
|------|-------|
| Converters | `crates/agentready-core/src/converters/*.rs` |
| Markdown normalization | `crates/agentready-core/src/agent_markdown.rs` |
| Text quality heuristics | `crates/agentready-core/src/text_quality.rs` |
| Shared text helpers | `crates/agentready-core/src/converters/plain_text_to_markdown.rs` |
| Converter unit tests | `#[cfg(test)]` modules inside the above |
| Fixtures (when adding a format) | `examples/sample-input/**` |
| Integration tests for your format | `crates/agentready-cli/tests/integration.rs` (only tests for the format you are improving) |

### FIXED surface (read-only unless user approves)

| Area | Files | Why fixed |
|------|-------|-----------|
| Validation rules | `crates/agentready-core/src/validation.rs` | Contract with server/CLI |
| Export package | `crates/agentready-core/src/export.rs` | Output schema invariant |
| Error codes / user messages | `crates/agentready-core/src/models.rs` | API contract |
| Job orchestration | `crates/agentready-core/src/job.rs` | Only add `match` arm when shipping a new format |
| CLI binary | `crates/agentready-cli/src/**` | Wire only when adding commands |
| Spec docs | `docs/**` | Source of truth — don't drift silently |
| Dependencies | `Cargo.toml` files | No new crates without user approval |

### METRIC

**Primary (gate):** all tests pass.

```bash
./scripts/autoresearch-verify.sh baseline   # fast: lib tests only
./scripts/autoresearch-verify.sh full       # gate: lib + integration
```

Record `unit_pass` and `integration_pass` (and `integration_total`) from script output.

**Secondary (quality):** fixture assertions for the format under work. After a passing gate, run the targeted integration test:

```bash
cargo test -p agentready convert_minimal_html   # when on feature/html
cargo test -p agentready convert_minimal_rtf    # when on feature/rtf
cargo test -p agentready convert_minimal_xlsx   # etc.
```

Manually inspect converted markdown when changing extraction logic:

```bash
cargo run -- convert examples/sample-input/ebooks/minimal.html --output /tmp/ar-out
cat /tmp/ar-out/documents/minimal-html.md
```

**Phase 5 KPI drop-in:** not yet specified in `docs/`. When a KPI script lands on `feature/kpi-drop-in`, add a tertiary metric column to `results.tsv`. Until then, skip.

### Simplicity criterion

All else equal, simpler is better. A tiny quality gain that adds 50 lines of fragile parsing is probably not worth it. Deleting code while keeping tests green is a win.

## Logging results

Log every experiment to `results.tsv` (tab-separated).

```
commit	branch	unit_pass	integration_pass	integration_total	status	description
```

1. Short git commit hash (7 chars), or `baseline` for the first row
2. Branch name (e.g. `feature/html`)
3. `cargo test --lib` pass count
4. Integration tests passed (full run)
5. Integration tests total (full run)
6. `keep`, `discard`, or `crash`
7. Short description of what changed

Example:

```
commit	branch	unit_pass	integration_pass	integration_total	status	description
4d64ef4	feature/html	96	15	17	baseline	HTML converter shipped; RTF tests fail until merge
a1b2c3d	feature/html	97	17	17	keep	fix HTML li spacing (no blank line between items)
b2c3d4e	feature/html	96	15	17	discard	nested table parser (regressed xlsx unrelated)
```

## The experiment loop

Runs on a feature branch (e.g. `feature/html`) or `autoresearch/<tag>` cut from master.

LOOP FOREVER:

1. Read git state: branch, last kept commit, `results.tsv` history.
2. Pick one focused hypothesis (converter bug, missing fixture assertion, markdown normalization, etc.).
3. Edit only the EDITABLE surface.
4. `/opt/homebrew/bin/git add <changed files> && /opt/homebrew/bin/git commit -m "experiment: <description>"`
5. Run `./scripts/autoresearch-verify.sh full > run.log 2>&1`
6. Parse results: `grep -E '^(unit_pass|integration_pass|integration_total):' run.log`
7. If empty or build failed: `tail -n 40 run.log`, fix trivial bugs once, else log `crash` and `git reset --hard <last kept commit>`.
8. Append row to `results.tsv`.
9. **Keep** if `integration_pass` did not decrease AND (`unit_pass` increased OR secondary fixture quality improved with same pass count).
10. **Discard** otherwise: `git reset --hard <last kept commit>`.

**Timeout:** If `full` verify exceeds 10 minutes, kill and treat as `crash`.

**NEVER STOP** (once loop started): do not ask the human whether to continue. Run until manually interrupted. If stuck, re-read spec docs, inspect failing fixtures, try smaller diffs.

## Quick reference commands

```bash
# Baseline
./scripts/autoresearch-verify.sh full

# Fast inner loop while hacking
cargo test --lib -p agentready-core
cargo test -p agentready convert_minimal_html

# Inspect output
cargo run -- convert examples/sample-input/ebooks/minimal.html --output /tmp/ar-out

# Revert failed experiment
git reset --hard <last-kept-commit>
```

## Current phase targets

| Phase | Branch | Status |
|-------|--------|--------|
| XLSX | `feature/xlsx` | Done |
| PPTX | `feature/pptx` | Done |
| RTF | `feature/rtf` | Done (merge to master pending) |
| HTML | `feature/html` | In progress — text-only converter |
| KPI drop-in | `feature/kpi-drop-in` | Spec not in docs yet |

## Overnight loop (Claude Code / Codex)

Paste into a new agent session:

```
Read program.md in the AgentReady repo root. You are on feature/html.
1) Run ./scripts/autoresearch-verify.sh full and log baseline to results.tsv if empty.
2) Enter the experiment loop in program.md. Never stop until I interrupt you.
3) Focus on HTML converter quality (lists, links, tables, whitespace) and agent_markdown normalization.
4) Use /opt/homebrew/bin/git for commits. Keep only improvements that pass full verify.
```
