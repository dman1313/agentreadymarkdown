// Kindle PDB formats (MOBI / AZW / AZW3-KF8) via `mobi` crate (MIT).
use std::path::Path;

use mobi::headers::Encryption;
use mobi::Mobi;

// EXTH record positions — https://wiki.mobileread.com/wiki/MOBI#EXTH_Header
const EXTH_DRM_SERVER_ID: u32 = 1;
const EXTH_DRM_COMMERCE_ID: u32 = 2;
const EXTH_DRM_EBOOKBASE_BOOK_ID: u32 = 3;
const EXTH_KF8_BOUNDARY_OFFSET: u32 = 121;

use crate::converters::ConversionResult;
use crate::models::{AgentReadyError, ErrorCode};
use crate::text_quality;

const MAX_KINDLE_TEXT_BYTES: usize = 50 * 1024 * 1024;

#[derive(Debug, Clone, Copy)]
pub enum KindleFormat {
    Mobi,
    Azw,
    Azw3,
}

impl KindleFormat {
    fn quality_warning(&self) -> &'static str {
        match self {
            KindleFormat::Mobi => "Some MOBI text may not have converted cleanly. Please review the output.",
            KindleFormat::Azw | KindleFormat::Azw3 => {
                "Some AZW/AZW3 text may not have converted cleanly. Please review the output."
            }
        }
    }
}

pub fn convert_kindle(path: &Path, format: KindleFormat) -> Result<ConversionResult, AgentReadyError> {
    let book =
        Mobi::from_path(path).map_err(|_| AgentReadyError::UserFacing(ErrorCode::ConversionFailed))?;

    ensure_drm_free(&book)?;

    let title = book.title();
    let body = extract_body(&book)?;
    let markdown = build_markdown(&title, &body);

    if markdown.trim().is_empty() {
        return Err(AgentReadyError::UserFacing(ErrorCode::NoReadableText));
    }

    if markdown.len() > MAX_KINDLE_TEXT_BYTES {
        return Err(AgentReadyError::UserFacing(ErrorCode::FileTooLarge));
    }

    if text_quality::looks_like_garbage(&markdown) {
        return Err(AgentReadyError::UserFacing(ErrorCode::NoReadableText));
    }

    let mut warning = None;
    if text_quality::readable_text_ratio(&markdown) < 0.75 {
        warning = Some(format.quality_warning().into());
    }

    Ok(ConversionResult {
        markdown,
        warning,
        raw_data: None,
    })
}

fn ensure_drm_free(book: &Mobi) -> Result<(), AgentReadyError> {
    if book.encryption() != Encryption::No {
        return Err(AgentReadyError::UserFacing(ErrorCode::PasswordProtected));
    }

    if has_drm_exth(book) {
        return Err(AgentReadyError::UserFacing(ErrorCode::PasswordProtected));
    }

    Ok(())
}

fn has_drm_exth(book: &Mobi) -> bool {
    for position in [
        EXTH_DRM_SERVER_ID,
        EXTH_DRM_COMMERCE_ID,
        EXTH_DRM_EBOOKBASE_BOOK_ID,
    ] {
        if let Some(values) = book.metadata.exth_record_at(position)
            && values.iter().any(|v| record_has_drm_value(v)) {
                return true;
            }
    }
    false
}

fn record_has_drm_value(bytes: &[u8]) -> bool {
    !bytes.is_empty() && bytes.iter().any(|b| *b != 0)
}

fn extract_body(book: &Mobi) -> Result<String, AgentReadyError> {
    if let Some(kf8) = extract_kf8_text(book)
        && !kf8.trim().is_empty() {
            return Ok(kf8);
        }

    Ok(book
        .content_as_string()
        .unwrap_or_else(|_| book.content_as_string_lossy()))
}

fn extract_kf8_text(book: &Mobi) -> Option<String> {
    let offset = kf8_boundary_offset(book)?;
    if offset >= book.content.len() {
        return None;
    }

    let slice = book.content[offset..].to_vec();
    let kf8 = Mobi::new(slice).ok()?;
    if kf8.encryption() != Encryption::No || has_drm_exth(&kf8) {
        return None;
    }

    kf8.content_as_string()
        .ok()
        .or_else(|| Some(kf8.content_as_string_lossy()))
}

fn kf8_boundary_offset(book: &Mobi) -> Option<usize> {
    let record = book.metadata.exth_record_at(EXTH_KF8_BOUNDARY_OFFSET)?;
    let bytes = record.first()?;
    if bytes.len() < 4 {
        return None;
    }
    let offset = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    if offset == 0 {
        None
    } else {
        Some(offset)
    }
}

pub(crate) fn build_markdown(title: &str, body: &str) -> String {
    let text = normalize_kindle_body(body);
    let mut out = String::new();

    let title_trim = title.trim();
    if !title_trim.is_empty() && !title_trim.eq_ignore_ascii_case("unknown") {
        out.push_str("# ");
        out.push_str(title_trim);
        out.push_str("\n\n");
    }

    out.push_str(&text);
    out
}

fn normalize_kindle_body(body: &str) -> String {
    let trimmed = body.trim();
    if trimmed.contains('<') && trimmed.contains('>') {
        strip_basic_html(trimmed)
    } else {
        collapse_blank_lines(trimmed)
    }
}

fn strip_basic_html(html: &str) -> String {
    let mut work = html.to_string();
    for tag in &["script", "style"] {
        work = strip_tag_block(&work, tag);
    }
    work = work
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
        .replace("</p>", "\n\n")
        .replace("</div>", "\n\n");

    let mut out = String::new();
    let mut in_tag = false;
    for ch in work.chars() {
        match ch {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    collapse_blank_lines(&out)
}

fn strip_tag_block(html: &str, tag: &str) -> String {
    let mut out = html.to_string();
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    while let Some(start) = out.to_lowercase().find(&open) {
        let Some(end_rel) = out[start..].to_lowercase().find(&close) else {
            break;
        };
        let end = start + end_rel + close.len();
        out.replace_range(start..end, "");
    }
    out
}

fn collapse_blank_lines(text: &str) -> String {
    let mut lines: Vec<&str> = Vec::new();
    let mut prev_blank = false;
    for line in text.lines() {
        let trimmed = line.trim();
        let blank = trimmed.is_empty();
        if blank {
            if !prev_blank {
                lines.push("");
            }
        } else {
            lines.push(trimmed);
        }
        prev_blank = blank;
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_strips_simple_html() {
        let md = normalize_kindle_body("<p>Hello <b>world</b>.</p>");
        assert!(md.contains("Hello"));
        assert!(!md.contains('<'));
    }

    #[test]
    fn build_markdown_adds_title() {
        let md = build_markdown("My Book", "Chapter one.");
        assert!(md.starts_with("# My Book"));
    }

    #[test]
    fn kf8_boundary_parses_be_u32() {
        let bytes = [0u8, 0, 4, 210]; // 1234 as big-endian u32
        let offset = u32::from_be_bytes(bytes) as usize;
        assert_eq!(offset, 1234);
    }
}
