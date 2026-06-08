use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use quick_xml::events::Event;
use quick_xml::Reader;
use zip::ZipArchive;

use crate::converters::ConversionResult;
use crate::models::{AgentReadyError, ErrorCode};
use crate::text_quality;

const MAX_UNCOMPRESSED_SIZE: u64 = 250 * 1024 * 1024;
const MAX_TOTAL_TEXT_BYTES: usize = 50 * 1024 * 1024;

pub fn convert_epub(path: &Path) -> Result<ConversionResult, AgentReadyError> {
    let file = File::open(path).map_err(AgentReadyError::Io)?;
    let mut archive = ZipArchive::new(file).map_err(AgentReadyError::Zip)?;

    let container = read_zip_text(&mut archive, "META-INF/container.xml")?;
    let opf_path = opf_path_from_container(&container)?;
    let opf_base = opf_base_dir(&opf_path);
    let opf_xml = read_zip_text(&mut archive, &opf_path)?;
    let (manifest, spine) = parse_opf(&opf_xml)?;

    let chapter_ids: Vec<String> = if spine.is_empty() {
        manifest
            .iter()
            .filter(|(_, href)| is_content_href(href))
            .map(|(id, _)| id.clone())
            .collect()
    } else {
        spine
    };

    if chapter_ids.is_empty() {
        return Err(AgentReadyError::UserFacing(ErrorCode::NoReadableText));
    }

    let mut markdown = String::new();
    let mut converted_chapters = 0usize;
    let mut skipped_chapters = 0usize;

    for id in chapter_ids {
        let Some(href) = manifest.get(&id) else {
            skipped_chapters += 1;
            continue;
        };
        if !is_content_href(href) {
            skipped_chapters += 1;
            continue;
        }

        let entry_path = join_epub_path(&opf_base, href);
        let html = match read_zip_text(&mut archive, &entry_path) {
            Ok(text) => text,
            Err(_) => {
                skipped_chapters += 1;
                continue;
            }
        };

        let chapter_md = html_to_markdown(&html);
        if chapter_md.trim().is_empty() {
            skipped_chapters += 1;
            continue;
        }

        if converted_chapters > 0 {
            markdown.push_str("\n---\n\n");
        }
        markdown.push_str(&chapter_md);
        markdown.push_str("\n\n");
        converted_chapters += 1;

        if markdown.len() > MAX_TOTAL_TEXT_BYTES {
            return Err(AgentReadyError::UserFacing(ErrorCode::FileTooLarge));
        }
    }

    if markdown.trim().is_empty() {
        return Err(AgentReadyError::UserFacing(ErrorCode::NoReadableText));
    }

    if text_quality::looks_like_garbage(&markdown) {
        return Err(AgentReadyError::UserFacing(ErrorCode::NoReadableText));
    }

    let mut warning = if skipped_chapters > 0 {
        Some(format!(
            "Converted {converted_chapters} chapter(s); {skipped_chapters} section(s) were skipped (images or unsupported content)."
        ))
    } else {
        None
    };

    if text_quality::readable_text_ratio(&markdown) < 0.75 {
        let quality_note =
            "Some EPUB text may not have converted cleanly. Please review the output.";
        warning = Some(match warning {
            Some(existing) => format!("{existing} {quality_note}"),
            None => quality_note.into(),
        });
    }

    Ok(ConversionResult {
        markdown,
        warning,
        raw_data: None,
    })
}

fn read_zip_text(archive: &mut ZipArchive<File>, name: &str) -> Result<String, AgentReadyError> {
    let normalized = name.replace('\\', "/");
    let mut file = archive
        .by_name(&normalized)
        .map_err(|_| AgentReadyError::UserFacing(ErrorCode::ConversionFailed))?;

    let size = file.size();
    if size > MAX_UNCOMPRESSED_SIZE {
        return Err(AgentReadyError::UserFacing(ErrorCode::ZipBombDetected));
    }

    let mut buf = Vec::with_capacity(size as usize);
    file.read_to_end(&mut buf).map_err(AgentReadyError::Io)?;

    if buf.len() as u64 > MAX_UNCOMPRESSED_SIZE {
        return Err(AgentReadyError::UserFacing(ErrorCode::ZipBombDetected));
    }

    Ok(String::from_utf8_lossy(&buf).into_owned())
}

fn opf_path_from_container(xml: &str) -> Result<String, AgentReadyError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(e)) | Ok(Event::Start(e)) => {
                if e.local_name().as_ref() == b"rootfile" {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"full-path" {
                            return Ok(String::from_utf8_lossy(&attr.value).into_owned());
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    Err(AgentReadyError::UserFacing(ErrorCode::ConversionFailed))
}

fn opf_base_dir(opf_path: &str) -> String {
    Path::new(opf_path)
        .parent()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default()
}

fn parse_opf(xml: &str) -> Result<(HashMap<String, String>, Vec<String>), AgentReadyError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    let mut manifest: HashMap<String, String> = HashMap::new();
    let mut spine = Vec::new();
    let mut in_spine = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                match e.local_name().as_ref() {
                    b"item" => {
                        let mut id = None;
                        let mut href = None;
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"id" => id = Some(String::from_utf8_lossy(&attr.value).into_owned()),
                                b"href" => href = Some(String::from_utf8_lossy(&attr.value).into_owned()),
                                _ => {}
                            }
                        }
                        if let (Some(id), Some(href)) = (id, href) {
                            manifest.insert(id, decode_href(&href));
                        }
                    }
                    b"spine" => in_spine = true,
                    b"itemref" if in_spine => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"idref" {
                                spine.push(String::from_utf8_lossy(&attr.value).into_owned());
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                if e.local_name().as_ref() == b"spine" {
                    in_spine = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => return Err(AgentReadyError::XmlParse),
            _ => {}
        }
        buf.clear();
    }

    Ok((manifest, spine))
}

