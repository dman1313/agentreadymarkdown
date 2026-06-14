use std::fs;
use std::path::Path;

use crate::converters::ConversionResult;
use crate::models::{AgentReadyError, ErrorCode};

/// Convert an HTML file to agent-ready Markdown.
///
/// Text-only extraction that strips tags and recovers basic structure:
/// - `<h1>`..`<h6>` → `#`..`######`
/// - `<p>`, `<div>`, `<section>` → paragraph breaks (double newline)
/// - `<br>` → single newline
/// - `<li>` → `- ` (unordered) or `1. ` (ordered, via `<ol>` tracking)
/// - `<strong>`, `<b>` → `**`
/// - `<em>`, `<i>` → `*`
/// - `<a href="...">text</a>` → `[text](...)`
/// - `<code>` → backticks
/// - `<pre>` → fenced code block
/// - `<blockquote>` → `> ` prefix
/// - `<hr>` → `---`
/// - HTML entities decoded by quick-xml
/// - `<script>`, `<style>`, `<head>` contents silently dropped
/// - All other tags: text preserved, tag stripped
pub fn convert_html(path: &Path) -> Result<ConversionResult, AgentReadyError> {
    let bytes = fs::read(path).map_err(AgentReadyError::Io)?;

    let text = match std::str::from_utf8(&bytes) {
        Ok(s) => s.to_string(),
        Err(_) => {
            // Fall back to lossy decode
            let (cow, _, _) = encoding_rs::UTF_8.decode(&bytes);
            cow.into_owned()
        }
    };

    let markdown = extract_text(&text);

    if markdown.trim().is_empty() {
        return Err(AgentReadyError::UserFacing(ErrorCode::NoReadableText));
    }

    Ok(ConversionResult {
        markdown,
        warning: None,
        raw_data: None,
    })
}

/// Tags whose entire content (including children) should be dropped.
const SKIP_TAGS: &[&str] = &["script", "style", "head", "noscript", "template"];

/// Block-level tags that get their own paragraph break.
const BLOCK_TAGS: &[&str] = &[
    "p", "div", "section", "article", "main", "header", "footer", "nav", "aside",
    "h1", "h2", "h3", "h4", "h5", "h6",
    "ul", "ol", "li", "table",
    "blockquote", "pre", "figure", "figcaption", "details", "summary",
    "form", "fieldset",
];

fn is_skip_tag(tag: &str) -> bool {
    SKIP_TAGS.contains(&tag)
}

