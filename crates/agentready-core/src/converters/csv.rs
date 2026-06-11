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
        .flexible(true)
        .from_reader(decoded.as_bytes());

    let headers: Vec<String> = reader
        .headers()
        .map_err(|_| AgentReadyError::UserFacing(crate::models::ErrorCode::ConversionFailed))?
        .iter()
        .map(escape_cell)
        .collect();

    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut ragged_rows = 0usize;
    let mut skipped_rows = 0usize;
    for result in reader.records() {
        match result {
            Ok(record) => {
                let row: Vec<String> = record.iter().map(escape_cell).collect();
                if row.len() != headers.len() {
                    ragged_rows += 1;
                }
                rows.push(row);
            }
            Err(_) => skipped_rows += 1,
        }
    }

    // Pad every row (and the header) to the widest row so the table stays valid.
    let width = rows
        .iter()
        .map(|r| r.len())
        .chain(std::iter::once(headers.len()))
        .max()
        .unwrap_or(0)
        .max(1);

    let mut markdown = String::new();
    markdown.push_str(&format!("| {} |\n", padded(&headers, width).join(" | ")));
    markdown.push_str(&format!("| {} |\n", vec!["---"; width].join(" | ")));
    for row in &rows {
        markdown.push_str(&format!("| {} |\n", padded(row, width).join(" | ")));
    }

    let warning = if skipped_rows > 0 {
        Some(format!(
            "{} row(s) could not be read and were skipped. Please review the output against the original CSV in data/.",
            skipped_rows
        ))
    } else if ragged_rows > 0 {
        Some(format!(
            "{} row(s) had a different number of columns than the header. Cells were padded to keep the table aligned.",
            ragged_rows
        ))
    } else {
        None
    };

    Ok(ConversionResult {
        markdown,
        warning,
        raw_data: Some(raw_bytes),
    })
}

/// Makes a CSV cell safe inside a Markdown table row.
fn escape_cell(cell: &str) -> String {
    cell.replace('|', "\\|")
        .replace(['\r', '\n'], " ")
        .trim()
        .to_string()
}

fn padded(row: &[String], width: usize) -> Vec<String> {
    let mut out = row.to_vec();
    out.resize(width, String::new());
    out
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
    fn ragged_rows_convert_with_warning() {
        let csv = b"Name,,Email,ExtraColumn\nAlice,Manager,alice@example.com,junk\nBob,,bob@example.com,morejunk,too many columns\n";
        let (dir, path) = write_temp_csv(csv);
        let result = convert_csv(&path).unwrap();

        assert!(result.warning.is_some(), "ragged CSV should carry a warning");
        // Bob's row has 5 columns, so every row is padded to 5 cells.
        assert!(result.markdown.contains("| Name |  | Email | ExtraColumn |  |"));
        assert!(result.markdown.contains("| Alice | Manager | alice@example.com | junk |  |"));
        assert!(result.markdown.contains("| Bob |  | bob@example.com | morejunk | too many columns |"));
        drop(dir);
    }

    #[test]
    fn short_rows_are_padded() {
        let csv = b"a,b,c\n1\n";
        let (dir, path) = write_temp_csv(csv);
        let result = convert_csv(&path).unwrap();
        assert!(result.warning.is_some());
        assert!(result.markdown.contains("| 1 |  |  |"));
        drop(dir);
    }

    #[test]
    fn escapes_pipes_and_newlines_in_cells() {
        let csv = b"name,desc\nAlice,\"a|b\"\nBob,\"line1\nline2\"\n";
        let (dir, path) = write_temp_csv(csv);
        let result = convert_csv(&path).unwrap();

        assert!(result.markdown.contains("a\\|b"));
        assert!(result.markdown.contains("| line1 line2 |"));
        // Every line of the table must still be a single Markdown row.
        for line in result.markdown.lines() {
            assert!(line.starts_with('|') && line.ends_with('|'), "broken row: {line}");
        }
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
