use std::fs::{self, File};
use std::path::Path;
use std::io::{Read, Write};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::models::{ConvertedFile, FileStatus, JobResult, JobStatus, JobSummary, ErrorCode, AgentReadyError};

pub fn create_export(
    input_paths: &[String],
    output_folder: &Path,
    zip_name: Option<&str>,
    results: Vec<ConvertedFile>,
) -> Result<JobResult, AgentReadyError> {
    // 1. Overwrite protection — refuse if output already exists
    if output_folder.exists() {
        return Err(AgentReadyError::UserFacing(ErrorCode::OutputAlreadyExists));
    }
    fs::create_dir_all(output_folder).map_err(AgentReadyError::Io)?;

    let docs_folder = output_folder.join("documents");
    fs::create_dir_all(&docs_folder).map_err(AgentReadyError::Io)?;

    let data_folder = output_folder.join("data");

    // 2. Resolve duplicates and write files
    let mut final_results = Vec::new();
    let mut name_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for mut converted in results {
        if converted.result.status == FileStatus::Failed || converted.result.status == FileStatus::Unsupported {
            final_results.push(converted);
            continue;
        }

        let base_name = Path::new(&converted.result.source_file)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("document")
            .to_lowercase()
            .replace(" ", "-")
            .replace("_", "-");

        let ext = Path::new(&converted.result.source_file)
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

        // Prepend frontmatter to the markdown content
        let content_with_frontmatter = inject_frontmatter(&converted.markdown, &converted.result);

        let out_path = docs_folder.join(&output_name);
        fs::write(&out_path, &content_with_frontmatter).map_err(AgentReadyError::Io)?;
        converted.result.output_file = Some(format!("documents/{}", output_name));

        // Write raw data if CSV
        if let Some(raw_bytes) = converted.raw_data.as_ref() {
            if !data_folder.exists() {
                fs::create_dir_all(&data_folder).map_err(AgentReadyError::Io)?;
            }
            let raw_name = output_name.replace(".md", ".csv");
            fs::write(data_folder.join(&raw_name), raw_bytes).map_err(AgentReadyError::Io)?;
        }

        final_results.push(converted);
    }

    // Populate user_message before metadata is written so the on-disk
    // report and manifest carry it too, not just the in-memory result.
    for res in &mut final_results {
        if res.result.user_message.is_none() {
            if let Some(ref code) = res.result.error_code {
                res.result.user_message = Some(code.user_message().to_string());
            } else if res.result.status == FileStatus::Partial
                && let Some(ref w) = res.result.warning {
                    res.result.user_message = Some(format!("This file was converted, but the output may need review: {}", w));
                }
        }
    }

    // 3. Write metadata files
    write_metadata_files(output_folder, &final_results)?;

    // 4. Create ZIP archive
    let zip_file_name = zip_name.map(|s| s.to_string())
        .unwrap_or_else(|| {
            output_folder.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("export")
                .to_string() + ".zip"
        });
    let zip_path = output_folder.with_file_name(&zip_file_name);
    create_zip_archive(output_folder, &zip_path)?;

    // 5. Generate JobSummary
    let mut converted = 0;
    let mut partial = 0;
    let mut failed = 0;
    let mut unsupported = 0;

    let mut export_files = Vec::new();

    for res in final_results {
        match res.result.status {
            FileStatus::Good => converted += 1,
            FileStatus::Partial => partial += 1,
            FileStatus::Failed => failed += 1,
            FileStatus::Unsupported => unsupported += 1,
            FileStatus::Cancelled => failed += 1,
        }
        export_files.push(res.result);
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
        JobStatus::ValidationError => 3,
        _ => 4,
    };

    let input_label = if input_paths.len() == 1 {
        input_paths[0].clone()
    } else {
        input_paths.join(", ")
    };

    Ok(JobResult {
        status: job_status,
        exit_code,
        input: input_label,
        output_folder: output_folder.to_string_lossy().to_string(),
        export_zip: zip_path.to_string_lossy().to_string(),
        summary,
        files: export_files,
    })
}

fn inject_frontmatter(markdown: &str, result: &crate::models::FileResult) -> String {
    let status_str = match result.status {
        FileStatus::Good => "good",
        FileStatus::Partial => "partial",
        _ => "unknown",
    };

    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(&format!("source_file: {}\n", result.source_file));
    out.push_str(&format!("source_type: {}\n", result.source_type));
    out.push_str("converted_by: agentready-v1\n");
    out.push_str(&format!("status: {}\n", status_str));
    out.push_str("---\n\n");

    if let Some(ref warning) = result.warning {
        out.push_str(&format!("> **AgentReady warning:** {}\n\n", warning));
    }

    out.push_str(markdown);
    out
}

