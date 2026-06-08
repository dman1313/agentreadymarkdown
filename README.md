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

---

## Supported formats

| Format | Extensions | Notes |
|--------|------------|-------|
| Plain text | `.txt` | UTF-8; BOM stripped |
| Markdown | `.md`, `.markdown` | Preserved structure (tables, code blocks) |
| CSV | `.csv` | Tabular Markdown + raw CSV in `data/` |
| Word | `.docx` | Structure extracted to Markdown |
| PDF | `.pdf` | Text extraction; garbage/mojibake rejected with a clear message |
| EPUB | `.epub` | Ebook chapters in spine order (HTML → Markdown) |
| MOBI | `.mobi` | Legacy Kindle ebooks (DRM-free only; MIT `mobi` crate) |

**Limits (V1):** up to **25 files** per job, **50 MB** per file.

**Not in V1:** AZW3/AZW (planned), OCR, audio, websites, XLSX, accounts, or AI enrichment.

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
┌─────────────┐     ┌──────────────────┐     ┌─────────────────┐
│  Web UI or  │────▶│  agentready-core │────▶│  Export folder  │
│  CLI upload │     │  converters +    │     │  + zip download │
└─────────────┘     │  validation      │     └─────────────────┘
                    └──────────────────┘
```

**Rust workspace**

| Crate | Role |
|-------|------|
| [`agentready-core`](crates/agentready-core) | Converters, validation, export, shared `job::run_job` pipeline |
| [`agentready`](crates/agentready-cli) | `convert` and `serve` commands; embedded HTML UI (Axum) |

The web server runs conversion **in-process**—no separate Node server or child CLI process. Legacy `apps/server` and `apps/web` are deprecated; see [`apps/README.md`](apps/README.md).

**Quality checks:** PDF text passes readability heuristics before export; unreadable extractions surface as user-facing errors instead of mojibake in preview.

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
