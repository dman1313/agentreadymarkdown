# Spec: Ebook Agent Export (MOBI + AZW3)

- **Status:** Approved — MOBI first (2026-06-07)
- **Owner:** Dwayne Primeau · **Agent:** cursor · **Date:** 2026-06-07

## Goal

Complete AgentReady V1 so any **DRM-free ebook or document** the user owns converts to **agent-ready Markdown** in the standard export package (`documents/`, `index.md`, zip) — including **MOBI** and **AZW3**, building on EPUB/PDF work already in tree.

## Background / context

- **Product SDD** lives in `docs/` (same pattern as [TeamConnect v2/docs SUMMARY](https://github.com/dman1313/TeamConnect/commit/3a9919b889e8a7b561211ec5359e8a0c778140c2)).
- Rust pipeline exists: `job::run_job` → converters → `export::create_export`.
- **Already implemented (uncommitted):** EPUB, PDF text quality, serve UI, `serve.sh`, legal notice (`docs/23`).
- **Gap:** MOBI, AZW3, AZW routing; ebook integration tests; README still lists MOBI/AZW as non-goals.

## Requirements

### Must

- Support **DRM-free** `.mobi`, `.azw3` (and route `.azw` where detectable) through CLI and web UI.
- Reject **encrypted / DRM-protected** ebooks with `PasswordProtected` and legal-aligned copy (`docs/23`).
- Apply `text_quality` heuristics on extracted ebook text (same bar as PDF).
- Wire each format through: converter → `validation.rs` → `job.rs` → `serve.rs` → `index.html` → `README.md`.
- Export package unchanged structurally (`docs/10_EXPORT_PACKAGE.md`).
- `cargo test` passes; no `unwrap()` on user-facing paths.
- Legal notice remains on upload UI and in export `README.md`.

### Should

- Shared `html_to_markdown` module if EPUB + AZW3 duplication exceeds ~80 lines.
- CLI integration test for EPUB; MOBI/AZW3 test with minimal fixture or `#[ignore]` + manual QA doc.
- `examples/sample-input/ebooks/README.md` explaining how to generate DRM-free test files (no copyrighted books in repo).

## Success criteria

1. User drops own DRM-free MOBI/AZW3 into `./scripts/serve.sh` UI → preview shows readable Markdown → zip downloads with `index.md` + `documents/*.md`.
2. `cargo run -- convert ./book.mobi --output ./out` exits 0 with markdown in `out/documents/`.
3. DRM Kindle file → failed status, user message mentions DRM-free requirement.
4. `cargo test` — 0 failures.
5. `docs/23`, `docs/13`, README aligned with shipped formats.

## Out of scope

- OCR for scanned PDFs
- DRM removal, decryption, password cracking
- MOBI/AZW3 **pirated** or **rental** content — user must own or be authorized
- XLSX, audio, website scraping, cloud processing, AI rewriting
- Reviving `apps/web` / Fastify / React UI
- Calibre CLI dependency **unless** Dwayne approves after failed pure-Rust spike

## Decisions (2026-06-07)

1. **MOBI first** — AZW3 in a follow-up pass after MOBI ships.
2. **Rust preferred** — use lightest practical approach; Calibre CLI acceptable only if Rust path is too heavy.
3. **Commit now** — baseline (EPUB, text quality, serve, legal) committed before MOBI work.
4. **Test files** — Dwayne will supply DRM-free local samples later; use synthetic/minimal fixtures in repo for now.
5. **XLSX** — unchanged; out of scope for this spec.

## Links

- `docs/SUMMARY.md` — product SDD at a glance
- `docs/10_EXPORT_PACKAGE.md` — export contract
- `docs/08_CONVERSION_PIPELINE.md` — pipeline
- `docs/23_CONTENT_OWNERSHIP_AND_LEGAL.md` — legal
- `docs/PROMPT-AUTOBUILD.md` — agent execution prompt
- `docs/PLAN-V1-EBOOK-AGENT-EXPORT.md` — phased roadmap (superseded by this spec's plan.md for execution)
- `crates/agentready-core/src/converters/epub.rs` — ebook template
