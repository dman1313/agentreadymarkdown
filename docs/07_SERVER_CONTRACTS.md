# 07 — Server Contracts

## Server

Fastify TypeScript server.

## Required routes

| Route | Purpose |
|---|---|
| `POST /api/jobs` | Upload files and create conversion job |
| `GET /api/jobs/:id` | Read job status |
| `POST /api/jobs/:id/cancel` | Cancel active job |
| `GET /api/jobs/:id/files/:fileId/preview` | Read converted Markdown preview |
| `GET /api/jobs/:id/download` | Download zip |
| `DELETE /api/jobs/:id` | Cleanup job files |

## Server responsibilities

- validate uploads before conversion
- block dangerous extensions
- enforce file limits
- create temp job directory
- call Rust CLI with `--json`
- map CLI error codes to UI messages
- clean source uploads and temp files
- never log document contents
