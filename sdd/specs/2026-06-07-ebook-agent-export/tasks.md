# Tasks: Ebook Agent Export (MOBI + AZW3)

> Plan: ./plan.md · **Do not start until spec.md is Approved.** Tick `- [x]` as you go.

---

## Phase 0 — Stabilize baseline

### Task 0.1: Verify in-flight work
**Files:** (read-only audit)
- [ ] Run `cargo test` — expect all tests pass (baseline ~55)
- [ ] Run `cargo check`
- [ ] Review `git diff --stat` — confirm EPUB, text_quality, serve, legal present
- [ ] Commit: `feat: EPUB converter, text quality checks, serve UI, legal notice` *(only if Dwayne approves)*

### Task 0.2: UI smoke
**Files:** none
- [ ] `./scripts/serve.sh --no-open --port 3099`
- [ ] Upload a DRM-free EPUB + text PDF — preview readable, zip downloads
- [ ] Confirm legal notice visible on upload page

---

## Phase 1 — MOBI

### Task 1.1: Research spike
**Files:** notes in `mobi.rs` header
- [ ] Search crates.io for MOBI parsers; check license (MIT/Apache/APACHE-2.0 only)
- [ ] Time-box 30 min; record chosen crate or "no crate — escalate to Dwayne"
- [ ] Verify: decision documented in plan or spec open questions resolved

### Task 1.2: Implement `convert_mobi`
**Files:** Create `crates/agentready-core/src/converters/mobi.rs`, Modify `mod.rs`, `Cargo.toml`
- [ ] `pub fn convert_mobi(path: &Path) -> Result<ConversionResult, AgentReadyError>`
- [ ] DRM/encryption sniff → `PasswordProtected`
- [ ] Chapter boundaries → `---` separators
- [ ] `text_quality::looks_like_garbage` → `NoReadableText`
- [ ] Unit test: minimal synthetic MOBI or public-domain fixture
- [ ] Verify: `cargo test converters::mobi`

### Task 1.3: Wire MOBI
**Files:** `validation.rs`, `job.rs`, `serve.rs`, `index.html`, `models.rs`, `README.md`
- [ ] Extensions: `mobi`
- [ ] Verify: `cargo test` + `cargo run -- convert test.mobi --output /tmp/out`

---

## Phase 2 — AZW3 + AZW

### Task 2.1: Implement `convert_azw3`
**Files:** Create `azw3.rs`, optionally `html_md.rs`, Modify `epub.rs` if extracting shared HTML MD
- [ ] Parse KF8/AZW3 container; extract HTML content
- [ ] Reuse HTML→Markdown path
- [ ] DRM reject + text_quality
- [ ] Verify: `cargo test converters::azw3`

### Task 2.2: AZW router
**Files:** `job.rs`, `validation.rs`, `serve.rs`, `index.html`
- [ ] `.azw` and `.azw3` in allowlists
- [ ] Sniff and route `.azw` → mobi or azw3
- [ ] Verify: manual convert on Dwayne's local `.azw3`

---

## Phase 3 — Integration

### Task 3.1: Fixtures doc
**Files:** Create `examples/sample-input/ebooks/README.md`
- [ ] Document Calibre/public-domain workflow for DRM-free test ebooks
- [ ] State: do not commit copyrighted books

### Task 3.2: CLI integration test
**Files:** `crates/agentready-cli/tests/integration.rs`
- [ ] Add EPUB round-trip test using in-memory or committed minimal epub
- [ ] Optional: `#[ignore]` MOBI test with instructions
- [ ] Verify: `cargo test -p agentready --test integration`

---

## Phase 4 — Docs & close

### Task 4.1: Align product SDD
**Files:** `README.md`, `docs/08_CONVERSION_PIPELINE.md`, `docs/SUMMARY.md`, `AGENTREADY_V1_SDD_MASTER.md` (non-goals section)
- [ ] MOBI/AZW3 in supported table; legal unchanged
- [ ] Verify: grep README for "not in V1" — MOBI/AZW removed

### Task 4.2: Manual QA matrix
**Files:** none (report in session-end / ACTIVITY)
- [ ] DRM-free MOBI — Good/Partial
- [ ] DRM-free AZW3 — Good/Partial
- [ ] DRM file — Failed + correct message
- [ ] Export README contains legal notice

### Task 4.3: Close spec
**Files:** `spec.md` status → Done
- [ ] Mark spec **Done** after Dwayne sign-off
- [ ] Log `milestone` to vault `ACTIVITY.md` if vault session active
