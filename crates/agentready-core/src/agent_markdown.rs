//! Final pass: lean, agent-scannable Markdown (fewer tokens, less noise).

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
        out.push_str(cleaned.trim());
        out.push('\n');
    }

    while out.ends_with("\n\n\n") {
        out.pop();
    }
    out.trim_end().to_string() + "\n"
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
    fn strips_zero_width_chars() {
        let md = "Hello\u{200b}world\n";
        let out = normalize_for_agents(md);
        assert_eq!(out, "Helloworld\n");
    }
}
