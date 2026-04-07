//! Generic text extraction from file bytes.
//!
//! Supports:
//! - **Text-based** formats (UTF-8 decode): `.txt`, `.md`, `.rs`, `.ts`, `.js`, etc.
//! - **PDF** → `pdf-extract` crate
//! - **DOCX** → ZIP with `word/document.xml` (Office Open XML)
//! - **XLSX** → ZIP with `xl/sharedStrings.xml` + `xl/worksheets/*.xml`
//! - **PPTX** → ZIP with `ppt/slides/*.xml`
//! - **RTF** → strip RTF control codes
//!
//! Returns `None` for unsupported binary formats (images, videos, archives).

use std::io::Read;

/// Extract readable text from raw file bytes.
///
/// `content_type` is the MIME type (e.g. `"application/pdf"`).
/// `name` is the filename, used as a fallback for format detection via extension.
pub fn extract(bytes: &[u8], content_type: &str, name: &str) -> Option<String> {
    let ext = name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();

    // Route by MIME type first, then fall back to extension
    match content_type {
        "application/pdf" => Some(extract_pdf(bytes)),
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
            Some(extract_docx(bytes))
        }
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => {
            Some(extract_xlsx(bytes))
        }
        "application/vnd.openxmlformats-officedocument.presentationml.presentation" => {
            Some(extract_pptx(bytes))
        }
        "application/rtf" | "text/rtf" => Some(extract_rtf(bytes)),
        ct if ct.starts_with("text/") => Some(extract_utf8(bytes)),
        "application/json" | "application/xml" | "application/javascript" => {
            Some(extract_utf8(bytes))
        }
        // Fall back to extension when MIME is generic
        _ => match ext.as_str() {
            "pdf" => Some(extract_pdf(bytes)),
            "docx" => Some(extract_docx(bytes)),
            "xlsx" => Some(extract_xlsx(bytes)),
            "pptx" => Some(extract_pptx(bytes)),
            "rtf" => Some(extract_rtf(bytes)),
            "txt" | "md" | "markdown" | "rs" | "ts" | "tsx" | "js" | "jsx" | "mjs" | "py"
            | "rb" | "go" | "java" | "c" | "h" | "cpp" | "cc" | "cxx" | "hpp" | "cs" | "swift"
            | "kt" | "kts" | "html" | "htm" | "css" | "xml" | "json" | "yaml" | "yml" | "toml"
            | "sql" | "sh" | "bash" | "zsh" | "csv" | "svg" | "svelte" | "vue" | "php" | "r"
            | "pl" | "lua" | "zig" | "nim" | "dart" | "scala" | "clj" | "ex" | "exs" | "erl"
            | "hs" | "ml" | "fs" | "fsx" | "tf" | "hcl" | "ini" | "cfg" | "conf" | "env"
            | "log" | "makefile" | "dockerfile" | "gitignore" | "editorconfig" | "prettierrc" => {
                Some(extract_utf8(bytes))
            }
            _ => None,
        },
    }
}

/// Decode bytes as UTF-8 text.
fn extract_utf8(bytes: &[u8]) -> String {
    match String::from_utf8(bytes.to_vec()) {
        Ok(text) => text,
        Err(_) => String::from_utf8_lossy(bytes).into_owned(),
    }
}

/// Extract text from a PDF using `pdf-extract`.
fn extract_pdf(bytes: &[u8]) -> String {
    match pdf_extract::extract_text_from_mem(bytes) {
        Ok(text) => {
            let trimmed = text.trim().to_string();
            if trimmed.is_empty() {
                "(PDF contained no extractable text — may be scanned/image-based)".to_string()
            } else {
                trimmed
            }
        }
        Err(e) => format!("(Failed to extract PDF text: {e})"),
    }
}

/// Extract text from a DOCX (Office Open XML word document).
///
/// DOCX is a ZIP archive containing `word/document.xml` with `<w:t>` text nodes.
fn extract_docx(bytes: &[u8]) -> String {
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = match zip::ZipArchive::new(cursor) {
        Ok(a) => a,
        Err(e) => return format!("(Failed to read DOCX archive: {e})"),
    };

    let mut xml = String::new();
    match archive.by_name("word/document.xml") {
        Ok(mut file) => {
            if file.read_to_string(&mut xml).is_err() {
                return "(Failed to read DOCX document.xml)".to_string();
            }
        }
        Err(_) => return "(DOCX archive missing word/document.xml)".to_string(),
    }

    strip_xml_to_text(&xml, &["w:t"])
}

/// Extract text from an XLSX (Office Open XML spreadsheet).
///
/// Reads shared strings from `xl/sharedStrings.xml` and inline strings from worksheets.
fn extract_xlsx(bytes: &[u8]) -> String {
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = match zip::ZipArchive::new(cursor) {
        Ok(a) => a,
        Err(e) => return format!("(Failed to read XLSX archive: {e})"),
    };

    let mut parts: Vec<String> = Vec::new();

    // Shared strings table
    if let Ok(mut file) = archive.by_name("xl/sharedStrings.xml") {
        let mut xml = String::new();
        if file.read_to_string(&mut xml).is_ok() {
            let text = strip_xml_to_text(&xml, &["t"]);
            if !text.is_empty() {
                parts.push(text);
            }
        }
    }

    // Worksheet content — collect names first to avoid borrow conflict
    let sheet_names: Vec<String> = (0..archive.len())
        .filter_map(|i| {
            let name = archive.by_index(i).ok()?.name().to_string();
            if name.starts_with("xl/worksheets/") && name.ends_with(".xml") {
                Some(name)
            } else {
                None
            }
        })
        .collect();

    for name in &sheet_names {
        if let Ok(mut file) = archive.by_name(name) {
            let mut xml = String::new();
            if file.read_to_string(&mut xml).is_ok() {
                let text = strip_xml_to_text(&xml, &["v", "t"]);
                if !text.is_empty() {
                    parts.push(text);
                }
            }
        }
    }

    if parts.is_empty() {
        "(XLSX contained no extractable text)".to_string()
    } else {
        parts.join("\n\n")
    }
}

