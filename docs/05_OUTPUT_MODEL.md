# 05 — Output Model

## Core entities

### ConversionJob

Represents one batch of uploaded files.

Fields:

- `job_id`
- `status`
- `created_at`
- `input_count`
- `output_folder`
- `zip_path`
- `files`

### FileResult

Represents one input file and its result.

Fields:

- `original_name`
- `safe_name`
- `source_type`
- `status`
- `output_paths`
- `warnings`
- `error_code`
- `user_message`

## Status values

- `converted`
- `partially_converted`
- `failed`
- `unsupported`
- `blocked`
- `skipped`

## Invariants

- Converted files have at least one output path.
- Failed files have an error code and user-facing message.
- Partially converted files are included in the export with a warning.
- Failed and unsupported files are not included as converted outputs.
