# SDD at a glance

**22 documents · 5 ADRs · 2 implementation prompts · 1 glossary · sample input/output fixtures** — read top-to-bottom. Each section answers one main question for a coding agent.

| # | Doc | Answers |
|---|---|---|
| 00 | `OVERVIEW` | What is AgentReady in one page? |
| 01 | `VISION_AND_PRD` | Why does it exist, what does V1 ship, and for whom? |
| 02 | `PERSONAS_AND_JOURNEYS` | Who is the first user and what does their flow look like? |
| 03 | `STATE_MACHINE` | What lifecycle does every upload batch move through? |
| 04 | `SYSTEM_ARCHITECTURE` | Rust engine, TypeScript web UI, server, and dataflow. |
| 05 | `OUTPUT_MODEL` | Export entities, statuses, folder structure, and invariants. |
| 06 | `CLI_CONTRACTS` | Public CLI surface and JSON output for the server. |
| 07 | `SERVER_CONTRACTS` | Fastify routes, temp jobs, validation, cancellation, cleanup. |
| 08 | `CONVERSION_PIPELINE` | How supported file types become clean Markdown/data outputs. |
| 09 | `MARKDOWN_STANDARD` | What good agent-ready Markdown must look like. |
| 10 | `EXPORT_PACKAGE` | What goes inside the Simple Export zip. |
| 11 | `ERRORS_AND_PARTIAL_SUCCESS` | Error codes, user messages, and partial conversion behavior. |
| 12 | `UI_UX_SPEC` | Screen-by-screen behavior and interaction states. |
| 13 | `INTERFACE_COPY` | Exact copy blocks for upload, privacy, export, and Markdown. |
| 14 | `ACCESSIBILITY_AND_I18N` | Non-technical language, keyboard, screen reader, and future translation rules. |
| 15 | `SECURITY_AND_PRIVACY` | Temporary processing, no AI in V1, safe logging, blocked files. |
| 16 | `OBSERVABILITY_AND_EVALS` | Logs, conversion reports, quality checks, and eval criteria. |
| 17 | `TESTING_STRATEGY` | Unit, fixture, CLI, server, and UI test requirements. |
| 18 | `DEPLOYMENT` | Local-first development and simple hosted beta expectations. |
| 19 | `DEMO_SCRIPT` | The 2-minute V1 demo flow. |
| 20 | `BUILD_ROADMAP` | Build order, milestones, and cut-lines. |
| 21 | `DECISIONS_Q107_Q261` | New product, UX, beta, credits, donation, and future anonymization decisions. |
| 22 | `DECISIONS_EDGE_CASES` | Duplicate resolution, partial success definitions, memory, timeouts, encodings, security, and concurrency. |
| 23 | `CONTENT_OWNERSHIP_AND_LEGAL` | Ownership warranty, DRM-free ebooks, disclaimer, export legal notice. |

**`sdd/specs/`** — feature-level SDD (spec → plan → tasks per build). Active: `2026-06-07-ebook-agent-export`. See `sdd/README.md`.  
**`docs/adr/`** — five important decisions.  
**`docs/prompts/`** — build prompts for Codex/VibeCode and future OpenClaw agents.  
**`docs/GLOSSARY.md`** — one source of truth for product and technical terms.  
**`docs/CHANGELOG.md`** — package update history.  
**`AGENTREADY_V1_SDD_MASTER.md`** — full master document retained for one-file ingestion.

> Read in order. `docs/21_DECISIONS_Q107_Q261.md` and `docs/22_DECISIONS_EDGE_CASES.md` are the latest decision addendums and override older sections when they conflict. The spec is the source of truth. If code and spec disagree, update one of them explicitly.
