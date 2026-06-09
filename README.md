# AgentReady

**Turn documents into agent-ready Markdown.**

AgentReady converts everyday files—PDFs, Word docs, spreadsheets, ebooks, and plain text—into clean Markdown and a portable zip package that AI agents can read and navigate. No accounts, no cloud upload pipeline, no AI rewriting: just structured conversion you can preview and download locally.

A **HumanGoodAI** product. V1 is spec-driven; the design package lives in [`docs/`](docs/SUMMARY.md).

---

## What you get

1. **Upload** one or more supported files (drag-and-drop or file picker).
2. **Preview** converted Markdown in the browser before you commit to an export.
3. **Download** an AgentReady zip with indexed Markdown, raw data copies, and a conversion report.

The export is built for RAG folders, Claude projects, ChatGPT knowledge, Obsidian vaults, and any workflow where an agent needs readable, linked source material—not binary PDFs or opaque DOCX internals.

### Before → after (why agents prefer this)

Agents can read PDFs and Word files, but those formats carry **layout noise**—binary structure, styles, tags, and positioning—that burns context before the model reaches the meaning. AgentReady strips that down to **lean Markdown**: headings, lists, tables, and short YAML frontmatter.

**Example — text PDF** ([`examples/sample-input/ebooks/minimal.pdf`](examples/sample-input/ebooks/minimal.pdf)):

| | What the agent sees |
|---|---------------------|
| **Before** | 663-byte PDF binary (or a messy text dump with no structure) |
| **After** | ~60 characters of content in clear paragraphs, plus a 6-line frontmatter block |

```markdown
---
source_file: minimal.pdf
source_type: pdf
converted_by: agentready-v1
status: good
---

Hello PDF reader.

AgentReady text extraction smoke test.
```

Try it yourself:

```bash
./scripts/demo-before-after.sh          # PDF + DOCX + EPUB in one shot
cargo run -- convert examples/sample-input/ebooks/minimal.pdf --output /tmp/ar-demo
cat /tmp/ar-demo/documents/minimal-pdf.md
```

**DOCX** ([`minimal.docx`](examples/sample-input/ebooks/minimal.docx)) shows the strongest structure — Word styles become `#` headings and `-` lists:

```markdown
# Staff Handbook

Welcome to the team. This is agent-ready content.

- Be kind
- Be clear
```

**What we claim:** cleaner structure, less formatting noise, easier agent navigation via `index.md` and `manifest.json`.

**What we do not claim:** exact token savings or fixed percentages—that depends on your model, prompt, and source file.

See [`docs/09_MARKDOWN_STANDARD.md`](docs/09_MARKDOWN_STANDARD.md) for the output contract.

---

## Use with ChatGPT and other chat agents

AgentReady does **not** include a chatbot. It prepares files so **you** can drop them into a bot’s knowledge base.

**Three steps:**

1. **Convert** your PDFs, Word docs, EPUBs, etc. in AgentReady (web UI or CLI).
2. **Download** the zip and unzip it on your computer.
3. **Upload to your agent’s knowledge base** — the files the bot should read.

### What to upload

| Upload this | Why |
|-------------|-----|
| **`documents/*.md`** | Main content — clean Markdown with headings and lists |
| **`index.md`** | Tells the bot what each file is and links to them |
| **`conversion-report.md`** | Optional — which files converted well vs failed |

You usually **do not** need to upload the original PDFs again. The Markdown in `documents/` is what ChatGPT, Claude, and similar tools read best.

### Where it works

| Tool | Typical flow |
|------|----------------|
| **ChatGPT** (Project or Custom GPT) | Project → Add files → select `documents/*.md` and `index.md` |
| **Claude** (Project) | Project knowledge → upload the same `.md` files |
| **Other agents / RAG apps** | Point the knowledge folder at `documents/` or upload the zip contents |

### Tips

