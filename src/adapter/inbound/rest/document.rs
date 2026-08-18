use std::io::{Cursor, Read};

pub fn extract_text(file_name: &str, bytes: &[u8], max_chars: usize) -> anyhow::Result<String> {
    let extension = file_name.rsplit('.').next().unwrap_or_default().to_ascii_lowercase();
    let text = match extension.as_str() {
        "txt" => std::str::from_utf8(bytes)
            .map_err(|_| anyhow::anyhow!("TXT files must be UTF-8 encoded"))?
            .to_string(),
        "docx" => extract_docx(bytes)?,
        "doc" => anyhow::bail!("Legacy .doc files are not supported; convert the document to .docx"),
        _ => anyhow::bail!("Only .txt and .docx files are supported"),
    };
    if text.trim().is_empty() { anyhow::bail!("Document does not contain text"); }
    if text.chars().count() > max_chars { anyhow::bail!("Document text exceeds the configured limit"); }
    Ok(text)
}

fn extract_docx(bytes: &[u8]) -> anyhow::Result<String> {
    if !bytes.starts_with(b"PK") { anyhow::bail!("DOCX file does not have a ZIP signature"); }
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|_| anyhow::anyhow!("DOCX file is not a valid ZIP archive"))?;
    let mut xml = String::new();
    archive.by_name("word/document.xml")
        .map_err(|_| anyhow::anyhow!("DOCX file does not contain word/document.xml"))?
        .read_to_string(&mut xml)
        .map_err(|_| anyhow::anyhow!("DOCX document XML is not valid UTF-8"))?;
    let mut text = String::new();
    let mut rest = xml.as_str();
    while let Some(start) = rest.find("<w:t") {
        let after_tag = &rest[start..];
        let Some(content_start) = after_tag.find('>') else { break };
        let content = &after_tag[content_start + 1..];
        let Some(end) = content.find("</w:t>") else { break };
        text.push_str(&xml_unescape(&content[..end]));
        text.push(' ');
        rest = &content[end + 6..];
    }
    Ok(text)
}

fn xml_unescape(value: &str) -> String {
    value.replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&apos;", "'")
}
