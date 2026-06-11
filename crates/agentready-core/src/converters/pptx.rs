use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;
use zip::ZipArchive;

use crate::converters::ConversionResult;
use crate::models::AgentReadyError;

const MAX_UNCOMPRESSED_SIZE: u64 = 250 * 1024 * 1024; // 250 MB

/// Convert a .pptx file to agent-ready Markdown (text-only).
///
/// Structure: PPTX is a zip; each slide is `ppt/slides/slide{N}.xml`, each note
/// is `ppt/notesSlides/notesSlide{N}.xml`. We walk each slide's XML and pull
/// out text from `<a:t>` runs, grouped by paragraph `<a:p>`. Speaker notes
/// are appended as a blockquote. Images are intentionally ignored (deferred to
/// the API tier that does OCR).
pub fn convert_pptx(path: &Path) -> Result<ConversionResult, AgentReadyError> {
    let file = File::open(path).map_err(AgentReadyError::Io)?;
    let mut archive = ZipArchive::new(file).map_err(AgentReadyError::Zip)?;

    // 1) Collect slide XML bodies in slide-number order.
    let mut slides: BTreeMap<u32, String> = BTreeMap::new();
    let mut notes: BTreeMap<u32, String> = BTreeMap::new();
    let mut image_count: u32 = 0;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(AgentReadyError::Zip)?;
        let name = entry.name().to_string();

        // Zip-bomb guard: reject if declared uncompressed size exceeds limit
        if entry.size() > MAX_UNCOMPRESSED_SIZE {
            return Err(AgentReadyError::UserFacing(
                crate::models::ErrorCode::ZipBombDetected,
            ));
        }

        if let Some(num) = slide_number(&name, "ppt/slides/slide", ".xml") {
            let mut buf = Vec::with_capacity(entry.size() as usize);
            entry.read_to_end(&mut buf).map_err(AgentReadyError::Io)?;
            if buf.len() as u64 > MAX_UNCOMPRESSED_SIZE {
                return Err(AgentReadyError::UserFacing(
                    crate::models::ErrorCode::ZipBombDetected,
                ));
            }
            let xml = String::from_utf8_lossy(&buf).into_owned();
            slides.insert(num, xml);
        } else if let Some(num) = slide_number(&name, "ppt/notesSlides/notesSlide", ".xml") {
            let mut buf = Vec::with_capacity(entry.size() as usize);
            entry.read_to_end(&mut buf).map_err(AgentReadyError::Io)?;
            let xml = String::from_utf8_lossy(&buf).into_owned();
            notes.insert(num, xml);
        } else if name.starts_with("ppt/media/") {
            image_count += 1;
        }
    }

    if slides.is_empty() {
        return Err(AgentReadyError::UserFacing(
            crate::models::ErrorCode::NoReadableText,
        ));
    }

    // 2) Render each slide in order. The export pipeline (export.rs →
    // agent_markdown::normalize_for_agents) will prepend a frontmatter block
    // for us, so we only emit the slide content here.
    let mut markdown = String::new();
    let _ = (slides.len(), image_count); // (slide_count, image_count available for future frontmatter if needed)

    for (num, xml) in slides.iter() {
        let paragraphs = extract_text_runs(xml);
        markdown.push_str(&format!("## Slide {}\n\n", num));
        if paragraphs.is_empty() {
            markdown.push_str("_(no text content)_\n\n");
        } else {
            for p in &paragraphs {
                let trimmed = p.trim();
                if trimmed.is_empty() {
                    continue;
                }
                markdown.push_str(&format!("- {}\n", trimmed));
            }
            markdown.push('\n');
        }

        // Append speaker notes if present
        if let Some(notes_xml) = notes.get(num) {
            let note_paras = extract_text_runs(notes_xml);
            if !note_paras.is_empty() {
                markdown.push_str("> **Notes:**\n");
                for p in &note_paras {
                    let trimmed = p.trim();
                    if !trimmed.is_empty() {
                        markdown.push_str(&format!("> {}\n", trimmed));
                    }
                }
                markdown.push('\n');
            }
        }
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

/// Extract the trailing number from a path like `ppt/slides/slide3.xml`.
fn slide_number(name: &str, prefix: &str, suffix: &str) -> Option<u32> {
    if name.starts_with(prefix) && name.ends_with(suffix) {
        let stem = &name[prefix.len()..name.len() - suffix.len()];
        stem.parse::<u32>().ok()
    } else {
        None
    }
}

/// Walk a slide/notes XML and pull out paragraph-level text runs.
/// Returns one string per `<a:p>` element.
fn extract_text_runs(xml: &str) -> Vec<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut buf = Vec::new();
    let mut paragraphs: Vec<String> = Vec::new();
    let mut current_para = String::new();
    let mut in_text = false;
    let mut in_para = false;

    loop {
        match reader.read_event_into(&mut buf) {
            // Begin a paragraph
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e))
                if local_name(e.name().as_ref()) == b"p" =>
            {
                if in_para {
                    // Nested <a:p>? Just append to current.
                } else {
                    in_para = true;
                    current_para.clear();
                }
                let _ = e; // silence
            }

            // Begin a text run inside a paragraph
            Ok(Event::Start(ref e)) if local_name(e.name().as_ref()) == b"t" => {
                in_text = true;
            }

            // Text content
            Ok(Event::Text(ref e)) if in_text => {
                let text = String::from_utf8_lossy(e.as_ref());
                current_para.push_str(&text);
            }

            // End text run
            Ok(Event::End(ref e)) if local_name(e.name().as_ref()) == b"t" => {
                in_text = false;
            }

            // End paragraph
            Ok(Event::End(ref e)) if local_name(e.name().as_ref()) == b"p" => {
                if in_para {
                    paragraphs.push(current_para.clone());
                    current_para.clear();
                    in_para = false;
                }
            }

            Ok(Event::Eof) => break,
            Err(_) => break, // best-effort: don't fail the whole file on one bad element
            _ => (),
        }
        buf.clear();
    }

    paragraphs
}

