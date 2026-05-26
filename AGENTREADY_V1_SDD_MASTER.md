# AgentReady V1 Software Design Document

Product: AgentReady  
Company: HumanGoodAI  
Repository name: `agentready-knowledge-packager`  
Tagline: **Is your data agent ready?**  
Version: V1 SDD  
Date: 2026-05-24  
Status: Build-ready draft

---

## 1. Product Summary

AgentReady prepares documents and data so AI agents can read them more clearly.

AgentReady V1 lets a non-technical user upload common files, convert them into clean Markdown and simple data outputs, preview the result, and download a portable zip package.

AgentReady does not run the agent, host a chatbot, permanently store user documents, or connect directly to the user's private downstream data systems.

### Core user promise

> Upload your files. Preview clean Markdown. Download an agent-ready zip.

### Supporting copy

> AgentReady converts your documents and data into clean, agent-ready files your AI tools can read faster, understand more clearly, and process with less unnecessary token waste.

### Primary user

AgentReady V1 is designed first for non-technical organization staff.

The primary user is someone who wants to upload files and download an agent-ready zip without needing to understand Markdown, RAG, vector databases, embeddings, JSONL, source maps, or command line tools.

Agent builders and developers are secondary users. The V1 output must still be useful for them, but the interface must stay simple.

---

## 2. V1 Scope

### V1 must do

1. Accept file upload through a web interface.
2. Support multiple file upload.
3. Support folder upload where browser support exists.
4. Validate file types and file sizes.
5. Convert supported files to clean Markdown or data outputs.
6. Show conversion progress.
7. Allow cancellation during conversion.
8. Continue converting valid files even if some files fail.
9. Show results with successful, partially converted, failed, and unsupported files.
10. Let users preview one converted Markdown file at a time.
11. Let users toggle between rendered Markdown and raw Markdown.
12. Let users copy raw Markdown for a selected file.
13. Let users download an individual converted Markdown file.
14. Generate a Simple Export zip.
15. Include `README.md`, `index.md`, `conversion-report.md`, converted files, data files, and assets when needed.
16. Delete uploaded source files and temporary working files after conversion, download, cancellation, or cleanup.
17. Use clear, friendly, non-technical language.

### V1 supported file types

Required V1 input types:

| Type | Extension examples | V1 behavior |
|---|---|---|
| PDF | `.pdf` | Extract readable text and structure where possible |
| DOCX | `.docx` | Convert headings, paragraphs, lists, links, and tables |
| Text | `.txt` | Lightly convert to Markdown |
| Markdown | `.md`, `.markdown` | Preserve structure and normalize only when safe |
| CSV | `.csv` | Convert to Markdown table and preserve CSV copy |
| XLSX | `.xlsx` | Convert sheets/tables to Markdown and preserve CSV copies |

### V1 file limits

| Limit | Value |
|---|---:|
| Files per batch | 25 |
| Max size per file | 50 MB |
| Long-running warning | Show after 60 seconds |
| Auto cancel after warning | No |

### V1 non-goals

AgentReady V1 shall not:

1. Scan websites.
2. Generate `llms.txt`.
3. Generate `agent.md` for websites.
4. Run a chatbot.
5. Query the exported knowledge.
6. Build a vector database.
7. Generate embeddings.
8. Provide JSONL chunks by default.
9. Permanently store user documents.
10. Require accounts or login.
11. Include billing.
12. Include persistent workspaces or history.
13. Send documents to an AI model for summaries.
14. Support OCR for scanned PDFs.
15. Process audio files.
16. Process EPUB files.
17. Ask users for passwords to unlock encrypted files.
18. Execute macros, scripts, or active file content.
19. Copy code, assets, tests, prompts, or implementation details from GPL projects.

---

## 3. Why Markdown

AgentReady V1 uses Markdown as the main knowledge format because it is simple, readable, portable, and agent-friendly.

AI agents can often read PDFs, Word documents, spreadsheets, and HTML, but those formats may include layout instructions, visual styling, hidden structure, document wrappers, tags, attributes, metadata, or formatting noise. That extra structure can make the agent process more information before it reaches the meaning.

