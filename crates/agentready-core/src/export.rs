use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::io::{Read, Write};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::models::{FileResult, FileStatus, JobResult, JobStatus, JobSummary, ErrorCode};

pub fn create_export(
    input_path: &Path,
    output_folder: &Path,
    results: Vec<(FileResult, String, Option<Vec<u8>>)> // FileResult, Markdown, RawBytes
) -> Result<JobResult, ErrorCode> {
    // 1. Create directory structure
    if output_folder.exists() {
        fs::remove_dir_all(output_folder).map_err(|_| ErrorCode::ConversionFailed)?;
    }
    fs::create_dir_all(output_folder).map_err(|_| ErrorCode::ConversionFailed)?;
    
    let docs_folder = output_folder.join("documents");
    fs::create_dir_all(&docs_folder).map_err(|_| ErrorCode::ConversionFailed)?;
    
    let data_folder = output_folder.join("data");
    
    // 2. Resolve duplicates
    let mut final_results = Vec::new();
    let mut name_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    
    for (mut result, markdown, raw) in results {
        if result.status == FileStatus::Failed || result.status == FileStatus::Unsupported {
            final_results.push((result, markdown, raw));
            continue;
        }

        let base_name = Path::new(&result.source_file)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("document")
            .to_lowercase()
            .replace(" ", "-")
            .replace("_", "-");

        let ext = Path::new(&result.source_file)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
            
        let name_key = format!("{}-{}", base_name, ext);
        let count = name_counts.entry(name_key.clone()).or_insert(0);
        *count += 1;
        
        let output_name = if *count == 1 {
            format!("{}-{}.md", base_name, ext)
        } else {
            format!("{}-{}-{}.md", base_name, ext, count)
        };
        
        // Write the markdown file
        let out_path = docs_folder.join(&output_name);
        fs::write(&out_path, &markdown).map_err(|_| ErrorCode::ConversionFailed)?;
        result.output_file = Some(format!("documents/{}", output_name));
        
        // Write raw data if CSV
        if let Some(raw_bytes) = raw.as_ref() {
            if !data_folder.exists() {
                fs::create_dir_all(&data_folder).map_err(|_| ErrorCode::ConversionFailed)?;
            }
            let raw_name = output_name.replace(".md", ".csv");
            fs::write(data_folder.join(&raw_name), raw_bytes).map_err(|_| ErrorCode::ConversionFailed)?;
        }
        
        final_results.push((result, markdown, raw));
    }
    
    // 3. Write metadata files
    write_metadata_files(output_folder, &final_results)?;
    
    // 4. Create ZIP archive
    let zip_path = output_folder.with_extension("zip");
    create_zip_archive(output_folder, &zip_path)?;
    
    // 5. Generate JobSummary
    let mut converted = 0;
    let mut partial = 0;
    let mut failed = 0;
    let mut unsupported = 0;
    
    let mut export_files = Vec::new();
    
    for (res, _, _) in final_results {
        match res.status {
            FileStatus::Good => converted += 1,
            FileStatus::Partial => partial += 1,
            FileStatus::Failed => failed += 1,
            FileStatus::Unsupported => unsupported += 1,
            FileStatus::Cancelled => failed += 1,
        }
        export_files.push(res);
    }
    
    let summary = JobSummary {
        total_files: export_files.len(),
        converted,
        partial,
        failed,
        unsupported,
    };
    
    let job_status = if failed == export_files.len() && !export_files.is_empty() {
        JobStatus::Failed
    } else if failed > 0 || partial > 0 {
        JobStatus::PartialSuccess
    } else {
        JobStatus::Success
    };
    
    let exit_code = match job_status {
        JobStatus::Success => 0,
        JobStatus::PartialSuccess => 1,
        JobStatus::Failed => 2,
        _ => 4,
    };
    
    Ok(JobResult {
        status: job_status,
        exit_code,
        input: input_path.to_string_lossy().to_string(),
        output_folder: output_folder.to_string_lossy().to_string(),
        export_zip: zip_path.to_string_lossy().to_string(),
        summary,
        files: export_files,
    })
}

fn write_metadata_files(folder: &Path, files: &[(FileResult, String, Option<Vec<u8>>)]) -> Result<(), ErrorCode> {
    // README.md
    let readme = "# AgentReady Export\n\nThis folder contains markdown representations of your files, optimized for AI agent consumption.\n";
    fs::write(folder.join("README.md"), readme).map_err(|_| ErrorCode::ConversionFailed)?;
    
    // index.md
    let mut index = String::from("# Export Index\n\n");
    for (res, _, _) in files {
        if let Some(out) = &res.output_file {
            index.push_str(&format!("- [{}]({})\n", res.source_file, out));
        }
    }
    fs::write(folder.join("index.md"), index).map_err(|_| ErrorCode::ConversionFailed)?;
    
    // conversion-report.md
    let mut report = String::from("# Conversion Report\n\n");
    for (res, _, _) in files {
        report.push_str(&format!("- {}: {:?}\n", res.source_file, res.status));
    }
    fs::write(folder.join("conversion-report.md"), report).map_err(|_| ErrorCode::ConversionFailed)?;
    
    Ok(())
}

fn create_zip_archive(folder: &Path, zip_path: &Path) -> Result<(), ErrorCode> {
    let zip_file = fs::File::create(zip_path).map_err(|_| ErrorCode::ZipCreationFailed)?;
    let mut zip = ZipWriter::new(zip_file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    
    // Simplistic recursive zip using walkdir or similar, but since we know the exact structure we can just iterate
    let mut buffer = Vec::new();
    
    let walker = walkdir::WalkDir::new(folder);
    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            let name = path.strip_prefix(folder).unwrap();
            #[allow(deprecated)]
            zip.start_file(name.to_string_lossy(), options).map_err(|_| ErrorCode::ZipCreationFailed)?;
            let mut f = File::open(path).map_err(|_| ErrorCode::ZipCreationFailed)?;
            f.read_to_end(&mut buffer).map_err(|_| ErrorCode::ZipCreationFailed)?;
            zip.write_all(&buffer).map_err(|_| ErrorCode::ZipCreationFailed)?;
            buffer.clear();
        }
    }
    
    zip.finish().map_err(|_| ErrorCode::ZipCreationFailed)?;
    Ok(())
}
