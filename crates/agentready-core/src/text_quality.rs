/// Heuristic: is this extracted text human-readable (not PDF/DOCX binary junk)?
pub fn readable_text_ratio(text: &str) -> f64 {
    let sample: String = text.chars().take(8000).collect();
    if sample.is_empty() {
        return 0.0;
    }

    let mut good = 0usize;
    let mut total = 0usize;
    for ch in sample.chars() {
        total += 1;
        if ch.is_ascii_alphanumeric() || matches!(ch, ' ' | '\n' | '\r' | '\t') {
            good += 1;
        } else if matches!(
            ch,
            '.' | ',' | ';' | ':' | '!' | '?' | '-' | '_' | '(' | ')' | '[' | ']'
                | '{' | '}' | '#' | '*' | '|' | '/' | '\\' | '\'' | '"' | '%' | '&' | '@'
                | '+' | '=' | '<' | '>'
        ) {
            good += 1;
        }
    }

    good as f64 / total as f64
}

fn word_like_ratio(text: &str) -> f64 {
    let words: Vec<&str> = text.split_whitespace().take(500).collect();
    if words.is_empty() {
        return 0.0;
    }
    let good_words = words
        .iter()
        .filter(|w| {
            let letters: usize = w.chars().filter(|c| c.is_ascii_alphabetic()).count();
            letters >= 2 && letters * 2 >= w.chars().count()
        })
        .count();
    good_words as f64 / words.len() as f64
}

pub fn looks_like_garbage(text: &str) -> bool {
    let sample: String = text.chars().take(8000).collect();
    if sample.trim().len() < 20 {
        return false;
    }

    let char_ratio = readable_text_ratio(&sample);
    let words = word_like_ratio(&sample);
    let space_ratio = sample.chars().filter(|c| c.is_whitespace()).count() as f64
        / sample.chars().count() as f64;

    // PDF binary streams: odd symbols, almost no real words, few spaces
    if char_ratio < 0.65 && words < 0.25 {
        return true;
    }
    if sample.len() > 80 && space_ratio < 0.03 && words < 0.2 {
        return true;
    }

    false
}

pub const GARBAGE_PREVIEW_MESSAGE: &str = "AgentReady could not show readable text for this file.

This often happens with scanned PDFs, image-only documents, or complex layouts. V1 does not include OCR.

Try exporting the original file from your app as TXT or DOCX, or re-save the PDF with a \"Save as text\" option, then upload again.";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_english_is_readable() {
        let text = "Hello world.\nThis is a clean note with punctuation!";
        assert!(!looks_like_garbage(text));
        assert!(readable_text_ratio(text) > 0.8);
    }

    #[test]
    fn binary_junk_is_garbage() {
        let text = "Î³-k¾ÇÇÂ^ÑÇËÄ^¾Ô^-Å²ÇÏËp×©Åx^-ÀÉþÎ³Ô^Ñ©Î³^²³þËp²Î";
        assert!(looks_like_garbage(text));
    }
}
