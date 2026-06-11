use std::fs;
use std::path::Path;

use calamine::{open_workbook, Data, Reader, Xlsx, XlsxError};

use crate::converters::ConversionResult;
use crate::models::{AgentReadyError, ErrorCode};

/// Maximum rows per sheet; anything beyond is truncated and marked partial.
const MAX_ROWS_PER_SHEET: usize = 1000;

pub fn convert_xlsx(path: &Path) -> Result<ConversionResult, AgentReadyError> {
    // Preserve raw bytes for the data/ folder.
    let raw_bytes = fs::read(path).map_err(AgentReadyError::Io)?;

    let mut workbook: Xlsx<_> = open_workbook(path).map_err(map_xlsx_error)?;

    let sheet_names: Vec<String> = workbook.sheet_names().to_vec();
    if sheet_names.is_empty() {
        return Err(AgentReadyError::UserFacing(ErrorCode::NoReadableText));
    }

    let mut markdown = String::new();
    let mut truncated_any = false;

    for (idx, name) in sheet_names.iter().enumerate() {
        let range = workbook.worksheet_range(name).map_err(map_xlsx_error)?;

        if idx > 0 {
            markdown.push_str("\n\n");
        }
        let mut sheet_md = String::new();
        sheet_md.push_str(&format!("## {}\n\n", name));

        let height = range.height();
        let width = range.width();

        if height == 0 || width == 0 {
            sheet_md.push_str("_(empty sheet)_\n");
            markdown.push_str(&sheet_md);
            continue;
        }

        let row_limit = height.min(MAX_ROWS_PER_SHEET);
        if height > MAX_ROWS_PER_SHEET {
            truncated_any = true;
        }

        // First non-empty row is the header (per spec: "Convert sheets/tables").
        // If row 0 is empty, scan down to first row with any non-empty cell.
        let header_row_idx = (0..row_limit)
            .find(|&r| (0..width).any(|c| !is_empty_cell(range.get((r, c)))))
            .unwrap_or(0);

        // Render header
        let headers: Vec<String> = (0..width)
            .map(|c| cell_to_string(range.get((header_row_idx, c))))
            .collect();
        sheet_md.push_str(&format!("| {} |\n", headers.join(" | ")));
        sheet_md.push_str(&format!(
            "| {} |\n",
            (0..width).map(|_| "---").collect::<Vec<_>>().join(" | ")
        ));

        // Render rows
        for r in (header_row_idx + 1)..row_limit {
            let row_str: Vec<String> = (0..width)
                .map(|c| escape_pipe(cell_to_string(range.get((r, c)))))
                .collect();
            sheet_md.push_str(&format!("| {} |\n", row_str.join(" | ")));
        }

        if height > MAX_ROWS_PER_SHEET {
            sheet_md.push_str(&format!(
                "\n_…truncated: sheet has {} rows; showing first {}._\n",
                height, MAX_ROWS_PER_SHEET
            ));
        }

        markdown.push_str(&sheet_md);
    }

    if markdown.trim().is_empty() {
        return Err(AgentReadyError::UserFacing(ErrorCode::NoReadableText));
    }

    let warning = if truncated_any {
        Some(format!(
            "One or more sheets exceeded {} rows; truncated. Raw data preserved in data/ folder.",
            MAX_ROWS_PER_SHEET
        ))
    } else {
        None
    };

    Ok(ConversionResult {
        markdown,
        warning,
        raw_data: Some(raw_bytes),
    })
}

fn is_empty_cell(cell: Option<&Data>) -> bool {
    match cell {
        None => true,
        Some(Data::Empty) => true,
        Some(Data::String(s)) if s.is_empty() => true,
        _ => false,
    }
}

fn cell_to_string(cell: Option<&Data>) -> String {
    match cell {
        None | Some(Data::Empty) => String::new(),
        Some(Data::String(s)) => s.clone(),
        Some(Data::Int(i)) => i.to_string(),
        Some(Data::Float(f)) => {
            if f.fract() == 0.0 && f.abs() < 1e16 {
                format!("{}", *f as i64)
            } else {
                format!("{}", f)
            }
        }
        Some(Data::Bool(b)) => b.to_string(),
        Some(Data::DateTime(dt)) => format!("{}", dt),
        Some(Data::DateTimeIso(s)) => s.clone(),
        Some(Data::DurationIso(s)) => s.clone(),
        Some(Data::Error(e)) => format!("{:?}", e),
    }
}

fn escape_pipe(s: String) -> String {
    s.replace('|', "\\|").replace('\n', " ").replace('\r', "")
}