Markdown keeps the important content clearer:

1. headings
2. sections
3. lists
4. tables
5. links
6. notes
7. source information

Markdown is also readable by people. A human can open it, review it, edit it, and store it anywhere. It is not a locked or proprietary AgentReady format.

### Plain-language UI explanation

```md
## Why Markdown?

AgentReady converts your files into Markdown because Markdown is simple, readable, and agent-friendly.

AI agents can read PDFs, Word documents, spreadsheets, and HTML, but those formats often include extra layout, styling, tags, hidden structure, or formatting noise. That means the agent has more to process before it gets to the actual meaning.

Markdown keeps the important content clearer: headings, sections, lists, tables, links, notes, and source information.

Markdown is also readable by people. You can open it, review it, edit it, and store it wherever you want. It is not a locked or proprietary AgentReady format.

Cleaner files can also reduce unnecessary token use. This does not guarantee a fixed savings amount, but reducing extra formatting and noise can help agents process the same knowledge more efficiently.
```

### Environmental wording rule

AgentReady may say that cleaner inputs can support more efficient AI use.

AgentReady must not promise exact token savings, exact electricity savings, exact carbon savings, or guaranteed environmental impact.

Allowed wording:

> Cleaner files can help reduce unnecessary token use. Lower token use can contribute to more efficient AI processing, depending on the model, infrastructure, and task.

Do not write:

> AgentReady always reduces electricity use by X percent.

---

## 4. User Flow

### Primary V1 flow

```text
Open AgentReady
-> Read simple upload page
-> Upload files or folder
-> Review selected files
-> Remove files if needed
-> Start conversion
-> See progress
-> Cancel if needed
-> See results
-> Preview converted Markdown
-> Download Simple Export zip
-> Thank you message
-> Start new conversion
```

### Upload screen must show

1. Product tagline.
2. Short product explanation.
3. How it works.
4. Supported file types.
5. File limits.
6. Privacy message.
7. What happens to my files.
8. Why Markdown.
9. Drag and drop upload area.
10. File picker button.
11. Folder upload option where supported.

### Selected files screen must show

For each selected file:

1. original filename
2. file type
3. file size
4. validation status
5. remove action

Validation statuses:

1. Ready
2. Unsupported file type
3. Too large
4. Blocked for safety
5. Duplicate name warning

### Conversion screen must show

1. overall status
2. basic per-file status
3. working icon or spinner
4. cancel button
5. long-running warning after 60 seconds

### Cancellation behavior

If the user cancels:

1. stop processing remaining files where possible
2. delete uploaded source files
3. delete temporary working files
4. show cancellation message
5. return user to selected file list
6. preserve selected file names in UI state if safe
7. allow user to remove files and restart
8. show problem files
9. offer “Remove problem files” action

### Results screen must show

1. successful files
2. partially converted files
3. failed files
4. unsupported files
5. basic reason for each failure
6. preview option
7. download zip button
8. short report
9. link or note that full report is inside the zip

### After download

Show:

> Thank you for using AgentReady. Your export has been downloaded. You can now start a new conversion.

Then clear the completed job and return the user to the upload screen without requiring a page refresh.

---

## 5. Simple Export Zip Standard

### Root package structure

```text
agentready-output/
  README.md
  index.md
  conversion-report.md
  documents/
  data/
  assets/        # only included if useful assets exist
```

### Folder rules

| Folder | Purpose |
|---|---|
| `documents/` | Converted PDF, DOCX, TXT, and Markdown files |
| `data/` | Converted CSV/XLSX outputs and CSV copies |
| `assets/` | Useful extracted images, diagrams, charts, maps, or screenshots |

The `assets/` folder shall only be included when assets exist.

### Simple Export contents

Simple Export includes:

1. converted Markdown files
2. `index.md`
3. `README.md`
4. `conversion-report.md`
5. CSV copies for spreadsheet data
6. useful extracted assets when needed

