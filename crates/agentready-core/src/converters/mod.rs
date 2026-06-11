pub mod azw3;
pub mod csv;
pub mod docx;
pub mod epub;
pub mod kindle;
pub mod mobi;
pub mod markdown;
pub mod pdf;
pub mod plain_text_to_markdown;
pub mod pptx;
pub mod txt;
pub mod xlsx;

use std::fs::File;
use std::io::Read;
use std::path::Path;

use encoding_rs_io::DecodeReaderBytesBuilder;

use crate::models::AgentReadyError;

#[derive(Debug, PartialEq)]
pub struct ConversionResult {
    pub markdown: String,
    pub warning: Option<String>,
    pub raw_data: Option<Vec<u8>>,
}

/// Reads a text file, guessing the encoding and stripping BOM.
pub fn read_text_file(path: &Path) -> Result<String, AgentReadyError> {
    let file = File::open(path).map_err(AgentReadyError::Io)?;
    let mut decoder = DecodeReaderBytesBuilder::new()
        .bom_override(true)
        .build(file);

    let mut content = String::new();
    decoder
        .read_to_string(&mut content)
        .map_err(AgentReadyError::Io)?;

    if content.trim().is_empty() {
        return Err(AgentReadyError::UserFacing(
            crate::models::ErrorCode::NoReadableText,
        ));
    }

    Ok(content)
}
