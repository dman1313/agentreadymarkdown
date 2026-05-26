# 21 Product and Implementation Decisions Q107–Q261

Status: build directive update  
Date: 2026-05-26  
Source: live product design Q&A with Dwayne Primeau

This document captures the product, UX, beta, pricing, donation, and future anonymization decisions made after the original V2-style SDD package was generated.

These decisions update and refine the earlier SDD. If this document conflicts with an earlier document, treat this document as the newer decision unless Dwayne explicitly changes it later.

---

## 21.1 Decision Policy for Future Build Work

### Decision

Codex, VibeCode, and implementation agents should not keep asking Dwayne every technical question. They should choose best-practice defaults for technical implementation details and only ask for input when the decision affects product strategy, user-facing experience, privacy promises, brand, pricing, access, or mission.

### Requirements

```md
REQ-DECISION-POLICY-001: Implementation agents shall auto-select best-practice defaults for routine technical implementation details.
REQ-DECISION-POLICY-002: Implementation agents shall ask Dwayne only for decisions that affect product promise, UX, privacy posture, branding, pricing, mission, access, or strategic tradeoffs.
REQ-DECISION-POLICY-003: Auto-selected technical decisions shall be documented briefly in the implementation log or pull request summary.
REQ-DECISION-POLICY-004: User-facing wording, business model choices, and privacy commitments shall not be invented without explicit product direction.
```

---

## 21.2 Build Sequence and First Demo

### Decisions

AgentReady should be built CLI first, then web app. The first local build should prove conversion with TXT, Markdown, CSV, and DOCX before adding harder formats. PDF and XLSX remain in V1 scope but may be implemented after the first local proof of concept.

The first demo should prioritize conversion quality over file type coverage or UI polish. Weak conversions should be handled conservatively.

### Requirements

```md
REQ-BUILD-SEQUENCE-001: AgentReady V1 shall build the Rust conversion engine before the web interface is treated as complete.
REQ-BUILD-SEQUENCE-002: The first working build shall expose file conversion through the Rust CLI.
REQ-BUILD-SEQUENCE-003: The CLI shall be able to convert supported files, create the output folder, create the export zip, and return structured status output.
REQ-BUILD-SEQUENCE-004: The web interface shall connect to the CLI after the CLI conversion path is working.
REQ-BUILD-SEQUENCE-005: The TypeScript web/server layer shall not duplicate conversion logic that belongs in the Rust engine.

REQ-RUN-LOCATION-001: AgentReady V1 shall be developed and tested locally first.
REQ-RUN-LOCATION-002: After the local CLI and basic web interface work, AgentReady may be deployed as a simple hosted test app.
REQ-RUN-LOCATION-003: The hosted test app shall use temporary processing and delete uploaded files after conversion, download, cancellation, or cleanup.
REQ-RUN-LOCATION-004: The hosted test app shall not require accounts, billing, or persistent document storage in V1.

REQ-FIRST-BUILD-001: The first local build shall support TXT, Markdown, CSV, and DOCX conversion.
REQ-FIRST-BUILD-002: PDF and XLSX support shall remain in V1 scope, but they may be implemented after the first local proof of concept.
REQ-FIRST-BUILD-003: The first local build shall still create the standard AgentReady output folder and zip.
REQ-FIRST-BUILD-004: The first local build shall include README.md, index.md, converted files, and conversion-report.md.

REQ-DEMO-PRIORITY-001: The first working demo shall prioritize clean, useful Markdown over broad but weak file type coverage.
REQ-DEMO-PRIORITY-002: Fewer file types are acceptable in the first demo if the supported conversions are reliable and agent-readable.
REQ-DEMO-PRIORITY-003: Conversion output shall be judged by readability, heading structure, section clarity, list quality, table handling, and reduced formatting noise.
REQ-DEMO-PRIORITY-004: The demo shall not claim success only because a file was technically converted. The converted output must be useful.

REQ-QUALITY-STRICTNESS-001: AgentReady V1 shall use a conservative quality policy.
REQ-QUALITY-STRICTNESS-002: If converted output may be unreliable, incomplete, poorly structured, or confusing for agents, AgentReady shall mark the file as partially converted or failed.
REQ-QUALITY-STRICTNESS-003: AgentReady shall not silently treat low-quality conversion output as successful.
REQ-QUALITY-STRICTNESS-004: Partially converted files shall include a visible warning at the top of the Markdown file.
REQ-QUALITY-STRICTNESS-005: Failed files shall be excluded from the export content but listed clearly in conversion-report.md.
```

---

## 21.3 Quality Status and Demo Fixtures

### Decisions

AgentReady V1 should include a simple quality status for every processed file: good, partial, failed, or unsupported. The repo should include clean and messy sample files so tests prove partial conversion and warnings from day one.

Demo examples should represent both mission-driven and business use cases, but the first screen examples should not divide users by sector.

### Requirements

```md
REQ-QUALITY-REPORT-001: AgentReady V1 shall include a simple quality status for each processed file.
REQ-QUALITY-REPORT-002: Quality status values shall be limited to good, partial, failed, and unsupported in V1.
REQ-QUALITY-REPORT-003: Quality status shall appear in the CLI JSON output, web results screen, index.md, and conversion-report.md.
REQ-QUALITY-REPORT-004: V1 shall avoid complex quality scoring unless added later as an advanced feature.

REQ-DEMO-FIXTURES-001: AgentReady V1 shall include sample input files for testing conversion from day one.
REQ-DEMO-FIXTURES-002: Sample files shall include clean TXT, Markdown, CSV, and DOCX examples.
REQ-DEMO-FIXTURES-003: Sample files shall also include intentionally messy examples to test partial conversion and warning behavior.
REQ-DEMO-FIXTURES-004: Test fixtures shall verify that AgentReady does not silently mark messy or unreliable conversions as successful.
REQ-DEMO-FIXTURES-005: Expected output examples shall be included so coding agents can compare generated output against known good output.

REQ-DEMO-DOCUMENTS-001: AgentReady V1 shall include sample files representing school, nonprofit, and small business use cases.
REQ-DEMO-DOCUMENTS-002: Demo files shall avoid real personal data. All names, emails, addresses, and phone numbers shall be fictional.

REQ-BEFORE-AFTER-001: AgentReady V1 shall include before-and-after examples in the demo documentation.
REQ-BEFORE-AFTER-002: The first app interface shall not require a side-by-side before-and-after comparison view.
REQ-BEFORE-AFTER-003: Before-and-after examples shall show how AgentReady improves structure, readability, and agent usability.
REQ-BEFORE-AFTER-004: Before-and-after examples shall avoid exaggerated claims and shall not imply guaranteed token savings.
```

Suggested sample fixture layout:

```text
examples/
  sample-input/
    clean/
      simple-note.txt
      policy.md
      volunteer-list.csv
      staff-handbook.docx
    messy/
      badly-structured-note.txt
      inconsistent-policy.md
      messy-volunteer-list.csv
      complex-formatting.docx
```

---

## 21.4 Export Model, Citation, Trust, and README

### Decisions

The default V1 export remains the Simple Export. Advanced developer export, project type selector, and agent-specific instructions are future features. V1 should include basic source citation guidance using the original source filename only.

The export README should include a quick start, folder map, trust warning, and practical next steps. Starting over should clear everything and remember nothing.

### Requirements

```md
REQ-ADVANCED-EXPORT-001: AgentReady V1 shall prioritize the simple export before advanced developer export features.
REQ-ADVANCED-EXPORT-002: Advanced developer export shall be planned as a later mode, not a blocker for the first working product.
REQ-ADVANCED-EXPORT-003: Future advanced export may include JSON metadata, JSONL chunks, source maps, database-ready files, and RAG ingestion support.
REQ-ADVANCED-EXPORT-004: The simple export shall remain the default experience for non-technical users.

REQ-PROJECT-TYPE-001: AgentReady V1 shall not require users to choose a project type before export.
REQ-PROJECT-TYPE-002: A project type selector may be added later as an advanced feature.
REQ-PROJECT-TYPE-003: Future project types may include Claude projects, ChatGPT projects, OpenClaw agents, NotebookLM, RAG databases, Google Drive knowledge folders, Obsidian vaults, and Notion workspaces.

REQ-AGENT-INSTRUCTIONS-001: AgentReady V1 shall not require agent-specific instruction files in the default simple export.
REQ-AGENT-INSTRUCTIONS-002: Future advanced exports may include an agent-instructions.md file.
REQ-AGENT-INSTRUCTIONS-003: Future agent instructions shall tell agents to check index.md, README.md, and conversion-report.md before relying on converted files.

REQ-CITATION-001: AgentReady V1 shall include basic source citation guidance in the simple export.
REQ-CITATION-002: The export shall tell users and agents to reference the original source file when using information from converted Markdown.
REQ-CITATION-003: Citation guidance shall appear in README.md and may also appear in index.md or conversion-report.md.
REQ-CITATION-004: Converted Markdown frontmatter shall preserve the original source filename.
REQ-CITATION-005: AgentReady V1 shall use simple source filename citation guidance only.
REQ-CITATION-006: V1 shall not require page-level, paragraph-level, or section-level citation mapping.

REQ-TRUST-WARNING-001: AgentReady V1 shall include a short trust warning in README.md.
REQ-TRUST-WARNING-002: The trust warning shall remind users to review converted files before treating them as official knowledge.
REQ-TRUST-WARNING-003: The trust warning shall point users to conversion-report.md for warnings and conversion status.

REQ-README-QUICKSTART-001: AgentReady V1 shall include a simple quick start section in the exported README.md.
REQ-README-QUICKSTART-002: The quick start shall tell users to open index.md first, check conversion-report.md, review partial files, and reference original source filenames.

REQ-README-FOLDER-MAP-001: AgentReady V1 shall include a simple folder map in the exported README.md.
REQ-README-FOLDER-MAP-002: The folder map shall explain README.md, index.md, conversion-report.md, documents/, data/, and assets/.
REQ-README-FOLDER-MAP-003: The assets/ folder shall only be shown or described as present when useful extracted assets exist, or the README shall clarify that it appears only when needed.

REQ-RESULTS-NEXT-STEPS-001: AgentReady V1 shall show practical next steps after the export is ready.
REQ-RESULTS-NEXT-STEPS-002: Next steps shall tell users to download the zip, open index.md, check conversion-report.md, review partial files, and store the export safely.

REQ-START-OVER-001: AgentReady V1 shall include a start over option after the export is ready.
REQ-START-OVER-002: Before clearing the current session, AgentReady shall ask users to confirm.
REQ-START-OVER-003: Starting over shall clear the current file list, preview state, conversion results, and temporary session state.
REQ-START-OVER-MEMORY-001: AgentReady V1 shall not retain document history after the user starts over.
REQ-START-OVER-MEMORY-002: AgentReady V1 shall not show recent conversions or recent documents.
```

Suggested README warning:

```md
## Review before using

AgentReady helps convert your files into cleaner, more agent-readable Markdown.

Before using the converted files as official knowledge, review the output and check conversion-report.md for warnings. Files marked as partial may be incomplete or less reliable.
```

Suggested quick start:

```md
## Quick start

1. Open index.md to see what is included in this export.
2. Review the converted Markdown files.
3. Check conversion-report.md for warnings or failed files.
4. Use the converted files in your AI tool, agent folder, knowledge base, or document workspace.
5. Reference the original source filename when using information from the export.
```

---

## 21.5 CLI and Rust Core Decisions

### Decisions

The CLI should use a simple `agentready convert` command. It should use human-readable output by default and `--json` for the web server or automation. It should continue after per-file failures, use detailed exit codes, and expose summary plus per-file JSON results.

Dry run, include-hidden, overwrite, zip-only, and open-output flags are future features unless implementation agents add them deliberately after core V1 is stable.

### Requirements

