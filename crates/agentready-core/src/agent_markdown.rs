//! Final pass: lean, agent-scannable Markdown (fewer tokens, less noise).

/// Markdown body for preview/copy — no YAML frontmatter or export warning block.
pub fn preview_body(stored: &str) -> String {
    strip_leading_warning_blockquote(strip_yaml_frontmatter(stored))
}

/// Normalize converter output before export. Preserves code fences and tables.
pub fn normalize_for_agents(markdown: &str) -> String {
    let mut out = String::with_capacity(markdown.len());
    let mut in_fence = false;
    let mut blank_run = 0usize;

    for line in markdown.replace('\r', "").lines() {
        let cleaned: String = if in_fence {
            line.to_string()
        } else {
            strip_invisible(&line.trim_end().to_string())
        };

        let fence = cleaned.trim_start().starts_with("```");
        if fence {
            in_fence = !in_fence;
            blank_run = 0;
            if !out.is_empty() && !out.ends_with('\n') {
                out.push('\n');
            }
            out.push_str(&cleaned);
            out.push('\n');
            continue;
        }

        if in_fence {
            out.push_str(line);
            out.push('\n');
            continue;
        }

        if cleaned.trim().is_empty() {
            blank_run += 1;
            if blank_run <= 1 {
                out.push('\n');
            }
            continue;
        }

        blank_run = 0;
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        // `cleaned` already has trailing whitespace stripped; keep leading
        // indentation so nested lists and indented blocks survive export.
        out.push_str(&cleaned);
        out.push('\n');
    }

    while out.ends_with("\n\n\n") {
        out.pop();
    }
    out.trim_end().to_string() + "\n"
}

fn strip_yaml_frontmatter(text: &str) -> &str {
    let trimmed = text.trim_start();
    if !trimmed.starts_with("---") {
        return text;
    }
    let Some(rest) = trimmed.strip_prefix("---") else {
        return text;
    };
    let rest = rest.trim_start_matches(['\n', '\r']);
    if let Some(end) = rest.find("\n---") {
        rest[end + 4..].trim_start_matches(['\n', '\r'])
    } else {
        text
    }
}

fn strip_leading_warning_blockquote(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines
        .first()
        .is_none_or(|l| !l.trim().starts_with("> **AgentReady warning:**"))
    {
        return text.trim().to_string();
    }
    let mut i = 0usize;
    while i < lines.len() && !lines[i].trim().is_empty() {
        i += 1;
    }
    while i < lines.len() && lines[i].trim().is_empty() {
        i += 1;
    }
    lines[i..].join("\n").trim().to_string()
}

fn strip_invisible(text: &str) -> String {
    text.chars()
        .filter(|c| {
            !matches!(
                c,
                '\u{feff}' | '\u{200b}' | '\u{200c}' | '\u{200d}' | '\u{2060}' | '\u{00ad}'
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapses_excessive_blank_lines() {
        let md = "One.\n\n\n\nTwo.\n";
        let out = normalize_for_agents(md);
        assert!(!out.contains("\n\n\n"));
        assert!(out.contains("One."));
        assert!(out.contains("Two."));
    }

    #[test]
    fn preserves_code_fences() {
        let md = "Intro\n\n```rust\nfn main() {\n    let x  =  1;\n}\n```\n\nEnd.\n";
        let out = normalize_for_agents(md);
        assert!(out.contains("    let x  =  1;"));
        assert!(out.contains("End."));
    }

    #[test]
    fn preserves_list_indentation() {
        let md = "- a\n  - b\n    - c\n";
        let out = normalize_for_agents(md);
        assert!(out.contains("\n  - b"), "nested indent lost: {:?}", out);
        assert!(out.contains("\n    - c"), "deep indent lost: {:?}", out);
    }

    #[test]
    fn still_strips_trailing_whitespace() {
        let md = "alpha   \nbeta\t\n";
        let out = normalize_for_agents(md);
        assert_eq!(out, "alpha\nbeta\n");
    }

    #[test]
    fn strips_zero_width_chars() {
        let md = "Hello\u{200b}world\n";
        let out = normalize_for_agents(md);
        assert_eq!(out, "Helloworld\n");
    }

    #[test]
    fn preview_body_strips_frontmatter_and_warning() {
        let stored = "---\nsource_file: book.pdf\nsource_type: pdf\nconverted_by: agentready-v1\nstatus: good\n---\n\n> **AgentReady warning:** partial\n\n# Title\n\nBody.\n";
        let body = preview_body(stored);
        assert!(!body.contains("source_file"));
        assert!(!body.contains("AgentReady warning"));
        assert!(body.contains("# Title"));
        assert!(body.contains("Body."));
    }
}