- Start with **`index.md`** so the bot knows the map of your export.
- Prefer **Good** files from the conversion results; skip or fix **Failed** ones.
- **DRM-free ebooks only** — encrypted Kindle/Adobe files are rejected by design.
- AgentReady runs **locally**; you choose what gets uploaded to ChatGPT or any cloud bot.

---

## Supported formats

| Format | Extensions | Notes |
|--------|------------|-------|
| Plain text | `.txt` | UTF-8; paragraphs, headings, and lists inferred where possible |
| Markdown | `.md`, `.markdown` | Preserved structure (tables, code blocks); normalized for export |
| CSV | `.csv` | Tabular Markdown + raw CSV in `data/` |
| Word | `.docx` | Structure extracted to Markdown |
| PDF | `.pdf` | Text → structured Markdown; garbage/mojibake rejected (no OCR) |
| EPUB | `.epub` | Ebook chapters in spine order (HTML → Markdown) |
| MOBI | `.mobi` | Legacy Kindle ebooks (DRM-free only) |
| AZW3 | `.azw3` | Kindle KF8 — prefers KF8 section when present |
| AZW | `.azw` | Kindle PDB (routed via same pipeline as MOBI/AZW3) |

**Limits (V1):** up to **25 files** per job, **50 MB** per file.

**Not in V1:** OCR, audio, websites, XLSX, accounts, or AI enrichment.

**Ebooks:** You must own or be authorized to convert files. **DRM-free only** — see [`docs/23_CONTENT_OWNERSHIP_AND_LEGAL.md`](docs/23_CONTENT_OWNERSHIP_AND_LEGAL.md).

---

## Quick start

### Requirements

