# 15 — Security and Privacy

## Core privacy promise

AgentReady does not keep source documents after conversion.

## V1 processing

- server-based temporary processing allowed for hosted beta
- local-first processing preferred long term
- source uploads and temporary files deleted after download, cancellation, or cleanup

## Logs

Logs may include:

- filename
- file type
- file size
- conversion stage
- error code

Logs must not include:

- document text
- spreadsheet contents
- extracted private content

## Safety checks

- validate file types
- enforce size limits
- block dangerous extensions
- never execute macros/scripts/active content
- skip encrypted/password-protected files

## AI processing rule

No AI summaries or AI enrichment in V1. Future AI enrichment must be opt-in and explicit.


---

## Latest privacy update from Q107–Q261

See `docs/21_DECISIONS_Q107_Q261.md`, sections 21.9 and 21.13, for beta support-file handling and future sensitive-file/anonymization decisions.

Important V1 clarification: AgentReady V1 does not anonymize or redact files. Users must review sensitive files before upload. Future anonymization and sensitive-file mode are planned but not part of the first V1 build.