fn decode_href(href: &str) -> String {
    let path = href.split('#').next().unwrap_or(href);
    path.replace("%20", " ")
        .replace("%28", "(")
        .replace("%29", ")")
}

fn join_epub_path(base: &str, href: &str) -> String {
    let href = href.trim_start_matches('/');
    if base.is_empty() {
        href.to_string()
    } else {
        format!("{}/{}", base.trim_end_matches('/'), href)
    }
}

fn is_content_href(href: &str) -> bool {
    let lower = href.to_lowercase();
    lower.ends_with(".xhtml")
        || lower.ends_with(".html")
        || lower.ends_with(".htm")
        || lower.ends_with(".xml")
}

fn html_to_markdown(html: &str) -> String {
    let mut work = html.to_string();

    for tag in &[
        "script", "style", "svg", "nav", "header", "footer", "aside",
    ] {
        work = strip_tag_block(&work, tag);
    }

    work = work.replace("<br>", "\n").replace("<br/>", "\n").replace("<br />", "\n");
    work = work.replace("</p>", "\n\n").replace("</div>", "\n\n").replace("</li>", "\n");
    work = work.replace("</h1>", "\n\n").replace("</h2>", "\n\n").replace("</h3>", "\n\n");
    work = work.replace("</h4>", "\n\n").replace("</h5>", "\n\n").replace("</h6>", "\n\n");

    let mut out = String::new();
    let mut in_tag = false;
    let mut tag_buf = String::new();
    let mut last_was_heading = false;

    for ch in work.chars() {
        match ch {
            '<' => {
                in_tag = true;
                tag_buf.clear();
            }
            '>' if in_tag => {
                in_tag = false;
                let tag = tag_buf.trim().to_lowercase();
                if let Some(rest) = tag.strip_prefix('/') {
                    let name: String = rest.chars().take_while(|c| c.is_alphanumeric()).collect();
                    if matches!(name.as_str(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6") {
                        out.push('\n');
                        last_was_heading = true;
                    }
                } else {
                    let name: String = tag
                        .chars()
                        .take_while(|c| c.is_alphanumeric())
                        .collect();
                    if let Some(level) = heading_level(&name) {
                        if !out.is_empty() && !out.ends_with("\n\n") {
                            out.push_str("\n\n");
                        }
                        out.push_str(&"#".repeat(level));
                        out.push(' ');
                        last_was_heading = true;
                    } else if name == "li" {
                        out.push_str("- ");
                    } else {
                        last_was_heading = false;
                    }
                }
                tag_buf.clear();
            }
            _ if in_tag => tag_buf.push(ch),
            _ => {
                if last_was_heading && !ch.is_whitespace() {
                    last_was_heading = false;
                }
                out.push(ch);
            }
        }
    }

    collapse_blank_lines(&out)
}

fn heading_level(name: &str) -> Option<usize> {
    match name {
        "h1" => Some(1),
        "h2" => Some(2),
        "h3" => Some(3),
        "h4" => Some(4),
        "h5" => Some(5),
        "h6" => Some(6),
        _ => None,
    }
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
    use std::io::Write;
    use std::path::PathBuf;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    fn write_minimal_epub(dir: &Path) -> PathBuf {
        let path = dir.join("test.epub");
        let file = File::create(&path).unwrap();
        let mut zip = ZipWriter::new(file);
        let opts = SimpleFileOptions::default();

        zip.start_file("mimetype", opts).unwrap();
        zip.write_all(b"application/epub+zip").unwrap();

        zip.start_file("META-INF/container.xml", opts).unwrap();
        zip.write_all(
            br#"<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#,
        )
        .unwrap();

        zip.start_file("OPS/content.opf", opts).unwrap();
        zip.write_all(
            br#"<?xml version="1.0"?>
<package version="2.0" xmlns="http://www.idpf.org/2007/opf">
  <manifest>
    <item id="c1" href="chapter1.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="c1"/>
  </spine>
</package>"#,
        )
        .unwrap();

        zip.start_file("OPS/chapter1.xhtml", opts).unwrap();
        zip.write_all(
            br#"<?xml version="1.0"?>
<html><body><h1>Chapter One</h1><p>Hello ebook reader.</p></body></html>"#,
        )
        .unwrap();

        zip.finish().unwrap();
        path
    }

    #[test]
    fn converts_minimal_epub() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_minimal_epub(dir.path());
        let result = convert_epub(&path).unwrap();
        assert!(result.markdown.contains("Chapter One"));
        assert!(result.markdown.contains("Hello ebook reader"));
    }

    #[test]
    fn decode_href_strips_fragment() {
        assert_eq!(decode_href("chapter1.xhtml#intro"), "chapter1.xhtml");
    }

    #[test]
    fn rejects_non_epub_zip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.epub");
        let file = File::create(&path).unwrap();
        let mut zip = ZipWriter::new(file);
        #[allow(deprecated)]
        zip.start_file("readme.txt", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"not an epub").unwrap();
        zip.finish().unwrap();

        assert!(convert_epub(&path).is_err());
    }
}
