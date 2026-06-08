use std::path::Path;

use crate::converters::plain_text_to_markdown::pdf_pages_to_markdown;
use crate::converters::ConversionResult;
use crate::models::{AgentReadyError, ErrorCode};
use crate::text_quality;

pub fn convert_pdf(path: &Path) -> Result<ConversionResult, AgentReadyError> {
    let pages = pdf_extract::extract_text_by_pages(path)
        .map_err(|_| AgentReadyError::PdfExtract)?;

    if pages.is_empty() {
        return Err(AgentReadyError::UserFacing(ErrorCode::NoReadableText));
    }

    let markdown = pdf_pages_to_markdown(&pages);

    if markdown.trim().is_empty() {
        return Err(AgentReadyError::UserFacing(ErrorCode::NoReadableText));
    }

    if text_quality::looks_like_garbage(&markdown) {
        return Err(AgentReadyError::UserFacing(ErrorCode::NoReadableText));
    }

    let warning = if text_quality::readable_text_ratio(&markdown) < 0.75 {
        Some(
            "Some PDF text may not have extracted cleanly. Please review the output.".into(),
        )
    } else {
        None
    };

    Ok(ConversionResult {
        markdown,
        warning,
        raw_data: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::converters::plain_text_to_markdown::page_text_to_markdown;

    #[test]
    fn rejects_nonexistent_file() {
        let result = convert_pdf(std::path::Path::new("/tmp/nonexistent-test-file.pdf"));
        assert!(result.is_err());
    }

    #[test]
    fn structures_extracted_lines_as_paragraphs() {
        let md = page_text_to_markdown("Hello PDF reader.\n\nAgentReady text extraction smoke test.");
        assert!(md.contains("Hello PDF reader."));
        assert!(md.contains("AgentReady text extraction smoke test."));
        assert!(md.contains("\n\n"));
    }
}
