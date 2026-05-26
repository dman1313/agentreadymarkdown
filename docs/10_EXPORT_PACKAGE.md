# 10 — Export Package

## Simple Export structure

```text
agentready-output/
  README.md
  index.md
  conversion-report.md
  documents/
  data/
  assets/   optional, only when useful assets exist
```

## File naming

- use the original filename as basis
- lowercase where appropriate
- replace spaces with hyphens
- remove unsafe characters
- resolve duplicates with source type suffix

Example:

```text
Staff Handbook.pdf  -> staff-handbook-pdf.md
Staff Handbook.docx -> staff-handbook-docx.md
```

## `index.md`

Agent navigation file listing converted files, source types, links, categories, notes, and original folder path when relevant.

## `README.md`

Human guidance file explaining how to use the export.

## `conversion-report.md`

Conversion audit showing successes, partial conversions, failed files, unsupported files, date, and privacy note.
