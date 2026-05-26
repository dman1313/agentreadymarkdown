# Rust CLI Contract

## Commands

```bash
agentready convert ./input-folder --output ./agentready-output
```

```bash
agentready convert staff-handbook.pdf volunteer-list.xlsx --output ./agentready-output
```

```bash
agentready convert ./input-folder --output ./agentready-output --json
```

## Flags

| Flag | Purpose |
|---|---|
| `--output <path>` | Output folder path |
| `--json` | Emit structured JSON |
| `--zip-name <name>` | Optional zip name override |
| `--max-files <n>` | Optional override, default 25 |
| `--max-file-size-mb <n>` | Optional override, default 50 |

No config file in V1.

## JSON output shape

```json
{
  "status": "completed_with_warnings",
  "output_folder": "./agentready-output",
  "zip_path": "./agentready-output.zip",
  "summary": {
    "total_files": 5,
    "converted": 3,
    "partially_converted": 1,
    "failed": 1,
    "unsupported": 0
  },
  "files": [],
  "errors": []
}
```
