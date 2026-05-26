# AgentReady Codex Build Prompt

You are building AgentReady V1 from a spec-driven design package.

## Source of truth

Read these files in order:

1. `docs/SUMMARY.md`
2. `docs/00_OVERVIEW.md` through `docs/20_BUILD_ROADMAP.md`
3. `docs/21_DECISIONS_Q107_Q261.md`
4. `docs/22_DECISIONS_EDGE_CASES.md`
5. `docs/adr/`
6. `AGENTREADY_V1_SDD_MASTER.md`
6. `examples/`

If code and spec disagree, stop and report the conflict.

## Build priorities

1. Make the V1 user flow work end-to-end.
2. Keep the UI simple for non-technical users.
3. Preserve privacy rules.
4. Keep Rust conversion logic separate from web UI.
5. Maintain deterministic folder structure and stable JSON contracts.
6. Do not add out-of-scope features.

## Do not build in V1

- OCR
- website scanning
- AI summaries
- accounts
- billing
- persistent workspaces
- vector database creation
- embeddings
- JSONL chunks by default
- GPL dependencies without explicit approval


## Latest build directive

Treat `docs/21_DECISIONS_Q107_Q261.md` and `docs/22_DECISIONS_EDGE_CASES.md` as the latest decision registry. If it conflicts with an older file, follow `docs/21` and `docs/22` and note the conflict.

Build order:

1. Rust core engine.
2. Rust CLI.
3. TXT, Markdown, CSV, and DOCX conversion first.
4. Simple Export folder and zip.
5. Quality statuses and conversion report.
6. Local web UI after CLI works.
7. Hosted private beta later with access codes.

Do not build payments, donations, anonymization, sensitive-file mode, project-specific exports, or advanced developer export in the first local V1 build unless explicitly instructed.