Simple Export does not include by default:

1. JSON metadata files
2. JSONL chunks
3. source maps
4. embeddings
5. database-ready chunk packages
6. AI-generated summaries

Those belong in a later Advanced Export mode.

---

## 6. Output File Standards

### Converted Markdown frontmatter

Every converted Markdown file shall include simple source notes as YAML frontmatter.

```yaml
---
source_file: Staff Handbook.pdf
source_type: PDF
converted_by: AgentReady
status: converted
---
```

Allowed `status` values:

1. `converted`
2. `partially_converted`

Do not include `converted_at` in each Markdown file. Conversion date belongs in `conversion-report.md`.

Do not include source hash in V1.

Do not include original folder path in frontmatter. Original folder path belongs in `index.md`.

### Partially converted warning

If a file is included but conversion quality is uncertain, put this warning at the top after frontmatter:

```md
> ⚠️ AgentReady warning: This file was partially converted.
> Some formatting, structure, tables, or text may be incomplete. Review before using this file as trusted agent knowledge.
```

### File naming rules

1. Use the original filename as the basis.
2. Clean unsafe characters.
3. Lowercase where appropriate.
4. Replace spaces with hyphens.
5. Preserve meaning.
6. Resolve duplicate names by appending source file type.

Examples:

| Source file | Output file |
|---|---|
| `Staff Handbook.pdf` | `staff-handbook-pdf.md` |
| `Staff Handbook.docx` | `staff-handbook-docx.md` |
| `Volunteer List.xlsx` | `volunteer-list-xlsx.md` |
| `policy.md` | `policy-md.md` if needed to avoid conflict |

### Markdown quality rules

AgentReady optimizes for agent understanding, not perfect visual reproduction.

Prioritize:

1. clean headings
2. logical sections
3. lists
4. readable tables
5. links
6. source notes
7. minimal formatting noise
8. navigation clarity

Do not prioritize:

1. exact visual layout
2. page-perfect formatting
3. decorative styling
4. font choices
5. Word or PDF layout fidelity

---

## 7. Document Conversion Rules

### PDF

V1 shall:

1. extract readable text
2. preserve headings where detectable
3. preserve lists where detectable
4. preserve tables where reliable
5. reduce repeated headers and footers where safe
6. reduce broken line wraps where safe
7. mark as partially converted if quality is uncertain
8. mark as failed if no readable text is found

V1 shall not do OCR.

If scanned PDF has little or no readable text:

1. do not create a misleading empty file
2. mark as failed or partially converted depending on recoverable content
3. show user-facing message: OCR is not supported in V1

### DOCX

V1 shall convert:

1. headings
2. paragraphs
3. lists
4. links
5. tables
6. simple images where useful

Agent readability matters more than Word layout fidelity.

### TXT

V1 shall:

1. preserve content
2. add basic Markdown structure when obvious
3. avoid over-processing

### Markdown

V1 shall:

1. preserve existing Markdown structure
2. normalize unsafe filenames
3. add AgentReady frontmatter if not already present
4. avoid rewriting already agent-readable content unless needed

### CSV

V1 shall:

1. preserve the original CSV copy in `data/`
2. create a Markdown representation
3. split large tables into readable sections when needed
4. not remove rows
5. not infer new values
6. not normalize dates or data fields in V1

### XLSX

V1 shall:

1. convert each sheet into Markdown
2. preserve each sheet as CSV copy where possible
3. include sheet names in headings
4. handle multi-sheet workbooks
5. not perform advanced cleaning

Large tables shall be split into smaller Markdown sections while preserving full CSV copies.

### Assets

V1 shall extract useful non-text assets when possible:

1. charts
2. diagrams
3. maps
4. screenshots
5. important embedded images

V1 should avoid extracting decorative or duplicate assets by default:

1. logos
2. repeated page decorations
3. background images
4. icons with no knowledge value

Assets shall be stored in `assets/` and linked from Markdown.

Do not base64 embed assets in Markdown.

---

## 8. Index, README, and Report

### `index.md`

