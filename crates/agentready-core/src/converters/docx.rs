use std::fs::File;
use std::io::Read;
use std::path::Path;
use quick_xml::events::Event;
use quick_xml::Reader;
use zip::ZipArchive;

use crate::converters::ConversionResult;
use crate::models::AgentReadyError;

const MAX_UNCOMPRESSED_SIZE: u64 = 250 * 1024 * 1024; // 250 MB

pub fn convert_docx(path: &Path) -> Result<ConversionResult, AgentReadyError> {
    let file = File::open(path).map_err(AgentReadyError::Io)?;
    let mut archive = ZipArchive::new(file).map_err(AgentReadyError::Zip)?;

    let mut doc_xml = String::new();
    let mut found_document = false;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(AgentReadyError::Zip)?;
        if file.name() == "word/document.xml" {
            found_document = true;

            // Defend against zip bombs: check declared size first
            let size = file.size();
            if size > MAX_UNCOMPRESSED_SIZE {
                return Err(AgentReadyError::UserFacing(
                    crate::models::ErrorCode::ZipBombDetected,
                ));
            }

            // Read full XML into Vec<u8> first, then decode as UTF-8 (fixes C1)
            let mut buf = Vec::with_capacity(size as usize);
            file.read_to_end(&mut buf).map_err(AgentReadyError::Io)?;

            // Reject if actual read exceeds limit (defense in depth)
            if buf.len() as u64 > MAX_UNCOMPRESSED_SIZE {
                return Err(AgentReadyError::UserFacing(
                    crate::models::ErrorCode::ZipBombDetected,
                ));
            }

            doc_xml = String::from_utf8_lossy(&buf).into_owned();
            break;
        }
    }

    if !found_document {
        return Err(AgentReadyError::UserFacing(
            crate::models::ErrorCode::ConversionFailed,
        ));
    }

    let markdown = parse_docx_xml(&doc_xml)?;

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

fn parse_docx_xml(xml: &str) -> Result<String, AgentReadyError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut markdown = String::new();
    let mut buf = Vec::new();

    // State tracking
    let mut in_text = false;
    let mut in_table = false;
    let mut table_row_count = 0;
    let mut current_row_cells: Vec<String> = Vec::new();
    let mut paragraph_style: Option<String> = None;
    let mut run_bold = false;
    let mut run_italic = false;
    let mut current_text = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            // Paragraph start
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"w:p" => {
                paragraph_style = None;
                current_text.clear();
            }

            // Paragraph properties — detect style
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"w:pPr" => {
                // Style will be read from w:pStyle child
            }

            Ok(Event::Empty(ref e)) if e.name().as_ref() == b"w:pStyle" => {
                for attr in e.attributes().flatten() {
                    if attr.key.as_ref() == b"w:val" {
                        paragraph_style = Some(String::from_utf8_lossy(&attr.value).to_string());
                    }
                }
            }

            // Run start — track bold/italic
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"w:r" => {
                run_bold = false;
                run_italic = false;
            }

            Ok(Event::Empty(ref e)) if e.name().as_ref() == b"w:b" => {
                run_bold = true;
            }

            Ok(Event::Empty(ref e)) if e.name().as_ref() == b"w:i" => {
                run_italic = true;
            }

            // Text content
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"w:t" => {
                in_text = true;
            }

            Ok(Event::Text(ref e)) if in_text => {
                let text = String::from_utf8_lossy(e.as_ref());
                if run_bold && run_italic {
                    current_text.push_str(&format!("***{}***", text));
                } else if run_bold {
                    current_text.push_str(&format!("**{}**", text));
                } else if run_italic {
                    current_text.push_str(&format!("*{}*", text));
                } else {
                    current_text.push_str(&text);
                }
            }

            // Text end
            Ok(Event::End(ref e)) if e.name().as_ref() == b"w:t" => {
                in_text = false;
            }

            Ok(Event::End(ref e)) if e.name().as_ref() == b"w:r" => {}

            // Paragraph end — emit markdown based on style
            Ok(Event::End(ref e)) if e.name().as_ref() == b"w:p" => {
                let text = current_text.trim().to_string();
                if text.is_empty() && !in_table {
                    buf.clear();
                    continue;
                }

                if in_table {
                    // Paragraph inside a table cell — text is already in current_text
                    buf.clear();
                    continue;
                }

                let style = paragraph_style.as_deref().unwrap_or("");

                if let Some(level) = parse_heading_level(style) {
                    let hashes = "#".repeat(level);
                    markdown.push_str(&format!("{} {}\n\n", hashes, text));
                } else if style == "ListBullet" {
                    // Add blank line before list if previous content wasn't a list
                    if !markdown.ends_with("- ") && !markdown.ends_with('\n') {
                        markdown.push('\n');
                    }
                    markdown.push_str(&format!("- {}\n", text));
                } else if style == "ListNumber" {
                    if !markdown.ends_with("1. ") && !markdown.ends_with('\n') {
                        markdown.push('\n');
                    }
                    markdown.push_str(&format!("1. {}\n", text));
                } else {
                    // Regular paragraph — add blank line before if not after a list
                    if !markdown.ends_with('\n') {
                        markdown.push('\n');
                    }
                    markdown.push_str(&text);
                    markdown.push_str("\n\n");
                }

                paragraph_style = None;
            }

            // Table
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"w:tbl" => {
                in_table = true;
                table_row_count = 0;
            }

            Ok(Event::End(ref e)) if e.name().as_ref() == b"w:tbl" => {
                in_table = false;
                markdown.push('\n');
            }

            // Table row
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"w:tr" => {
                current_row_cells.clear();
            }

            Ok(Event::End(ref e)) if e.name().as_ref() == b"w:tr" => {
                table_row_count += 1;

                // First row is header
                if table_row_count == 1 {
                    let table_col_count = current_row_cells.len();
                    markdown.push_str(&format!("| {} |\n", current_row_cells.join(" | ")));
                    markdown.push_str(&format!("| {} |\n", vec!["---"; table_col_count].join(" | ")));
                } else {
                    markdown.push_str(&format!("| {} |\n", current_row_cells.join(" | ")));
                }
            }

            // Table cell
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"w:tc" => {
                current_text.clear();
            }

            Ok(Event::End(ref e)) if e.name().as_ref() == b"w:tc" => {
                current_row_cells.push(current_text.trim().to_string());
                current_text.clear();
            }

            Ok(Event::Eof) => break,
            Err(_) => return Err(AgentReadyError::XmlParse),
            _ => (),
        }
        buf.clear();
    }

    Ok(markdown)
}

