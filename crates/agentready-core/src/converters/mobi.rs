// MOBI conversion via `mobi` crate (MIT, https://github.com/vv9k/mobi-rs).
use std::path::Path;

use mobi::headers::Encryption;
use mobi::Mobi;

use crate::converters::ConversionResult;
use crate::models::{AgentReadyError, ErrorCode};
use crate::text_quality;

const MAX_MOBI_TEXT_BYTES: usize = 50 * 1024 * 1024;

pub fn convert_mobi(path: &Path) -> Result<ConversionResult, AgentReadyError> {
    let book = Mobi::from_path(path).map_err(|_| AgentReadyError::UserFacing(ErrorCode::ConversionFailed))?;

    if book.encryption() != Encryption::No {
        return Err(AgentReadyError::UserFacing(ErrorCode::PasswordProtected));
    }

    let title = book.title();
    let body = book
        .content_as_string()
        .unwrap_or_else(|_| book.content_as_string_lossy());

    let markdown = build_markdown(&title, &body);

    if markdown.trim().is_empty() {
        return Err(AgentReadyError::UserFacing(ErrorCode::NoReadableText));
    }

    if markdown.len() > MAX_MOBI_TEXT_BYTES {
        return Err(AgentReadyError::UserFacing(ErrorCode::FileTooLarge));
    }

    if text_quality::looks_like_garbage(&markdown) {
        return Err(AgentReadyError::UserFacing(ErrorCode::NoReadableText));
    }

    let mut warning = None;
    if text_quality::readable_text_ratio(&markdown) < 0.75 {
        warning = Some(
            "Some MOBI text may not have converted cleanly. Please review the output.".into(),
        );
    }

    Ok(ConversionResult {
        markdown,
        warning,
        raw_data: None,
    })
}

fn build_markdown(title: &str, body: &str) -> String {
    let text = normalize_mobi_body(body);
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

fn normalize_mobi_body(body: &str) -> String {
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
    fn rejects_nonexistent_file() {
        let result = convert_mobi(Path::new("/tmp/nonexistent-agentready.mobi"));
        assert!(result.is_err());
    }

    #[test]
    fn rejects_invalid_mobi_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.mobi");
        std::fs::write(&path, b"not a mobi file").unwrap();
        assert!(convert_mobi(&path).is_err());
    }

    #[test]
    fn normalize_strips_simple_html() {
        let md = normalize_mobi_body("<p>Hello <b>world</b>.</p>");
        assert!(md.contains("Hello"));
        assert!(md.contains("world"));
        assert!(!md.contains('<'));
    }

    #[test]
    fn build_markdown_adds_title() {
        let md = build_markdown("My Book", "Chapter one.");
        assert!(md.starts_with("# My Book"));
        assert!(md.contains("Chapter one."));
    }
}
