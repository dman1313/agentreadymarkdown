# AgentReady — Autonomous Build & Verify Prompt

**Copy everything below the line into Cursor, Claude Code, or MacH → Cursor inbox dispatch.**

Use this when you want an agent to **build, extend, and verify** AgentReady without hand-holding: any document in → clean Markdown out → agent-ready export folder.

---

## PROMPT START

You are building **AgentReady V1** — a local-first Rust app that converts documents into **agent-ready Markdown** packaged for RAG, Claude projects, Obsidian, and similar workflows.

**Workspace (absolute path):**
```
/Volumes/M2 Media/Coding Dwayne/agentready Mark down-v1-sdd-v2style-package
```

**Product promise:** Upload files → preview Markdown → download a zip/folder an AI agent can use as a knowledge base.

**Company:** HumanGoodAI. **Spec is law:** read `docs/SUMMARY.md` first, then `docs/10_EXPORT_PACKAGE.md`, `docs/08_CONVERSION_PIPELINE.md`, `docs/22_DECISIONS_EDGE_CASES.md`.

---

### 1. Mission

Build and verify a program that:

1. Accepts **any supported document** (single file, multi-file, folder via CLI).
2. Converts each file to **clean Markdown** (not a visual replica — optimized for agent reading).
3. Writes outputs into a **standard export folder** suitable as an agent database:
   ```
   output/
     README.md              # human guide
     index.md               # agent navigation table (LLMs start here)
     conversion-report.md   # per-file audit
     manifest.json          # machine metadata
     documents/*.md         # converted content + YAML frontmatter
     data/                  # raw CSV copies where applicable
   ```
4. Produces a matching **`.zip`** for download.
5. Runs **entirely locally** — no cloud upload, no AI rewriting, no accounts.

---

### 2. Ebook & document formats — target matrix

| Format | Extensions | Priority | Notes |
|--------|------------|----------|-------|
| Plain text | `.txt` | ✅ exists | BOM strip, encoding guess |
| Markdown | `.md`, `.markdown` | ✅ exists | preserve structure |
| CSV | `.csv` | ✅ exists | MD table + raw copy in `data/` |
| Word | `.docx` | ✅ exists | headings, lists, tables |
| PDF | `.pdf` | ✅ exists | text extract; **no OCR**; reject garbage via `text_quality` |
| EPUB | `.epub` | ✅ exists | spine order, HTML→MD, zip-bomb guards |
| MOBI | `.mobi` | 🔴 **build this** | Legacy Kindle; may overlap AZW; handle DRM-free only |
| AZW3 | `.azw3` | 🔴 **build this** | KF8; structurally EPUB-like; DRM-free only |
| AZW | `.azw` | 🟡 optional | Often MOBI wrapper; detect and route |

**User also said `.MOB`** — treat as typo for **`.mobi`** unless sample files prove otherwise.

**Explicit non-goals unless Dwayne asks:** OCR, DRM breaking, password cracking, website scraping, embeddings, vector DB, cloud APIs, AI summaries.

---

### 2b. Legal — content ownership & DRM (mandatory)

Read and implement `docs/23_CONTENT_OWNERSHIP_AND_LEGAL.md` in all user-facing surfaces.

**User warrants:**
- They **own** each file or have **explicit authorization** to convert it.
- All ebooks are **DRM-free** (no Kindle DRM, Adobe ACS, etc.).

**AgentReady / HumanGoodAI:**
- Conversion tool only — **does not verify ownership**, **does not review content**, **accepts no responsibility** for user uploads or use of exports.
- Software provided **"as is"**; user solely responsible for copyright/licensing compliance.
- **Never** implement DRM removal or decryption.

**Must wire into:**
- Upload UI (`static/index.html`) — short notice visible before convert
- Export `README.md` (`export.rs`) — full legal notice in every zip
- `ErrorCode::PasswordProtected` — DRM/encrypted rejection message
- `docs/13_INTERFACE_COPY.md` — canonical copy blocks

---

### 3. Architecture — do not reinvent

```
crates/agentready-core/     converters, validation, text_quality, export, job::run_job
crates/agentready-cli/      `convert` + `serve` (Axum + embedded index.html)
```

**Single pipeline:** CLI and web UI both call `job::run_job` in-process.

