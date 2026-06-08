use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

fn agentready_bin() -> PathBuf {
    std::env::var("CARGO_BIN_EXE_agentready")
        .expect("CARGO_BIN_EXE_agentready not set")
        .into()
}

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

#[test]
fn convert_single_txt() {
    let bin = agentready_bin();
    let dir = TempDir::new().unwrap();
    let input_dir = dir.path().join("input");
    fs::create_dir_all(&input_dir).unwrap();
    fs::write(input_dir.join("hello.txt"), "Hello world").unwrap();

    let output = dir.path().join("output");
    let status = Command::new(&bin)
        .args(["convert", input_dir.to_str().unwrap(), "--output", output.to_str().unwrap()])
        .current_dir(project_root())
        .status()
        .unwrap();

    assert!(status.success(), "Expected exit code 0");
    assert!(output.join("documents").exists(), "documents/ folder missing");
    assert!(output.join("README.md").exists(), "README.md missing");
    assert!(output.join("manifest.json").exists(), "manifest.json missing");
    assert!(output.with_extension("zip").exists(), "zip file missing");
}

#[test]
fn json_flag_produces_valid_json() {
    let bin = agentready_bin();
    let dir = TempDir::new().unwrap();
    let input_dir = dir.path().join("input");
    fs::create_dir_all(&input_dir).unwrap();
    fs::write(input_dir.join("test.txt"), "Test content").unwrap();

    let output = dir.path().join("output");
    let output2 = Command::new(&bin)
        .args(["convert", input_dir.to_str().unwrap(), "--output", output.to_str().unwrap(), "--json"])
        .current_dir(project_root())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output2.stdout);
    let json: serde_json::Value = serde_json::from_str(stdout.trim()).expect("stdout is not valid JSON");
    assert_eq!(json["status"], "success");
    assert_eq!(json["exit_code"], 0);
}

#[test]
fn overwrite_protection_exits_3() {
    let bin = agentready_bin();
    let dir = TempDir::new().unwrap();
    let input_dir = dir.path().join("input");
    fs::create_dir_all(&input_dir).unwrap();
    fs::write(input_dir.join("test.txt"), "Content").unwrap();

    let output = dir.path().join("output");

    // First run succeeds
    let status1 = Command::new(&bin)
        .args(["convert", input_dir.to_str().unwrap(), "--output", output.to_str().unwrap()])
        .current_dir(project_root())
        .status()
        .unwrap();
    assert!(status1.success());

    // Second run exits 3
    let status2 = Command::new(&bin)
        .args(["convert", input_dir.to_str().unwrap(), "--output", output.to_str().unwrap()])
        .current_dir(project_root())
        .status()
        .unwrap();
    assert_eq!(status2.code(), Some(3));
}

#[test]
fn no_input_files_exits_3() {
    let bin = agentready_bin();
    let dir = TempDir::new().unwrap();
    let empty_dir = dir.path().join("empty");
    fs::create_dir_all(&empty_dir).unwrap();

    let output = dir.path().join("output");
    let status = Command::new(&bin)
        .args(["convert", empty_dir.to_str().unwrap(), "--output", output.to_str().unwrap()])
        .current_dir(project_root())
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(3));
}

#[test]
fn recursive_includes_subdirs() {
    let bin = agentready_bin();
    let root = project_root();
    let input = root.join("examples/sample-input");

    // Without recursive — only top-level files
    let dir1 = TempDir::new().unwrap();
    let out1 = dir1.path().join("output");
    let result1 = Command::new(&bin)
        .args(["convert", input.to_str().unwrap(), "--output", out1.to_str().unwrap(), "--json"])
        .current_dir(&root)
        .output()
        .unwrap();
    let json1: serde_json::Value = serde_json::from_str(String::from_utf8_lossy(&result1.stdout).trim()).unwrap();
    let flat_count = json1["summary"]["total_files"].as_u64().unwrap();

    // With recursive — includes subdirs
    let dir2 = TempDir::new().unwrap();
    let out2 = dir2.path().join("output");
    let result2 = Command::new(&bin)
        .args(["convert", input.to_str().unwrap(), "--output", out2.to_str().unwrap(), "--recursive", "--json"])
        .current_dir(&root)
        .output()
        .unwrap();
    let json2: serde_json::Value = serde_json::from_str(String::from_utf8_lossy(&result2.stdout).trim()).unwrap();
    let recursive_count = json2["summary"]["total_files"].as_u64().unwrap();

    assert!(recursive_count > flat_count, "Recursive ({}) should find more files than flat ({})", recursive_count, flat_count);
}

