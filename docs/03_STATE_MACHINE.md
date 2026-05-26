# 03 — Upload Batch State Machine

Every conversion batch moves through a clear lifecycle.

```text
S0_IDLE
-> S1_FILES_SELECTED
-> S2_VALIDATING
-> S3_READY_TO_CONVERT
-> S4_CONVERTING
-> S5_RESULTS_READY
-> S6_ZIP_READY
-> S7_DOWNLOADED_OR_CLEANED
```

## Error and cancellation branches

```text
S4_CONVERTING -> S4_CANCEL_REQUESTED -> S8_CANCELLED_AND_CLEANED
S4_CONVERTING -> S5_RESULTS_READY_WITH_FAILURES
S6_ZIP_READY -> S9_ZIP_CREATION_FAILED
```

## State invariants

- Invalid files never block valid files from converting.
- Failed files are not silently hidden.
- Source uploads and temp files must be cleaned after cancellation, download, or cleanup expiry.
- Results must include successful, partially converted, failed, and unsupported files.