- [Rust](https://rustup.rs/) (2024 edition toolchain)
- macOS, Linux, or Windows with a terminal

### Web UI (recommended)

```bash
git clone https://github.com/dman1313/agentreadymarkdown.git
cd agentreadymarkdown
./scripts/serve.sh
```

This builds the project, starts a local server (default `http://127.0.0.1:3000`), and opens your browser. **Keep the terminal open** while you use the app.

Options:

```bash
./scripts/serve.sh --port 3001      # different port
./scripts/serve.sh --no-open        # don't open browser
```

Health check: `GET /health` on the same host/port.

### CLI

Convert a folder without the UI:

```bash
cargo run -- convert ./my-files --output ./agentready-output
```

Useful flags:

```bash
cargo run -- convert ./docs --output ./out --recursive
cargo run -- convert ./file.pdf --output ./out --zip-name my-export.zip
cargo run -- convert ./docs --output ./out --max-files 10 --max-file-size-mb 25
cargo run -- convert ./docs --output ./out --json   # machine-readable job result
```

Release binary:

```bash
cargo build --release
./target/release/agentready convert ./my-files --output ./agentready-output
```

---

## Export package

Each successful job produces a folder (and matching `.zip`) like:

```text
agentready-output/
  README.md              # Human guide to the export
  index.md               # Agent navigation table (start here for LLMs)
  conversion-report.md   # Per-file status and audit trail
  documents/             # Converted Markdown (TXT, MD, DOCX, PDF, EPUB)
  data/                  # Raw CSV copies where applicable
  assets/                # Optional; only when useful assets exist
```

File names are derived from originals (lowercase, hyphenated, type suffix on duplicates). Example: `Staff Handbook.pdf` → `documents/staff-handbook-pdf.md`.

See [`docs/10_EXPORT_PACKAGE.md`](docs/10_EXPORT_PACKAGE.md) for the full spec.

---

## How it works

```text
Upload → validate → convert to Markdown → reject garbage → normalize for agents → export zip
```

```text
┌─────────────┐     ┌──────────────────────────────────────┐     ┌─────────────────┐
│  Web UI or  │────▶│  agentready-core                     │────▶│  Export folder  │
│  CLI upload │     │  converters · text_quality ·         │     │  + zip download │
└─────────────┘     │  agent_markdown · export             │     └─────────────────┘
                    └──────────────────────────────────────┘
```

**Rust workspace**

| Crate | Role |
|-------|------|
| [`agentready-core`](crates/agentready-core) | Converters, validation, export, shared `job::run_job` pipeline |
| [`agentready`](crates/agentready-cli) | `convert` and `serve` commands; embedded HTML UI (Axum) |

The web server runs conversion **in-process**—no separate Node server or child CLI process. Legacy `apps/server` and `apps/web` are deprecated; see [`apps/README.md`](apps/README.md).

**Agent-ready output:** converters emit structure (headings, lists, tables where detected); `text_quality` blocks unreadable PDF/ebook extract; `normalize_for_agents` removes invisible characters and extra blank lines before export.

---

## Development

```bash
cargo check              # Type-check
cargo test               # Unit + integration tests
cargo build --release    # Optimized binary
```

Project conventions: [`CLAUDE.md`](CLAUDE.md).

### Scripts

| Script | Purpose |
|--------|---------|
| [`scripts/serve.sh`](scripts/serve.sh) | Build, pick a free port, start UI |
| [`scripts/gh-auth.sh`](scripts/gh-auth.sh) | Fix invalid `gh` keyring tokens |

---

## Privacy (V1)

- Processing is **local and temporary**—files are written to a temp job directory for the conversion, then served for download.
- **No accounts**, no persistent workspace history, no AI enrichment in V1.
- See [`docs/15_SECURITY_AND_PRIVACY.md`](docs/15_SECURITY_AND_PRIVACY.md) and [`docs/adr/ADR-0004-temporary-processing-privacy.md`](docs/adr/ADR-0004-temporary-processing-privacy.md).

## Content & legal (V1)

- Upload only files **you own** or are **authorized** to convert.
- Ebooks (EPUB, PDF, MOBI, AZW3) must be **DRM-free** — AgentReady cannot remove copy protection.
- AgentReady is a conversion tool; **HumanGoodAI accepts no responsibility** for the legality of your uploads or use of exports. You are solely responsible for copyright compliance.
- Full notice: [`docs/23_CONTENT_OWNERSHIP_AND_LEGAL.md`](docs/23_CONTENT_OWNERSHIP_AND_LEGAL.md). Every export zip includes the legal notice in `README.md`.

---

## Spec and design package

This repository is both the **V1 implementation** and the **spec-driven design (SDD) package** for AgentReady.

| Start here | Contents |
|------------|----------|
| [`docs/SUMMARY.md`](docs/SUMMARY.md) | One-screen overview |
| [`docs/00_OVERVIEW.md`](docs/00_OVERVIEW.md) … [`docs/20_BUILD_ROADMAP.md`](docs/20_BUILD_ROADMAP.md) | Implementation-ready sections |
| [`docs/21_DECISIONS_Q107_Q261.md`](docs/21_DECISIONS_Q107_Q261.md) | Latest product and UX decisions |
| [`AGENTREADY_V1_SDD_MASTER.md`](AGENTREADY_V1_SDD_MASTER.md) | Full master SDD in one file |

**Rule:** The spec is the source of truth. If code and spec disagree, update one deliberately—do not silently drift.

---

## Roadmap (high level)

V1 focuses on upload → convert → preview → zip. Future directions (not committed here) include hosted beta, credits, richer project types, and optional anonymization for sensitive files. See the decisions doc and roadmap in `docs/`.

---

## Troubleshooting

| Issue | What to do |
|-------|------------|
| `ERR_CONNECTION_REFUSED` | Server not running—start `./scripts/serve.sh` and leave the terminal open |
| Stale UI or missing features | Rebuild: `./scripts/serve.sh` (HTML is embedded at compile time) |
| Large PDF fails in browser | Ensure you're on a build after the 2 MB body-limit fix; max 50 MB per file |
| iCloud drag shows 0-byte files | Copy files to Desktop or use click-to-browse |
| `gh auth` invalid token | Run `./scripts/gh-auth.sh` |

---

## License

No license file is included yet. Contact the repository owner before redistributing or using in production.

---

## Tagline

> Upload your files, preview the AgentReady Markdown, and download an agent-ready zip.
