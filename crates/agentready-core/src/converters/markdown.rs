use std::path::Path;

use crate::converters::{read_text_file, ConversionResult};
use crate::models::AgentReadyError;

pub fn convert_markdown(path: &Path) -> Result<ConversionResult, AgentReadyError> {
    let content = read_text_file(path)?;

    Ok(ConversionResult {
        markdown: content,
        warning: None,
        raw_data: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ErrorCode;
    use std::fs;
    use tempfile::TempDir;

    fn write_temp_file(content: &str) -> (TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.md");
        fs::write(&path, content).unwrap();
        (dir, path)
    }

    #[test]
    fn preserves_markdown_content() {
        let md = "# Heading\n\nParagraph with **bold** and *italic*.\n\n- Item 1\n- Item 2\n";
        let (dir, path) = write_temp_file(md);
        let result = convert_markdown(&path).unwrap();
        assert_eq!(result.markdown, md);
        drop(dir);
    }

    #[test]
    fn rejects_empty_markdown() {
        let (dir, path) = write_temp_file("");
        let result = convert_markdown(&path);
        assert_eq!(result.unwrap_err().to_error_code(), ErrorCode::NoReadableText);
        drop(dir);
    }

    #[test]
    fn preserves_tables() {
        let md = "| A | B |\n|---|---|\n| 1 | 2 |\n";
        let (dir, path) = write_temp_file(md);
        let result = convert_markdown(&path).unwrap();
        assert!(result.markdown.contains("| A | B |"));
        drop(dir);
    }

    #[test]
    fn preserves_code_blocks() {
        let md = "```rust\nfn main() {}\n```\n";
        let (dir, path) = write_temp_file(md);
        let result = convert_markdown(&path).unwrap();
        assert!(result.markdown.contains("fn main()"));
        drop(dir);
    }
}