```md
REQ-CLI-COMMAND-001: AgentReady V1 shall use agentready convert as the primary CLI command.
REQ-CLI-COMMAND-002: The CLI shall support folder input using agentready convert ./input-folder --output ./agentready-output.
REQ-CLI-COMMAND-003: The CLI shall support explicit file input using agentready convert file1.docx file2.csv --output ./agentready-output.
REQ-CLI-COMMAND-004: The CLI shall create the output folder and export zip automatically.

REQ-CLI-OUTPUT-001: AgentReady CLI shall use human-readable output by default.
REQ-CLI-OUTPUT-002: Default CLI output shall show simple progress, converted files, partial files, failed files, and final export location.
REQ-CLI-OUTPUT-003: AgentReady CLI shall support a --json flag for structured machine-readable output.
REQ-CLI-OUTPUT-004: The Fastify server shall use the CLI --json output when calling the Rust CLI.
REQ-CLI-OUTPUT-005: Human-readable CLI output shall avoid printing document text, spreadsheet contents, or private file content.

REQ-CLI-PARTIAL-SUCCESS-001: AgentReady CLI shall continue processing remaining files when one file fails.
REQ-CLI-PARTIAL-SUCCESS-002: Failed files shall not stop the entire batch conversion.
REQ-CLI-PARTIAL-SUCCESS-003: Failed files shall be excluded from the export content but listed in conversion-report.md.
REQ-CLI-PARTIAL-SUCCESS-004: If all files fail, the CLI shall not create a misleading successful export and shall return a clear failure result.

REQ-CLI-EXIT-001: AgentReady CLI shall use detailed exit codes in V1.
REQ-CLI-EXIT-002: Exit code 0 shall mean all files were converted successfully.
REQ-CLI-EXIT-003: Exit code 1 shall mean partial success.
REQ-CLI-EXIT-004: Exit code 2 shall mean all files failed, were unsupported, or produced no usable output.
REQ-CLI-EXIT-005: Exit code 3 shall mean bad input, invalid command usage, missing input path, missing output path, unsupported arguments, or validation error.
REQ-CLI-EXIT-006: Exit code 4 shall mean system error, such as temporary file failure, zip creation failure, permission error, or unexpected internal failure.

REQ-CLI-JSON-001: AgentReady CLI --json output shall include both an overall summary and per-file results.
REQ-CLI-JSON-002: The JSON summary shall include overall status, exit code, input path, output folder path, export zip path, and file counts.
REQ-CLI-JSON-003: Per-file results shall include source filename, source type, status, output file path when available, error code when available, and warning when available.
REQ-CLI-JSON-004: The CLI JSON output shall not include document text, spreadsheet contents, or private source content.

REQ-CLI-DRY-RUN-001: AgentReady V1 first build shall not require dry run mode.
REQ-CLI-DRY-RUN-002: Dry run mode may be added later as a CLI validation feature.

REQ-CLI-RECURSIVE-001: AgentReady CLI shall support recursive folder input through an explicit --recursive flag.
REQ-CLI-RECURSIVE-002: By default, folder input shall process only files directly inside the selected folder.
REQ-CLI-RECURSIVE-003: When --recursive is used, AgentReady shall scan supported files inside nested subfolders.
REQ-CLI-RECURSIVE-004: Original relative folder paths shall be recorded in index.md when recursive input is used.
REQ-CLI-RECURSIVE-005: The export folder structure shall remain simple and shall not fully mirror the original folder tree in V1.

REQ-HIDDEN-FILES-001: AgentReady V1 shall ignore hidden files and system files by default.
REQ-HIDDEN-FILES-002: AgentReady shall skip common operating system files such as .DS_Store and Thumbs.db.
REQ-HIDDEN-FILES-003: AgentReady shall skip development folders such as .git/ and node_modules/.
REQ-HIDDEN-FILES-004: A future CLI version may include an --include-hidden flag for advanced users.

REQ-OUTPUT-FOLDER-001: AgentReady V1 shall not overwrite an existing output folder by default.
REQ-OUTPUT-FOLDER-002: If the requested output folder already exists, the CLI shall stop before conversion begins.
REQ-OUTPUT-FOLDER-003: Existing output folder conflicts shall use exit code 3.
REQ-OUTPUT-OVERWRITE-001: A future CLI version may include an --overwrite flag for advanced users and automated workflows.

REQ-ZIP-NAMING-001: AgentReady V1 shall name the export zip to match the output folder name.
REQ-ZIP-NAMING-002: If the output folder is agentready-output, the zip shall be agentready-output.zip.
REQ-ZIP-EXISTS-001: AgentReady V1 shall not overwrite an existing export zip by default.
REQ-ZIP-EXISTS-002: Existing zip conflicts shall use exit code 3.
REQ-ZIP-LOCATION-001: AgentReady V1 shall create the output folder and matching export zip in the same parent location by default.
REQ-OUTPUT-CLEANUP-001: AgentReady V1 shall keep both the generated output folder and matching export zip by default.
REQ-OPEN-OUTPUT-001: AgentReady V1 first CLI build shall not require an open output folder command.
REQ-OPEN-OUTPUT-002: A future CLI version may include an --open-output flag.

REQ-CLI-VERSION-001: AgentReady V1 CLI shall include a version command in the first build.
REQ-CLI-VERSION-002: The CLI shall support agentready --version.
REQ-CLI-HELP-001: AgentReady V1 CLI shall include help output in the first build.
REQ-CLI-HELP-002: The CLI shall support agentready --help and agentready convert --help.

REQ-CLI-PARSER-001: AgentReady V1 CLI shall use clap for Rust command line argument parsing.
REQ-CLI-PARSER-002: clap shall handle top-level help output, version output, subcommands, required arguments, and flags.

REQ-CORE-LOCATION-001: AgentReady V1 conversion logic shall live in the agentready-core crate.
REQ-CORE-LOCATION-002: The agentready-cli crate shall act as a command line wrapper around agentready-core.
REQ-CORE-LOCATION-003: The CLI shall not duplicate conversion, Markdown generation, report generation, or export package logic.
REQ-CORE-RESULT-001: agentready-core shall return a structured conversion result object after processing.
REQ-CORE-RESULT-002: The structured result shall include overall status, output paths, summary counts, per-file results, warnings, and errors.
REQ-CORE-STATUS-001: agentready-core shall use detailed batch status values.
REQ-CORE-STATUS-002: Batch status values shall include success, partial_success, failed, cancelled, validation_error, and system_error.
REQ-CORE-STATUS-003: Status values shall be separate from CLI exit codes.
```

Example JSON shape:

```json
{
  "status": "partial_success",
  "exit_code": 1,
  "input": "./sample-input",
  "output_folder": "./agentready-output",
  "export_zip": "./agentready-output.zip",
  "summary": {
    "total_files": 4,
    "converted": 2,
    "partial": 1,
    "failed": 1,
    "unsupported": 0
  },
  "files": [
    {
      "source_file": "simple-note.txt",
      "source_type": "txt",
      "status": "good",
      "output_file": "documents/simple-note.md",
      "error_code": null,
      "warning": null
    },
    {
      "source_file": "staff-handbook.docx",
      "source_type": "docx",
      "status": "partial",
      "output_file": "documents/staff-handbook.md",
      "error_code": "PARTIALLY_CONVERTED",
      "warning": "AgentReady converted this file, but some structure or content may be incomplete."
    }
  ]
}
```

---

## 21.6 User-Facing App Copy

### Core public demo message

```text
Upload your files, preview the AgentReady Markdown, and download an agent-ready zip.

AgentReady helps convert your documents and data into a clean format that is easier for AI agents to read.
```

### Main CTAs

```text
First screen primary button: Upload files
After files are selected: Create AgentReady export
Downloaded zip label: AgentReady export
Download button: Download your AgentReady export as a zip
Private beta button: Request beta access
Service CTA: Get your organization agent-ready
Donation CTA: Help an organization become agent-ready
```

### Privacy and processing copy

```text
Your files are only used to create your AgentReady export zip.

After your AgentReady export is created, your uploaded files and temporary working files are deleted to help protect your privacy.
```

```text
Privacy-first by design.
```

```text
Creating your AgentReady export...

Privacy-first by design. Your files are being processed temporarily to create your export zip.
```

### Completion and error copy

```text
Your AgentReady export zip is ready to download.
```

```text
Your AgentReady export zip is ready to download. Some files did not convert, so please review the results before use.
```

