use std::path::Path;

use crate::models::ErrorCode;

pub const MAX_FILE_SIZE_BYTES: u64 = 50 * 1024 * 1024; // 50 MB

pub fn validate_file(path: &Path, file_size: u64) -> Result<(), ErrorCode> {
    if file_size > MAX_FILE_SIZE_BYTES {
        return Err(ErrorCode::FileTooLarge);
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "txt" | "md" | "csv" | "docx" => Ok(()),
        "pdf" | "xlsx" => Ok(()), // Supported but might not be fully implemented in Phase 1
        _ => Err(ErrorCode::UnsupportedFile),
    }
}
