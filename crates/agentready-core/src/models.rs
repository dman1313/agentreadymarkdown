use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Success,
    PartialSuccess,
    Failed,
    Cancelled,
    ValidationError,
    SystemError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileStatus {
    Good,
    Partial,
    Failed,
    Unsupported,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    UnsupportedFile,
    FileTooLarge,
    NoReadableText,
    PasswordProtected,
    ConversionFailed,
    Cancelled,
    TempFileError,
    ZipCreationFailed,
    TimeoutExceeded,
    ZipBombDetected,
    OutputAlreadyExists,
    ValidationError,
}

impl ErrorCode {
    pub fn user_message(&self) -> &'static str {
        match self {
            ErrorCode::UnsupportedFile => "This file type is not supported in AgentReady V1. Try TXT, Markdown, CSV, DOCX, PDF, EPUB, MOBI, AZW3, or AZW.",
            ErrorCode::FileTooLarge => "This file is larger than the V1 file size limit.",
            ErrorCode::NoReadableText => "AgentReady could not find readable text in this file. OCR is not supported in V1.",
            ErrorCode::PasswordProtected => "This file appears to be password-protected, encrypted, or DRM-protected. AgentReady only supports DRM-free files you own or are authorized to convert. It cannot remove copy protection.",
            ErrorCode::ConversionFailed => "AgentReady could not convert this file.",
            ErrorCode::Cancelled => "The conversion was cancelled.",
            ErrorCode::TempFileError => "AgentReady had a temporary file processing problem.",
            ErrorCode::ZipCreationFailed => "AgentReady could not create the export zip.",
            ErrorCode::TimeoutExceeded => "Conversion timed out after 30 seconds.",
            ErrorCode::ZipBombDetected => "This file appears to be a zip bomb and was rejected for safety.",
            ErrorCode::OutputAlreadyExists => "The output directory already exists.",
            ErrorCode::ValidationError => "Input validation failed. Please check your files and try again.",
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.user_message())
    }
}

impl std::error::Error for ErrorCode {}

#[derive(Error, Debug)]
pub enum AgentReadyError {
    #[error("{0}")]
    UserFacing(#[from] ErrorCode),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("CSV parse error: {0}")]
    Csv(#[from] csv::Error),
    #[error("XML parse error")]
    XmlParse,
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("PDF extract error")]
    PdfExtract,
    #[error("Encoding error: {0}")]
    Encoding(String),
    #[error("Timeout exceeded")]
    Timeout,
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl AgentReadyError {
    /// Maps this error to a user-facing ErrorCode for CLI/JSON output.
    pub fn to_error_code(&self) -> ErrorCode {
        match self {
            AgentReadyError::UserFacing(code) => *code,
            AgentReadyError::Timeout => ErrorCode::TimeoutExceeded,
            _ => ErrorCode::ConversionFailed,
        }
    }

    /// Returns a human-readable message suitable for end users.
    pub fn user_message(&self) -> String {
        match self {
            AgentReadyError::UserFacing(code) => code.user_message().to_string(),
            AgentReadyError::Timeout => ErrorCode::TimeoutExceeded.user_message().to_string(),
            _ => ErrorCode::ConversionFailed.user_message().to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSummary {
    pub total_files: usize,
    pub converted: usize,
    pub partial: usize,
    pub failed: usize,
    pub unsupported: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileResult {
    pub source_file: String,
    pub source_type: String,
    pub status: FileStatus,
    pub output_file: Option<String>,
    pub error_code: Option<ErrorCode>,
    pub warning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    pub status: JobStatus,
    pub exit_code: i32,
    pub input: String,
    pub output_folder: String,
    pub export_zip: String,
    pub summary: JobSummary,
    pub files: Vec<FileResult>,
}

/// Holds the result of converting a single file, used inside the export pipeline.
#[derive(Debug, Clone)]
pub struct ConvertedFile {
    pub result: FileResult,
    pub markdown: String,
    pub raw_data: Option<Vec<u8>>,
}

#[cfg(test)]
pub mod test_helpers {
    use super::*;

    pub fn assert_err_code(result: Result<(), AgentReadyError>, expected: ErrorCode) {
        match result {
            Err(AgentReadyError::UserFacing(code)) => assert_eq!(code, expected),
            Err(other) => panic!("Expected UserFacing({:?}), got {:?}", expected, other),
            Ok(_) => panic!("Expected error {:?}, got Ok", expected),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_code_user_messages_are_non_empty() {
        let codes = [
            ErrorCode::UnsupportedFile,
            ErrorCode::FileTooLarge,
            ErrorCode::NoReadableText,
            ErrorCode::PasswordProtected,
            ErrorCode::ConversionFailed,
            ErrorCode::Cancelled,
            ErrorCode::ZipCreationFailed,
            ErrorCode::TimeoutExceeded,
            ErrorCode::ZipBombDetected,
            ErrorCode::OutputAlreadyExists,
            ErrorCode::ValidationError,
        ];
        for code in &codes {
            assert!(!code.user_message().is_empty(), "{:?} has empty user_message", code);
        }
    }

    #[test]
    fn file_status_serialization_roundtrip() {
        let statuses = [FileStatus::Good, FileStatus::Partial, FileStatus::Failed, FileStatus::Unsupported, FileStatus::Cancelled];
        for status in &statuses {
            let json = serde_json::to_string(status).unwrap();
            let back: FileStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(*status, back);
        }
    }

    #[test]
    fn job_status_serialization_roundtrip() {
        let statuses = [JobStatus::Success, JobStatus::PartialSuccess, JobStatus::Failed, JobStatus::Cancelled, JobStatus::ValidationError, JobStatus::SystemError];
        for status in &statuses {
            let json = serde_json::to_string(status).unwrap();
            let back: JobStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(*status, back);
        }
    }

    #[test]
    fn file_result_skips_none_user_message() {
        let result = FileResult {
            source_file: "test.txt".into(),
            source_type: "txt".into(),
            status: FileStatus::Good,
            output_file: None,
            error_code: None,
            warning: None,
            user_message: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("user_message"));
    }

    #[test]
    fn file_result_includes_some_user_message() {
        let result = FileResult {
            source_file: "test.txt".into(),
            source_type: "txt".into(),
            status: FileStatus::Failed,
            output_file: None,
            error_code: Some(ErrorCode::UnsupportedFile),
            warning: None,
            user_message: Some("error msg".into()),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("user_message"));
    }

    #[test]
    fn agent_ready_error_maps_to_error_code() {
        let err = AgentReadyError::UserFacing(ErrorCode::FileTooLarge);
        assert_eq!(err.to_error_code(), ErrorCode::FileTooLarge);
        assert_eq!(err.user_message(), ErrorCode::FileTooLarge.user_message());

        let io_err = AgentReadyError::Io(std::io::Error::new(std::io::ErrorKind::Other, "boom"));
        assert_eq!(io_err.to_error_code(), ErrorCode::ConversionFailed);
    }
}
