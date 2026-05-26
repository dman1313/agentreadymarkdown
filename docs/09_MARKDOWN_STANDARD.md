# 09 — Markdown Standard

## Goal

The output should be readable by humans and easier for agents to scan, understand, and reuse.

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

## Why Markdown

Markdown is simple, portable, agent-friendly, and human-readable. It may reduce unnecessary token use by removing layout noise, but AgentReady must not promise exact savings.