/// Strip XML namespace from a tag name (`p:sld` → `sld`).
fn local_name(name: &[u8]) -> &[u8] {
    match name.iter().position(|&b| b == b':') {
        Some(i) => &name[i + 1..],
        None => name,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;
    use zip::write::SimpleFileOptions;

    #[test]
    fn slide_number_parses_correctly() {
        assert_eq!(slide_number("ppt/slides/slide1.xml", "ppt/slides/slide", ".xml"), Some(1));
        assert_eq!(slide_number("ppt/slides/slide42.xml", "ppt/slides/slide", ".xml"), Some(42));
        assert_eq!(slide_number("ppt/notesSlides/notesSlide3.xml", "ppt/notesSlides/notesSlide", ".xml"), Some(3));
        assert_eq!(slide_number("ppt/slides/_rels/slide1.xml.rels", "ppt/slides/slide", ".xml"), None);
        assert_eq!(slide_number("ppt/theme/theme1.xml", "ppt/slides/slide", ".xml"), None);
    }

    #[test]
    fn local_name_strips_namespace() {
        assert_eq!(local_name(b"p:sld"), b"sld");
        assert_eq!(local_name(b"a:t"), b"t");
        assert_eq!(local_name(b"p"), b"p");
    }

    #[test]
    fn extract_text_runs_groups_by_paragraph() {
        let xml = r#"<p:sld>
            <p:cSld>
                <p:sp>
                    <p:txBody>
                        <a:p><a:r><a:t>Hello world</a:t></a:r></a:p>
                        <a:p><a:r><a:t>Line </a:t></a:r><a:r><a:t>two</a:t></a:r></a:p>
                    </p:txBody>
                </p:sp>
            </p:cSld>
        </p:sld>"#;
        let runs = extract_text_runs(xml);
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0], "Hello world");
        assert_eq!(runs[1], "Line two");
    }

    #[test]
    fn converts_minimal_pptx_fixture() {
        let dir = TempDir::new().unwrap();
        let pptx_path = dir.path().join("minimal.pptx");
        write_minimal_pptx(&pptx_path, 3, true);

        let result = convert_pptx(&pptx_path).expect("convert should succeed");
        assert!(result.markdown.contains("## Slide 1"), "got: {}", result.markdown);
        assert!(result.markdown.contains("## Slide 2"));
        assert!(result.markdown.contains("## Slide 3"));
        assert!(result.markdown.contains("- Welcome to AgentReady"));
        assert!(result.markdown.contains("- Slide two content"));
        assert!(result.markdown.contains("> **Notes:**"));
        assert!(result.markdown.contains("> Speaker note here"));
    }

    #[test]
    fn pptx_without_notes_renders_cleanly() {
        let dir = TempDir::new().unwrap();
        let pptx_path = dir.path().join("no_notes.pptx");
        write_minimal_pptx(&pptx_path, 2, false);

        let result = convert_pptx(&pptx_path).expect("convert should succeed");
        assert!(!result.markdown.contains("**Notes:**"));
    }

    #[test]
    fn empty_pptx_yields_no_readable_text() {
        let dir = TempDir::new().unwrap();
        let pptx_path = dir.path().join("empty.pptx");
        // A pptx with no slides
        let file = fs::File::create(&pptx_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let opts: SimpleFileOptions = SimpleFileOptions::default();
        zip.start_file("[Content_Types].xml", opts).unwrap();
        zip.write_all(br#"<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"/>"#).unwrap();
        zip.finish().unwrap();

        let err = convert_pptx(&pptx_path).unwrap_err();
        match err {
            AgentReadyError::UserFacing(crate::models::ErrorCode::NoReadableText) => (),
            other => panic!("expected NoReadableText, got {:?}", other),
        }
    }

    /// Build a minimal pptx with `n_slides` and (optionally) speaker notes.
    /// Each slide has a single text box with a known content.
    /// Also drops a 1-byte image into `ppt/media/` to exercise image_count.
    fn write_minimal_pptx(path: &Path, n_slides: u32, with_notes: bool) {
        let file = fs::File::create(path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let opts: SimpleFileOptions = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        // [Content_Types].xml
        zip.start_file("[Content_Types].xml", opts).unwrap();
        let mut types = String::from(
            r#"<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>"#,
        );
        for i in 1..=n_slides {
            types.push_str(&format!(
                r#"<Override PartName="/ppt/slides/slide{}.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>"#,
                i
            ));
        }
        if with_notes {
            for i in 1..=n_slides {
                types.push_str(&format!(
                    r#"<Override PartName="/ppt/notesSlides/notesSlide{}.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.notesSlide+xml"/>"#,
                    i
                ));
            }
        }
        types.push_str("</Types>");
        zip.write_all(types.as_bytes()).unwrap();

        // _rels/.rels
        zip.start_file("_rels/.rels", opts).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/></Relationships>"#,
        ).unwrap();

        // ppt/_rels/presentation.xml.rels
        zip.start_file("ppt/_rels/presentation.xml.rels", opts).unwrap();
        let mut pres_rels = String::from(
            r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
        );
        for i in 1..=n_slides {
            pres_rels.push_str(&format!(
                r#"<Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide{}.xml"/>"#,
                i, i
            ));
        }
        if with_notes {
            for i in 1..=n_slides {
                pres_rels.push_str(&format!(
                    r#"<Relationship Id="rIdN{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide" Target="notesSlides/notesSlide{}.xml"/>"#,
                    i, i
                ));
            }
        }
        pres_rels.push_str("</Relationships>");
        zip.write_all(pres_rels.as_bytes()).unwrap();

        // ppt/presentation.xml
        zip.start_file("ppt/presentation.xml", opts).unwrap();
        let mut pres = String::from(
            r#"<?xml version="1.0" encoding="UTF-8"?><p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><p:sldIdLst>"#,
        );
        for i in 1..=n_slides {
            pres.push_str(&format!(r#"<p:sldId id="{}" r:id="rId{}"/>"#, 255 + i, i));
        }
        pres.push_str("</p:sldIdLst></p:presentation>");
        zip.write_all(pres.as_bytes()).unwrap();

        // ppt/slides/slide{N}.xml
        for i in 1..=n_slides {
            let text = match i {
                1 => "Welcome to AgentReady",
                2 => "Slide two content",
                _ => "Final slide",
            };
            zip.start_file(format!("ppt/slides/slide{}.xml", i), opts).unwrap();
            zip.write_all(format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:cSld>
    <p:sp>
      <p:txBody>
        <a:p><a:r><a:t>{}</a:t></a:r></a:p>
      </p:txBody>
    </p:sp>
  </p:cSld>
</p:sld>"#,
                text
            ).as_bytes()).unwrap();
        }

        // ppt/notesSlides/notesSlide{N}.xml
        if with_notes {
            for i in 1..=n_slides {
                zip.start_file(format!("ppt/notesSlides/notesSlide{}.xml", i), opts).unwrap();
                zip.write_all(
                    r#"<?xml version="1.0" encoding="UTF-8"?>
<p:notes xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:cSld>
    <p:sp>
      <p:txBody>
        <a:p><a:r><a:t>Speaker note here</a:t></a:r></a:p>
      </p:txBody>
    </p:sp>
  </p:cSld>
</p:notes>"#.as_bytes(),
                ).unwrap();
            }
        }

        // ppt/media/image1.png (1 byte stub; the converter just counts files)
        zip.start_file("ppt/media/image1.png", opts).unwrap();
        zip.write_all(&[0x89, b'P', b'N', b'G']).unwrap();

        zip.finish().unwrap();
    }
}