```text
AgentReady could not create an export from these files. Please review the file types and try again.
```

```text
AgentReady does not support this file type yet. Please use PDF, DOCX, TXT, Markdown, CSV, or XLSX.
```

```text
AgentReady converted this file, but some structure or content may be incomplete.
```

```text
AgentReady could not process this file because it is over the 50 MB limit.
```

### Requirements

```md
REQ-DEMO-MESSAGE-001: The first public demo message shall explain that users can upload files, preview the AgentReady Markdown, and download an agent-ready zip.
REQ-DEMO-MESSAGE-002: The demo message shall explain that AgentReady converts files into a cleaner format that is easier for AI agents to read.
REQ-DEMO-MESSAGE-003: The message shall avoid exaggerated claims, exact token-saving promises, or technical language that non-technical users may not understand.

REQ-FIRST-SCREEN-CTA-001: The first screen primary button shall say Upload files.
REQ-UPLOADED-FILES-CTA-001: After users select files, the primary action button shall say Create AgentReady export.
REQ-DOWNLOAD-NAME-001: The user-facing app shall refer to the downloaded zip as the AgentReady export.
REQ-DOWNLOAD-CTA-001: The results screen download button shall say Download your AgentReady export as a zip.

REQ-UPLOAD-PRIVACY-COPY-001: The upload screen shall explain that user files are only used to create the AgentReady export zip.
REQ-UPLOAD-PRIVACY-COPY-002: The upload screen shall explain that uploaded files and temporary working files are deleted after the AgentReady export is created.
REQ-UPLOAD-PRIVACY-COPY-003: Privacy wording shall avoid unsupported absolute guarantees such as privacy assured.
REQ-UPLOAD-PRIVACY-COPY-004: The upload screen may use the supporting phrase Privacy-first by design.

REQ-PROCESSING-COPY-001: While files are being converted, AgentReady shall show the message Creating your AgentReady export...
REQ-PROCESSING-COPY-002: The processing screen may include the supporting phrase Privacy-first by design.
REQ-FINISHED-COPY-001: When conversion is complete, AgentReady shall show the message Your AgentReady export zip is ready to download.
REQ-PARTIAL-FINISHED-COPY-001: If the export is created but some files failed, AgentReady shall show the partial success message listed above.
REQ-NO-EXPORT-COPY-001: If no files convert successfully, AgentReady shall not create a misleading export zip.
REQ-UNSUPPORTED-FILE-COPY-001: If a user selects an unsupported file type, AgentReady shall show the unsupported file message listed above.
REQ-PARTIAL-FILE-COPY-001: If a file is converted but may need review, AgentReady shall show the partial conversion warning listed above.
REQ-FILE-SIZE-COPY-001: If a file is over the V1 size limit, AgentReady shall show the file size message listed above.
```

---

## 21.7 Visual Identity and First Page UX

### Decisions

AgentReady should use a clean SaaS style with HumanGoodAI warmth. The app should visibly say `AgentReady by HumanGoodAI` in the header and footer. The footer should say `A HumanGoodAI tool · Privacy-first by design`.

AgentReady should use the HumanGoodAI brand family with a slight product accent communicating trust, privacy, clarity, and organization. The icon should be a soft rounded document-to-agent visual, not a cartoon robot.

The first screen should show the three-step flow, supported file types, file limits, a short Markdown explanation, and a lower-page example section.

### Requirements

```md
REQ-VISUAL-STYLE-001: AgentReady V1 shall use a clean SaaS-style interface with HumanGoodAI warmth.
REQ-VISUAL-STYLE-002: AgentReady shall visually feel like part of the HumanGoodAI brand family.
REQ-VISUAL-STYLE-003: The interface shall feel modern, simple, professional, human-centered, and trustworthy.
REQ-VISUAL-STYLE-004: AgentReady shall avoid a cold developer-dashboard feel in the default user interface.
REQ-VISUAL-STYLE-005: AgentReady shall avoid overly playful, cartoonish, or gimmicky visual design.

REQ-BRAND-VISIBILITY-001: AgentReady V1 shall visibly connect the product to HumanGoodAI.
REQ-BRAND-VISIBILITY-002: The app shall use the wording AgentReady by HumanGoodAI.
REQ-BRAND-PLACEMENT-001: AgentReady V1 shall show the HumanGoodAI brand connection in both the header and footer.
REQ-FOOTER-001: AgentReady V1 shall include a simple footer.
REQ-FOOTER-002: The footer shall include the wording A HumanGoodAI tool · Privacy-first by design.

REQ-COLOR-PALETTE-001: AgentReady V1 shall use the HumanGoodAI brand color family.
REQ-COLOR-PALETTE-002: AgentReady may include a slight product-specific accent color to distinguish it from the main HumanGoodAI website.
REQ-ACCENT-STYLE-001: AgentReady’s product accent shall communicate trust, privacy, clarity, and organization.
REQ-ACCENT-STYLE-002: The visual design shall support the idea that AgentReady makes messy files cleaner and easier for agents to read.

REQ-ICON-001: AgentReady V1 shall use a small document-to-agent visual symbol.
REQ-ICON-002: The icon shall suggest that messy documents are transformed into clean, structured, agent-readable files.
REQ-ICON-STYLE-001: AgentReady V1 shall use a soft rounded icon style.
REQ-ICON-STYLE-002: The icon shall feel professional, warm, simple, and human-centered.
REQ-ICON-STYLE-003: The icon shall avoid harsh developer-tool styling and overly cartoonish robot imagery.

REQ-FIRST-SCREEN-STEPS-001: AgentReady V1 shall show a simple three-step explanation on the first screen.
REQ-FIRST-SCREEN-STEPS-002: The three steps shall be Upload your files, Preview the AgentReady Markdown, and Download your export zip.
REQ-SUPPORTED-FILES-DISPLAY-001: AgentReady V1 shall clearly show supported file types before upload.
REQ-SUPPORTED-FILES-DISPLAY-002: The message shall say Supported files: PDF, DOCX, TXT, Markdown, CSV, and XLSX.
REQ-FILE-LIMITS-DISPLAY-001: AgentReady V1 shall clearly show upload limits before users select files.
REQ-FILE-LIMITS-DISPLAY-002: The message shall say Limits: Up to 25 files per batch. Up to 50 MB per file.
REQ-WHY-MARKDOWN-DISPLAY-001: AgentReady V1 shall include a short explanation of why Markdown matters on the first screen.
REQ-WHY-MARKDOWN-DISPLAY-002: The first screen Markdown explanation shall say Cleaner Markdown helps agents understand your files with less formatting noise.
REQ-WHY-MARKDOWN-DISPLAY-003: More detailed token or environmental efficiency messaging shall not appear in the main upload area.

REQ-FIRST-SCREEN-EXAMPLE-001: AgentReady V1 shall include a small example preview lower on the first screen.
REQ-FIRST-SCREEN-EXAMPLE-DISPLAY-001: AgentReady V1 shall show simple examples but shall not divide examples into school, nonprofit, or business categories.
REQ-EXAMPLE-SECTION-TITLE-001: AgentReady V1 shall include an example section titled From messy files to clean Markdown.
REQ-EXAMPLE-COUNT-001: AgentReady V1 shall show two examples in that section.
REQ-EXAMPLE-VISUAL-001: The example section shall use simple visual storytelling.
REQ-EXAMPLE-VISUAL-002: One visual shall show an agent or robot trying to read a messy document and appearing confused.
REQ-EXAMPLE-VISUAL-003: A second visual shall show the agent reading clean AgentReady documents and appearing calm, happy, or confident.
REQ-EXAMPLE-VISUAL-004: The visuals shall avoid overly cartoonish or childish robot imagery.
```

