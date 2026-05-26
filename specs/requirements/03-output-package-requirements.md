# Output Package Requirements

## Simple Export structure

```text
agentready-output/
  README.md
  index.md
  conversion-report.md
  documents/
  data/
  assets/        # only when useful assets exist
```

## Required files

1. `README.md` for humans
2. `index.md` for agents
3. `conversion-report.md` for conversion status
4. converted Markdown files
5. CSV copies for spreadsheet data when needed
6. useful assets when needed

## Markdown frontmatter

```yaml
---
source_file: Staff Handbook.pdf
source_type: PDF
converted_by: AgentReady
status: converted
---
```

Allowed statuses:

1. `converted`
2. `partially_converted`

## Partially converted warning

```md
> ⚠️ AgentReady warning: This file was partially converted.
> Some formatting, structure, tables, or text may be incomplete. Review before using this file as trusted agent knowledge.
```