**Adding a format requires touching ALL of:**
- `crates/agentready-core/src/converters/{format}.rs` + `mod.rs`
- `crates/agentready-core/src/validation.rs` (allowed extensions)
- `crates/agentready-core/src/job.rs` (`convert_with_timeout` match arm)
- `crates/agentready-cli/src/serve.rs` (upload allowlist)
- `crates/agentready-cli/static/index.html` (accept + copy)
- `crates/agentready-core/src/models.rs` (`ErrorCode::user_message` if needed)
- Tests in converter module + validation tests
- `README.md` supported formats table

**Conventions:**
- Rust edition 2024, `ErrorCode` enum, no `unwrap()` on user paths
- Converters return `Result<ConversionResult, AgentReadyError>`
- 30s per-file timeout (already in `job.rs`)
- 50 MB/file, 25 files/job
- Frontmatter injected in **export layer only** (`export.rs`), not converters
- Use `text_quality::looks_like_garbage` + `readable_text_ratio` on extracted text (see `pdf.rs`, `epub.rs`)

---

### 4. Autonomous workflow — follow in order

#### Phase A — Audit (read before writing)

```bash
cd "/Volumes/M2 Media/Coding Dwayne/agentready Mark down-v1-sdd-v2style-package"
git status
cargo check
cargo test
```

Read these files completely:
- `crates/agentready-core/src/job.rs`
- `crates/agentready-core/src/export.rs`
- `crates/agentready-core/src/text_quality.rs`
- `crates/agentready-core/src/converters/epub.rs` (template for ebook parsing)
- `crates/agentready-core/src/converters/pdf.rs`
- `crates/agentready-cli/src/serve.rs`
- `docs/10_EXPORT_PACKAGE.md`

Log what is **done vs missing**. Do not duplicate working code.

#### Phase B — MOBI converter

**Goal:** `convert_mobi(path) -> ConversionResult`

Research approach (pick best pure-Rust path; document choice in a one-line comment at top of file):

1. **Preferred:** Rust crate that parses MOBI/PalmDOC records (search crates.io: `mobi`, `kindle`, etc.). Evaluate license (no GPL code in product).
2. **Fallback:** If file is KF8/AZW3 inside MOBI container, delegate internally to AZW3/EPUB logic.
3. **Reject clearly:** DRM-protected, encrypted, or unreadable → `ErrorCode::PasswordProtected` or `NoReadableText` with friendly message.

**Implementation checklist:**
- [ ] `converters/mobi.rs` with unit tests (minimal synthetic MOBI fixture or documented test skip + manual verify note)
- [ ] Wire through validation, job, serve, UI, README
- [ ] Chapter/section boundaries → `---` separators like EPUB
- [ ] `text_quality` pass on output
- [ ] Partial warnings when sections skipped

#### Phase C — AZW3 converter

**Goal:** `convert_azw3(path) -> ConversionResult`

AZW3 (KF8) is often a zip-like container with HTML/XHTML content similar to EPUB.

**Approach:**
1. Try reusing EPUB HTML→Markdown path after extracting content manifest.
2. If structure differs, parse KF8 container per format docs (agent: research, cite source in code comment).
3. `.azw` files: sniff magic/header — route to MOBI or AZW3 handler.

**Implementation checklist:**
- [ ] `converters/azw3.rs` (+ optional `azw.rs` router or shared `kindle.rs` module)
- [ ] Same wiring as MOBI
- [ ] Tests + garbage rejection

#### Phase D — Integration & UI

- [ ] `serve.sh` still works: `./scripts/serve.sh`
- [ ] Upload EPUB, PDF, MOBI, AZW3 in UI — preview + zip download
- [ ] CLI: `cargo run -- convert ./ebooks --output ./out --json`
- [ ] Error messages mention all supported ebook types

#### Phase E — Verify (evidence required)

Run and paste summaries:

```bash
cargo check
cargo test                    # all pass
cargo clippy -p agentready-core -p agentready -- -D warnings  # fix or justify
cargo run -- convert examples/sample-input/clean --output /tmp/ar-test-out
./scripts/serve.sh --no-open --port 3099   # smoke: POST a file, poll job, preview
```