---

## 21.8 First Page Product Positioning and Preview Screen

### Decisions

The first page should say no login is required and files are deleted after the export is created. AgentReady should include a short `Who AgentReady is for` section emphasizing non-technical users and small organizations.

The preview screen should use best-practice responsive layout: file list on the left and preview on the right for desktop, one-file selector on mobile. Users can copy raw Markdown and download individual Markdown files, but cannot edit Markdown inside AgentReady V1.

### Requirements

```md
REQ-NO-LOGIN-001: AgentReady V1 shall clearly communicate that no login is required.
REQ-NO-LOGIN-002: The first screen shall include the phrase No login required.
REQ-NO-LOGIN-FILE-STORAGE-001: The upload screen shall include the trust line No login required. Files are deleted after your export is created.
REQ-NO-LOGIN-FILE-STORAGE-002: AgentReady shall avoid saying No files stored without context because files are temporarily processed during conversion.

REQ-WHO-FOR-001: AgentReady V1 shall include a short Who AgentReady is for section on the first page.
REQ-WHO-FOR-002: The section shall explain that AgentReady is for teams preparing documents, data, and internal knowledge for AI tools or agents.
REQ-WHO-FOR-006: The section shall emphasize non-technical users and small organizations.
REQ-WHO-FOR-008: The section may reference small teams, schools, nonprofits, small businesses, and mission-driven organizations.
REQ-WHO-FOR-009: The section shall avoid making AgentReady feel limited to only one sector.

REQ-DOES-NOT-DO-001: AgentReady V1 shall not include a What AgentReady does not do section on the first page.
REQ-DOES-NOT-DO-002: Clarifications about AgentReady not running agents, answering questions, or permanently storing files may appear later in FAQ or help documentation.

REQ-PREVIEW-LAYOUT-001: AgentReady V1 shall use a two-panel preview layout on desktop.
REQ-PREVIEW-LAYOUT-002: The desktop preview shall show the converted file list on the left and the selected Markdown preview on the right.
REQ-PREVIEW-LAYOUT-003: The mobile preview shall show one file at a time using a dropdown, selector, or compact file navigation pattern.
REQ-PREVIEW-LAYOUT-004: The preview screen shall show file status clearly, including good, partial, failed, and unsupported.
REQ-PREVIEW-COPY-001: AgentReady V1 shall allow users to copy raw Markdown from each converted file.
REQ-PREVIEW-COPY-002: After copying, the interface shall show a simple confirmation such as Markdown copied.
REQ-PREVIEW-DOWNLOAD-001: AgentReady V1 shall allow users to download individual converted Markdown files from the preview screen.
REQ-PREVIEW-DOWNLOAD-002: Individual Markdown download shall not replace the main AgentReady export zip workflow.
REQ-PREVIEW-EDITING-001: AgentReady V1 shall not include in-app Markdown editing.
REQ-PREVIEW-EDITING-002: Users can review, copy, download one Markdown file, or download the full zip, but cannot edit inside AgentReady V1.
```

Suggested who-for copy:

```text
AgentReady is for small teams and organizations that want to prepare their documents and data for AI tools without learning technical AI workflows.

It helps turn files into cleaner Markdown so agents can read them more easily.
```

---

## 21.9 Hosted Beta Access and Feedback

### Decisions

The first hosted test version should have a public product page but private beta conversion access. Users request access through a short form. Approved testers receive a unique beta access code. No full login is required.

The private beta is free with limits. Feedback should be collected after export and through selected email follow-up. Original files are not automatically collected for debugging.

### Requirements