#[test]
fn fixture_structure_is_correct() {
    let bin = agentready_bin();
    let root = project_root();
    let input = root.join("examples/sample-input/clean");

    let dir = TempDir::new().unwrap();
    let output = dir.path().join("output");

    let status = Command::new(&bin)
        .args(["convert", input.to_str().unwrap(), "--output", output.to_str().unwrap()])
        .current_dir(&root)
        .status()
        .unwrap();
    assert!(status.success());

    // Folder structure
    assert!(output.join("documents").exists());
    assert!(output.join("README.md").exists());
    assert!(output.join("index.md").exists());
    assert!(output.join("conversion-report.md").exists());
    assert!(output.join("manifest.json").exists());

    // Frontmatter in at least one file
    let mut docs = fs::read_dir(output.join("documents")).unwrap();
    let has_frontmatter = docs.any(|e| {
        let content = fs::read_to_string(e.unwrap().path()).unwrap();
        content.starts_with("---\n")
    });
    assert!(has_frontmatter, "At least one document should have frontmatter");

    // manifest.json is valid
    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["version"], "1.0");
    assert_eq!(manifest["generated_by"], "agentready-v1");
}

#[test]
fn convert_minimal_docx() {
    let bin = agentready_bin();
    let root = project_root();
    let input = root.join("examples/sample-input/ebooks/minimal.docx");

    let dir = TempDir::new().unwrap();
    let output = dir.path().join("output");

    let status = Command::new(&bin)
        .args(["convert", input.to_str().unwrap(), "--output", output.to_str().unwrap()])
        .current_dir(&root)
        .status()
        .unwrap();
    assert!(status.success(), "DOCX convert should exit 0");

    let doc = output.join("documents/minimal-docx.md");
    assert!(doc.exists(), "minimal-docx.md missing");
    let content = fs::read_to_string(doc).unwrap();
    assert!(content.contains("# Staff Handbook"));
    assert!(content.contains("- Be kind"));
    assert!(content.contains("- Be clear"));
}

#[test]
fn convert_minimal_pdf() {
    let bin = agentready_bin();
    let root = project_root();
    let input = root.join("examples/sample-input/ebooks/minimal.pdf");

    let dir = TempDir::new().unwrap();
    let output = dir.path().join("output");

    let status = Command::new(&bin)
        .args(["convert", input.to_str().unwrap(), "--output", output.to_str().unwrap()])
        .current_dir(&root)
        .status()
        .unwrap();
    assert!(status.success(), "PDF convert should exit 0");

    let doc = output.join("documents/minimal-pdf.md");
    assert!(doc.exists(), "minimal-pdf.md missing");
    let content = fs::read_to_string(doc).unwrap();
    assert!(content.contains("Hello PDF reader"));
    assert!(content.contains("AgentReady text extraction smoke test"));
}

#[test]
fn convert_minimal_epub() {
    let bin = agentready_bin();
    let root = project_root();
    let input = root.join("examples/sample-input/ebooks/minimal.epub");

    let dir = TempDir::new().unwrap();
    let output = dir.path().join("output");

    let status = Command::new(&bin)
        .args(["convert", input.to_str().unwrap(), "--output", output.to_str().unwrap()])
        .current_dir(&root)
        .status()
        .unwrap();
    assert!(status.success(), "EPUB convert should exit 0");

    let docs_dir = output.join("documents");
    assert!(docs_dir.exists(), "documents/ folder missing");

    let mut found_content = false;
    for entry in fs::read_dir(&docs_dir).unwrap() {
        let content = fs::read_to_string(entry.unwrap().path()).unwrap();
        if content.contains("Chapter One") && content.contains("Hello ebook reader") {
            found_content = true;
            break;
        }
    }
    assert!(found_content, "EPUB markdown should contain chapter text");
    assert!(output.join("README.md").exists(), "README.md missing");
    assert!(output.join("index.md").exists(), "index.md missing");
}

/// Run manually when you have a DRM-free MOBI on disk:
/// `AGENTREADY_MOBI_SAMPLE=/path/to/book.mobi cargo test -p agentready convert_mobi_sample -- --ignored`
#[test]
#[ignore = "requires AGENTREADY_MOBI_SAMPLE env pointing to a local DRM-free .mobi"]
fn convert_mobi_sample() {
    let sample = std::env::var("AGENTREADY_MOBI_SAMPLE").expect("set AGENTREADY_MOBI_SAMPLE");
    let bin = agentready_bin();
    let root = project_root();

    let dir = TempDir::new().unwrap();
    let output = dir.path().join("output");

    let status = Command::new(&bin)
        .args(["convert", &sample, "--output", output.to_str().unwrap()])
        .current_dir(&root)
        .status()
        .unwrap();
    assert!(status.success());
    assert!(output.join("documents").exists());
}
