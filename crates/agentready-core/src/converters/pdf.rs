use std::path::Path;
use pdf_extract;

use crate::converters::ConversionResult;
use crate::models::ErrorCode;

pub fn convert_pdf(path: &Path) -> Result<ConversionResult, ErrorCode> {
    // pdf_extract reads the file and extracts text.
    let out = pdf_extract::extract_text(path).map_err(|_| ErrorCode::ConversionFailed)?;

    let trimmed = out.trim().to_string();
    if trimmed.is_empty() {
        return Err(ErrorCode::NoReadableText);
    }

    Ok(ConversionResult {
        markdown: trimmed,
        warning: None,
        raw_data: None,
    })
}
