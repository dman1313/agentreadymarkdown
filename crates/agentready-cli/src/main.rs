mod serve;

use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};

use agentready_core::job::{ConvertOptions, RunJobError};

#[derive(Parser)]
#[command(name = "agentready")]
#[command(version)]
#[command(about = "AgentReady V1 - Spec Driven AI Knowledge Packager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert documents to agent-ready Markdown
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

        /// Recurse into subdirectories (default: flat)
        #[arg(long)]
        recursive: bool,
    },
    /// Local web UI (Rust-only, in-process conversion)
    Serve {
        /// Host interface
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// TCP port
        #[arg(long, default_value_t = 3000)]
        port: u16,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Convert {
            inputs,
            output,
            json,
            zip_name,
            max_files,
            max_file_size_mb,
            recursive,
        } => {
            let options = ConvertOptions {
                max_files,
                max_file_size_bytes: max_file_size_mb * 1024 * 1024,
                recursive,
                zip_name,
            };

            match agentready_core::job::run_job(&inputs, &output, &options, None) {
                Ok(job_result) => {
                    if json {
                        match serde_json::to_string_pretty(&job_result) {
                            Ok(json_str) => println!("{}", json_str),
                            Err(_) => {
                                eprintln!("System Error: Failed to serialize results.");
                                process::exit(4);
                            }
                        }
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
                Err(RunJobError::Validation { message, exit_code }) => {
                    if json {
                        eprintln!(
                            r#"{{"status": "validation_error", "exit_code": {}, "error": "{}"}}"#,
                            exit_code, message
                        );
                    } else {
                        eprintln!("Error: {}", message);
                    }
                    process::exit(exit_code);
                }
                Err(RunJobError::System(e)) => {
                    if json {
                        println!(
                            r#"{{"status": "system_error", "exit_code": 4, "error": "{}"}}"#,
                            e.user_message()
                        );
                    } else {
                        eprintln!(
                            "System Error: {} ({:?})",
                            e.user_message(),
                            e.to_error_code()
                        );
                    }
                    process::exit(4);
                }
            }
        }
        Commands::Serve { host, port } => {
            if let Err(e) = serve::run(&host, port) {
                eprintln!("Server error: {e}");
                process::exit(1);
            }
        }
    }
}
