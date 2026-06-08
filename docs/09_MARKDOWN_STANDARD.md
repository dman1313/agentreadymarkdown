# 09 — Markdown Standard

## Goal

The output should be readable by humans and easier for agents to scan, understand, and reuse — **with less formatting noise than PDF, Word, HTML, or spreadsheet sources**, so agents spend fewer tokens on structure and more on meaning.

This is AgentReady’s core promise: **documents in → lean agent Markdown out**.

## Required qualities

- clear heading hierarchy
- logical sections
- clean lists
- readable tables
- minimal formatting noise
- source frontmatter
- visible warnings for partial conversion

## Frontmatter

```yaml
---
source_file: Staff Handbook.pdf
source_type: PDF
converted_by: AgentReady
status: converted
---
```

## Partial conversion warning

```md
> ⚠️ AgentReady warning: This file was partially converted.
> Some formatting, structure, tables, or text may be incomplete. Review before using this file as trusted agent knowledge.
```

## Agent pipeline (implementation)

Every successful conversion passes through:

1. **Format converter** — extract meaning (DOCX styles → headings/lists/tables; PDF/TXT → `plain_text_to_markdown` heuristics; EPUB HTML → Markdown; etc.).
2. **Garbage rejection** — `text_quality` blocks mojibake and unreadable extract (fail loud, don’t export noise).
3. **`normalize_for_agents`** — collapse extra blank lines, strip invisible Unicode, preserve code fences and tables.
4. **Export frontmatter** — small YAML header (`source_file`, `source_type`, `status`) for agent navigation.

## Why Markdown

Markdown is simple, portable, agent-friendly, and human-readable. Removing PDF/Word/HTML layout noise **typically** reduces token load versus feeding raw binaries or markup to an agent, but AgentReady must not promise exact savings percentages.