fn parse_heading_level(style: &str) -> Option<usize> {
    match style {
        "Heading1" | "heading 1" => Some(1),
        "Heading2" | "heading 2" => Some(2),
        "Heading3" | "heading 3" => Some(3),
        "Heading4" | "heading 4" => Some(4),
        "Heading5" | "heading 5" => Some(5),
        "Heading6" | "heading 6" => Some(6),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ErrorCode;
    use std::fs;
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    fn create_minimal_docx() -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.docx");
        let file = fs::File::create(&path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = SimpleFileOptions::default();

        let doc_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Title</w:t></w:r></w:p>
    <w:p><w:r><w:t>Paragraph.</w:t></w:r></w:p>
    <w:p><w:pPr><w:pStyle w:val="ListBullet"/></w:pPr><w:r><w:t>Item</w:t></w:r></w:p>
    <w:p><w:pPr><w:pStyle w:val="ListNumber"/></w:pPr><w:r><w:t>Step one</w:t></w:r></w:p>
    <w:tbl>
      <w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr>
      <w:tr><w:tc><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc></w:tr>
      <w:tr><w:tc><w:p><w:r><w:t>1</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>2</w:t></w:r></w:p></w:tc></w:tr>
    </w:tbl>
    <w:p><w:r><w:rPr><w:b/></w:rPr><w:t>Bold</w:t></w:r><w:t> and </w:t><w:r><w:rPr><w:i/></w:rPr><w:t>italic</w:t></w:r></w:p>
  </w:body>
</w:document>"#;

        #[allow(deprecated)]
        zip.start_file("word/document.xml", options).unwrap();
        zip.write_all(doc_xml.as_bytes()).unwrap();
        zip.finish().unwrap();
        (dir, path)
    }

    #[test]
    fn converts_docx_structure() {
        let (_dir, path) = create_minimal_docx();
        let result = convert_docx(&path).unwrap();

        assert!(result.markdown.contains("# Title"), "Missing h1");
        assert!(result.markdown.contains("Paragraph."), "Missing paragraph");
        assert!(result.markdown.contains("- Item"), "Missing bullet");
        assert!(result.markdown.contains("1. Step one"), "Missing numbered list");
        assert!(result.markdown.contains("| A | B |"), "Missing table header");
        assert!(result.markdown.contains("| 1 | 2 |"), "Missing table row");
        assert!(result.markdown.contains("**Bold**"), "Missing bold");
        assert!(result.markdown.contains("*italic*"), "Missing italic");
    }

    #[test]
    fn rejects_non_docx() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.docx");
        fs::write(&path, b"not a docx").unwrap();
        assert!(convert_docx(&path).is_err());
    }

    #[test]
    fn rejects_empty_docx() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.docx");
        let file = fs::File::create(&path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = SimpleFileOptions::default();
        #[allow(deprecated)]
        zip.start_file("word/document.xml", options).unwrap();
        zip.write_all(b"<?xml version=\"1.0\"?><w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:body></w:body></w:document>").unwrap();
        zip.finish().unwrap();
        let result = convert_docx(&path);
        assert_eq!(result.unwrap_err().to_error_code(), ErrorCode::NoReadableText);
    }
}