`index.md` is for agent navigation.

It shall include:

1. export title
2. short description
3. file list
4. links to converted files
5. original filename
6. converted filename
7. source file type
8. basic category
9. status
10. short notes when available
11. original folder path when folder upload is used

Example:

```md
# AgentReady Index

This index helps an AI agent navigate the exported files.

## Files

| Original file | Converted file | Type | Status | Notes |
|---|---|---|---|---|
| Staff Handbook.pdf | documents/staff-handbook-pdf.md | PDF | converted | Staff policy document |
| Volunteer List.xlsx | data/volunteer-list-xlsx.md | XLSX | partially_converted | Review tables before use |
```

### `README.md`

`README.md` is for humans.

It shall explain:

1. what the export is
2. how to use the files
3. why Markdown is used
4. where to put the zip or extracted folder
5. privacy note
6. warning to review files before trusting them fully

### `conversion-report.md`

`conversion-report.md` shall include:

1. conversion date
2. export structure
3. successful files
4. partially converted files
5. failed files
6. unsupported files
7. failure reasons
8. privacy note
9. limits note
10. no-AI-processing note for V1

The results screen shows a short report. The zip includes the full report.

---

## 9. Error Code Registry

Rust CLI, Fastify server, and React UI must use the same registry.

New error codes require an SDD update.

| Code | User-facing message | Status | Include in zip? |
|---|---|---|---|
| `UNSUPPORTED_FILE` | This file type is not supported in AgentReady V1. | failed | No |
| `FILE_TOO_LARGE` | This file is larger than the V1 file size limit. | failed | No |
| `NO_READABLE_TEXT` | AgentReady could not find readable text in this file. OCR is not supported in V1. | failed | No |
| `PASSWORD_PROTECTED` | This file appears to be password protected or encrypted. Password protected files are not supported in V1. | failed | No |
| `CONVERSION_FAILED` | AgentReady could not convert this file. | failed | No |
| `PARTIALLY_CONVERTED` | This file was converted, but the output may need review. | partially_converted | Yes |
| `CANCELLED` | The conversion was cancelled. | cancelled | No |
| `TEMP_FILE_ERROR` | AgentReady had a temporary file processing problem. | failed | No |
| `ZIP_CREATION_FAILED` | AgentReady could not create the export zip. | failed | No |
| `BLOCKED_EXTENSION` | This file type is blocked for safety. | failed | No |
| `INVALID_ARCHIVE_PATH` | AgentReady blocked an unsafe file path. | failed | No |
| `BATCH_LIMIT_EXCEEDED` | This batch has more files than AgentReady V1 supports. | failed | No |

---

## 10. Privacy and Security

### Product privacy principle

AgentReady is a preparation tool. It does not keep source documents.

### Upload page privacy copy

```md
## What happens to my files?

Your files are processed temporarily so AgentReady can create your export.

AgentReady does not keep your source documents after conversion. Once your AgentReady zip is created, the uploaded files and temporary working files are deleted.

You decide where to store the final zip, such as Google Drive, Dropbox, your computer, or your own agent knowledge folder.
```

### Required privacy behavior

1. Uploads are temporary.
2. Temporary working files are temporary.
3. Source documents are deleted after conversion, download, cancellation, or cleanup.
4. Temporary logs are deleted after conversion, download, cancellation, or cleanup.
5. Logs shall not include document text, spreadsheet contents, or private content.
6. Logs may include filename, file type, size, status, conversion stage, error code, and failure reason.
7. V1 shall not send documents to an AI model.
8. V1 shall not persist user workspaces.
9. V1 shall not require accounts.

### Safety checks

V1 shall:

1. validate file extensions
2. validate MIME types where possible
3. enforce file size limits
4. enforce batch size limits
5. block dangerous extensions
6. never execute macros
7. never execute scripts
8. never open active content as code
9. store temp files in isolated job folders
10. prevent path traversal in zip generation
11. delete temp job folders after completion or cleanup
12. report unsupported or suspicious files clearly

### Password-protected files