/// Map calamine XlsxError to AgentReady's error model.
fn map_xlsx_error(e: XlsxError) -> AgentReadyError {
    match e {
        XlsxError::Password => AgentReadyError::UserFacing(ErrorCode::PasswordProtected),
        _ => AgentReadyError::UserFacing(ErrorCode::ConversionFailed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use calamine::Data;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn escape_pipe_escapes_table_chars() {
        assert_eq!(escape_pipe("a|b".to_string()), "a\\|b");
        assert_eq!(escape_pipe("a\nb".to_string()), "a b");
        assert_eq!(escape_pipe("a\rb".to_string()), "ab");
    }

    #[test]
    fn cell_to_string_handles_all_data_variants() {
        assert_eq!(cell_to_string(None), "");
        assert_eq!(cell_to_string(Some(&Data::Empty)), "");
        assert_eq!(cell_to_string(Some(&Data::String("hi".into()))), "hi");
        assert_eq!(cell_to_string(Some(&Data::Int(42))), "42");
        assert_eq!(cell_to_string(Some(&Data::Int(-7))), "-7");
        assert_eq!(cell_to_string(Some(&Data::Float(1.5))), "1.5");
        assert_eq!(cell_to_string(Some(&Data::Float(2.0))), "2");
        assert_eq!(cell_to_string(Some(&Data::Bool(true))), "true");
        assert_eq!(cell_to_string(Some(&Data::Bool(false))), "false");
        assert_eq!(
            cell_to_string(Some(&Data::DateTimeIso("2026-06-11".into()))),
            "2026-06-11"
        );
    }

    #[test]
    fn is_empty_cell_distinguishes_empty_from_zero() {
        assert!(is_empty_cell(None));
        assert!(is_empty_cell(Some(&Data::Empty)));
        assert!(is_empty_cell(Some(&Data::String("".into()))));
        assert!(!is_empty_cell(Some(&Data::Int(0))));
        assert!(!is_empty_cell(Some(&Data::Float(0.0))));
        assert!(!is_empty_cell(Some(&Data::String("x".into()))));
    }

    #[test]
    fn converts_simple_xlsx_fixture() {
        let dir = TempDir::new().unwrap();
        let xlsx_path = dir.path().join("minimal.xlsx");
        write_minimal_xlsx(&xlsx_path);

        let result = convert_xlsx(&xlsx_path).expect("convert should succeed");
        assert!(
            result.markdown.contains("## Sheet1"),
            "markdown should have a Sheet1 heading; got: {}",
            result.markdown
        );
        assert!(result.markdown.contains("| name | age |"));
        assert!(result.markdown.contains("| --- | --- |"));
        assert!(result.markdown.contains("| Alice | 30 |"));
        assert!(result.markdown.contains("| Bob | 25 |"));
        assert!(result.raw_data.is_some(), "raw bytes should be preserved");
    }

    #[test]
    fn multiple_sheets_each_get_a_section() {
        let dir = TempDir::new().unwrap();
        let xlsx_path = dir.path().join("two_sheets.xlsx");
        write_minimal_xlsx_with_sheets(&xlsx_path, &["Alpha", "Beta"]);

        let result = convert_xlsx(&xlsx_path).expect("convert should succeed");
        assert!(result.markdown.contains("## Alpha"));
        assert!(result.markdown.contains("## Beta"));
    }

    #[test]
    fn truncated_sheet_warns_and_marks_partial() {
        let dir = TempDir::new().unwrap();
        let xlsx_path = dir.path().join("big.xlsx");
        write_minimal_xlsx_with_rows(&xlsx_path, MAX_ROWS_PER_SHEET + 100);

        let result = convert_xlsx(&xlsx_path).expect("convert should succeed");
        assert!(result.warning.is_some(), "should warn on truncation");
        assert!(result.warning.unwrap().contains("truncated"));
    }

    #[test]
    fn non_xlsx_file_yields_conversion_failed() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("notreally.xlsx");
        // Plain text content that isn't a real xlsx
        let mut f = fs::File::create(&path).unwrap();
        f.write_all(b"this is just plain text, not a real xlsx")
            .unwrap();

        let err = convert_xlsx(&path).unwrap_err();
        match err {
            AgentReadyError::UserFacing(ErrorCode::ConversionFailed) => (),
            other => panic!("expected ConversionFailed, got {:?}", other),
        }
    }

    /// Build a minimal xlsx file (single sheet "Sheet1" with two rows of data).
    /// Hand-rolled OPC package using the `zip` crate at test time.
    fn write_minimal_xlsx(path: &Path) {
        let file = fs::File::create(path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let opts: zip::write::SimpleFileOptions = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("[Content_Types].xml", opts).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/><Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/><Override PartName="/xl/sharedStrings.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml"/></Types>"#,
        ).unwrap();

        zip.start_file("_rels/.rels", opts).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>"#,
        ).unwrap();

        zip.start_file("xl/_rels/workbook.xml.rels", opts).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings" Target="sharedStrings.xml"/></Relationships>"#,
        ).unwrap();

        zip.start_file("xl/workbook.xml", opts).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="Sheet1" sheetId="1" r:id="rId1"/></sheets></workbook>"#,
        ).unwrap();

        zip.start_file("xl/sharedStrings.xml", opts).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="4" uniqueCount="4"><si><t>name</t></si><si><t>age</t></si><si><t>Alice</t></si><si><t>Bob</t></si></sst>"#,
        ).unwrap();

        zip.start_file("xl/worksheets/sheet1.xml", opts).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData><row r="1"><c r="A1" t="s"><v>0</v></c><c r="B1" t="s"><v>1</v></c></row><row r="2"><c r="A2" t="s"><v>2</v></c><c r="B2" t="n"><v>30</v></c></row><row r="3"><c r="A3" t="s"><v>3</v></c><c r="B3" t="n"><v>25</v></c></row></sheetData></worksheet>"#,
        ).unwrap();

        zip.finish().unwrap();
    }

    fn write_minimal_xlsx_with_sheets(path: &Path, names: &[&str]) {
        let file = fs::File::create(path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let opts: zip::write::SimpleFileOptions = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("[Content_Types].xml", opts).unwrap();
        let mut types = String::from(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>"#,
        );
        for i in 1..=names.len() {
            types.push_str(&format!(
                r#"<Override PartName="/xl/worksheets/sheet{}.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>"#,
                i
            ));
        }
        types.push_str("</Types>");
        zip.write_all(types.as_bytes()).unwrap();

        zip.start_file("_rels/.rels", opts).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>"#,
        ).unwrap();

        zip.start_file("xl/_rels/workbook.xml.rels", opts).unwrap();
        let mut rels = String::from(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
        );
        for i in 1..=names.len() {
            rels.push_str(&format!(
                r#"<Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet{}.xml"/>"#,
                i, i
            ));
        }
        rels.push_str("</Relationships>");
        zip.write_all(rels.as_bytes()).unwrap();

        zip.start_file("xl/workbook.xml", opts).unwrap();
        let mut wb = String::from(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets>"#,
        );
        for (i, name) in names.iter().enumerate() {
            wb.push_str(&format!(
                r#"<sheet name="{}" sheetId="{}" r:id="rId{}"/>"#,
                name,
                i + 1,
                i + 1
            ));
        }
        wb.push_str("</sheets></workbook>");
        zip.write_all(wb.as_bytes()).unwrap();

        for i in 1..=names.len() {
            zip.start_file(format!("xl/worksheets/sheet{}.xml", i), opts)
                .unwrap();
            zip.write_all(
                br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData><row r="1"><c r="A1" t="inlineStr"><is><t>cell</t></is></c></row></sheetData></worksheet>"#,
            ).unwrap();
        }

        zip.finish().unwrap();
    }

    fn write_minimal_xlsx_with_rows(path: &Path, total_rows: usize) {
        let file = fs::File::create(path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let opts: zip::write::SimpleFileOptions = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("[Content_Types].xml", opts).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/><Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/></Types>"#,
        ).unwrap();

        zip.start_file("_rels/.rels", opts).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>"#,
        ).unwrap();

        zip.start_file("xl/_rels/workbook.xml.rels", opts).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/></Relationships>"#,
        ).unwrap();

        zip.start_file("xl/workbook.xml", opts).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="Sheet1" sheetId="1" r:id="rId1"/></sheets></workbook>"#,
        ).unwrap();

        let mut sheet = String::from(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData>"#,
        );
        sheet.push_str(r#"<row r="1"><c r="A1" t="inlineStr"><is><t>row</t></is></c></row>"#);
        for r in 2..=total_rows {
            sheet.push_str(&format!(
                r#"<row r="{}"><c r="A{}" t="n"><v>{}</v></c></row>"#,
                r, r, r
            ));
        }
        sheet.push_str("</sheetData></worksheet>");
        zip.start_file("xl/worksheets/sheet1.xml", opts).unwrap();
        zip.write_all(sheet.as_bytes()).unwrap();

        zip.finish().unwrap();
    }
}
