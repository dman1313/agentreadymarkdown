# Spec-Driven Development — AgentReady

Follow the fleet protocol: **spec → plan → tasks → execute → verify.**

Full protocol: vault `sdd/README.md`  
Templates: `sdd/templates/` (copy into each spec folder)

## Two layers (like TeamConnect)

| Layer | Location | Purpose |
|-------|----------|---------|
| **Product SDD** | `docs/00`–`23`, `docs/SUMMARY.md` | Whole product — read top-to-bottom (mirrors [TeamConnect v2/docs](https://github.com/dman1313/TeamConnect/tree/main/v2/docs)) |
| **Feature SDD** | `sdd/specs/YYYY-MM-DD-<slug>/` | One build effort — spec.md, plan.md, tasks.md |

## Active specs

| Date | Slug | Status |
|------|------|--------|
| 2026-06-07 | [ebook-agent-export](./specs/2026-06-07-ebook-agent-export/spec.md) | Draft — awaiting Dwayne 👍 |

## Rules

- Spec before code. Get 👍 on `spec.md` before executing `tasks.md`.
- One feature = one dated folder.
- Verify with evidence (`cargo test`, manual upload, diff stat).
- ADRs for irreversible choices → `docs/adr/`.
