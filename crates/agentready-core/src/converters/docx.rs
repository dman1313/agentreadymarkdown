use std::fs::File;
use std::io::Read;
use std::path::Path;
use quick_xml::events::Event;
use quick_xml::Reader;
use zip::ZipArchive;

use crate::converters::ConversionResult;
use crate::models::ErrorCode;

const MAX_UNCOMPRESSED_SIZE: u64 = 250 * 1024 * 1024; // 250 MB

pub fn convert_docx(path: &Path) -> Result<ConversionResult, ErrorCode> {
    let file = File::open(path).map_err(|_| ErrorCode::ConversionFailed)?;
    let mut archive = ZipArchive::new(file).map_err(|_| ErrorCode::ConversionFailed)?;

    // Protect against Zip Bombs
    let mut doc_xml = String::new();
    let mut found_document = false;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|_| ErrorCode::ZipBombDetected)?;
        if file.name() == "word/document.xml" {
            found_document = true;
            let mut buffer = [0; 8192];
            let mut total_read = 0;
            
            loop {
                let n = file.read(&mut buffer).map_err(|_| ErrorCode::ConversionFailed)?;
                if n == 0 {
                    break;
                }
                total_read += n as u64;
                if total_read > MAX_UNCOMPRESSED_SIZE {
                    return Err(ErrorCode::ZipBombDetected);
                }
                
                let chunk = String::from_utf8_lossy(&buffer[..n]);
                doc_xml.push_str(&chunk);
            }
            break;
        }
    }

    if !found_document {
        return Err(ErrorCode::ConversionFailed);
    }

    let mut reader = Reader::from_str(&doc_xml);
    reader.config_mut().trim_text(false);
    
    let mut markdown = String::new();
    let mut in_text = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"w:t" => {
                in_text = true;
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"w:t" => {
                in_text = false;
            }
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"w:p" => {
                // New paragraph
                if !markdown.ends_with("\n\n") {
                    markdown.push_str("\n\n");
                }
            }
            Ok(Event::Text(e)) if in_text => {
                let text = String::from_utf8_lossy(e.as_ref());
                markdown.push_str(&text);
            }
            Ok(Event::Eof) => break,
            Err(_) => return Err(ErrorCode::ConversionFailed),
            _ => (),
        }
        buf.clear();
    }

    let trimmed = markdown.trim().to_string();
    if trimmed.is_empty() {
        return Err(ErrorCode::NoReadableText);
    }

    Ok(ConversionResult {
        markdown: trimmed,
        warning: None,
        raw_data: None,
    })
}
