use std::fs;
use std::io::Read;
use std::path::Path;

use csv::ReaderBuilder;
use encoding_rs_io::DecodeReaderBytesBuilder;

use crate::converters::ConversionResult;
use crate::models::AgentReadyError;

pub fn convert_csv(path: &Path) -> Result<ConversionResult, AgentReadyError> {
    // Read raw bytes for preservation in the data/ folder
    let raw_bytes = fs::read(path).map_err(AgentReadyError::Io)?;

    // Decode text with BOM stripping and encoding fallback (fixes C3)
    let decoded = decode_csv_text(path)?;

    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(decoded.as_bytes());

    let mut markdown = String::new();

    // Headers
    if let Ok(headers) = reader.headers() {
        let header_str = headers.iter().collect::<Vec<_>>().join(" | ");
        markdown.push_str(&format!("| {} |\n", header_str));

        let separator = headers.iter().map(|_| "---").collect::<Vec<_>>().join(" | ");
        markdown.push_str(&format!("| {} |\n", separator));
    }

    // Rows
    for result in reader.records() {
        if let Ok(record) = result {
            let row_str = record.iter().collect::<Vec<_>>().join(" | ");
            markdown.push_str(&format!("| {} |\n", row_str));
        } else {
            return Err(AgentReadyError::UserFacing(
                crate::models::ErrorCode::ConversionFailed,
            ));
        }
    }

    if markdown.trim().is_empty() {
        return Err(AgentReadyError::UserFacing(
            crate::models::ErrorCode::NoReadableText,
        ));
    }

    Ok(ConversionResult {
        markdown,
        warning: None,
        raw_data: Some(raw_bytes),
    })
}

/// Decodes a CSV file to UTF-8 String, stripping BOM and handling legacy encodings.
fn decode_csv_text(path: &Path) -> Result<String, AgentReadyError> {
    let file = fs::File::open(path).map_err(AgentReadyError::Io)?;
    let mut decoder = DecodeReaderBytesBuilder::new()
        .bom_override(true)
        .build(file);

    let mut content = String::new();
    decoder
        .read_to_string(&mut content)
        .map_err(AgentReadyError::Io)?;

    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_temp_csv(content: &[u8]) -> (TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.csv");
        fs::write(&path, content).unwrap();
        (dir, path)
    }

    #[test]
    fn converts_basic_csv() {
        let csv = b"name,age,city\nAlice,30,NYC\nBob,25,LA\n";
        let (dir, path) = write_temp_csv(csv);
        let result = convert_csv(&path).unwrap();

        assert!(result.markdown.contains("| name | age | city |"));
        assert!(result.markdown.contains("| --- | --- | --- |"));
        assert!(result.markdown.contains("| Alice | 30 | NYC |"));
        assert!(result.markdown.contains("| Bob | 25 | LA |"));
        assert!(result.raw_data.is_some());
        drop(dir);
    }

    #[test]
    fn empty_csv_produces_minimal_table() {
        // An empty CSV file produces headers-only output
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.csv");
        fs::write(&path, b"").unwrap();
        let result = convert_csv(&path).unwrap();
        // The csv crate produces a single empty header column
        assert!(result.markdown.contains("|"));
        drop(dir);
    }

    #[test]
    fn single_column_csv() {
        let csv = b"name\nAlice\nBob\n";
        let (dir, path) = write_temp_csv(csv);
        let result = convert_csv(&path).unwrap();
        assert!(result.markdown.contains("| name |"));
        assert!(result.markdown.contains("| Alice |"));
        drop(dir);
    }

    #[test]
    fn csv_with_quoted_commas() {
        let csv = b"name,desc\nAlice,\"Hello, world\"\n";
        let (dir, path) = write_temp_csv(csv);
        let result = convert_csv(&path).unwrap();
        assert!(result.markdown.contains("Hello, world"));
        drop(dir);
    }

    #[test]
    fn raw_data_preserved() {
        let csv = b"a,b\n1,2\n";
        let (dir, path) = write_temp_csv(csv);
        let result = convert_csv(&path).unwrap();
        assert_eq!(result.raw_data.unwrap().as_slice(), csv.as_slice());
        drop(dir);
    }

    #[test]
    fn strips_bom() {
        let csv = b"\xef\xbb\xbfname,age\nAlice,30\n";
        let (dir, path) = write_temp_csv(csv);
        let result = convert_csv(&path).unwrap();
        assert!(!result.markdown.starts_with('\u{feff}'));
        assert!(result.markdown.contains("| name | age |"));
        drop(dir);
    }
}
