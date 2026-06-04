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
