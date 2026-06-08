use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use crate::converters;
use crate::export::create_export;
use crate::models::{
    AgentReadyError, ConvertedFile, ErrorCode, FileResult, FileStatus, JobResult,
};
use crate::validation::{self, validate_file};

pub const FILE_TIMEOUT_SECS: u64 = 30;
pub const DEFAULT_MAX_FILES: usize = 25;

#[derive(Debug, Clone)]
pub struct JobProgress {
    pub done: usize,
    pub total: usize,
    pub current_file: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ConvertOptions {
    pub max_files: usize,
    pub max_file_size_bytes: u64,
    pub recursive: bool,
    pub zip_name: Option<String>,
}

impl Default for ConvertOptions {
    fn default() -> Self {
        Self {
            max_files: DEFAULT_MAX_FILES,
            max_file_size_bytes: validation::MAX_FILE_SIZE_BYTES,
            recursive: false,
            zip_name: None,
        }
    }
}

#[derive(Debug)]
pub enum RunJobError {
    Validation { message: String, exit_code: i32 },
    System(AgentReadyError),
}

impl RunJobError {
    pub fn exit_code(&self) -> i32 {
        match self {
            RunJobError::Validation { exit_code, .. } => *exit_code,
            RunJobError::System(_) => 4,
        }
    }

    pub fn message(&self) -> String {
        match self {
            RunJobError::Validation { message, .. } => message.clone(),
            RunJobError::System(e) => e.user_message(),
        }
    }
}

/// Full convert + export pipeline (CLI and web server share this).
pub fn run_job(
    inputs: &[PathBuf],
    output: &Path,
    options: &ConvertOptions,
    mut on_progress: Option<&mut dyn FnMut(JobProgress)>,
) -> Result<JobResult, RunJobError> {
    if output.exists() {
        return Err(RunJobError::Validation {
            message: format!(
                "Output directory already exists: {}",
                output.display()
            ),
            exit_code: 3,
        });
    }

    let mut files_to_process = Vec::new();
    for input in inputs {
        if input.is_dir() {
            gather_files(input, options.recursive, &mut files_to_process);
        } else if input.is_file() {
            if !is_hidden_file(input) {
                files_to_process.push(input.clone());
            }
        }
    }

    if files_to_process.is_empty() {
        return Err(RunJobError::Validation {
            message: "No input files found.".into(),
            exit_code: 3,
        });
    }

    if files_to_process.len() > options.max_files {
        return Err(RunJobError::Validation {
            message: format!(
                "Input contains {} files, but max_files is {}.",
                files_to_process.len(),
                options.max_files
            ),
            exit_code: 3,
        });
    }

    let total = files_to_process.len();
    let mut results = Vec::new();

    for (index, path) in files_to_process.into_iter().enumerate() {
        if let Some(ref mut progress) = on_progress {
            progress(JobProgress {
                done: index,
                total,
                current_file: path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string()),
            });
        }

        let metadata = match fs::metadata(&path) {
            Ok(m) => m,
            Err(_) => {
                results.push(create_failed_result(&path, ErrorCode::ConversionFailed));
                continue;
            }
        };

        if metadata.len() > options.max_file_size_bytes {
            results.push(create_failed_result(&path, ErrorCode::FileTooLarge));
            continue;
        }

        if let Err(err) = validate_file(&path, metadata.len()) {
            results.push(create_failed_result(&path, err.to_error_code()));
            continue;
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let conversion_result = convert_with_timeout(&path, &ext);

        match conversion_result {
            Ok(res) => {
                let status = if res.warning.is_some() {
                    FileStatus::Partial
                } else {
                    FileStatus::Good
                };
                let file_result = FileResult {
                    source_file: path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    source_type: ext.clone(),
                    status,
                    output_file: None,
                    error_code: None,
                    warning: res.warning.clone(),
                    user_message: None,
                };
                results.push(ConvertedFile {
                    result: file_result,
                    markdown: res.markdown,
                    raw_data: res.raw_data,
                });
            }
            Err(err) => {
                results.push(create_failed_result(&path, err.to_error_code()));
            }
        }
    }

    if let Some(ref mut progress) = on_progress {
        progress(JobProgress {
            done: total,
            total,
            current_file: None,
        });
    }

    let input_labels: Vec<String> = inputs
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    create_export(
        &input_labels,
        output,
        options.zip_name.as_deref(),
        results,
    )
    .map_err(RunJobError::System)
}

fn convert_with_timeout(
    path: &Path,
    ext: &str,
) -> Result<converters::ConversionResult, AgentReadyError> {
    let (tx, rx) = mpsc::channel();
    let path = path.to_path_buf();
    let ext = ext.to_string();

    let handle = std::thread::spawn(move || {
        let result = match ext.as_str() {
            "txt" => converters::txt::convert_txt(&path),
            "md" => converters::markdown::convert_markdown(&path),
            "csv" => converters::csv::convert_csv(&path),
            "docx" => converters::docx::convert_docx(&path),
            "pdf" => converters::pdf::convert_pdf(&path),
            "epub" => converters::epub::convert_epub(&path),
            _ => Err(AgentReadyError::UserFacing(ErrorCode::UnsupportedFile)),
        };
        let _ = tx.send(result);
    });

    match rx.recv_timeout(Duration::from_secs(FILE_TIMEOUT_SECS)) {
        Ok(Ok(res)) => {
            let _ = handle.join();
            Ok(res)
        }
        Ok(Err(err)) => {
            let _ = handle.join();
            Err(err)
        }
        Err(_) => Err(AgentReadyError::Timeout),
    }
}

pub fn gather_files(dir: &Path, recursive: bool, files: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if is_hidden_file(&path) {
            continue;
        }
        if path.is_symlink() {
            continue;
        }
        if path.is_dir() {
            if recursive {
                gather_files(&path, recursive, files);
            }
        } else if path.is_file() {
            files.push(path);
        }
    }
}

pub fn is_hidden_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with('.'))
        .unwrap_or(false)
}

pub fn sanitize_filename(name: &str) -> String {
    let base = Path::new(name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");
    base.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn create_failed_result(path: &Path, err: ErrorCode) -> ConvertedFile {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let status = if err == ErrorCode::UnsupportedFile {
        FileStatus::Unsupported
    } else {
        FileStatus::Failed
    };

    ConvertedFile {
        result: FileResult {
            source_file: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            source_type: ext,
            status,
            output_file: None,
            error_code: Some(err),
            warning: None,
            user_message: None,
        },
        markdown: String::new(),
        raw_data: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn run_job_converts_txt() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("input");
        fs::create_dir_all(&input).unwrap();
        fs::write(input.join("hello.txt"), "Hello world").unwrap();
        let output = dir.path().join("output");

        let result = run_job(
            &[input.clone()],
            &output,
            &ConvertOptions::default(),
            None,
        )
        .unwrap();

        assert_eq!(result.exit_code, 0);
        assert!(output.join("documents").exists());
    }

    #[test]
    fn rejects_existing_output() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("input");
        fs::create_dir_all(&input).unwrap();
        fs::write(input.join("a.txt"), "x").unwrap();
        let output = dir.path().join("output");
        fs::create_dir_all(&output).unwrap();

        let err = run_job(&[input], &output, &ConvertOptions::default(), None).unwrap_err();
        assert_eq!(err.exit_code(), 3);
    }
}
