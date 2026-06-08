use std::path::Path;

use crate::converters::plain_text_to_markdown::page_text_to_markdown;
use crate::converters::{read_text_file, ConversionResult};
use crate::models::AgentReadyError;

pub fn convert_txt(path: &Path) -> Result<ConversionResult, AgentReadyError> {
    let content = read_text_file(path)?;

    Ok(ConversionResult {
        markdown: page_text_to_markdown(&content),
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

    fn write_temp_file(content: &[u8], ext: &str) -> (TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(format!("test.{}", ext));
        fs::write(&path, content).unwrap();
        (dir, path)
    }

    #[test]
    fn converts_basic_text() {
        let (dir, path) = write_temp_file(b"Hello world", "txt");
        let result = convert_txt(&path).unwrap();
        assert!(result.markdown.contains("Hello world"));
        drop(dir);
    }

    #[test]
    fn rejects_empty_file() {
        let (dir, path) = write_temp_file(b"", "txt");
        let result = convert_txt(&path);
        assert_eq!(result.unwrap_err().to_error_code(), ErrorCode::NoReadableText);
        drop(dir);
    }

    #[test]
    fn rejects_whitespace_only_file() {
        let (dir, path) = write_temp_file(b"   \n\t  ", "txt");
        let result = convert_txt(&path);
        assert_eq!(result.unwrap_err().to_error_code(), ErrorCode::NoReadableText);
        drop(dir);
    }

    #[test]
    fn handles_utf8() {
        let (dir, path) = write_temp_file("Café résumé naïve".as_bytes(), "txt");
        let result = convert_txt(&path).unwrap();
        assert!(result.markdown.contains("Café"));
        drop(dir);
    }

    #[test]
    fn strips_bom() {
        let content = b"\xef\xbb\xbfHello world";
        let (dir, path) = write_temp_file(content, "txt");
        let result = convert_txt(&path).unwrap();
        assert!(!result.markdown.starts_with('\u{feff}'));
        assert!(result.markdown.contains("Hello world"));
        drop(dir);
    }
}
