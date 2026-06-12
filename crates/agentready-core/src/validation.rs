use std::path::Path;

use crate::models::{AgentReadyError, ErrorCode};

pub const MAX_FILE_SIZE_BYTES: u64 = 50 * 1024 * 1024; // 50 MB

pub fn validate_file(path: &Path, file_size: u64) -> Result<(), AgentReadyError> {
    if file_size > MAX_FILE_SIZE_BYTES {
        return Err(AgentReadyError::UserFacing(ErrorCode::FileTooLarge));
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "txt" | "md" | "csv" | "docx" | "pdf" | "epub" | "mobi" | "azw3" | "azw" | "xlsx"
        | "pptx" | "html" | "htm" => Ok(()),
        _ => Err(AgentReadyError::UserFacing(ErrorCode::UnsupportedFile)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn accepts_supported_extensions() {
        for ext in &[
            "txt", "md", "csv", "docx", "pdf", "epub", "mobi", "azw3", "azw", "xlsx", "pptx",
        ] {
            let path = PathBuf::from(format!("test.{}", ext));
            assert!(validate_file(&path, 1024).is_ok(), "Extension .{} should be supported", ext);
        }
    }

    #[test]
    fn rejects_unsupported_extensions() {
        for ext in &["exe", "png", "jpg", "zip", "js", "rtf", "doc", ""] {
            let path = PathBuf::from(format!("test.{}", ext));
            assert_eq!(validate_file(&path, 1024).unwrap_err().to_error_code(), ErrorCode::UnsupportedFile, "Extension .{} should be unsupported", ext);
        }
    }

    #[test]
    fn rejects_files_over_size_limit() {
        let path = PathBuf::from("test.txt");
        assert_eq!(validate_file(&path, MAX_FILE_SIZE_BYTES + 1).unwrap_err().to_error_code(), ErrorCode::FileTooLarge);
    }

    #[test]
    fn accepts_file_at_size_limit() {
        let path = PathBuf::from("test.txt");
        assert!(validate_file(&path, MAX_FILE_SIZE_BYTES).is_ok());
    }

    #[test]
    fn extension_case_insensitive() {
        let path = PathBuf::from("test.TXT");
        assert!(validate_file(&path, 1024).is_ok());
    }

    #[test]
    fn no_extension_rejected() {
        let path = PathBuf::from("Makefile");
        assert_eq!(validate_file(&path, 1024).unwrap_err().to_error_code(), ErrorCode::UnsupportedFile);
    }
}
