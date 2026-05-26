# 06 — CLI Contracts

## Main command

```bash
agentready convert ./input-folder --output ./agentready-output
```

## Multi-file command

```bash
agentready convert staff-handbook.pdf volunteer-list.xlsx --output ./agentready-output
```

## JSON mode for server

```bash
agentready convert ./input-folder --output ./agentready-output --json
```

## CLI requirements

- human-readable logs by default
- structured JSON with `--json`
- stable status fields
- no document text in logs
- creates output folder and zip automatically
- continues after per-file failure

## JSON response shape

```json
{
  "job_status": "completed_with_warnings",
  "output_folder": "./agentready-output",
  "zip_path": "./agentready-output.zip",
  "files": []
}
```


---

## Latest CLI update from Q107–Q261

See `docs/21_DECISIONS_Q107_Q261.md`, section 21.5, for the current CLI contract details.

Required first CLI build:

```bash
agentready --help
agentready --version
agentready convert ./input-folder --output ./agentready-output
agentready convert ./input-folder --output ./agentready-output --json
agentready convert ./input-folder --output ./agentready-output --recursive
```

The CLI uses human-readable output by default, supports `--json`, uses detailed exit codes 0 through 4, ignores hidden/system files by default, does not overwrite output folders or zip files, and keeps both the output folder and matching zip.