fn write_metadata_files(folder: &Path, files: &[ConvertedFile]) -> Result<(), AgentReadyError> {
    // README.md
    let readme = r#"# AgentReady Export

Markdown representations of your files, optimized for AI agent consumption.

## Quick start

1. Read `manifest.json` for structured metadata about all converted files.
2. Browse `documents/` for the Markdown output of each file.
3. Check `data/` for raw copies of tabular data (CSV).

## Folder map

```
├── documents/        # Converted Markdown files (with YAML frontmatter)
├── data/             # Raw copies of CSV files
├── manifest.json     # Structured metadata for all files
├── index.md          # Linked list of converted files
├── conversion-report.md  # Status table for every file
└── README.md         # This file
```

## Trust

Your files were processed locally. No content was sent to any cloud service.

## Legal notice

**Your content.** This export was created from files you supplied. You represent and warrant that you own each file or have all rights and permissions needed to convert and use it, including for AI agent workflows.

**DRM-free only.** AgentReady supports DRM-free documents only. It does not decrypt, strip, or bypass digital rights management, Kindle DRM, Adobe ACS, or similar copy protection. Encrypted or DRM-protected files are rejected.

**No responsibility.** AgentReady and HumanGoodAI provide this software and export format **"as is"** without warranty of any kind. We do not monitor, verify, or take responsibility for the legality, accuracy, or appropriateness of your source files or how you use converted Markdown. **You are solely responsible** for your uploads, exports, and compliance with applicable copyright, contract, and privacy laws.

**Not legal advice.** This notice is not legal advice. If you are unsure whether you may convert a file, consult qualified counsel or the rights holder before uploading.

## Citation

If you use this export in an AI pipeline, cite AgentReady V1 by HumanGoodAI.
"#;
    fs::write(folder.join("README.md"), readme).map_err(AgentReadyError::Io)?;

    // index.md
    let mut index = String::from("# Export Index\n\n");
    for converted in files {
        if let Some(out) = &converted.result.output_file {
            index.push_str(&format!("- [{}]({})\n", converted.result.source_file, out));
        }
    }
    fs::write(folder.join("index.md"), index).map_err(AgentReadyError::Io)?;

    // conversion-report.md
    let mut report = String::from("# Conversion Report\n\n");
    report.push_str("| File | Status | Error | Message |\n");
    report.push_str("|------|--------|-------|--------|\n");
    for converted in files {
        let res = &converted.result;
        let status = format!("{:?}", res.status);
        let error = res.error_code
            .as_ref()
            .map(|c| format!("{:?}", c))
            .unwrap_or_else(|| "—".to_string());
        let msg = res.user_message
            .as_deref()
            .unwrap_or("—");
        report.push_str(&format!("| {} | {} | {} | {} |\n", res.source_file, status, error, msg));
    }
    fs::write(folder.join("conversion-report.md"), report).map_err(AgentReadyError::Io)?;

    // manifest.json — structured metadata for programmatic consumers
    let manifest_files: Vec<_> = files.iter().map(|converted| {
        serde_json::json!({
            "source_file": converted.result.source_file,
            "source_type": converted.result.source_type,
            "status": converted.result.status,
            "output_file": converted.result.output_file,
            "error_code": converted.result.error_code,
            "warning": converted.result.warning,
            "user_message": converted.result.user_message,
            "markdown_bytes": converted.markdown.len(),
        })
    }).collect();

    let manifest = serde_json::json!({
        "version": "1.0",
        "generated_by": "agentready-v1",
        "files": manifest_files,
    });

    let manifest_json = serde_json::to_string_pretty(&manifest).map_err(AgentReadyError::Json)?;
    fs::write(folder.join("manifest.json"), manifest_json).map_err(AgentReadyError::Io)?;

    Ok(())
}

