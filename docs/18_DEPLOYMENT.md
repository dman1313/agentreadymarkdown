# 18 — Deployment

## Development order

Local development first, then simple hosted beta.

## Local development

- Rust CLI runs locally
- TypeScript server calls CLI locally
- Vite app talks to local server

## Hosted beta

Hosted beta may process files temporarily on the server.

Requirements:

- temporary upload storage
- cleanup after download/cancel/expiry
- size limits enforced
- no accounts in V1
- clear privacy message before upload

## Future deployment

A future desktop or local-first version can keep all processing on the user's machine.
