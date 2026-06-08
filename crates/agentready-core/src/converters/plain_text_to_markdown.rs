//! Heuristic plain text → Markdown for PDF page extraction (no layout/OCR).

/// Convert one page of extracted PDF text into Markdown blocks.
pub fn page_text_to_markdown(page: &str) -> String {
    page_text_to_markdown_inner(page)
}

fn page_text_to_markdown_inner(page: &str) -> String {
    let paragraphs = reflow_into_paragraphs(page);
    let mut out = String::new();
    let mut in_list = false;
    let mut roster_section = false;

    for para in paragraphs {
        let raw = para.trim();
        if raw.is_empty() {
            continue;
        }

        if is_roman_page_marker(raw) || is_plain_page_marker(raw) {
            continue;
        }

        if let Some(entry) = toc_entry(raw) {
            if !in_list && !out.is_empty() && !out.ends_with("\n\n") {
                out.push('\n');
            }
            out.push_str("- ");
            out.push_str(&entry);
            out.push('\n');
            in_list = true;
            continue;
        }

        let line = fix_pdf_word_breaks(raw);

        if let Some(item) = bullet_list_item(&line) {
            if !in_list && !out.is_empty() && !out.ends_with("\n\n") {
                out.push('\n');
            }
            out.push_str("- ");
            out.push_str(item);
            out.push('\n');
            in_list = true;
            continue;
        }

        if let Some(item) = numbered_list_item(&line) {
            if !in_list && !out.is_empty() && !out.ends_with("\n\n") {
                out.push('\n');
            }
            out.push_str(&format!("1. {item}\n"));
            in_list = true;
            continue;
        }

        in_list = false;

        if let Some((level, title)) = heading_line(&line) {
            if !out.is_empty() && !out.ends_with("\n\n") {
                out.push('\n');
            }
            let lower = title.to_lowercase();
            roster_section = matches!(lower.as_str(), "panel" | "staff");
            out.push_str(&"#".repeat(level));
            out.push(' ');
            out.push_str(title);
            out.push_str("\n\n");
            continue;
        }

        if roster_section && is_roster_line(&line) {
            if !in_list && !out.is_empty() && !out.ends_with("\n\n") {
                out.push('\n');
            }
            out.push_str("- ");
            out.push_str(&line);
            out.push('\n');
            in_list = true;
            continue;
        }

        roster_section = false;

        if let Some(subtitle) = subtitle_line(&line) {
            if !out.is_empty() && !out.ends_with("\n\n") {
                out.push('\n');
            }
            out.push_str("## ");
            out.push_str(subtitle);
            out.push_str("\n\n");
            continue;
        }

        if !out.is_empty() && !out.ends_with("\n\n") {
            out.push('\n');
        }
        out.push_str(&line);
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
        if total > 1 && idx > 0 {
            out.push_str("\n\n---\n\n");
            out.push_str(&format!("<!-- Page {} -->\n\n", page_idx + 1));
        }
        let page_md = if *page_idx == 0 {
            let body = page_text_to_markdown_inner(text);
            match document_title_prefix(text) {
                Some(title) => {
                    let body = strip_duplicate_title_lines(&body, &title);
                    format!("# {title}\n\n{body}")
                }
                None => body,
            }
        } else {
            page_text_to_markdown_inner(text)
        };
        out.push_str(&page_md);
    }

    dedupe_consecutive_paragraphs(&out)
}

fn document_title_prefix(page: &str) -> Option<String> {
    let lines: Vec<&str> = page
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .take(8)
        .collect();
    if lines.is_empty() {
        return None;
    }
    let first = lines[0];
    if first.len() < 12 || first.len() > 120 || first.ends_with('.') {
        return None;
    }
    let first_lower = first.to_lowercase();
    let repeats = lines
        .iter()
        .filter(|l| l.to_lowercase() == first_lower || l.contains(first))
        .count();
    if repeats >= 2 || (lines.len() <= 4 && title_case_ratio(first) > 0.5) {
        Some(fix_pdf_word_breaks(first))
    } else {
        None
    }
}

fn title_case_ratio(line: &str) -> f64 {
    let words: Vec<&str> = line.split_whitespace().collect();
    if words.is_empty() {
        return 0.0;
    }
    let titled = words
        .iter()
        .filter(|w| {
            w.chars()
                .next()
                .is_some_and(|c| c.is_ascii_uppercase())
        })
        .count();
    titled as f64 / words.len() as f64
}

fn dedupe_consecutive_paragraphs(md: &str) -> String {
    let mut blocks: Vec<String> = Vec::new();
    for block in md.split("\n\n") {
        let trimmed = block.trim();
        if trimmed.is_empty() {
            continue;
        }
        if blocks.last().is_some_and(|prev| prev == trimmed) {
            continue;
        }
        blocks.push(trimmed.to_string());
    }
    blocks.join("\n\n")
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

    const H2_LABELS: &[&str] = &[
        "contents",
        "table of contents",
        "panel",
        "staff",
        "disclaimer",
        "overview",
        "introduction",
        "summary of the recommendations",
        "ies practice guide",
    ];
    if H2_LABELS.iter().any(|label| lower == *label) {
        return Some((2, trimmed));
    }

    if numbered_section_heading(trimmed) {
        return Some((2, trimmed));
    }

    if is_all_caps_heading(trimmed) {
        return Some((2, trimmed));
    }

    None
}

