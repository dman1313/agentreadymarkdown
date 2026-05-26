# ADR-0002 — Simple Export before Advanced Export

## Decision

V1 ships Simple Export only.

## Reason

The primary user is non-technical. A clean zip with Markdown, index, README, report, data copies, and assets is easier to understand than advanced metadata/chunk exports.

## Consequence

JSONL chunks, source maps, embeddings, and database-ready export belong to a later Advanced Export mode.
