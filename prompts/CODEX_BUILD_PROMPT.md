# Codex Build Prompt: AgentReady V1

You are building **AgentReady V1** for HumanGoodAI.

Use `AGENTREADY_V1_SDD_MASTER.md` as the source of truth.

## Goal

Build a monorepo for `agentready-knowledge-packager`.

V1 flow:

```text
Upload files -> Convert to Markdown -> Preview Markdown -> Download Simple Export zip
```

## Core stack

1. Rust conversion core
2. Rust CLI
3. Fastify TypeScript server
4. Vite plus React web app

## Critical rules

1. Keep Rust conversion core independent from web UI.
2. No login, billing, persistent storage, AI summaries, OCR, website scanning, embeddings, or vector database in V1.
3. Do not send documents to AI models.
4. Delete source uploads and temp files after conversion, download, cancellation, or cleanup.
5. Do not log document contents.
6. Avoid GPL dependencies unless Dwayne explicitly approves them.
7. Use the SDD error code registry.
8. Continue conversion when some files fail.
9. Never silently hide failed files.
10. Optimize Markdown for agent understanding, not visual layout reproduction.

## First implementation steps

1. Create monorepo structure.
2. Create Rust workspace with `agentready-core` and `agentready-cli`.
3. Create TypeScript apps: `apps/server` and `apps/web`.
4. Add sample fixtures.
5. Implement file validation, filename sanitizer, and error registry first.
6. Implement TXT, Markdown, and CSV conversion before PDF/DOCX/XLSX.
7. Add tests and CI early.

## Acceptance criteria

Follow section 20 of `AGENTREADY_V1_SDD_MASTER.md`.


## Latest build directive

Treat `docs/21_DECISIONS_Q107_Q261.md` as the latest decision registry. If it conflicts with an older file, follow `docs/21` and note the conflict.

Build order:

1. Rust core engine.
2. Rust CLI.
3. TXT, Markdown, CSV, and DOCX conversion first.
4. Simple Export folder and zip.
5. Quality statuses and conversion report.
6. Local web UI after CLI works.
7. Hosted private beta later with access codes.

Do not build payments, donations, anonymization, sensitive-file mode, project-specific exports, or advanced developer export in the first local V1 build unless explicitly instructed.