```md
REQ-HOSTED-ACCESS-001: The first hosted AgentReady test version shall include a public product page.
REQ-HOSTED-ACCESS-002: File upload and conversion in the first hosted version shall be limited to approved private beta testers.
REQ-HOSTED-ACCESS-003: The public page shall explain what AgentReady does, who it is for, and how the export works.
REQ-HOSTED-ACCESS-004: The hosted beta shall avoid open public file uploads until privacy, security, cost controls, and conversion quality are ready.

REQ-BETA-ACCESS-001: The first hosted AgentReady version shall include a short private beta request form.
REQ-BETA-ACCESS-002: The beta request form shall collect name, email, organization, and a short description of what the user wants to prepare for AI.
REQ-BETA-CTA-001: The private beta button shall say Request beta access.
REQ-BETA-CONFIRMATION-001: After a user submits the private beta request form, AgentReady shall show this confirmation: Thanks for requesting beta access. We’ll review your request and contact you if AgentReady is a good fit.

REQ-BETA-TESTERS-001: AgentReady early beta testers shall include a mix of users with messy real documents, small organizations, and AI builders.
REQ-BETA-TESTERS-002: Early beta selection shall prioritize users with real document problems over users who only want to test the tool casually.
REQ-BETA-PRICING-001: AgentReady private beta shall be free.
REQ-BETA-LIMITS-001: Private beta users shall be limited to 5 exports per week.
REQ-BETA-LIMITS-002: Each export shall allow up to 25 files.
REQ-BETA-LIMITS-003: Each file shall be limited to 50 MB.

REQ-BETA-CODE-001: The first hosted private beta shall use beta access codes to unlock upload and conversion.
REQ-BETA-CODE-002: The public AgentReady page shall remain visible without a beta code.
REQ-BETA-CODE-003: Upload and conversion shall require a valid beta access code during the private beta.
REQ-BETA-CODE-004: Beta access codes shall avoid requiring full user accounts or login in the first beta.
REQ-BETA-CODE-005: Approved testers should receive unique beta access codes rather than one shared password.
REQ-BETA-CODE-COPY-001: The beta code screen shall say AgentReady conversion is currently in private beta. Enter your access code to continue.
REQ-BETA-NO-CODE-LINK-001: The beta code screen shall include a Request beta access link.
REQ-BETA-INVALID-CODE-001: Invalid code message shall say This beta access code is not valid. Please check the code or request beta access.

REQ-BETA-CODE-LIMITS-001: Each private beta access code shall have usage limits attached to it.
REQ-BETA-CODE-LIMITS-002: Each beta code shall allow 5 exports per week.
REQ-BETA-USAGE-DISPLAY-001: AgentReady shall show beta testers how many exports they have left for the week.
REQ-BETA-USAGE-DISPLAY-002: The usage message shall use simple wording such as You have 3 exports left this week.
REQ-BETA-USAGE-RESET-001: AgentReady shall show beta testers when their weekly export limit resets.
REQ-BETA-USAGE-RESET-002: The reset message shall include the weekday and exact date.
REQ-BETA-CODE-EXPIRY-001: Private beta access codes shall expire after 90 days.
REQ-BETA-EXPIRED-CODE-001: Expired code message shall say This beta access code has expired. Contact HumanGoodAI if you need more testing time.
REQ-BETA-EXPIRY-REMINDER-001: AgentReady shall send a reminder 7 days before the beta access code expires.
REQ-BETA-EXTENSION-001: HumanGoodAI shall be able to manually extend beta access for selected testers.

REQ-BETA-FEEDBACK-001: AgentReady private beta shall collect feedback from testers.
REQ-BETA-FEEDBACK-002: After export, AgentReady shall show a simple feedback form.
REQ-BETA-FEEDBACK-003: The feedback form shall ask what worked, what failed, whether the Markdown was useful, and what should improve first.
REQ-BETA-FEEDBACK-004: HumanGoodAI may follow up by email with selected beta testers.
REQ-BETA-FEEDBACK-CONSENT-001: AgentReady private beta shall ask testers for permission before using feedback beyond internal review.
REQ-BETA-FEEDBACK-CONSENT-002: The feedback form shall include a checkbox for using anonymized feedback to improve AgentReady.
REQ-TESTIMONIAL-CONSENT-001: Testimonial permission wording shall say You may contact me before using any of my feedback publicly as a testimonial.

REQ-SUPPORT-FILES-001: AgentReady shall not automatically collect original source files through the feedback form.
REQ-SUPPORT-FILES-002: Failed files shall not be attached to feedback automatically.
REQ-SUPPORT-FILES-003: Users may choose to share a problem file separately if they want support with a failed conversion.
REQ-SUPPORT-FILE-SHARING-COPY-001: Support message shall say We do not automatically collect your files.
REQ-SUPPORT-FILE-SHARING-COPY-002: Support message shall explain that users may choose to share a problem file if they want help with a failed conversion.
REQ-SUPPORT-FILE-RETENTION-001: If a user explicitly shares a problem file for support, HumanGoodAI shall delete the file after support is complete.
```

Suggested support file sharing copy:

```text
We do not automatically collect your files.

If you want help with a failed conversion, you can choose to share the problem file with us. Only share a file if you are comfortable with HumanGoodAI reviewing it for support and debugging.
```

---

## 21.10 HumanGoodAI Service Page

### Decisions

HumanGoodAI should eventually have a service page titled `Get Your Organization Agent-Ready`. The service offer should cover the full package: file preparation, AgentReady export, agent setup, and workflow guidance.

The main service promise should combine human-centered AI and agent-ready transformation.

### Requirements

```md
REQ-HUMANGOODAI-SERVICE-001: HumanGoodAI may offer a service page around AgentReady.
REQ-HUMANGOODAI-SERVICE-002: AgentReady shall remain available as a self-serve tool.
REQ-SERVICE-PAGE-001: The future HumanGoodAI service page around AgentReady shall use the title Get Your Organization Agent-Ready.
REQ-SERVICE-OFFER-001: The future HumanGoodAI service page shall offer a full AgentReady support package.
REQ-SERVICE-OFFER-002: The service package may include file preparation, AgentReady export creation, agent setup, and workflow guidance.
REQ-SERVICE-PROMISE-001: The future HumanGoodAI AgentReady service page shall combine the themes of human-centered AI and agent-ready transformation.
REQ-SERVICE-PROMISE-002: The main service promise shall say We help your organization become agent-ready, from files to workflows, so people have more time for people.
REQ-SERVICE-CTA-001: The future HumanGoodAI service page shall use Get your organization agent-ready as the main call-to-action button.
REQ-SERVICE-CONTACT-001: The service CTA shall open a short contact form.
REQ-SERVICE-CONTACT-002: The form shall collect name, email, organization, and what the user needs help with.
REQ-SERVICE-FORM-CHECKBOX-001: The form shall include checkboxes for File preparation, AgentReady export, AI tool or agent setup, Workflow guidance, and Not sure yet.
REQ-SERVICE-FORM-CONFIRMATION-001: Confirmation message shall say Thanks for reaching out. We’ll review your request and contact you about the best next step.
REQ-SERVICE-PRICING-001: The future HumanGoodAI service page shall not show fixed pricing at first.
```

Suggested service structure:

```text
1. Prepare your files
2. Create your AgentReady export
3. Connect the export to your AI tools
4. Redesign the workflow so people get more time back for people
```

---

## 21.11 Credits, Pricing, and Mission-Aligned Access

### Decisions

After the free private beta, AgentReady should eventually support simple self-serve paid plans based on AgentReady credits. One credit processes one file. Pay-as-you-go credits do not expire. Failed and unsupported files do not use credits; good and partial conversions use one credit.

Free starter credits should be available for approved schools, nonprofits, and small mission-driven organizations that are trying to improve the world and make it a better place.

### Requirements