If a file is password protected or encrypted:

1. skip it
2. mark it as failed
3. show the `PASSWORD_PROTECTED` message
4. do not ask for the password in V1
5. do not attempt bypass

---

## 11. Technical Architecture

### Architecture decision

AgentReady V1 uses:

1. Rust conversion engine
2. Rust CLI
3. TypeScript Fastify server
4. Vite plus React web interface

### Why

Rust handles conversion, file safety, deterministic outputs, and zip generation.

TypeScript handles web upload, progress, preview, results, and download.

The Rust engine must stay independent from the web interface so the same conversion logic can later be used by a CLI, desktop app, API worker, background job, or local-first tool.

### Monorepo structure

```text
agentready-knowledge-packager/
  README.md
  specs/
  crates/
    agentready-core/
    agentready-cli/
  apps/
    web/
    server/
  examples/
    sample-input/
    expected-output/
  scripts/
```

### Rust crates

#### `agentready-core`

Responsibilities:

1. file validation
2. file type detection
3. conversion orchestration
4. output naming
5. Markdown generation
6. index generation
7. README generation
8. conversion report generation
9. asset handling
10. zip creation
11. error code registry
12. deterministic output structure

#### `agentready-cli`

Responsibilities:

1. parse command line flags
2. call `agentready-core`
3. support folder conversion
4. support individual files
5. print human-readable logs by default
6. output JSON when `--json` is passed

### TypeScript server

Use Fastify.

Responsibilities:

1. receive uploads
2. validate obvious limits before calling CLI
3. create temporary job folder
4. call Rust CLI with `--json`
5. stream or poll progress if supported
6. serve preview files
7. serve zip download
8. delete temp files after completion or cleanup
9. map Rust error codes to UI responses

### Web app

Use Vite plus React.

Responsibilities:

1. upload UI
2. selected file review
3. conversion progress
4. cancel action
5. results view
6. preview rendered Markdown
7. preview raw Markdown
8. copy raw Markdown
9. download individual Markdown
10. download zip
11. thank you and start over state

---

## 12. Rust CLI Contract

### Commands

Convert folder:

```bash
agentready convert ./input-folder --output ./agentready-output
```

Convert individual files:

```bash
agentready convert staff-handbook.pdf volunteer-list.xlsx --output ./agentready-output
```

JSON mode for server:

```bash
agentready convert ./input-folder --output ./agentready-output --json
```

### Required flags

| Flag | Purpose |
|---|---|
| `--output <path>` | Output folder path |
| `--json` | Emit structured JSON for server use |
| `--zip-name <name>` | Optional zip name override |
| `--max-files <n>` | Optional override, defaults to 25 |
| `--max-file-size-mb <n>` | Optional override, defaults to 50 |

No config file in V1. Command line flags only.

### JSON output shape

```json
{
  "status": "completed",
  "output_folder": "./agentready-output",
  "zip_path": "./agentready-output.zip",
  "summary": {
    "total_files": 5,
    "converted": 3,
    "partially_converted": 1,
    "failed": 1,
    "unsupported": 0
  },
  "files": [
    {
      "original_path": "Staff Handbook.pdf",
      "original_filename": "Staff Handbook.pdf",
      "source_type": "PDF",
      "status": "converted",
      "output_path": "documents/staff-handbook-pdf.md",
      "error_code": null,
      "message": null
    }
  ],
  "errors": []
}
```

Allowed top-level `status` values:

1. `completed`
2. `completed_with_warnings`
3. `failed`
4. `cancelled`

Allowed file `status` values:

1. `converted`
2. `partially_converted`
3. `failed`
4. `unsupported`
5. `cancelled`

### Determinism

The CLI shall produce mostly deterministic outputs:

1. stable folder structure
2. stable filenames
3. stable Markdown content where parser output is stable
4. stable index links
5. stable README structure
6. stable report sections
7. stable JSON output shape

Byte-for-byte reproducible zip is not required in V1 because zip metadata and timestamps may vary.

---

## 13. Fastify Server Contract