/// Extract text from a PPTX (Office Open XML presentation).
///
/// Reads text from `ppt/slides/*.xml` `<a:t>` elements.
fn extract_pptx(bytes: &[u8]) -> String {
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = match zip::ZipArchive::new(cursor) {
        Ok(a) => a,
        Err(e) => return format!("(Failed to read PPTX archive: {e})"),
    };

    // Collect slide file names first
    let slide_names: Vec<String> = (0..archive.len())
        .filter_map(|i| {
            let name = archive.by_index(i).ok()?.name().to_string();
            if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
                Some(name)
            } else {
                None
            }
        })
        .collect();

    let mut parts: Vec<String> = Vec::new();
    for name in &slide_names {
        if let Ok(mut file) = archive.by_name(name) {
            let mut xml = String::new();
            if file.read_to_string(&mut xml).is_ok() {
                let text = strip_xml_to_text(&xml, &["a:t"]);
                if !text.is_empty() {
                    parts.push(text);
                }
            }
        }
    }

    if parts.is_empty() {
        "(PPTX contained no extractable text)".to_string()
    } else {
        parts.join("\n\n")
    }
}

/// Extract text from RTF by stripping control codes.
fn extract_rtf(bytes: &[u8]) -> String {
    let raw = String::from_utf8_lossy(bytes);
    let mut result = String::with_capacity(raw.len() / 2);
    let mut chars = raw.chars().peekable();
    let mut depth: i32 = 0;

    while let Some(ch) = chars.next() {
        match ch {
            '{' => depth += 1,
            '}' => depth -= 1,
            '\\' => {
                // Read the control word
                let mut word = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_alphabetic() {
                        word.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                // Skip optional numeric parameter
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() || c == '-' {
                        chars.next();
                    } else {
                        break;
                    }
                }
                // Skip trailing space
                if let Some(&' ') = chars.peek() {
                    chars.next();
                }
                match word.as_str() {
                    "par" | "line" => result.push('\n'),
                    "tab" => result.push('\t'),
                    _ => {}
                }
            }
            _ if depth <= 1 => result.push(ch),
            _ => {}
        }
    }

    let trimmed = result.trim().to_string();
    if trimmed.is_empty() {
        "(RTF contained no extractable text)".to_string()
    } else {
        trimmed
    }
}

/// Simple XML text extractor: collects inner text of elements matching any of the given tag names.
///
/// This is a lightweight approach that avoids pulling in a full XML parser dependency.
/// It handles the OOXML patterns we need (e.g., `<w:t>text</w:t>`, `<a:t>text</a:t>`).
fn strip_xml_to_text(xml: &str, tag_names: &[&str]) -> String {
    let mut result = String::new();
    for tag in tag_names {
        let open = format!("<{}", tag);
        let close = format!("</{}>", tag);
        let mut search_from = 0;

        while let Some(start_pos) = xml[search_from..].find(&open) {
            let abs_start = search_from + start_pos;
            // Find the end of the opening tag (handle attributes)
            if let Some(tag_end) = xml[abs_start..].find('>') {
                let content_start = abs_start + tag_end + 1;
                if let Some(end_pos) = xml[content_start..].find(&close) {
                    let text = &xml[content_start..content_start + end_pos];
                    if !text.is_empty() {
                        if !result.is_empty() {
                            result.push(' ');
                        }
                        result.push_str(text);
                    }
                    search_from = content_start + end_pos + close.len();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_utf8() {
        let text = "Hello, world!";
        let result = extract(text.as_bytes(), "text/plain", "hello.txt");
        assert_eq!(result, Some("Hello, world!".to_string()));
    }

    #[test]
    fn test_extract_json() {
        let json = r#"{"key": "value"}"#;
        let result = extract(json.as_bytes(), "application/json", "data.json");
        assert_eq!(result, Some(r#"{"key": "value"}"#.to_string()));
    }

    #[test]
    fn test_extract_unknown_binary() {
        let result = extract(&[0xFF, 0xD8, 0xFF], "image/jpeg", "photo.jpg");
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_by_extension_fallback() {
        let code = "fn main() {}";
        // MIME is generic but extension is .rs
        let result = extract(code.as_bytes(), "application/octet-stream", "main.rs");
        assert_eq!(result, Some("fn main() {}".to_string()));
    }

    #[test]
    fn test_strip_xml_to_text() {
        let xml = r#"<w:p><w:r><w:t>Hello</w:t></w:r><w:r><w:t> World</w:t></w:r></w:p>"#;
        let result = strip_xml_to_text(xml, &["w:t"]);
        assert_eq!(result, "Hello  World");
    }

    #[test]
    fn test_rtf_basic() {
        let rtf = r"{\rtf1\ansi Hello World\par Second line}";
        let result = extract_rtf(rtf.as_bytes());
        assert!(result.contains("Hello World"));
        assert!(result.contains("Second line"));
    }
}