```md
REQ-PAID-PLANS-001: AgentReady shall begin with a free private beta before introducing paid plans.
REQ-PAID-PLANS-002: Self-serve paid plans may be introduced after beta feedback proves clear user value.
REQ-PAID-PLAN-BASIS-001: Future AgentReady self-serve paid plans shall be based primarily on the number of files processed.
REQ-FILE-CREDITS-001: Future AgentReady self-serve pricing shall support pay-as-you-go file credits.
REQ-FILE-CREDIT-EXPIRY-001: Future AgentReady file credits shall not expire.

REQ-AGENTREADY-CREDITS-001: Future AgentReady paid usage shall use the name AgentReady credits.
REQ-AGENTREADY-CREDITS-002: AgentReady credits may be purchased by users for self-serve file processing.
REQ-AGENTREADY-CREDITS-003: AgentReady credits may also be granted as free starter credits to approved mission-aligned organizations.
REQ-AGENTREADY-CREDITS-004: AgentReady credits may be donated by supporters to help approved schools, nonprofits, and mission-driven organizations.

REQ-CREDIT-USAGE-001: One AgentReady credit shall process one file.
REQ-CREDIT-USAGE-002: Credit usage shall be simple and understandable to non-technical users.
REQ-CREDIT-USAGE-003: AgentReady shall avoid charging users based on tokens, embeddings, compute time, or technical processing units.
REQ-CREDIT-FAILED-001: AgentReady shall not charge a credit for files that fail to convert and produce no usable output.
REQ-CREDIT-FAILED-002: AgentReady shall charge one credit for files that convert successfully.
REQ-CREDIT-FAILED-003: AgentReady shall charge one credit for files that are partially converted and included in the AgentReady export.
REQ-CREDIT-FAILED-004: The results screen shall clearly show which files used credits and which files did not.
REQ-CREDIT-COPY-001: Main credit explanation shall say You only use credits when AgentReady creates a usable file for your export. Failed files do not use credits.
REQ-CREDIT-COPY-002: Supporting copy shall explain that good and partial conversions use one AgentReady credit, while failed or unsupported files do not use credits.

REQ-FREE-STARTER-001: After private beta, AgentReady may offer a small free starter credit amount for approved users.
REQ-FREE-STARTER-ELIGIBILITY-001: Free starter credits may be offered to approved schools, nonprofits, and small mission-driven organizations.
REQ-FREE-STARTER-ELIGIBILITY-006: Free starter credits shall be limited to organizations that are mission-aligned with HumanGoodAI.
REQ-FREE-STARTER-ELIGIBILITY-007: Eligible organizations shall demonstrate a purpose connected to improving people’s lives, strengthening communities, supporting education, helping vulnerable groups, or making the world better in a practical way.
REQ-FREE-STARTER-ELIGIBILITY-009: HumanGoodAI may manually review free starter credit requests for mission fit.
```

Credit explanation:

```text
You only use credits when AgentReady creates a usable file for your export. Failed files do not use credits.

Good and partial conversions use one AgentReady credit. Failed or unsupported files do not use credits.
```

---

## 21.12 Donate Credits Program

### Decisions

AgentReady should include a future donate credit option so supporters can help approved schools, nonprofits, and mission-driven organizations use AgentReady. Donated credits should first go into a shared HumanGoodAI donation pool. Category-based donation may come later.

The donate button should say `Help an organization become agent-ready`. The donation flow should show an explanation page before the donation form.

### Requirements

```md
REQ-DONATE-CREDITS-001: AgentReady shall plan for a future donate credit option.
REQ-DONATE-CREDITS-002: The donate credit option shall allow supporters to help schools, nonprofits, and mission-driven organizations access AgentReady credits.
REQ-DONATE-CREDITS-003: Donated credits shall support organizations aligned with HumanGoodAI’s mission.
REQ-DONATE-CREDIT-POOL-001: Donated AgentReady credits shall first go into a shared HumanGoodAI donation pool.
REQ-DONATE-CREDIT-POOL-002: HumanGoodAI shall assign donated credits to approved mission-aligned organizations.
REQ-DONATE-CREDIT-POOL-004: A future version may allow donors to choose a broad support category such as schools, nonprofits, or mission-driven organizations.
REQ-DONATE-CREDIT-POOL-005: Donors shall not choose specific recipient organizations in the first donate credit version.

REQ-DONATE-CTA-001: The donate credits button shall say Help an organization become agent-ready.
REQ-DONATE-FLOW-001: The donate CTA shall open a donation explanation page before showing the donation form.
REQ-DONATE-PAGE-EMPHASIS-001: The donation explanation page shall emphasize practical help, access and fairness, and HumanGoodAI’s human-centered mission.
REQ-DONATE-PAGE-TITLE-001: The donation page shall use the message Help organizations that are trying to make the world a better place become agent-ready.
REQ-DONATE-PAGE-TITLE-002: The donation page shall include the supporting CTA copy Donate AgentReady credits here.

REQ-DONATE-CREDIT-AMOUNTS-001: The donate credits flow shall include preset AgentReady credit donation amounts.
REQ-DONATE-CREDIT-AMOUNTS-002: Preset donation options shall include 25, 50, 100, and 250 AgentReady credits.
REQ-DONATE-CREDIT-AMOUNTS-003: The donate credits flow shall include a custom credit amount option.
REQ-DONATE-IMPACT-001: The donation page shall show simple impact examples for donated AgentReady credits.
REQ-DONATE-IMPACT-002: Impact examples shall explain what donated credits can help organizations do, not only how many files can be processed.
REQ-DONATE-CONFIRMATION-001: Confirmation message shall say Thank you for donating AgentReady credits. Your donation will help approved schools, nonprofits, and mission-driven organizations prepare their files for AI.

REQ-DONOR-UPDATES-001: Donors shall be able to opt in to occasional updates about donated AgentReady credits.
REQ-DONOR-UPDATES-002: Donor updates shall be optional, not automatic.
REQ-DONOR-UPDATES-CONTENT-001: Donor updates may include both general donation pool updates and impact stories.
REQ-DONOR-UPDATES-CONTENT-003: Impact stories may only be shared when the supported organization gives permission.
REQ-SUPPORTED-ORG-PUBLICITY-001: HumanGoodAI shall ask supported organizations before naming them publicly.
REQ-SUPPORTED-ORG-PUBLICITY-002: HumanGoodAI may share anonymous impact examples without naming the organization.
REQ-DONATED-CREDIT-FEEDBACK-001: Organizations that receive donated AgentReady credits shall be asked to provide basic product feedback.
REQ-DONATED-CREDIT-FEEDBACK-003: Story permission, public naming, testimonials, logos, and case studies shall remain separate and optional.
REQ-DONATED-CREDIT-ONBOARDING-001: Organizations receiving donated AgentReady credits shall receive a simple onboarding guide.
REQ-DONATED-CREDIT-CHECKLIST-001: Organizations receiving donated AgentReady credits shall receive a starter checklist before uploading files.
```

Donation page copy:

```text
Help organizations that are trying to make the world a better place become agent-ready.

Donate AgentReady credits to support schools, nonprofits, and mission-driven teams preparing their files for AI.
```

Impact examples:

```text
25 credits: Help a small team prepare a starter knowledge pack.
50 credits: Help an organization prepare key policies, guides, or internal documents for AI tools.
100 credits: Help a school, nonprofit, or mission-driven team prepare a larger set of working documents.
250 credits: Help an organization prepare a more complete agent-ready document collection.
```

---

## 21.13 Sensitive Files and Future Anonymization

### Decisions

AgentReady V1 does not include anonymization or redaction. Users must be warned not to upload sensitive personal, student, medical, legal, financial, or child-related data unless they have permission and a clear reason.

Future AgentReady versions should include anonymization focused first on student and child data, then general personal data. Users should choose between replacing details with labels and removing them completely. Future anonymization should include an anonymization report, preview, and required confirmation.

### Requirements