fn create_zip_archive(folder: &Path, zip_path: &Path) -> Result<(), AgentReadyError> {
    let zip_file = fs::File::create(zip_path).map_err(AgentReadyError::Io)?;
    let mut zip = ZipWriter::new(zip_file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let mut buffer = Vec::new();

    let walker = walkdir::WalkDir::new(folder);
    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            let name = match path.strip_prefix(folder) {
                Ok(n) => n,
                Err(_) => continue,
            };
            #[allow(deprecated)]
            zip.start_file(name.to_string_lossy(), options).map_err(AgentReadyError::Zip)?;
            let mut f = File::open(path).map_err(AgentReadyError::Io)?;
            f.read_to_end(&mut buffer).map_err(AgentReadyError::Io)?;
            zip.write_all(&buffer).map_err(|_| AgentReadyError::UserFacing(ErrorCode::ZipCreationFailed))?;
            buffer.clear();
        }
    }

    zip.finish().map_err(AgentReadyError::Zip)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_file_result(name: &str, ext: &str, status: FileStatus) -> crate::models::FileResult {
        crate::models::FileResult {
            source_file: name.to_string(),
            source_type: ext.to_string(),
            status,
            output_file: None,
            error_code: None,
            warning: None,
            user_message: None,
        }
    }

    fn make_converted(name: &str, ext: &str, status: FileStatus, markdown: &str, raw: Option<Vec<u8>>) -> ConvertedFile {
        ConvertedFile {
            result: make_file_result(name, ext, status),
            markdown: markdown.to_string(),
            raw_data: raw,
        }
    }

    #[test]
    fn creates_folder_structure() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("output");
        let input = dir.path().join("input");
        fs::create_dir_all(&input).unwrap();

        let results = vec![
            make_converted("test.txt", "txt", FileStatus::Good, "Hello", None),
        ];

        let result = create_export(&["input".to_string()], &output, None, results).unwrap();
        assert!(output.join("documents").exists());
        assert!(output.join("documents/test-txt.md").exists());
        assert!(output.join("README.md").exists());
        assert!(output.join("index.md").exists());
        assert!(output.join("conversion-report.md").exists());
        assert!(output.join("manifest.json").exists());
        assert!(output.with_extension("zip").exists());
        assert_eq!(result.status, JobStatus::Success);
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn rejects_existing_output() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("output");
        fs::create_dir_all(&output).unwrap();

        let input = dir.path().join("input");
        fs::create_dir_all(&input).unwrap();

        let results = vec![
            make_converted("test.txt", "txt", FileStatus::Good, "Hello", None),
        ];

        let result = create_export(&["input".to_string()], &output, None, results);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_error_code(), ErrorCode::OutputAlreadyExists);
    }

    #[test]
    fn injects_frontmatter() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("output");
        let input = dir.path().join("input");
        fs::create_dir_all(&input).unwrap();

        let results = vec![
            make_converted("test.txt", "txt", FileStatus::Good, "Hello", None),
        ];

        create_export(&["input".to_string()], &output, None, results).unwrap();
        let content = fs::read_to_string(output.join("documents/test-txt.md")).unwrap();
        assert!(content.starts_with("---\n"));
        assert!(content.contains("source_file: test.txt"));
        assert!(content.contains("source_type: txt"));
        assert!(content.contains("converted_by: agentready-v1"));
        assert!(content.contains("status: good"));
    }

    #[test]
    fn injects_warning_for_partial() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("output");
        let input = dir.path().join("input");
        fs::create_dir_all(&input).unwrap();

        let mut fr = make_file_result("test.txt", "txt", FileStatus::Partial);
        fr.warning = Some("Some text was dropped".into());

        let results = vec![
            ConvertedFile { result: fr, markdown: "Hello".into(), raw_data: None },
        ];

        create_export(&["input".to_string()], &output, None, results).unwrap();
        let content = fs::read_to_string(output.join("documents/test-txt.md")).unwrap();
        assert!(content.contains("status: partial"));
        assert!(content.contains("AgentReady warning"));
        assert!(content.contains("Some text was dropped"));
    }

    #[test]
    fn resolves_duplicate_filenames() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("output");
        let input = dir.path().join("input");
        fs::create_dir_all(&input).unwrap();

        let results = vec![
            make_converted("test.txt", "txt", FileStatus::Good, "First", None),
            make_converted("test.txt", "txt", FileStatus::Good, "Second", None),
        ];

        create_export(&["input".to_string()], &output, None, results).unwrap();
        assert!(output.join("documents/test-txt.md").exists());
        assert!(output.join("documents/test-txt-2.md").exists());
    }

    #[test]
    fn csv_raw_data_in_data_folder() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("output");
        let input = dir.path().join("input");
        fs::create_dir_all(&input).unwrap();

        let results = vec![
            make_converted("data.csv", "csv", FileStatus::Good, "| a |\n| --- |\n| 1 |\n", Some(b"a\n1\n".to_vec())),
        ];

        create_export(&["input".to_string()], &output, None, results).unwrap();
        assert!(output.join("data/data-csv.csv").exists());
        let raw = fs::read_to_string(output.join("data/data-csv.csv")).unwrap();
        assert_eq!(raw, "a\n1\n");
    }

    #[test]
    fn conversion_report_is_table() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("output");
        let input = dir.path().join("input");
        fs::create_dir_all(&input).unwrap();

        let results = vec![
            make_converted("test.txt", "txt", FileStatus::Good, "Hello", None),
        ];

        create_export(&["input".to_string()], &output, None, results).unwrap();
        let report = fs::read_to_string(output.join("conversion-report.md")).unwrap();
        assert!(report.contains("| File | Status |"));
    }

    #[test]
    fn report_on_disk_includes_partial_message() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("output");
        let input = dir.path().join("input");
        fs::create_dir_all(&input).unwrap();

        let mut partial = make_file_result("messy.csv", "csv", FileStatus::Partial);
        partial.warning = Some("2 row(s) had a different number of columns".to_string());

        let results = vec![
            ConvertedFile { result: partial, markdown: "| a |\n| --- |\n".into(), raw_data: None },
        ];

        create_export(&["input".to_string()], &output, None, results).unwrap();
        let report = fs::read_to_string(output.join("conversion-report.md")).unwrap();
        assert!(
            report.contains("the output may need review"),
            "partial message missing from on-disk report: {report}"
        );
        let manifest = fs::read_to_string(output.join("manifest.json")).unwrap();
        assert!(manifest.contains("the output may need review"));
    }

    #[test]
    fn partial_success_exit_code() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("output");
        let input = dir.path().join("input");
        fs::create_dir_all(&input).unwrap();

        let mut failed = make_file_result("bad.txt", "txt", FileStatus::Failed);
        failed.error_code = Some(ErrorCode::ConversionFailed);
        failed.user_message = Some(ErrorCode::ConversionFailed.user_message().to_string());

        let results = vec![
            make_converted("good.txt", "txt", FileStatus::Good, "Hello", None),
            ConvertedFile { result: failed, markdown: String::new(), raw_data: None },
        ];

        let result = create_export(&["input".to_string()], &output, None, results).unwrap();
        assert_eq!(result.status, JobStatus::PartialSuccess);
        assert_eq!(result.exit_code, 1);
    }

    #[test]
    fn all_failed_exit_code() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("output");
        let input = dir.path().join("input");
        fs::create_dir_all(&input).unwrap();

        let mut failed = make_file_result("bad.txt", "txt", FileStatus::Failed);
        failed.error_code = Some(ErrorCode::ConversionFailed);

        let results = vec![
            ConvertedFile { result: failed, markdown: String::new(), raw_data: None },
        ];

        let result = create_export(&["input".to_string()], &output, None, results).unwrap();
        assert_eq!(result.status, JobStatus::Failed);
        assert_eq!(result.exit_code, 2);
    }

    #[test]
    fn manifest_json_contains_file_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("output");
        let input = dir.path().join("input");
        fs::create_dir_all(&input).unwrap();

        let mut bad = make_file_result("bad.txt", "txt", FileStatus::Failed);
        bad.error_code = Some(ErrorCode::ConversionFailed);
        bad.user_message = Some("AgentReady could not convert this file.".into());

        let results = vec![
            make_converted("good.txt", "txt", FileStatus::Good, "Hello world", None),
            ConvertedFile { result: bad, markdown: String::new(), raw_data: None },
        ];

        create_export(&["input".to_string()], &output, None, results).unwrap();
        let manifest: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output.join("manifest.json")).unwrap()).unwrap();

        assert_eq!(manifest["version"], "1.0");
        assert_eq!(manifest["generated_by"], "agentready-v1");
        assert_eq!(manifest["files"].as_array().unwrap().len(), 2);
        assert_eq!(manifest["files"][0]["source_file"], "good.txt");
        assert_eq!(manifest["files"][0]["status"], "good");
        assert_eq!(manifest["files"][0]["markdown_bytes"], 11);
        assert_eq!(manifest["files"][1]["status"], "failed");
        assert_eq!(manifest["files"][1]["error_code"], "CONVERSION_FAILED");
    }

    #[test]
    fn custom_zip_name_is_used() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("output");
        let input = dir.path().join("input");
        fs::create_dir_all(&input).unwrap();

        let results = vec![
            make_converted("test.txt", "txt", FileStatus::Good, "Hello", None),
        ];

        let result = create_export(&["input".to_string()], &output, Some("my-export.zip"), results).unwrap();
        assert!(result.export_zip.ends_with("my-export.zip"));
    }
}
