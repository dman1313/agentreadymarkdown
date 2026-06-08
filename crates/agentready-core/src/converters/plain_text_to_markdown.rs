//! Heuristic plain text → Markdown for PDF page extraction (no layout/OCR).

/// Convert one page of extracted PDF text into Markdown blocks.
pub fn page_text_to_markdown(page: &str) -> String {
    let paragraphs = reflow_into_paragraphs(page);
    let mut out = String::new();
    let mut in_list = false;

    for para in paragraphs {
        let line = para.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(item) = bullet_list_item(line) {
            if !in_list && !out.is_empty() && !out.ends_with("\n\n") {
                out.push('\n');
            }
            out.push_str("- ");
            out.push_str(item);
            out.push('\n');
            in_list = true;
            continue;
        }

        if let Some(item) = numbered_list_item(line) {
            if !in_list && !out.is_empty() && !out.ends_with("\n\n") {
                out.push('\n');
            }
            out.push_str(&format!("1. {item}\n"));
            in_list = true;
            continue;
        }

        in_list = false;

        if let Some((level, title)) = heading_line(line) {
            if !out.is_empty() && !out.ends_with("\n\n") {
                out.push('\n');
            }
            out.push_str(&"#".repeat(level));
            out.push(' ');
            out.push_str(title);
            out.push_str("\n\n");
            continue;
        }

        if !out.is_empty() && !out.ends_with("\n\n") {
            out.push('\n');
        }
        out.push_str(line);
        out.push_str("\n\n");
    }

    out.trim_end().to_string()
}

/// Join multiple PDF pages with Markdown page headings when needed.
pub fn pdf_pages_to_markdown(pages: &[String]) -> String {
    let non_empty: Vec<(usize, &str)> = pages
        .iter()
        .enumerate()
        .filter_map(|(i, p)| {
            let t = p.trim();
            if t.is_empty() {
                None
            } else {
                Some((i, t))
            }
        })
        .collect();

    if non_empty.is_empty() {
        return String::new();
    }

    let total = non_empty.len();
    let mut out = String::new();

    for (idx, (page_idx, text)) in non_empty.iter().enumerate() {
        if total > 1 {
            if !out.is_empty() {
                out.push_str("\n\n---\n\n");
            }
            out.push_str(&format!("## Page {}\n\n", page_idx + 1));
        }
        out.push_str(&page_text_to_markdown(text));
        if idx + 1 < total {
            out.push_str("\n\n");
        }
    }

    out
}

fn reflow_into_paragraphs(page: &str) -> Vec<String> {
    let raw_lines: Vec<&str> = page.lines().map(str::trim).collect();
    if raw_lines.is_empty() {
        return Vec::new();
    }

    let mut paragraphs: Vec<String> = Vec::new();
    let mut current = String::new();

    let mut i = 0;
    while i < raw_lines.len() {
        let line = raw_lines[i];
        if line.is_empty() {
            if !current.is_empty() {
                paragraphs.push(current.trim().to_string());
                current.clear();
            }
            i += 1;
            continue;
        }

        if current.is_empty() {
            current.push_str(line);
        } else if should_merge_lines(&current, line) {
            if current.ends_with('-') {
                current.pop();
                current.push_str(line);
            } else {
                current.push(' ');
                current.push_str(line);
            }
        } else {
            paragraphs.push(current.trim().to_string());
            current = line.to_string();
        }
        i += 1;
    }

    if !current.is_empty() {
        paragraphs.push(current.trim().to_string());
    }

    paragraphs
}

fn should_merge_lines(prev: &str, next: &str) -> bool {
    if looks_like_block_start(next) || looks_like_block_start(prev) {
        return false;
    }

    if prev.ends_with('-') {
        return true;
    }

    let prev_ends_sentence = prev
        .chars()
        .rev()
        .find(|c| !c.is_whitespace())
        .is_some_and(|c| matches!(c, '.' | '!' | '?' | ':' | ';'));

    if prev_ends_sentence {
        return false;
    }

    next.chars()
        .next()
        .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

fn looks_like_block_start(line: &str) -> bool {
    bullet_list_item(line).is_some()
        || numbered_list_item(line).is_some()
        || heading_line(line).is_some()
}

fn bullet_list_item(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    for prefix in ["- ", "* ", "• ", "○ ", "▪ ", "▫ "] {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            let item = rest.trim();
            if !item.is_empty() {
                return Some(item);
            }
        }
    }
    None
}

