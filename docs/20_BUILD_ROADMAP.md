# 20 — Build Roadmap

## Phase 0 — Repo and CI

- monorepo structure
- Rust workspace
- Vite app
- Fastify server
- CI checks
- sample fixtures

## Phase 1 — Rust core and CLI skeleton

- file detection
- safe filename handling
- job model
- JSON output
- folder and zip generation

## Phase 2 — TXT, Markdown, CSV

- text conversion
- Markdown preservation
- CSV to Markdown and CSV copy

## Phase 3 — DOCX, PDF, XLSX

- document conversion
- spreadsheet sheet handling
- partial conversion warnings

## Phase 4 — Export package

- README
- index
- conversion report
- frontmatter
- assets folder when needed

## Phase 5 — Fastify server

- upload endpoint
- temp jobs
- call CLI
- cancel
- preview
- download
- cleanup

## Phase 6 — React UI

- upload
- selected files
- progress
- results
- preview
- download
- start over

## Phase 7 — Hardening

- tests
- fixtures
- error mapping
- privacy/log checks
- hosted beta readiness

## Cut-lines

Do not add OCR, website scanning, AI summaries, accounts, billing, embeddings, JSONL chunks, or persistent storage to V1.


---

## Latest roadmap update from Q107–Q261

See `docs/21_DECISIONS_Q107_Q261.md` for the current build cut line.

Current build order:

1. Build `agentready-core`.
2. Build `agentready-cli` around the core.
3. Support TXT, Markdown, CSV, and DOCX first.
4. Produce the Simple Export folder and zip.
5. Implement quality statuses and conversion-report.md.
6. Add local web UI after the CLI path works.
7. Prepare hosted private beta later with access codes.

Do not block the first local build on PDF, XLSX, payments, donations, advanced export, project type selector, sensitive-file mode, anonymization, accounts, or persistent workspaces.
