# ADR-0001 — Rust engine with TypeScript interface

## Decision

Use Rust for the reusable conversion engine and TypeScript for the web interface/server.

## Reason

Rust is a strong fit for file processing, deterministic output, CLI reuse, and later local-first/desktop builds. TypeScript is a strong fit for the web UI and Fastify server.

## Consequence

The Rust engine must not depend on the web UI. The TypeScript server calls the Rust CLI with `--json` in V1.
