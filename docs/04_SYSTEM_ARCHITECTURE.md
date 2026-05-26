# 04 — System Architecture

## Architecture choice

AgentReady V1 uses a Rust conversion engine with a TypeScript web interface.

## Components

| Component | Responsibility |
|---|---|
| Vite React app | Upload, selected files, progress, preview, results, download |
| Fastify server | Accept uploads, validate, create temp job, call Rust CLI, serve zip, cleanup |
| Rust CLI | Batch conversion, JSON status output, folder and zip generation |
| Rust core crate | File detection, conversion pipeline, Markdown rendering, export generation |

## Runtime flow

```text
Browser
-> Fastify upload endpoint
-> temporary job directory
-> Rust CLI with --json
-> output folder + zip
-> Fastify returns result metadata
-> Browser previews/downloads
-> cleanup
```

## Boundary rule

The Rust engine must remain independent from the web UI so it can later support CLI-only use, desktop use, API use, and background workers.
