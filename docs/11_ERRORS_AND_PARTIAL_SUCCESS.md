# 11 — Errors and Partial Success

## Principle

One bad file must not break the whole batch.

## Error code registry

| Code | User message |
|---|---|
| `UNSUPPORTED_FILE` | This file type is not supported in AgentReady V1. |
| `FILE_TOO_LARGE` | This file is larger than the V1 file size limit. |
| `NO_READABLE_TEXT` | AgentReady could not find readable text in this file. OCR is not supported in V1. |
| `PASSWORD_PROTECTED` | This file appears to be password protected or encrypted. Password protected files are not supported in V1. |
| `CONVERSION_FAILED` | AgentReady could not convert this file. |
| `PARTIALLY_CONVERTED` | This file was converted, but the output may need review. |
| `CANCELLED` | The conversion was cancelled. |
| `TEMP_FILE_ERROR` | AgentReady had a temporary file processing problem. |
| `ZIP_CREATION_FAILED` | AgentReady could not create the export zip. |

## Rules

- Failed files are listed on screen and in `conversion-report.md`.
- Partially converted files are included with warnings.
- Unsupported files are skipped and reported.
- Password-protected files are skipped; do not ask for passwords.