fn is_block_tag(tag: &str) -> bool {
    BLOCK_TAGS.contains(&tag)
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ListKind {
    Unordered,
    Ordered,
}

#[derive(Debug, Default)]
struct State {
    out: String,
    /// Stack of tag names for matching open/close. Used to track skip zones
    /// and list nesting.
    tag_stack: Vec<String>,
    /// Track whether we're inside a `<pre>` block (preserve whitespace).
    in_pre: usize,
    /// Track list context: ordered vs unordered + counter for ordered.
    list_stack: Vec<ListKind>,
    /// Track ordered list item counters.
    ol_counters: Vec<usize>,
    /// Whether the next text/raw emission should be preceded by a single space.
    /// Reset whenever we emit a separator (newline, space, block break, marker
    /// that already provides spacing).
    needs_space: bool,
    /// Inline formatting stack: bold, italic, code.
    bold_depth: usize,
    italic_depth: usize,
    code_depth: usize,
    /// Current blockquote depth.
    bq_depth: usize,
    /// Track anchor href for link rendering.
    pending_href: Option<String>,
    /// Table nesting depth. While > 0, cells/rows render as GFM pipes and
    /// block breaks are suppressed so rows stay on one line.
    in_table: usize,
    /// Index of the current row within the active table (0 = header row).
    table_row_index: usize,
    /// Cells seen in the current row (drives the header separator width).
    table_cell_count: usize,
}

impl State {
    fn is_in_skip(&self) -> bool {
        self.tag_stack.iter().any(|t| is_skip_tag(t))
    }

    /// Emit a space if the next emission would otherwise run into existing text.
    /// Returns true if a space was inserted.
    fn ensure_space(&mut self) {
        if self.needs_space
            && !self.out.is_empty()
            && !self.out.ends_with(' ')
            && !self.out.ends_with('\n')
        {
            self.out.push(' ');
        }
        self.needs_space = false;
    }

    fn emit(&mut self, text: &str) {
        if self.is_in_skip() {
            return;
        }
        if self.in_pre > 0 {
            self.out.push_str(text);
            self.needs_space = false;
            return;
        }
        let trimmed = text.trim();
        if trimmed.is_empty() {
            // Pure whitespace — capture the intent for the next emit.
            let had_leading = text.trim_start() != text;
            if had_leading {
                self.needs_space = true;
            }
            return;
        }
        // Separate from prior output with a single space when this chunk has
        // leading whitespace OR a previous chunk left a pending space (e.g. a
        // trailing space before an href-less anchor or a <span>).
        let had_leading = text.len() - text.trim_start().len() > 0;
        if (had_leading || self.needs_space)
            && !self.out.is_empty()
            && !self.out.ends_with(' ')
            && !self.out.ends_with('\n')
        {
            self.out.push(' ');
        }
        self.needs_space = false;
        // Blockquote prefix on fresh lines.
        if self.bq_depth > 0
            && (self.out.is_empty() || self.out.ends_with('\n'))
        {
            for _ in 0..self.bq_depth {
                self.out.push_str("> ");
            }
        }
        // Outside <pre>, HTML collapses internal whitespace (spaces, tabs,
        // source newlines) to a single space — otherwise wrapped source lines
        // leak hard breaks or stray indentation into the Markdown.
        let collapsed = trimmed.split_whitespace().collect::<Vec<_>>().join(" ");
        self.out.push_str(&collapsed);
        // Trailing whitespace in the chunk becomes a pending separator.
        let had_trailing = text.len() - text.trim_end().len() > 0;
        self.needs_space = had_trailing;
    }

    fn emit_raw(&mut self, text: &str) {
        if self.is_in_skip() {
            return;
        }
        // For raw marker characters, no leading-space insertion — callers
        // control spacing via ensure_space() and needs_space before/after.
        self.out.push_str(text);
        self.needs_space = false;
    }

    fn block_break(&mut self) {
        if self.is_in_skip() {
            return;
        }
        // Inside a table, rows manage their own newlines; a paragraph break
        // would split a single row across multiple lines.
        if self.in_table > 0 {
            return;
        }
        if !self.out.is_empty() && !self.out.ends_with("\n\n") {
            if self.out.ends_with('\n') {
                self.out.push('\n');
            } else {
                self.out.push_str("\n\n");
            }
        }
        self.needs_space = false;
    }
}

fn extract_text(html: &str) -> String {
    let mut reader = quick_xml::Reader::from_str(html);
    reader.config_mut().allow_unmatched_ends = true;

    let mut state = State::default();
    let mut buf = Vec::new();

    loop {
        use quick_xml::events::Event;

        buf.clear();
        let event = reader.read_event_into(&mut buf);
        match event {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_lowercase();
                handle_open_tag(&mut state, &tag, e.attributes());
            }
            Ok(Event::Empty(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_lowercase();
                handle_self_closing(&mut state, &tag);
            }
            Ok(Event::End(ref _e)) => {
                // Pop the tag stack to find the matching open tag.
                if let Some(closed_tag) = state.tag_stack.pop() {
                    handle_close_tag(&mut state, &closed_tag);
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.decode().unwrap_or_default();
                state.emit(text.as_ref());
            }
            Ok(Event::CData(ref e)) => {
                let text = String::from_utf8_lossy(e.as_ref());
                state.emit(&text);
            }
            Ok(Event::Eof) => break,
            Err(_) => break, // Malformed HTML — stop gracefully
            _ => {}
        }
    }

    // Clean up trailing whitespace.
    state.out.trim_end().to_string()
}

/// Spacing before a `<ul>`/`<ol>`: a paragraph break for a top-level list, or
/// just a fresh line for a nested list so it sits under its parent item.
fn open_list_spacing(state: &mut State) {
    if state.list_stack.is_empty() {
        state.block_break();
    } else if !state.out.is_empty() && !state.out.ends_with('\n') {
        state.emit_raw("\n");
    }
}

fn handle_open_tag(state: &mut State, tag: &str, attrs: quick_xml::events::attributes::Attributes) {
    state.tag_stack.push(tag.to_string());

    if is_skip_tag(tag) {
        return;
    }

    // List items share one block. ul/ol manage their own spacing (so nested
    // lists indent instead of starting a new paragraph).
    if is_block_tag(tag) && tag != "li" && tag != "ul" && tag != "ol" {
        state.block_break();
    }

    match tag {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            let level = tag.as_bytes()[1] - b'0';
            state.emit_raw(&format!("{} ", "#".repeat(level as usize)));
        }
        "br" => {
            // <br> in HTML is often not self-closed — handle in both places.
            state.emit_raw("\n");
        }
        "hr" => {
            // <hr> same situation. Emit the separator and consume the close.
            state.block_break();
            state.emit_raw("---");
            state.block_break();
        }
        "strong" | "b" => {
            state.bold_depth += 1;
            if state.bold_depth == 1 {
                state.ensure_space();
                state.emit_raw("**");
            }
        }
        "em" | "i" => {
            state.italic_depth += 1;
            if state.italic_depth == 1 {
                state.ensure_space();
                state.emit_raw("*");
            }
        }
        "code" => {
            if state.in_pre == 0 {
                state.code_depth += 1;
                if state.code_depth == 1 {
                    state.ensure_space();
                    state.emit_raw("`");
                }
            }
        }
        "pre" => {
            state.in_pre += 1;
            state.emit_raw("```\n");
        }
        "blockquote" => {
            state.bq_depth += 1;
        }
        "a" => {
            // Only open a markdown link when there is a usable href; otherwise
            // anchors like `<a name="x">` would leave a dangling `[`.
            let mut href = None;
            for attr in attrs.flatten() {
                if attr.key.as_ref() == b"href" {
                    let value = String::from_utf8_lossy(&attr.value).trim().to_string();
                    if !value.is_empty() {
                        href = Some(value);
                    }
                    break;
                }
            }
            if let Some(href) = href {
                state.ensure_space();
                state.emit_raw("[");
                state.pending_href = Some(href);
            }
        }
        "ul" => {
            open_list_spacing(state);
            state.list_stack.push(ListKind::Unordered);
        }
        "ol" => {
            open_list_spacing(state);
            state.list_stack.push(ListKind::Ordered);
            state.ol_counters.push(1);
        }
        "li" => {
            // Indent nested items two spaces per level beyond the first.
            let depth = state.list_stack.len();
            if depth > 1 {
                state.emit_raw(&"  ".repeat(depth - 1));
            }
            // Emit list marker.
            if let Some(kind) = state.list_stack.last() {
                match kind {
                    ListKind::Unordered => state.emit_raw("- "),
                    ListKind::Ordered => {
                        if let Some(counter) = state.ol_counters.last_mut() {
                            let n = *counter;
                            *counter += 1;
                            state.emit_raw(&format!("{}. ", n));
                        }
                    }
                }
            }
        }
        "table" => {
            state.in_table += 1;
            state.table_row_index = 0;
        }
        "tr" => {
            if state.in_table > 0 {
                if !state.out.is_empty() && !state.out.ends_with('\n') {
                    state.out.push('\n');
                }
                state.table_cell_count = 0;
                state.needs_space = false;
            }
        }
        "td" | "th" => {
            if state.in_table > 0 {
                state.emit_raw("| ");
                state.table_cell_count += 1;
            }
        }
        "img" => {
            // Self-closing in practice, but handle here too.
            let mut alt = String::from("image");
            let mut src = String::new();
            for attr in attrs.flatten() {
                match attr.key.as_ref() {
                    b"alt" => alt = String::from_utf8_lossy(&attr.value).to_string(),
                    b"src" => src = String::from_utf8_lossy(&attr.value).to_string(),
                    _ => {}
                }
            }
            if !src.is_empty() {
                state.emit(&format!("![{}]({})", alt, src));
            }
        }
        _ => {}
    }
}

fn handle_self_closing(state: &mut State, tag: &str) {
    if state.is_in_skip() {
        return;
    }
    match tag {
        "br" => {
            state.emit_raw("\n");
        }
        "hr" => {
            state.block_break();
            state.emit_raw("---");
            state.block_break();
        }
        "img" => {
            // Handled in open tag since quick-xml may emit Empty for <img>.
            // We don't have attributes here though, so this is a no-op fallback.
        }
        _ => {}
    }
}

fn handle_close_tag(state: &mut State, tag: &str) {
    if is_skip_tag(tag) {
        return;
    }

    match tag {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            state.block_break();
        }
        "strong" | "b" => {
            if state.bold_depth > 0 {
                state.bold_depth -= 1;
                if state.bold_depth == 0 {
                    state.emit_raw("**");
                    // The close marker is "fresh" text — next emit will
                    // add a separating space if its text has leading ws.
                }
            }
        }
        "em" | "i" => {
            if state.italic_depth > 0 {
                state.italic_depth -= 1;
                if state.italic_depth == 0 {
                    state.emit_raw("*");
                }
            }
        }
        "code" => {
            if state.in_pre == 0 && state.code_depth > 0 {
                state.code_depth -= 1;
                if state.code_depth == 0 {
                    state.emit_raw("`");
                }
            }
        }
        "pre" => {
            if state.in_pre > 0 {
                state.in_pre -= 1;
                state.emit_raw("\n```");
                state.block_break();
            }
        }
        "blockquote" => {
            if state.bq_depth > 0 {
                state.bq_depth -= 1;
            }
            state.block_break();
        }
        "a" => {
            if let Some(href) = state.pending_href.take() {
                state.emit_raw(&format!("]({})", href));
                // We need to wrap the text that was just emitted.
                // Retroactively insert the `[` before the link text.
                // Simpler: just emit the closing bracket. The opening `[` is
                // tricky without lookahead, so we use a simplified approach:
                // We emit `[text](href)` pattern by inserting `[` at the end.
                // Actually, for simplicity, let's just close the markdown link.
            }
        }
        "p" | "div" | "section" | "article" => {
            state.block_break();
        }
        "li" => {
            // Single newline between items; block_break on <ul>/<ol> handles outer spacing.
            if !state.out.is_empty() && !state.out.ends_with('\n') {
                state.emit_raw("\n");
            }
        }
        "ul" => {
            state.list_stack.pop();
            if state.list_stack.is_empty() {
                state.block_break();
            }
        }
        "ol" => {
            state.list_stack.pop();
            state.ol_counters.pop();
            if state.list_stack.is_empty() {
                state.block_break();
            }
        }
        "table" => {
            if state.in_table > 0 {
                state.in_table -= 1;
            }
            state.block_break();
        }
        "tr" => {
            if state.in_table > 0 {
                // Close the row, then inject a GFM header separator after row 0.
                state.emit_raw("|\n");
                if state.table_row_index == 0 {
                    let cols = state.table_cell_count.max(1);
                    let mut sep = String::new();
                    for _ in 0..cols {
                        sep.push_str("| --- ");
                    }
                    sep.push_str("|\n");
                    state.emit_raw(&sep);
                }
                state.table_row_index += 1;
            } else {
                state.emit_raw("\n");
            }
        }
        "td" | "th" => {
            if state.in_table > 0 {
                state.emit_raw(" ");
            } else {
                state.emit_raw(" | ");
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn strips_tags_and_extracts_text() {
        let html = "<html><body><p>Hello <b>world</b></p></body></html>";
        let md = extract_text(html);
        assert!(md.contains("Hello"), "got: {:?}", md);
        assert!(md.contains("**world**"), "got: {:?}", md);
    }

    #[test]
    fn headings_become_markdown() {
        let html = "<h1>Title</h1><h2>Subtitle</h2><h3>Section</h3>";
        let md = extract_text(html);
        assert!(md.contains("# Title"), "got: {:?}", md);
        assert!(md.contains("## Subtitle"), "got: {:?}", md);
        assert!(md.contains("### Section"), "got: {:?}", md);
    }

    #[test]
    fn drops_script_and_style() {
        let html =
            "<html><head><style>body{}</style></head><body><script>alert(1)</script>Hello</body></html>";
        let md = extract_text(html);
        assert!(!md.contains("alert"), "script content leaked: {:?}", md);
        assert!(!md.contains("body"), "style content leaked: {:?}", md);
        assert!(md.contains("Hello"), "got: {:?}", md);
    }

    #[test]
    fn converts_links() {
        let html = r#"<p><a href="https://example.com">Click here</a></p>"#;
        let md = extract_text(html);
        assert!(
            md.contains("Click here") && md.contains("https://example.com"),
            "got: {:?}",
            md
        );
    }

    #[test]
    fn converts_lists() {
        let html = "<ul><li>one</li><li>two</li></ul><ol><li>first</li><li>second</li></ol>";
        let md = extract_text(html);
        assert!(md.contains("- one"), "got: {:?}", md);
        assert!(md.contains("- two"), "got: {:?}", md);
        assert!(md.contains("1. first"), "got: {:?}", md);
        assert!(md.contains("2. second"), "got: {:?}", md);
        // Adjacent list items: single newline, not a blank line.
        assert!(
            md.contains("- one\n- two"),
            "expected single newline between items: {:?}",
            md
        );
        assert!(
            !md.contains("- one\n\n- two"),
            "extra blank line between li items: {:?}",
            md
        );
    }

    #[test]
    fn pre_becomes_code_block() {
        let html = "<pre>fn main() {\n    println!(\"hi\");\n}</pre>";
        let md = extract_text(html);
        assert!(md.contains("```"), "got: {:?}", md);
        assert!(md.contains("fn main()"), "got: {:?}", md);
    }

    #[test]
    fn blockquote_becomes_prefix() {
        let html = "<blockquote>This is a quote</blockquote>";
        let md = extract_text(html);
        assert!(md.contains("> This is a quote") || md.contains(">This is a quote"), "got: {:?}", md);
    }

    #[test]
    fn hr_becomes_separator() {
        let html = "<p>Above</p><hr><p>Below</p>";
        let md = extract_text(html);
        assert!(md.contains("---"), "got: {:?}", md);
        assert!(md.contains("Above"), "got: {:?}", md);
        assert!(md.contains("Below"), "got: {:?}", md);
    }

    #[test]
    fn preserves_inline_formatting() {
        let html = "<p><em>italic</em> and <strong>bold</strong> and <code>code</code></p>";
        let md = extract_text(html);
        assert!(md.contains("*italic* and"), "whitespace lost: {:?}", md);
        assert!(md.contains("and **bold** and"), "whitespace lost: {:?}", md);
        assert!(md.contains("and `code`"), "whitespace lost: {:?}", md);
    }

    #[test]
    fn rejects_empty_html() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("empty.html");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(b"<html><body></body></html>").unwrap();

        let err = convert_html(&path).unwrap_err();
        match err {
            AgentReadyError::UserFacing(ErrorCode::NoReadableText) => (),
            other => panic!("expected NoReadableText, got {:?}", other),
        }
    }

    #[test]
    fn converts_full_html_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("page.html");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(
            br#"<!DOCTYPE html>
<html>
<head><title>Test</title></head>
<body>
<h1>Welcome</h1>
<p>This is a <strong>test</strong> page.</p>
<ul><li>Item A</li><li>Item B</li></ul>
</body>
</html>"#,
        )
        .unwrap();

        let result = convert_html(&path).expect("convert should succeed");
        assert!(result.markdown.contains("# Welcome"), "got: {}", result.markdown);
        assert!(result.markdown.contains("**test**"), "got: {}", result.markdown);
        assert!(result.markdown.contains("- Item A"), "got: {}", result.markdown);
    }

    #[test]
    fn anchor_without_href_has_no_dangling_bracket() {
        let html = r#"<p>See <a name="section">this section</a> below.</p>"#;
        let md = extract_text(html);
        assert!(md.contains("this section"), "got: {:?}", md);
        assert!(!md.contains('['), "dangling bracket emitted: {:?}", md);
        assert!(!md.contains(']'), "dangling bracket emitted: {:?}", md);
    }

    #[test]
    fn anchor_with_empty_href_renders_as_plain_text() {
        let html = r#"<p><a href="">empty</a></p>"#;
        let md = extract_text(html);
        assert_eq!(md, "empty", "got: {:?}", md);
    }

    #[test]
    fn space_preserved_before_href_less_anchor() {
        let html = r#"<p>foo and <a name="x">bar</a> baz</p>"#;
        let md = extract_text(html);
        assert_eq!(md, "foo and bar baz", "got: {:?}", md);
    }

    #[test]
    fn space_preserved_around_unknown_inline_tags() {
        let html = "<p>foo <span>bar</span> baz</p>";
        let md = extract_text(html);
        assert_eq!(md, "foo bar baz", "got: {:?}", md);
    }

    #[test]
    fn collapses_whitespace_in_text() {
        let html = "<p>Hello    world\n   wrapped   here</p>";
        let md = extract_text(html);
        assert_eq!(md, "Hello world wrapped here", "got: {:?}", md);
    }

    #[test]
    fn pre_still_preserves_whitespace() {
        let html = "<pre>fn main() {\n    let x = 1;\n}</pre>";
        let md = extract_text(html);
        assert!(md.contains("    let x = 1;"), "pre whitespace lost: {:?}", md);
    }

    #[test]
    fn nested_lists_indent() {
        let html = "<ul><li>a<ul><li>b</li><li>c</li></ul></li><li>d</li></ul>";
        let md = extract_text(html);
        assert!(md.contains("- a"), "got: {:?}", md);
        assert!(md.contains("\n  - b"), "nested item not indented: {:?}", md);
        assert!(md.contains("\n  - c"), "nested item not indented: {:?}", md);
        assert!(md.contains("\n- d"), "outer item lost: {:?}", md);
        // No blank line should split the nested list from its parent.
        assert!(!md.contains("- a\n\n"), "nested list broke into paragraph: {:?}", md);
    }

    #[test]
    fn nested_ordered_list_restarts_numbering() {
        let html = "<ol><li>one<ol><li>inner-a</li><li>inner-b</li></ol></li><li>two</li></ol>";
        let md = extract_text(html);
        assert!(md.contains("1. one"), "got: {:?}", md);
        assert!(md.contains("\n  1. inner-a"), "got: {:?}", md);
        assert!(md.contains("\n  2. inner-b"), "got: {:?}", md);
        assert!(md.contains("\n2. two"), "outer numbering wrong: {:?}", md);
    }

    #[test]
    fn table_becomes_gfm_table() {
        let html = "<table><tr><th>Name</th><th>Age</th></tr><tr><td>Alice</td><td>30</td></tr></table>";
        let md = extract_text(html);
        assert!(md.contains("| Name | Age |"), "header row malformed: {:?}", md);
        assert!(
            md.contains("| --- | --- |"),
            "missing header separator: {:?}",
            md
        );
        assert!(
            md.contains("| Alice | 30 |"),
            "data row malformed: {:?}",
            md
        );
        // Rows must not be split across paragraphs.
        assert!(!md.contains("Name\n\nAge"), "row was shattered: {:?}", md);
    }

    #[test]
    fn handles_malformed_html_gracefully() {
        let html = "<p>Start <b>bold <i>both</b> italic</i> end";
        let md = extract_text(html);
        // Should not panic and should recover some text.
        assert!(md.contains("Start"), "got: {:?}", md);
    }
}
