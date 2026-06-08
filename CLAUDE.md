# CLAUDE.md — AgentReady V1

## Build Commands

```bash
cargo check              # Type-check
cargo test               # Run all tests
cargo build --release    # Release build
cargo run -- convert <input> --output <dir>  # CLI
./scripts/serve.sh       # Local web UI (recommended)
./scripts/serve.sh --port 3001
cargo run -- serve       # Or run directly after cargo build
```

## Architecture

Rust workspace:

- `crates/agentready-core` — converters, validation, export, `job::run_job` pipeline
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