### Endpoints

| Method | Path | Purpose |
|---|---|---|
| `POST` | `/api/jobs` | Upload files and start conversion |
| `GET` | `/api/jobs/:jobId` | Get job status |
| `POST` | `/api/jobs/:jobId/cancel` | Cancel job |
| `GET` | `/api/jobs/:jobId/files/:fileId/preview` | Get converted Markdown preview |
| `GET` | `/api/jobs/:jobId/files/:fileId/download` | Download individual converted file |
| `GET` | `/api/jobs/:jobId/download` | Download export zip |
| `DELETE` | `/api/jobs/:jobId` | Cleanup job files |

### Job states

1. `created`
2. `validating`
3. `converting`
4. `packaging`
5. `ready`
6. `completed_with_warnings`
7. `failed`
8. `cancelled`
9. `cleaned_up`

### Temporary file lifecycle

1. Create isolated job folder.
2. Save uploads into job input folder.
3. Run CLI with job output folder.
4. Serve preview and zip while job is active.
5. Delete job folder after download, cancellation, explicit cleanup, or cleanup timeout.

### Server must not

1. store files permanently
2. log document contents
3. send documents to AI
4. trust filenames without sanitization
5. allow path traversal
6. expose raw filesystem paths to the browser

---

## 14. React Web Interface Spec

### Visual direction

The web interface should be polished SaaS-style, but V1 does not need a final brand system.

Style words:

1. clean
2. modern
3. professional
4. trustworthy
5. simple
6. calm
7. non-technical

### Interface tone

Use friendly and simple wording.

Avoid:

1. unnecessary technical language
2. enterprise jargon
3. developer-first labels
4. hype-heavy marketing claims
5. unsupported environmental promises

### Main page sections

1. Hero
2. Upload area
3. How it works
4. Supported files and limits
5. What happens to my files?
6. Why Markdown?
7. Simple footer

### Hero copy

```md
# Is your data agent ready?

Upload your documents and data. AgentReady converts them into clean Markdown files that your AI tools can read faster, understand more clearly, and process with less unnecessary token waste.
```

### How it works copy

```md
## How it works

1. Upload your files.
2. AgentReady converts them.
3. Preview the Markdown.
4. Download your AgentReady zip.
```

### Supported files copy

```md
## Supported files

PDF, DOCX, TXT, Markdown, CSV, XLSX

Up to 25 files per batch. Up to 50 MB per file.
```

### Export explanation copy

```md
## What is in your AgentReady export?

Your export is a zip file containing clean Markdown versions of your documents and data.

Markdown is a simple text format that AI agents can read more easily than messy PDFs, Word files, spreadsheets, or HTML. It is also readable by humans, so you can open it, review it, edit it, and store it wherever you want.

Your export may include:

- README.md, which explains how to use the export
- index.md, which helps your agent navigate the files
- converted Markdown files
- CSV copies of spreadsheet data when needed
- useful images or diagrams when needed
- conversion-report.md, which shows what converted successfully and what did not
```

### Preview screen

Preview one converted file at a time.

Left side:

1. file list
2. status badges
3. failure warnings

Main area:

1. rendered Markdown
2. raw Markdown toggle
3. copy raw Markdown button
4. download individual Markdown button

No Markdown editing in V1.

---

## 15. Testing and CI

### Test principles

V1 requires strong tests because the product handles user files and privacy-sensitive workflows.

### Required tests

1. Rust unit tests for core logic
2. Rust integration tests for CLI
3. fixture tests with sample input and expected output
4. zip structure tests
5. error code tests
6. file naming tests
7. partial conversion warning tests
8. report generation tests
9. Fastify server endpoint tests
10. upload validation tests
11. cleanup tests
12. basic UI flow tests
13. preview toggle tests
14. download tests
15. cancel conversion tests

### Required CI

CI shall run:

1. Rust format check
2. Rust build
3. Rust tests
4. TypeScript type check
5. Fastify server build
6. Vite web app build
7. frontend tests
8. linting where practical

