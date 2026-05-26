# 22 Product and Implementation Decisions: Edge Cases & Technical Constraints

Status: build directive update
Date: 2026-05-26
Source: System SDD Review

This document captures technical edge cases, error state definitions, and system constraints identified during the SDD review process. It ensures the implementation agent has strict rules for scenarios not fully detailed in the initial specs.

These decisions override older sections when they conflict.

---

## 22.1 Duplicate Filename Resolution

### Decision

When the CLI processes a batch with `--recursive`, multiple files with the same name from different folders may collide in the flattened export directory. The core engine must resolve duplicates by appending an incremental counter before the extension.

### Requirements

```md
REQ-DUPLICATE-NAMES-001: The export folder structure shall remain flat, even when --recursive is used.
REQ-DUPLICATE-NAMES-002: If multiple source files result in the same output filename (e.g., staff-handbook-pdf.md), the engine shall append a counter to subsequent files.
REQ-DUPLICATE-NAMES-003: The naming pattern for duplicates shall be filename-ext-2.md, filename-ext-3.md, etc.
```

---

## 22.2 "Partial Success" Technical Definition

### Decision

A conversion is "partial" when the core engine successfully parses the document structure but encounters elements it cannot safely convert without data loss, or when a file type parser throws a non-fatal warning.

### Requirements

```md
REQ-PARTIAL-DEF-001: The core engine shall mark a file as partial if it successfully extracts readable text but drops complex structural elements (e.g., embedded objects, complex nested tables in DOCX).
REQ-PARTIAL-DEF-002: The core engine shall mark a file as partial if an image extraction fails but the surrounding text is preserved.
REQ-PARTIAL-DEF-003: The core engine shall mark a file as failed (not partial) if it cannot extract any readable text or if the file is corrupted.
```

---

## 22.3 Memory Limits and Streaming

### Decision

To prevent Out-of-Memory (OOM) errors on large files (up to 50MB), the Rust core should prefer streaming parsers or buffered reading where possible, particularly for CSV. DOCX and PDF parsing may require in-memory loading, which is acceptable for V1, provided overall memory is bounded.

### Requirements

```md
REQ-MEMORY-LIMIT-001: The Rust core shall use buffered reading for TXT and CSV files.
REQ-MEMORY-LIMIT-002: In-memory parsing is acceptable for DOCX and PDF in V1, as long as the file size is strictly validated against the 50MB limit before parsing begins.
```

---

## 22.4 Processing Timeouts

### Decision

A single file must not hang the conversion batch indefinitely. The engine must enforce a strict processing timeout per file.

### Requirements

```md
REQ-TIMEOUT-001: The core engine shall enforce a 30-second maximum processing timeout per file.
REQ-TIMEOUT-002: If a file exceeds the timeout, the engine shall abort processing for that file and mark it as failed with error code CONVERSION_FAILED or a new TIMEOUT_EXCEEDED code.
```

---

## 22.5 Text Encodings & BOMs

### Decision

TXT and CSV files are not guaranteed to be UTF-8. The core engine should attempt to decode common encodings or strip BOMs before failing.

### Requirements

```md
REQ-ENCODING-001: The core engine shall strip Byte Order Marks (BOM) from TXT and CSV files before parsing.
REQ-ENCODING-002: If a file is not valid UTF-8, the engine may attempt a fallback decoding (e.g., Windows-1252) or gracefully fail the file rather than panicking.
```

---

## 22.6 Security: Zip Bombs

### Decision

Since DOCX and XLSX are ZIP archives, the engine must protect against decompression bombs.

### Requirements

```md
REQ-ZIP-BOMB-001: The core engine shall enforce a maximum decompression ratio and a maximum uncompressed file size limit (e.g., 250MB) when extracting DOCX or XLSX files.
REQ-ZIP-BOMB-002: If the extraction exceeds the uncompressed limit, the file shall immediately fail.
```

---

## 22.7 Concurrency

### Decision

Batch processing in the Rust CLI should be concurrent to maximize speed, provided it does not violate memory constraints.

### Requirements

```md
REQ-CONCURRENCY-001: The Rust CLI may process the batch of files concurrently (e.g., using Rayon).
REQ-CONCURRENCY-002: If concurrency is implemented, error collection and report generation must remain deterministic.
```
