# AgentReady V1 — Improvement Rubric Prompt

> **How to use:** Give this entire file to a coding agent (Claude Code, Codex, Goose, etc.) as a prompt. The agent will evaluate the project and produce a scored improvement rubric. You can ask the agent to focus on specific modules by saying "only evaluate modules 1 and 3" or "skip module 6."

---

## Your task

You are evaluating **AgentReady V1**, a tool that converts documents into clean Markdown optimized for AI agents. Evaluate the full project and produce a scored improvement rubric.

## What AgentReady does

AgentReady takes user-uploaded files (TXT, Markdown, CSV, DOCX, PDF) and converts them into clean Markdown with YAML frontmatter, structured into an export folder and zip. The stack is Rust (core + CLI), Fastify (server), and React (web UI).

## Key files to read

Start with:
- `CLAUDE.md` — project conventions and build commands
- `docs/SUMMARY.md` — spec overview
- `docs/22_DECISIONS_EDGE_CASES.md` — latest decisions (overrides older docs when conflicting)

Then explore:
- `crates/agentready-core/src/` — converters, models, validation, export
- `crates/agentready-cli/src/main.rs` — CLI entry point
- `apps/server/src/` — Fastify server
- `apps/web/src/` — React frontend
- `docs/` — 22 spec sections + ADRs
- `examples/` — sample input and expected output fixtures

## Commands to run

```bash
cargo check          # Compile (should be zero warnings)
cargo test           # Run tests
cargo run -- convert <input> --output <dir>   # Test CLI
```

## Scoring scale

| Score | Meaning |
|-------|---------|
| 5 | Excellent — production-ready, no meaningful gaps |
| 4 | Good — minor issues, easy fixes |
| 3 | Adequate — functional but has notable gaps |
| 2 | Needs work — significant issues affecting quality |
| 1 | Poor — missing, broken, or fundamentally insufficient |

---

## Module 1: Agent-Readiness of Output

Evaluate how well the Markdown output serves AI agent consumers.

**Check:**
- Does every converted file have YAML frontmatter (source_file, source_type, converted_by, status)?
- Are headings, lists, tables, and formatting preserved from DOCX?
- Do multi-page PDFs have page break markers?
- Is CSV converted to proper Markdown tables with raw CSV copy in `data/`?
- Does `manifest.json` exist with structured per-file metadata?
- Are partial-conversion warnings embedded in the output?
- Is the output token-efficient (no redundant whitespace, no formatting noise)?
- Can an agent navigate the export package programmatically?

**Read:** `crates/agentready-core/src/export.rs`, `crates/agentready-core/src/converters/*.rs`

## Module 2: Code Quality & Architecture

Evaluate the Rust codebase health.

**Check:**
- Does `cargo check` produce zero warnings?
- Are error types well-structured (thiserror, ErrorCode enum)?
- Is the module boundary clean (core vs CLI)?
- Are there any unwraps on user-facing paths?
- Is there dead code or unused dependencies?
- Is the separation of concerns clear (converters, validation, export, models)?
- Are converters consistent in their interface (`Result<ConversionResult, ErrorCode>`)?

**Read:** `crates/agentready-core/src/models.rs`, `crates/agentready-core/src/lib.rs`

## Module 3: Spec Compliance

Evaluate how well the code matches the spec docs.

**Check:**
- Read `docs/SUMMARY.md` then spot-check 3-4 spec docs against the implementation
- Does the CLI match `docs/06_CLI_CONTRACTS.md`? (flags, exit codes, JSON output)
- Does the server match `docs/07_SERVER_CONTRACTS.md`? (all 6 routes)
- Does the export match `docs/05_OUTPUT_MODEL.md`? (fields, statuses)
- Do error codes match `docs/11_ERRORS_AND_PARTIAL_SUCCESS.md`?
- Does the UI match `docs/12_UI_UX_SPEC.md`? (screens, copy, flow)
- Does `docs/22_DECISIONS_EDGE_CASES.md` have any unimplemented decisions?

**Read:** `docs/06_CLI_CONTRACTS.md`, `docs/07_SERVER_CONTRACTS.md`, `docs/05_OUTPUT_MODEL.md`, `docs/11_ERRORS_AND_PARTIAL_SUCCESS.md`

## Module 4: Test Coverage & Reliability

Evaluate test quality and coverage.

**Check:**
- How many tests exist? What do they cover?
- Are all converters tested? (txt, md, csv, docx, pdf)
- Is the export pipeline tested? (folder structure, frontmatter, zip, manifest)
- Are edge cases tested? (empty files, BOM, encoding, duplicates, large files)
- Are CLI integration tests present? (exit codes, JSON output, flags)
- Do fixture tests compare actual output against expected output?
- Run `cargo test` — do all tests pass?

**Read:** All `#[cfg(test)]` blocks, `examples/expected-output/`

## Module 5: Performance & Security

Evaluate performance characteristics and security posture.

**Check:**
- Is there a per-file timeout (spec: 30 seconds)?
- Is zip bomb protection implemented for DOCX?
- Are file size limits enforced?
- Is there any risk of unbounded memory usage?
- Is the output directory overwrite-protected?
- Are hidden/system files filtered during directory walking?
- Does the server validate uploads before processing?
- Is there concurrency support (or is it sequential)?

**Read:** `crates/agentready-core/src/validation.rs`, `crates/agentready-cli/src/main.rs`, `apps/server/src/routes/jobs.ts`

## Module 6: UX & Accessibility

Evaluate the user experience across CLI, server, and web UI.

**Check:**
- Does the upload page show the tagline, supported types, limits, and privacy message?
- Is the preview screen functional with file sidebar, rendered/raw toggle, copy button?
- Are CLI error messages user-friendly?
- Is `--help` clear and complete?
- Does the web UI have proper ARIA labels and keyboard support?
- Is the cancel endpoint available for long-running conversions?
- Does the "start over" flow clean up server state?

**Read:** `apps/web/src/App.tsx`, `apps/web/src/index.css`, `apps/server/src/routes/jobs.ts`

---

## Output format

Produce a Markdown document with this structure:

```markdown
# AgentReady V1 — Improvement Rubric

**Date:** <today>
**Evaluator:** <agent name>
**Commit:** <latest git hash>

## Summary

| Module | Score | Status |
|--------|-------|--------|
| 1. Agent-Readiness | ?/5 | ... |
| 2. Code Quality | ?/5 | ... |
| 3. Spec Compliance | ?/5 | ... |
| 4. Test Coverage | ?/5 | ... |
| 5. Performance & Security | ?/5 | ... |
| 6. UX & Accessibility | ?/5 | ... |

## Module 1: Agent-Readiness of Output — ?/5

### Strengths
- ...

### Gaps
- ...

### Actions (priority order)
1. [High] ...
2. [Medium] ...
3. [Low] ...

(Repeat for each module)

## Cross-Cutting Priority Ranking

Rank all actions across all modules by impact:

1. [Module N] Action description — effort: S/M/L
2. ...
```
