use std::path::Path;

use crate::converters::{read_text_file, ConversionResult};
use crate::models::ErrorCode;

pub fn convert_markdown(path: &Path) -> Result<ConversionResult, ErrorCode> {
    let content = read_text_file(path)?;

    Ok(ConversionResult {
        markdown: content,
        warning: None,
        raw_data: None,
    })
}
