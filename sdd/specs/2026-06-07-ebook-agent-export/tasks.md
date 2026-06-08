# Tasks: Ebook Agent Export (MOBI + AZW3)

> Plan: ./plan.md · **Do not start until spec.md is Approved.** Tick `- [x]` as you go.

---

## Phase 0 — Stabilize baseline

### Task 0.1: Verify in-flight work
**Files:** (read-only audit)
- [x] Run `cargo test` — expect all tests pass (baseline ~55)
- [x] Run `cargo check`
- [x] Review `git diff --stat` — confirm EPUB, text_quality, serve, legal present
- [x] Commit: `feat: EPUB converter, text quality checks, serve UI, legal notice` (89ad458)

### Task 0.2: UI smoke
**Files:** none
- [ ] `./scripts/serve.sh --no-open --port 3099`
- [ ] Upload a DRM-free EPUB + text PDF — preview readable, zip downloads
- [ ] Confirm legal notice visible on upload page

---

## Phase 1 — MOBI

### Task 1.1: Research spike
**Files:** notes in `mobi.rs` header
- [x] Search crates.io — chose `mobi` v0.8 (MIT, vv9k/mobi-rs)
- [x] Calibre fallback not needed for initial implementation
- [x] Decision documented in `mobi.rs` header comment

### Task 1.2: Implement `convert_mobi`
**Files:** Create `crates/agentready-core/src/converters/mobi.rs`, Modify `mod.rs`, `Cargo.toml`
- [x] `pub fn convert_mobi(path: &Path) -> Result<ConversionResult, AgentReadyError>`
- [x] DRM/encryption sniff → `PasswordProtected`
- [x] Title as `#` heading; basic HTML strip when present
- [x] `text_quality::looks_like_garbage` → `NoReadableText`
- [x] Unit tests: invalid file, HTML normalize, title build
- [x] Verify: `cargo test converters::mobi` — pass

### Task 1.3: Wire MOBI
**Files:** `validation.rs`, `job.rs`, `serve.rs`, `index.html`, `models.rs`, `README.md`
- [x] Extensions: `mobi`
- [x] `cargo test` — 59 total pass
- [ ] Manual: `cargo run -- convert your.mobi --output /tmp/out` when Dwayne supplies DRM-free sample

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
