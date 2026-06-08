# Plan — AgentReady V1: Ebook Agent Export

**Date:** 2026-06-07  
**Owner:** Dwayne Primeau / HumanGoodAI  
**Workspace:** `/Volumes/M2 Media/Coding Dwayne/agentready Mark down-v1-sdd-v2style-package`  
**Status:** Superseded for execution by `sdd/specs/2026-06-07-ebook-agent-export/` (spec → plan → tasks). Keep as narrative overview.

---

## 1. Executive summary

AgentReady converts documents into **agent-ready Markdown** packaged in a standard export folder (`documents/`, `index.md`, `manifest.json`, zip). The core pipeline exists in Rust. **EPUB, PDF, text quality, serve UI, and legal disclaimers are largely built but uncommitted.**

**Remaining product gap:** DRM-free **MOBI** and **AZW3** (plus optional **AZW** routing) so your ebook library can feed agent databases without Calibre manual steps.

**Recommended approach:** Three phases over ~3–5 focused sessions — stabilize → MOBI → AZW3 → verify → ship.

---

## 2. Goal (what “done” looks like)

A user (you) can:

1. Drop **TXT, MD, CSV, DOCX, PDF, EPUB, MOBI, AZW3** into the local UI or CLI.
2. Get a zip/folder where **`index.md` + `documents/*.md`** are ready for Claude Projects, RAG, Obsidian, etc.
3. See **clear failures** for DRM, scanned PDFs, and garbage text — never mojibake in preview.
4. Read **legal notice** on upload and in every export `README.md` (ownership + DRM-free + no liability).

**Out of scope:** OCR, DRM removal, XLSX (unless you reprioritize), cloud, accounts, AI rewriting.

---

## 3. Current state inventory

### Done (in tree, mostly uncommitted)

| Area | Evidence |
|------|----------|
| EPUB converter | `converters/epub.rs` — spine, HTML→MD, zip-bomb limits |
| PDF quality | `text_quality.rs` + `pdf.rs` garbage rejection |
| Web UI | Drag-drop fixes, EPUB accept, preview garbage UX |
| Serve API | 50 MB body limit, preview endpoint |
| `serve.sh` | One-command local UI |
| Legal | `docs/23_CONTENT_OWNERSHIP_AND_LEGAL.md`, UI + export README |
| Autobuild prompt | `docs/PROMPT-AUTOBUILD.md` |
| Tests | 55 passing (`cargo test`) |

### Not started

| Area | Notes |
|------|-------|
| MOBI (`.mobi`) | README still says “not in V1” |
| AZW3 (`.azw3`) | Same |
| AZW (`.azw`) | Optional router |
| Ebook test fixtures | `examples/sample-input/ebooks/` empty/missing |
| Integration test for EPUB/MOBI CLI | Only TXT fixtures in `integration.rs` |

### Architecture (keep)

```
Upload (UI/CLI) → job::run_job → converters → export → zip
                     ↑ validation, 30s timeout, text_quality
```

Single pipeline — **do not** revive `apps/web` or Fastify server.

---

## 4. Phased roadmap

### Phase 0 — Stabilize baseline (½ session)

**Why:** Clean foundation before new formats.

| # | Task | Verify |
|---|------|--------|
| 0.1 | Review uncommitted diff; fix any clippy warnings | `cargo clippy` |
| 0.2 | Run full test suite | `cargo test` |
| 0.3 | Smoke UI: `./scripts/serve.sh --no-open` + upload EPUB/PDF | Manual |
| 0.4 | **Commit** “EPUB + text quality + serve + legal” (when Dwayne approves) | `git log -1` |

**Exit:** One commit you trust as the ebook baseline.

---

### Phase 1 — MOBI converter (1–2 sessions)

**Research spike (first 30 min)**

| Option | Pros | Cons |
|--------|------|------|
| Pure Rust crate (`mobi`, etc.) | Local-first, no deps | License check; format edge cases |
| Reuse EPUB path for KF8-in-MOBI | Less code | Not all MOBI files |
| Calibre CLI fallback | Battle-tested | Violates “pure Rust” spirit; install burden |

**Decision rule:** Pure Rust first. Calibre only if spike fails and Dwayne approves.

**Implementation checklist**

| # | Task | Files |
|---|------|-------|
| 1.1 | Spike: evaluate crates.io MOBI parsers; pick one (MIT/Apache only) | notes in `mobi.rs` header |
| 1.2 | `convert_mobi()` — extract text by chapter/record | `converters/mobi.rs` |
| 1.3 | DRM sniff — reject encrypted → `PasswordProtected` | same |
| 1.4 | `text_quality` pass on output | same |
| 1.5 | Wire: `mod.rs`, `validation.rs`, `job.rs`, `serve.rs`, `index.html` | 5 files |
| 1.6 | Update `ErrorCode` messages if needed | `models.rs` |
| 1.7 | Unit tests + minimal synthetic fixture | `mobi.rs` tests |
| 1.8 | Update README + `docs/08_CONVERSION_PIPELINE.md` | docs |

**Exit:** `cargo run -- convert sample.mobi --output /tmp/out` → `documents/*.md` with frontmatter.

---

### Phase 2 — AZW3 + AZW routing (1 session)

AZW3 (KF8) is structurally close to EPUB (zip + HTML/XHTML).