fn fix_pdf_word_breaks(s: &str) -> String {
    let words: Vec<&str> = s.split_whitespace().collect();
    let mut out: Vec<String> = Vec::new();
    for word in words {
        if let Some(last) = out.last_mut() {
            if should_merge_pdf_tokens(last, word) {
                last.push_str(word);
                continue;
            }
        }
        out.push(word.to_string());
    }
    out.join(" ")
}

fn should_merge_pdf_tokens(prev: &str, next: &str) -> bool {
    if !(2..=3).contains(&prev.len()) {
        return false;
    }
    if !prev.chars().all(|c| c.is_ascii_alphabetic()) {
        return false;
    }
    // Keep acronyms separate: "PDF reader" not "PDFreader".
    if prev.chars().all(|c| c.is_ascii_uppercase()) {
        return false;
    }
    const STOP_WORDS: &[&str] = &["and", "for", "the", "with", "from", "not"];
    if STOP_WORDS.contains(&prev) {
        return false;
    }
    if !next
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_lowercase())
    {
        return false;
    }
    // Typical PDF break: "Re" + "gional" or "edu" + "cation".
    prev.chars().next().is_some_and(|c| c.is_ascii_uppercase())
        || prev.chars().all(|c| c.is_ascii_lowercase())
}

fn is_plain_page_marker(line: &str) -> bool {
    let lower = line.trim().to_lowercase();
    lower.strip_prefix("page ")
        .and_then(|n| n.trim().parse::<u32>().ok())
        .is_some()
}

fn is_roster_line(line: &str) -> bool {
    let t = line.trim();
    if t.len() < 6 || t.len() > 100 {
        return false;
    }
    if t.ends_with('.') || t.starts_with('#') {
        return false;
    }
    t.chars()
        .next()
        .is_some_and(|c| c.is_ascii_uppercase())
}

fn subtitle_line(line: &str) -> Option<&str> {
    let t = line.trim();
    if t.len() < 8 || t.len() > 100 || t.ends_with('.') {
        return None;
    }
    if t.starts_with('(') {
        Some(t)
    } else {
        None
    }
}

fn strip_duplicate_title_lines(body: &str, title: &str) -> String {
    let title_lower = title.to_lowercase();
    body.split("\n\n")
        .filter(|block| {
            let b = block.trim();
            if b.is_empty() {
                return false;
            }
            let b_lower = b.to_lowercase();
            b_lower != title_lower && !title_lower.contains(&b_lower) && !b_lower.contains(&title_lower)
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn is_roman_page_marker(line: &str) -> bool {
    let t = line.trim();
    let inner = t
        .strip_prefix('(')
        .and_then(|s| s.strip_suffix(')'))
        .map(str::trim);
    let Some(inner) = inner else {
        return false;
    };
    !inner.is_empty()
        && inner
            .chars()
            .all(|c| matches!(c, 'i' | 'v' | 'x' | 'l' | 'c' | 'd' | 'm' | 'I' | 'V' | 'X' | 'L' | 'C' | 'D' | 'M'))
}

fn toc_entry(line: &str) -> Option<String> {
    let trimmed = line.trim();
    // Table-of-contents rows use a wide gap before the page number.
    let (title, page) = trimmed.rsplit_once("  ")?;
    let title = title.trim();
    let page = page.trim();
    if title.len() < 3 || !page.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    if title.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(format!("{title} (p. {page})"))
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
    fn multi_page_uses_page_comments() {
        let pages = vec!["Page one text.".into(), "Page two text.".into()];
        let md = pdf_pages_to_markdown(&pages);
        assert!(!md.contains("## Page 1"));
        assert!(md.contains("<!-- Page 2 -->"));
        assert!(md.contains("---"));
    }

    #[test]
    fn single_page_no_page_heading() {
        let md = pdf_pages_to_markdown(&["Just one page.".into()]);
        assert!(!md.contains("<!-- Page"));
    }

    #[test]
    fn fixes_pdf_word_breaks() {
        let md = page_text_to_markdown("Re gional Assistance and edu cation policy.");
        assert!(md.contains("Regional"));
        assert!(md.contains("education"));
    }

    #[test]
    fn contents_and_toc_become_markdown() {
        let page = "Contents\n\nIntroduction  1\n\nOverview  4";
        let md = page_text_to_markdown(page);
        assert!(md.contains("## Contents"));
        assert!(md.contains("- Introduction (p. 1)"));
        assert!(md.contains("- Overview (p. 4)"));
    }

    #[test]
    fn ies_style_labels_are_headings() {
        let md = page_text_to_markdown("IES PRACTICE GUIDE\n\nPanel\n\nRussell Gersten (Chair)");
        assert!(md.contains("## IES PRACTICE GUIDE"));
        assert!(md.contains("## Panel"));
        assert!(md.contains("- Russell Gersten (Chair)"));
    }

    #[test]
    fn skips_plain_page_markers() {
        let md = page_text_to_markdown("Page 2\n\nReal paragraph.");
        assert!(!md.contains("Page 2"));
        assert!(md.contains("Real paragraph"));
    }
}
