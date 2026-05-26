# Error Code Registry

| Code | User-facing message | Include in zip? |
|---|---|---|
| `UNSUPPORTED_FILE` | This file type is not supported in AgentReady V1. | No |
| `FILE_TOO_LARGE` | This file is larger than the V1 file size limit. | No |
| `NO_READABLE_TEXT` | AgentReady could not find readable text in this file. OCR is not supported in V1. | No |
| `PASSWORD_PROTECTED` | This file appears to be password protected or encrypted. Password protected files are not supported in V1. | No |
| `CONVERSION_FAILED` | AgentReady could not convert this file. | No |
| `PARTIALLY_CONVERTED` | This file was converted, but the output may need review. | Yes |
| `CANCELLED` | The conversion was cancelled. | No |
| `TEMP_FILE_ERROR` | AgentReady had a temporary file processing problem. | No |
| `ZIP_CREATION_FAILED` | AgentReady could not create the export zip. | No |
| `BLOCKED_EXTENSION` | This file type is blocked for safety. | No |
| `INVALID_ARCHIVE_PATH` | AgentReady blocked an unsafe file path. | No |
| `BATCH_LIMIT_EXCEEDED` | This batch has more files than AgentReady V1 supports. | No |

New error codes require an SDD update.
