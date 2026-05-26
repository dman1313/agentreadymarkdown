# ADR-0004 — Temporary processing privacy model

## Decision

Uploaded files and temporary working files are deleted after conversion, download, cancellation, or cleanup expiry.

## Reason

AgentReady should prepare files, not become a permanent document store.

## Consequence

The server needs clear temp job directories, cleanup jobs, and privacy-safe logs.
