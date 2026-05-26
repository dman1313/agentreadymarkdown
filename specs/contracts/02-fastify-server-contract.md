# Fastify Server Contract

## Endpoints

| Method | Path | Purpose |
|---|---|---|
| `POST` | `/api/jobs` | Upload files and start conversion |
| `GET` | `/api/jobs/:jobId` | Get job status |
| `POST` | `/api/jobs/:jobId/cancel` | Cancel job |
| `GET` | `/api/jobs/:jobId/files/:fileId/preview` | Get Markdown preview |
| `GET` | `/api/jobs/:jobId/files/:fileId/download` | Download individual file |
| `GET` | `/api/jobs/:jobId/download` | Download export zip |
| `DELETE` | `/api/jobs/:jobId` | Cleanup job files |

## Job states

1. `created`
2. `validating`
3. `converting`
4. `packaging`
5. `ready`
6. `completed_with_warnings`
7. `failed`
8. `cancelled`
9. `cleaned_up`

## Server rules

The server must not store files permanently, log document contents, send documents to AI, trust filenames, allow path traversal, or expose raw filesystem paths to the browser.
