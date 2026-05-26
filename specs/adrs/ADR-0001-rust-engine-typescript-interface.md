# ADR-0001: Rust Engine and TypeScript Interface

## Decision

AgentReady V1 uses a Rust conversion engine and a TypeScript web interface.

## Rationale

Rust is responsible for file validation, conversion, deterministic output structure, reports, and zip creation.

TypeScript is responsible for upload, progress, preview, results, and download.

The conversion core remains independent so it can later support CLI, desktop, API, or local-first use.
