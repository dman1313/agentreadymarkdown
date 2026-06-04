# Legacy TypeScript apps (deprecated)

The V1 UI and API now live in Rust:

```bash
cargo build --release
cargo run -- serve
```

Open http://127.0.0.1:3000 — upload, progress, preview, and download run in-process via `agentready-core` (no Node, no CLI subprocess).

The `server/` and `web/` folders are kept for reference only and are no longer required to run AgentReady.