### Example fixtures

Include:

```text
examples/
  sample-input/
    staff-handbook.txt
    policy.md
    volunteer-list.csv
    simple-data.xlsx
    sample-document.pdf
    sample-document.docx
  expected-output/
    agentready-output/
      README.md
      index.md
      conversion-report.md
      documents/
      data/
```

---

## 16. Dependency and License Policy

### License rule

Avoid GPL dependencies unless Dwayne explicitly approves them.

Prefer:

1. MIT
2. Apache 2.0
3. BSD
4. permissive dual licenses

### Marker rule

Marker and similar GPL projects may be used for inspiration only.

Do not copy:

1. code
2. tests
3. prompts
4. assets
5. implementation details
6. internal architecture if protected by license

### Dependency strategy

Prefer Rust-native libraries where practical.

External tools may be considered if:

1. quality is significantly better
2. license is acceptable
3. security risk is acceptable
4. deployment complexity is acceptable
5. the coding agent documents why the dependency was chosen

---

## 17. Phase 2 Roadmap

Phase 2 ideas are explicitly out of scope for V1, but the architecture should not block them.

### Future advanced export

Advanced Export may include:

1. metadata JSON
2. source maps
3. JSONL chunks
4. database-ready files
5. RAG ingestion packages
6. agent framework specific packages
7. optional chunking rules
8. optional source hash

### Future website readiness

Future website features may include:

1. website scanning
2. page-to-Markdown conversion
3. `llms.txt`
4. `agent.md`
5. service descriptions
6. structured organization profile
7. agent-readable site map

### Future AI enrichment

AI enrichment must be opt-in only.

Future features may include:

1. summaries
2. tags
3. suggested categories
4. knowledge map
5. missing-information detection

Rules:

1. user must opt in
2. user chooses provider or brings API key
3. documents are not sent to AI by default
4. generated content is clearly labeled as generated helper content

### Future OCR

Future OCR may support:

1. scanned PDFs
2. image documents
3. photos of documents

### Future audio

Future audio may support:

1. MP3
2. WAV
3. M4A
4. audiobooks
5. meeting recordings
6. transcripts

Possible pipeline:

```text
Audio file
-> audio decoding and metadata extraction
-> speech-to-text
-> transcript cleanup
-> Markdown generation
-> index entry
-> AgentReady export zip
```

Symphonia can be researched as a Rust audio decoding/demuxing candidate, but it is not a transcription engine.

---

## 18. Implementation Plan for Codex or VibeCode

### Phase 0: Repository setup

Create monorepo:

```text
agentready-knowledge-packager/
  specs/
  crates/
    agentready-core/
    agentready-cli/
  apps/
    web/
    server/
  examples/
  scripts/
```

Add:

1. root README
2. license decision note
3. CI workflow
4. Rust workspace
5. TypeScript workspace
6. sample fixtures

### Phase 1: Rust core skeleton

Build:

1. file type enum
2. conversion status enum
3. error code enum
4. file validation
5. filename sanitizer
6. output path generator
7. report data model
8. basic Markdown writer

### Phase 2: CLI skeleton

Build:

1. `agentready convert`
2. folder input support
3. individual file input support
4. `--output`
5. `--json`
6. default human-readable logs
7. output folder creation
8. zip creation stub

### Phase 3: Simple converters

Build first:

1. TXT converter
2. Markdown passthrough plus frontmatter
3. CSV converter

Then add:

1. XLSX converter
2. DOCX converter
3. PDF text extraction

### Phase 4: Export package

Build:

1. `README.md` generator
2. `index.md` generator
3. `conversion-report.md` generator
4. data folder outputs
5. document folder outputs
6. assets folder creation only when needed
7. zip creation
8. deterministic structure tests

### Phase 5: Server

Build Fastify server:

1. upload endpoint
2. temp job folder
3. call CLI with `--json`
4. status endpoint
5. preview endpoint
6. individual download endpoint
7. zip download endpoint
8. cancel endpoint
9. cleanup endpoint

### Phase 6: Web UI

