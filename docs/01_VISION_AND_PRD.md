# 01 — Vision and PRD

## Vision

Organizations are accumulating documents, PDFs, spreadsheets, policies, and knowledge files faster than humans can organize them. AgentReady helps prepare that information for safe, practical AI use without forcing non-technical people to understand AI infrastructure.

## V1 goal

Deliver a simple web app that converts supported files into a clean, portable AgentReady export zip.

## Target users

### Primary

Non-technical organization staff who want to prepare knowledge for AI tools.

### Secondary

Agent builders and developers who want clean files to load into an agent, RAG system, or knowledge folder.

## Must ship in V1

- multiple file upload
- supported file validation
- conversion to Markdown/data outputs
- partial success behavior
- preview
- Simple Export zip
- privacy-focused temporary processing
- no accounts
- no AI enrichment

## Non-goals

- website scanning
- `llms.txt`
- `agent.md`
- OCR
- audio
- EPUB
- embeddings
- vector database creation
- accounts or billing
- persistent workspace history

## Success criteria

V1 is successful when a non-technical user can understand the upload page, process a small folder of files, preview the Markdown, and download a useful zip without support.
