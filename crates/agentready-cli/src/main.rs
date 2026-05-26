use std::path::PathBuf;
use clap::{Parser, Subcommand};
use std::process;
use std::fs;
use serde_json;

use agentready_core::models::{FileResult, FileStatus, ErrorCode};
use agentready_core::validation::validate_file;
use agentready_core::converters;
use agentready_core::export::create_export;

#[derive(Parser)]
#[command(name = "agentready")]
#[command(about = "AgentReady V1 - Spec Driven AI Knowledge Packager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Convert {
        /// Input files or directories
        #[arg(required = true)]
        inputs: Vec<PathBuf>,

        /// Output directory
        #[arg(long, required = true)]
        output: PathBuf,

        /// Emit structured JSON for server use
        #[arg(long)]
        json: bool,

        /// Override the zip file name
        #[arg(long)]
        zip_name: Option<String>,

        /// Maximum files to process (default: 25)
        #[arg(long, default_value_t = 25)]
        max_files: usize,

        /// Maximum file size in MB (default: 50)
        #[arg(long, default_value_t = 50)]
        max_file_size_mb: u64,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Convert {
            inputs,
            output,
            json,
            zip_name: _,
            max_files,
            max_file_size_mb,
        } => {
            let max_bytes = *max_file_size_mb * 1024 * 1024;
            
            // 1. Gather files
            let mut files_to_process = Vec::new();
            for input in inputs {
                if input.is_dir() {
                    let walker = walkdir::WalkDir::new(input);
                    for entry in walker.into_iter().filter_map(|e| e.ok()) {
                        if entry.path().is_file() {
                            files_to_process.push(entry.path().to_path_buf());
                        }
                    }
                } else if input.is_file() {
                    files_to_process.push(input.clone());
                } else if !*json {
                    eprintln!("Warning: Skipping invalid input path: {}", input.display());
                }
            }
            
            if files_to_process.len() > *max_files {
                files_to_process.truncate(*max_files);
                if !*json {
                    eprintln!("Warning: Max files limit ({}) reached. Truncating input list.", max_files);
                }
            }
            
            // 2. Convert files
            let mut results = Vec::new();
            
            for path in files_to_process {
                let metadata = match fs::metadata(&path) {
                    Ok(m) => m,
                    Err(_) => {
                        results.push((create_failed_result(&path, ErrorCode::ConversionFailed), String::new(), None));
                        continue;
                    }
                };
                
                if let Err(err) = validate_file(&path, metadata.len()) {
                    // Check if it's because of our override limit
                    let err = if metadata.len() > max_bytes {
                        ErrorCode::FileTooLarge
                    } else {
                        err
                    };
                    results.push((create_failed_result(&path, err), String::new(), None));
                    continue;
                }
                
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                
                let conversion = match ext.as_str() {
                    "txt" => converters::txt::convert_txt(&path),
                    "md" => converters::markdown::convert_markdown(&path),
                    "csv" => converters::csv::convert_csv(&path),
                    "docx" => converters::docx::convert_docx(&path),
                    "pdf" => converters::pdf::convert_pdf(&path),
                    _ => Err(ErrorCode::UnsupportedFile),
                };
                
                match conversion {
                    Ok(res) => {
                        let status = if res.warning.is_some() { FileStatus::Partial } else { FileStatus::Good };
                        let file_result = FileResult {
                            source_file: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                            source_type: ext.clone(),
                            status,
                            output_file: None, // Filled by export module
                            error_code: None,
                            warning: res.warning.clone(),
                        };
                        results.push((file_result, res.markdown, res.raw_data));
                    }
                    Err(err) => {
                        results.push((create_failed_result(&path, err), String::new(), None));
                    }
                }
            }
            
            // 3. Export
            // Using the first input as a label
            let input_label = inputs.first().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("unknown"));
            match create_export(&input_label, output, results) {
                Ok(job_result) => {
                    if *json {
                        println!("{}", serde_json::to_string_pretty(&job_result).unwrap());
                    } else {
                        println!("Job completed with status: {:?}", job_result.status);
                        println!("Output saved to: {}", job_result.output_folder);
                        println!("Export zip created at: {}", job_result.export_zip);
                        println!("Summary:");
                        println!("  Total Files: {}", job_result.summary.total_files);
                        println!("  Converted: {}", job_result.summary.converted);
                        println!("  Partial: {}", job_result.summary.partial);
                        println!("  Failed: {}", job_result.summary.failed);
                        println!("  Unsupported: {}", job_result.summary.unsupported);
                    }
                    process::exit(job_result.exit_code);
                }
                Err(e) => {
                    if *json {
                        println!(r#"{{"status": "system_error", "exit_code": 4, "error": "{:?}"}}"#, e);
                    } else {
                        eprintln!("System Error: Failed to create export package ({:?})", e);
                    }
                    process::exit(4);
                }
            }
        }
    }
}

fn create_failed_result(path: &PathBuf, err: ErrorCode) -> FileResult {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let status = if err == ErrorCode::UnsupportedFile {
        FileStatus::Unsupported
    } else {
        FileStatus::Failed
    };
    
    FileResult {
        source_file: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
        source_type: ext,
        status,
        output_file: None,
        error_code: Some(err),
        warning: None,
    }
}