Build React UI:

1. upload screen
2. selected files screen
3. progress screen
4. results screen
5. preview screen
6. raw/rendered toggle
7. copy raw Markdown
8. download zip
9. thank you/start over

### Phase 7: Tests and polish

Add:

1. fixture tests
2. zip structure tests
3. error behavior tests
4. privacy cleanup tests
5. UI smoke tests
6. copy polish
7. accessibility checks

---

## 19. Build Rules for the Coding Agent

The coding agent must follow these rules:

1. Use this SDD as the source of truth.
2. Do not add login, billing, persistent storage, AI summaries, OCR, website scanning, embeddings, or database export unless Dwayne asks.
3. Do not introduce GPL dependencies without explicit approval.
4. Keep the Rust conversion core independent from the web UI.
5. Keep user-facing language simple and non-technical.
6. Preserve privacy behavior.
7. Continue conversion when some files fail.
8. Never silently hide failed files.
9. Use the error code registry.
10. Update the SDD before adding new error codes or major behavior changes.
11. Optimize Markdown for agent understanding rather than visual reproduction.
12. Use tests before expanding scope.

---

## 20. Acceptance Criteria

AgentReady V1 is acceptable when:

1. A user can upload supported files.
2. Unsupported or oversized files are clearly shown.
3. The user can remove selected files before conversion.
4. Conversion continues even if some files fail.
5. The user sees progress.
6. The user can cancel conversion.
7. The user sees results with clear status for each file.
8. The user can preview converted Markdown.
9. The user can toggle rendered and raw Markdown.
10. The user can copy raw Markdown.
11. The user can download individual Markdown files.
12. The user can download a Simple Export zip.
13. The zip contains the required structure.
14. Partially converted files include the warning block.
15. Failed files are not silently included.
16. `index.md` helps agents navigate the files.
17. `README.md` helps humans understand the export.
18. `conversion-report.md` lists successful and failed files.
19. Uploaded source files and temporary files are deleted after completion or cleanup.
20. Logs do not include document contents.
21. The UI is clear for non-technical staff.
22. Tests and CI pass.

---

## 21. Source Notes

This SDD uses public technical references only for general format reasoning:

1. CommonMark publishes a formal Markdown specification and test suite.
2. MDN documents HTML as a web markup language made from elements, tags, attributes, nesting, document structure, and related syntax.
3. OpenAI API documentation shows that prompt size, reusable prompt structure, caching, latency, and input token costs are related operational concerns. AgentReady should therefore make careful, non-guaranteed efficiency claims rather than promise fixed savings.


---

## Latest decision addendum Q107–Q261

The latest product, UX, beta, credits, donation, service, and future anonymization decisions are captured in:

```text
docs/21_DECISIONS_Q107_Q261.md
docs/22_DECISIONS_EDGE_CASES.md
```

This addendum updates and refines the original master SDD. If the addendum conflicts with an earlier part of this master document, the addendum is the newer product decision unless Dwayne explicitly changes it later.

Core updates:

1. Build CLI first, then web app.
2. Local first, then hosted private beta.
3. First local build supports TXT, Markdown, CSV, and DOCX before PDF/XLSX.
4. Prioritize conversion quality over broad weak file coverage.
5. Use conservative quality handling with statuses: good, partial, failed, unsupported.
6. Use detailed CLI exit codes and structured JSON output.
7. Use AgentReady by HumanGoodAI brand connection.
8. Public hosted version has a public page but private beta conversion access using beta codes.
9. Future paid model uses AgentReady credits, with donation credits for mission-aligned organizations.
10. V1 does not include anonymization. Future sensitive-file mode and anonymization are planned.
11. Handle name collisions in `--recursive` mode by appending a counter.
12. Establish strict definition of partial success (e.g., failed to extract image, but text preserved).
13. Memory limits: stream TXT/CSV, cap DOCX/PDF to 50MB. Enforce 30-second processing timeout per file.
14. Ensure engine protects against Zip bombs and drops TXT/CSV BOMs.
