use std::path::Path;

use crate::converters::ConversionResult;
use crate::models::AgentReadyError;

pub fn convert_pdf(path: &Path) -> Result<ConversionResult, AgentReadyError> {
    let pages = pdf_extract::extract_text_by_pages(path)
        .map_err(|_| AgentReadyError::PdfExtract)?;

    if pages.is_empty() {
        return Err(AgentReadyError::UserFacing(
            crate::models::ErrorCode::NoReadableText,
        ));
    }

    let mut markdown = String::new();
    let total_pages = pages.len();

    for (i, page) in pages.iter().enumerate() {
        let trimmed = page.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Add page separator between pages
        if i > 0 && !markdown.is_empty() {
            markdown.push_str("\n---\n\n");
        }

        // Add page header
        if total_pages > 1 {
            markdown.push_str(&format!("<!-- Page {} of {} -->\n\n", i + 1, total_pages));
        }

        markdown.push_str(trimmed);
        markdown.push('\n');
    }

    if markdown.trim().is_empty() {
        return Err(AgentReadyError::UserFacing(
            crate::models::ErrorCode::NoReadableText,
        ));
    }

    Ok(ConversionResult {
        markdown,
        warning: None,
        raw_data: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_nonexistent_file() {
        let result = convert_pdf(std::path::Path::new("/tmp/nonexistent-test-file.pdf"));
        assert!(result.is_err());
    }
}
