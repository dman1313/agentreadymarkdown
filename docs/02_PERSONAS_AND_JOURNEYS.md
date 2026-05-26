# 02 — Personas and Journeys

## Primary persona: non-technical organization staff

This user has documents and spreadsheets but does not know how to prepare them for an AI agent.

They need plain language, clear privacy wording, simple results, and a download they can store in Google Drive, Dropbox, a local folder, or an agent knowledge folder.

## Secondary persona: agent builder

This user cares about clean Markdown, index files, conversion reports, and predictable folder structure.

They may later want advanced metadata, JSONL chunks, source maps, and database-ready exports, but those are not default V1 features.

## Journey A — Simple conversion

```text
Open upload page
Read how it works
Upload PDF, DOCX, XLSX, CSV
Start conversion
Preview Markdown
Download zip
Store zip where needed
```

## Journey B — Mixed success

```text
Upload valid files plus one unsupported file
AgentReady validates files
Valid files continue
Unsupported file is shown with reason
User downloads zip containing successful outputs only
conversion-report.md records what happened
```

## Journey C — Cancel and restart

```text
Upload files
Start conversion
Cancel
Temporary files deleted
User returns to selected file list
Problem files remain visible
User removes problem files and restarts
```
