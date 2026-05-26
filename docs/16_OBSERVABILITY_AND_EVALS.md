# 16 — Observability and Evals

## Observability goals

Give developers enough information to debug conversion while protecting document privacy.

## Required logs

- job started
- file validation result
- conversion started/completed/failed
- zip created/failed
- cleanup completed/failed

## Required report

`conversion-report.md` must include:

- converted files
- partially converted files
- failed files
- unsupported files
- basic reasons
- privacy note

## Quality eval criteria

A converted file is acceptable when:

- main text is preserved
- headings are understandable
- lists and tables are readable where possible
- warnings are present for partial quality
- the output helps an agent navigate the content
