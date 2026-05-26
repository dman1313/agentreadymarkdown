use std::fs;
use std::path::Path;

use csv::ReaderBuilder;

use crate::converters::ConversionResult;
use crate::models::ErrorCode;

pub fn convert_csv(path: &Path) -> Result<ConversionResult, ErrorCode> {
    // Read the raw file to get both the markdown and the raw bytes for the export
    let raw_bytes = fs::read(path).map_err(|_| ErrorCode::ConversionFailed)?;

    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)
        .map_err(|_| ErrorCode::ConversionFailed)?;

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
            return Err(ErrorCode::ConversionFailed);
        }
    }

    if markdown.trim().is_empty() {
        return Err(ErrorCode::NoReadableText);
    }

    Ok(ConversionResult {
        markdown,
        warning: None,
        raw_data: Some(raw_bytes),
    })
}