```md
REQ-ANONYMIZATION-001: AgentReady V1 shall not include automatic anonymization or redaction.
REQ-ANONYMIZATION-002: AgentReady V1 shall clearly tell users that they are responsible for reviewing files before upload.
REQ-ANONYMIZATION-003: The donated credit starter checklist shall warn users not to upload sensitive personal, student, medical, legal, financial, or child-related data unless they have permission and a clear reason.
REQ-ANONYMIZATION-004: A future AgentReady version may include anonymization tools for student names and sensitive documents.

REQ-ANONYMIZATION-FOCUS-001: Future AgentReady anonymization shall focus first on student and child-related data.
REQ-ANONYMIZATION-FOCUS-002: Future anonymization shall also support general personal data such as names, emails, phone numbers, addresses, and personal identifiers.
REQ-ANONYMIZATION-FOCUS-003: Student and child data shall be treated as the highest-priority anonymization use case because of the sensitivity of school and youth-serving organization documents.

REQ-ANONYMIZATION-METHOD-001: Future AgentReady anonymization shall allow users to choose between replacing sensitive details with labels or removing sensitive details completely.
REQ-ANONYMIZATION-METHOD-002: Label replacement may use simple labels such as Student 1, Student 2, Parent 1, Email 1, and Phone 1.
REQ-ANONYMIZATION-METHOD-003: Full removal may replace sensitive details with placeholders such as [removed].
REQ-ANONYMIZATION-REPORT-001: Future AgentReady anonymization shall create an anonymization report.
REQ-ANONYMIZATION-REPORT-002: The anonymization report shall show what types of sensitive information were replaced or removed.
REQ-ANONYMIZATION-REPORT-003: The anonymization report shall avoid exposing the original sensitive data.
REQ-ANONYMIZATION-PREVIEW-001: Future AgentReady anonymization shall allow users to preview the anonymized version before export.
REQ-ANONYMIZATION-PREVIEW-002: Users shall be required to confirm the anonymized version before AgentReady creates the final export.
REQ-ANONYMIZATION-PROCESS-001: Future AgentReady anonymization shall use both pre-conversion and post-conversion checks.
REQ-ANONYMIZATION-PROCESS-002: AgentReady shall attempt to anonymize sensitive text before Markdown conversion.
REQ-ANONYMIZATION-PROCESS-003: AgentReady shall scan the converted Markdown again before export to check for remaining sensitive information.

REQ-SENSITIVE-MODE-001: Future AgentReady anonymization shall support both the normal upload flow and a clear sensitive-file mode.
REQ-SENSITIVE-MODE-003: Users shall be able to choose a dedicated sensitive-file pathway when preparing files that may contain student, child, or personal data.
REQ-SENSITIVE-MODE-BUTTON-001: Future AgentReady sensitive-file mode shall use the button label Prepare sensitive files.
REQ-SENSITIVE-MODE-COPY-001: Sensitive-file mode shall explain that it is for files containing student, child, or personal data.
REQ-SENSITIVE-MODE-COPY-002: Sensitive-file mode copy shall explain that AgentReady can help anonymize these files.
REQ-SENSITIVE-MODE-COPY-003: Sensitive-file mode copy shall make clear that users must review and confirm the anonymized version before export.
REQ-SENSITIVE-MODE-ACCESS-001: Future AgentReady sensitive-file mode shall be available to paid users and approved mission-aligned organizations.
REQ-SENSITIVE-MODE-CREDITS-001: Future AgentReady sensitive-file mode may use a different credit model than normal file conversion.
REQ-SENSITIVE-MODE-CREDITS-002: Paid users may be charged more credits for sensitive-file processing because anonymization adds extra processing, review, and privacy steps.
REQ-SENSITIVE-MODE-CREDITS-003: Approved schools, nonprofits, and mission-driven organizations may access sensitive-file mode through donated credits, free starter credits, or manual approval.
REQ-SENSITIVE-MODE-PRICING-001: AgentReady shall not set a fixed sensitive-file mode credit cost before the feature is tested.
```

V1 sensitive file warning:

```text
Sensitive files

AgentReady V1 does not anonymize or redact files yet. Before uploading, review your documents and avoid sensitive personal, student, medical, legal, financial, or child-related data unless you have permission and a clear reason to process it.
```

Future sensitive-file mode copy:

```text
Use this mode for files with student, child, or personal data.

AgentReady can help anonymize these files, but you must review and confirm the anonymized version before creating your export.
```

Future anonymization flow:

```text
Upload sensitive files
→ Choose anonymization settings
→ Anonymize before conversion
→ Convert to AgentReady Markdown
→ Scan final Markdown again
→ Show anonymized preview
→ User confirms
→ Create AgentReady export
```

---

## 21.14 Environmental Efficiency Placement

### Decision

AgentReady may mention that cleaner Markdown can reduce formatting noise and unnecessary token use, but this should appear in a deeper explanation section, not the main upload area. No guaranteed token savings or energy savings should be claimed.

### Requirements

```md
REQ-ENV-EFFICIENCY-001: AgentReady may mention efficiency benefits from cleaner, simpler files.
REQ-ENV-EFFICIENCY-002: Environmental or processing efficiency shall be treated as a secondary point, not the main product promise.
REQ-ENV-EFFICIENCY-003: AgentReady may state that cleaner Markdown can help reduce formatting noise and unnecessary token use.
REQ-ENV-EFFICIENCY-004: AgentReady shall not claim guaranteed token savings, guaranteed energy savings, or exact environmental impact reductions.
REQ-ENV-EFFICIENCY-PLACEMENT-001: AgentReady V1 shall not make environmental or token-efficiency messaging a main first-screen focus.
REQ-ENV-EFFICIENCY-PLACEMENT-002: Efficiency messaging shall appear in a deeper explanation section such as Why Markdown?, FAQ, or help documentation.
```

Suggested wording:

```text
Cleaner Markdown can help agents process your files with less formatting noise and unnecessary token use. This can support more efficient AI workflows.
```

---

## 21.15 Current Build Cut Line

### Build now

```text
Rust core engine
Rust CLI
TXT, Markdown, CSV, DOCX first
Simple Export folder and zip
README.md, index.md, conversion-report.md
Quality statuses: good, partial, failed, unsupported
Human-readable CLI output
--json output
Detailed exit codes
Clean sample and messy sample fixtures
Basic local web UI after CLI works
Preview, copy Markdown, individual Markdown download, full zip download
No login required messaging
Privacy-first temporary processing and deletion
```

### Do not build yet

```text
Advanced developer export
Project type selector
agent-instructions.md
Sensitive-file mode
Anonymization/redaction
Payments and credit purchase flow
Donate credit checkout
Public open uploads
Persistent workspaces or history
Accounts/login
Website scanning
OCR
Audio/EPUB processing
AI summaries
Vector DB/embedding generation
```

### Future planned tracks

```text
Hosted private beta with access codes
AgentReady credits
Donate credits program
HumanGoodAI service page
Sensitive-file mode and anonymization
Project-specific exports
Developer-ready export
```
