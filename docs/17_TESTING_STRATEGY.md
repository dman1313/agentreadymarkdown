# 17 — Testing Strategy

## Required test categories

- Rust unit tests
- Rust CLI integration tests
- fixture tests using sample input/output
- zip structure tests
- Fastify server tests
- React UI flow tests
- error code mapping tests
- privacy/logging tests

## CI checks

- Rust format
- Rust build
- Rust tests
- TypeScript type check
- Fastify build
- Vite build
- fixture conversion check

## Fixture rule

Sample input and expected output must stay in the repo. If conversion behavior changes, update fixtures intentionally.
