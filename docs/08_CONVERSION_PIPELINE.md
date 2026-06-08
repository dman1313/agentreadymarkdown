# 08 — Conversion Pipeline

## Pipeline

```text
Detect file type
-> Validate safety and size
-> Parse source
-> Normalize structure
-> Render Markdown/data output
-> Extract useful assets when needed
-> Add frontmatter
-> Add warnings when partial
-> Update index and report
-> Zip export
```

## File type behavior

| File type | Behavior |
|---|---|
| PDF | Extract readable text and structure where possible. No OCR in V1. |
| DOCX | Convert headings, paragraphs, lists, tables, and links. |
| TXT | Lightly convert to Markdown. |
| Markdown | Preserve structure and normalize only when safe. |
| CSV | Create Markdown table and preserve CSV copy. |
| EPUB | Spine-ordered HTML → Markdown; zip-bomb guards. |
| MOBI | Kindle PDB via MIT `mobi` crate; DRM rejected. |
| AZW3 / AZW | KF8 boundary preferred when EXTH offset present; else MOBI layer. |
| XLSX | Convert sheets/tables and preserve CSV copies. *(out of scope V1 build)* |

## Quality priority

AgentReady optimizes for agent understanding, not visual reproduction.