**Manual matrix** (create minimal fixtures if none exist):

| Input | Expected |
|-------|----------|
| `.txt` | Good status, frontmatter in `documents/` |
| `.pdf` (text-based) | Good or Partial; no mojibake in preview |
| `.epub` | Chapters in spine order |
| `.mobi` (DRM-free sample) | Readable Markdown or clear error |
| `.azw3` (DRM-free sample) | Readable Markdown or clear error |
| Scanned/image PDF | Failed + `NoReadableText` message (not garbage in preview) |
| 51 MB file | `FileTooLarge` |
| `.exe` | `UnsupportedFile` |

#### Phase F — Document & handoff

- [ ] Update `README.md` supported formats (remove "MOBI/AZW not in V1" if implemented)
- [ ] Update `CLAUDE.md` if build commands changed
- [ ] Do **not** commit unless Dwayne asks — report `git diff --stat` instead

---

### 5. Export package contract (agents consuming output)

Every successful job must satisfy `docs/10_EXPORT_PACKAGE.md`:

- `index.md` lists every converted file with path, source type, status
- `conversion-report.md` is human-auditable
- `manifest.json` version `1.0`, `generated_by: agentready-v1`
- Each `documents/*.md` starts with YAML frontmatter (`source_file`, `source_type`, `status`, etc.)
- Zip sits beside folder: `output.zip` or custom `--zip-name`

**Agent database usage:** Dwayne feeds the `documents/` folder (or whole zip) to Claude Projects, RAG tools, etc. Quality bar = readable prose, clear headings, no binary junk.

---

### 6. Quality rules

1. **Garbage detection** — never export PDF/MOBI/EPUB mojibake; fail loud (`text_quality.rs`).
2. **Partial success** — job continues if one file fails; export includes report of failures.
3. **Security** — zip-bomb limits on all zip-based formats (see `epub.rs` constants).
4. **No scope creep** — no React revival of `apps/web`, no new databases, no cloud.
5. **Minimal diff** — match existing code style; no over-abstraction.

---

### 7. Failure modes to handle

| Situation | Error code | User message tone |
|-----------|------------|-------------------|
| DRM / encrypted ebook | `PasswordProtected` | "Password-protected or encrypted — not supported in V1" |
| Scanned PDF | `NoReadableText` | Suggest re-export as TXT/DOCX; no OCR in V1 |
| Corrupt file | `ConversionFailed` | Plain language, no stack traces |
| Timeout (>30s) | `TimeoutExceeded` | Suggest smaller file or split |
| Empty/iCloud 0-byte upload | validation message | iCloud hint (already in serve.rs) |

---

### 8. Definition of done

You are finished when:

1. **EPUB, PDF, MOBI, AZW3** all wired end-to-end (CLI + web + validation + tests).
2. `cargo test` — **0 failures**.
3. Export folder structure matches spec and contains navigable `index.md`.
4. You provide Dwayne a short report:
   - What you built
   - What you could not support (e.g. DRM) and why
   - Commands to run UI + CLI
   - List of test evidence

---

### 9. If blocked

- **No DRM-free MOBI/AZW3 test file:** create a minimal fixture using public format docs, or document `cargo test` with ignored integration test + manual Calibre-generated samples in `examples/sample-input/ebooks/` (do not commit copyrighted books).
- **No pure-Rust MOBI library:** propose thinnest acceptable approach to Dwayne in report before shelling out to Calibre CLI.
- **Spec conflict:** `docs/` wins; note conflict in report.

---

### 10. MacH / fleet dispatch (optional)

If dispatched via vault inbox, read first:
```
/Users/dwayne-primeau/Library/Mobile Documents/com~apple~CloudDocs/Agent Memory/Coding/
Agent Inbox/cursor.md
Reference/coding-factory-routing.md
```

Log `milestone` to `ACTIVITY.md` when done.

---

## PROMPT END

---

## Quick dispatch (one-liner for MacH)

```
Dispatch to Cursor — workspace: agentready Mark down-v1-sdd-v2style-package.
Execute docs/PROMPT-AUTOBUILD.md end-to-end: audit repo, implement MOBI + AZW3 ebook converters, wire CLI/UI/validation/tests, verify export package, report evidence. Do not commit unless asked.
```
