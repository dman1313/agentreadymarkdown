use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    UnsupportedFile,
    FileTooLarge,
    NoReadableText,
    PasswordProtected,
    ConversionFailed,
    PartiallyConverted,
    Cancelled,
    TempFileError,
    ZipCreationFailed,
    TimeoutExceeded,
    ZipBombDetected,
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
