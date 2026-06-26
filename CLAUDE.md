# CLAUDE.md — AgentReady V1

## Build Commands

```bash
cargo check              # Type-check
cargo test               # Run all tests
cargo build --release    # Release build
cargo run -- convert <input> --output <dir>  # CLI
./scripts/serve.sh       # Local web UI (recommended)
./scripts/serve.sh --port 3001
./scripts/demo-before-after.sh  # PDF/DOCX/EPUB before→after demo
cargo run -- serve       # Or run directly after cargo build
```

## Architecture

Rust workspace:

- `crates/agentready-core` — converters, `text_quality`, `agent_markdown`, export, `job::run_job` pipeline
- `crates/agentready-cli` — `convert` + `serve` (axum, embedded HTML UI)

Legacy `apps/server` and `apps/web` are deprecated; use `agentready serve` instead.

## Spec

The spec is the source of truth. All 22 spec docs live in `docs/`.
Start with `docs/SUMMARY.md` for an overview.
Latest decisions: `docs/21_DECISIONS_Q107_Q261.md` and `docs/22_DECISIONS_EDGE_CASES.md`.

## Conventions

- Edition 2024, resolver 2
- Errors: `ErrorCode` enum, no unwraps on user-facing paths
- Converters return `Result<ConversionResult, AgentReadyError>`
- Export writes folder + zip; frontmatter in export layer only
- Web server calls `job::run_job` in-process (not a child CLI process)

## gstack

The [gstack](https://github.com/garrytan/gstack) skill suite is installed.

- **Web browsing:** Use the `/browse` skill from gstack for all web browsing.
  Never use `mcp__claude-in-chrome__*` tools.

### Available skills

`/office-hours`, `/plan-ceo-review`, `/plan-eng-review`, `/plan-design-review`,
`/design-consultation`, `/design-shotgun`, `/design-html`, `/review`, `/ship`,
`/land-and-deploy`, `/canary`, `/benchmark`, `/browse`, `/connect-chrome`,
`/qa`, `/qa-only`, `/design-review`, `/setup-browser-cookies`, `/setup-deploy`,
`/setup-gbrain`, `/retro`, `/investigate`, `/document-release`,
`/document-generate`, `/codex`, `/cso`, `/autoplan`, `/plan-devex-review`,
`/devex-review`, `/careful`, `/freeze`, `/guard`, `/unfreeze`,
`/gstack-upgrade`, `/learn`