fn numbered_list_item(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    let mut digits = 0usize;
    for (i, ch) in trimmed.char_indices() {
        if ch.is_ascii_digit() {
            digits += 1;
            continue;
        }
        if digits == 0 {
            return None;
        }
        if matches!(ch, '.' | ')') {
            let rest = trimmed[i + ch.len_utf8()..].trim_start();
            if !rest.is_empty() && !heading_number_prefix(trimmed, digits) {
                return Some(rest);
            }
        }
        break;
    }
    None
}

fn heading_number_prefix(line: &str, digits: usize) -> bool {
    // "1. Introduction" is a heading; "1. first item" in a list is handled elsewhere.
    if digits > 2 {
        return true;
    }
    if let Some(rest) = line.split_once('.') {
        let title = rest.1.trim();
        return title.len() > 3
            && title
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_uppercase());
    }
    false
}

fn heading_line(line: &str) -> Option<(usize, &str)> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.len() > 120 {
        return None;
    }

    let hashes = trimmed.chars().take_while(|&c| c == '#').count();
    if (1..=6).contains(&hashes) {
        let title = trimmed[hashes..].trim();
        if !title.is_empty() {
            return Some((hashes, title));
        }
    }

    let lower = trimmed.to_lowercase();
    for prefix in ["chapter ", "section ", "part ", "appendix "] {
        if lower.starts_with(prefix) {
            return Some((1, trimmed));
        }
    }

    if numbered_section_heading(trimmed) {
        return Some((2, trimmed));
    }

    if is_all_caps_heading(trimmed) {
        return Some((2, trimmed));
    }

    None
}

fn numbered_section_heading(line: &str) -> bool {
    let mut digit_run = 0usize;
    for (i, ch) in line.char_indices() {
        if ch.is_ascii_digit() {
            digit_run += 1;
            continue;
        }
        if digit_run == 0 {
            return false;
        }
        if matches!(ch, '.' | ')') {
            let rest = line[i + ch.len_utf8()..].trim();
            return rest.len() >= 3
                && rest.chars().next().is_some_and(|c| c.is_ascii_uppercase());
        }
        return false;
    }
    false
}

fn is_all_caps_heading(line: &str) -> bool {
    let letters: Vec<char> = line.chars().filter(|c| c.is_alphabetic()).collect();
    if letters.len() < 3 || letters.len() > 80 {
        return false;
    }
    if !letters.iter().all(|c| c.is_uppercase()) {
        return false;
    }
    !line.ends_with('.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reflows_soft_wrapped_lines() {
        let md = page_text_to_markdown("Hello PDF reader.\nAgentReady smoke test.");
        assert!(md.contains("Hello PDF reader."));
        assert!(md.contains("AgentReady smoke test."));
        assert!(md.contains("\n\n"));
    }

    #[test]
    fn all_caps_becomes_heading() {
        let md = page_text_to_markdown("INTRODUCTION\n\nBody text here.");
        assert!(md.contains("## INTRODUCTION"));
        assert!(md.contains("Body text here."));
    }

    #[test]
    fn bullet_list_markdown() {
        let md = page_text_to_markdown("- first item\n- second item");
        assert!(md.contains("- first item"));
        assert!(md.contains("- second item"));
    }

    #[test]
    fn multi_page_uses_page_headings() {
        let pages = vec!["Page one text.".into(), "Page two text.".into()];
        let md = pdf_pages_to_markdown(&pages);
        assert!(md.contains("## Page 1"));
        assert!(md.contains("## Page 2"));
        assert!(md.contains("---"));
    }

    #[test]
    fn single_page_no_page_heading() {
        let md = pdf_pages_to_markdown(&["Just one page.".into()]);
        assert!(!md.contains("## Page"));
    }
}