| # | Task | Approach |
|---|------|----------|
| 2.1 | Magic-byte / container sniff | Shared `kindle_sniff.rs` or inline |
| 2.2 | `convert_azw3()` — extract content, reuse HTML→MD from `epub.rs` | Extract shared `html_to_markdown` to `converters/html_md.rs` if duplication > ~80 lines |
| 2.3 | `.azw` router — delegate to MOBI or AZW3 | `job.rs` match arm |
| 2.4 | DRM reject (same as MOBI) | |
| 2.5 | Wire + tests + UI accept list | extensions: `.azw3`, `.azw` |
| 2.6 | Legal copy already covers “ebooks” — add MOBI/AZW3 to UI supported line | `index.html` |

**Exit:** DRM-free AZW3 converts; DRM files fail with legal-aligned message.

---

### Phase 3 — Integration hardening (½–1 session)

| # | Task | Verify |
|---|------|--------|
| 3.1 | Add `examples/sample-input/ebooks/README.md` — how to generate DRM-free test files (Calibre, public domain) | No copyrighted books in repo |
| 3.2 | CLI integration test: EPUB round-trip | `integration.rs` |
| 3.3 | Optional: MOBI/AZW3 integration test with committed minimal fixture | `#[ignore]` if fixture too large |
| 3.4 | Export package audit vs `docs/10_EXPORT_PACKAGE.md` | checklist |
| 3.5 | Update `docs/PROMPT-AUTOBUILD.md` status column (MOBI/AZW3 ✅) | |

---

### Phase 4 — Polish & release prep (½ session)

| # | Task |
|---|------|
| 4.1 | README: remove “MOBI/AZW not in V1”; update supported table |
| 4.2 | `CLAUDE.md` + `docs/20_BUILD_ROADMAP.md` add ebook phase note |
| 4.3 | Manual matrix (below) on real files from your library |
| 4.4 | Tag release or push when ready |

---

## 5. Shared module refactor (optional, decide in Phase 2)

If MOBI and AZW3 both need HTML→Markdown:

```
converters/
  html_md.rs      # shared html_to_markdown, strip_tag_block
  epub.rs         # uses html_md
  mobi.rs
  azw3.rs         # uses html_md
```

**Trigger:** Only if duplicating >80 lines. Don’t refactor preemptively in Phase 0.

---

## 6. Legal & compliance (complete — maintain)

| Requirement | Where |
|-------------|-------|
| Ownership + DRM-free notice | Upload UI |
| Full disclaimer | Export `README.md` |
| No DRM circumvention | Converters reject encrypted |
| Spec | `docs/23_CONTENT_OWNERSHIP_AND_LEGAL.md` |

**Build rule:** Any new ebook format must reject DRM before extracting text.

---

## 7. Verification matrix (manual QA)

Run after Phase 2–3 on **your own DRM-free files only**:

| Input | Expected status | Notes |
|-------|-----------------|-------|
| Own EPUB | Good | chapters in order |
| Own PDF (text) | Good/Partial | no garbage preview |
| Own MOBI (DRM-free) | Good/Partial | |
| Own AZW3 (DRM-free) | Good/Partial | |
| Kindle DRM purchase | Failed | `PasswordProtected` |
| Scanned PDF | Failed | `NoReadableText` + helpful message |
| 51 MB file | Failed | `FileTooLarge` |
| Export zip | — | `README.md` has legal notice; `index.md` links all docs |

---

## 8. Risks & mitigations

| Risk | Mitigation |
|------|------------|
| No good Rust MOBI crate | Time-box spike; report alternatives to Dwayne |
| AZW3 variants differ | Fixture-driven tests; partial + warning |
| User uploads DRM Kindle files | Already rejected + legal copy |
| Copyright exposure | Legal notice; no DRM stripping; user warranty |
| Scope creep (XLSX, OCR) | Cut-line in this plan; defer unless asked |

---

## 9. Decisions needed from you

Before build starts, confirm:

1. **MOBI/AZW3 priority** — build both now, or MOBI first?
2. **Calibre fallback** — allowed if pure Rust fails, or hard no?
3. **Commit timing** — commit Phase 0 (current work) before MOBI, or one big commit at end?
4. **Test ebooks** — will you provide DRM-free sample files locally (not committed), or use public-domain only?
5. **XLSX** — still out of scope?

---

## 10. Suggested execution order (for Cursor / MacH)

```
Session 1: Phase 0 (stabilize + commit approval)
Session 2: Phase 1 (MOBI spike + implement)
Session 3: Phase 2 (AZW3 + AZW router)
Session 4: Phase 3–4 (integration tests + manual QA + docs)
```

**MacH dispatch template:**

```
Dispatch to Cursor — workspace: agentready Mark down-v1-sdd-v2style-package.
Execute docs/PLAN-V1-EBOOK-AGENT-EXPORT.md Phase {N} only. Follow docs/23 legal rules. Report evidence; do not commit unless asked.
```

---

## 11. Success criteria

- [ ] All target formats in CLI + UI + validation
- [ ] `cargo test` green
- [ ] Legal notice on upload + in export
- [ ] DRM files rejected, never decrypted
- [ ] Your own DRM-free ebooks convert to usable `documents/*.md`
- [ ] README and spec aligned with reality

---

## 12. Related docs

| Doc | Purpose |
|-----|---------|
| `docs/PROMPT-AUTOBUILD.md` | Agent execution prompt |
| `docs/23_CONTENT_OWNERSHIP_AND_LEGAL.md` | Legal copy |
| `docs/10_EXPORT_PACKAGE.md` | Export contract |
| `docs/08_CONVERSION_PIPELINE.md` | Pipeline behavior |
| `AGENTREADY_V1_SDD_MASTER.md` | Master spec (note: EPUB listed as non-goal — update when shipping) |
