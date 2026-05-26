# Privacy and Security Requirements

## Core rule

AgentReady does not keep source documents after conversion.

## File lifecycle

1. Upload files temporarily.
2. Process files in isolated temporary job folders.
3. Generate output folder and zip.
4. Delete source uploads and temporary working files after conversion, download, cancellation, or cleanup.
5. Delete temporary logs.

## Logging rule

Logs must not include document text, spreadsheet contents, or private content.

Logs may include filename, type, size, status, stage, error code, and failure reason.

## V1 AI rule

V1 does not send documents to AI models.

## Safety checks

1. validate file extensions
2. validate MIME where possible
3. enforce limits
4. block dangerous extensions
5. never execute macros or scripts
6. prevent path traversal
7. report unsupported or suspicious files
