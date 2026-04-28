use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaResult {
    pub kind: String,
    pub mimetype: Option<String>,
    pub data_url: Option<String>,
    pub text: Option<String>,
    pub base64: Option<String>,
    pub is_valid_base64: bool,
    pub file_size_bytes: Option<usize>,
    pub html_title: Option<String>,
    pub html_author: Option<String>,
}

impl MediaResult {
    pub fn empty() -> Self {
        Self {
            kind: "none".to_string(),
            mimetype: None,
            data_url: None,
            text: None,
            base64: None,
            is_valid_base64: false,
            file_size_bytes: None,
            html_title: None,
            html_author: None,
        }
    }
}

pub fn media_from_payload(payload: &[u8]) -> (MediaResult, Option<Value>, Option<String>) {
    let text = String::from_utf8_lossy(payload).trim().to_string();
    let json = serde_json::from_str::<Value>(&text).ok();

    if let Some(value) = json.as_ref() {
        if let Some(description) = value.get("description").and_then(Value::as_str) {
            if let Some(media) = parse_data_url(description) {
                return (media, json, Some(text));
            }
        }

        if let Some(media) = parse_src_protocol_media(value) {
            return (media, json, Some(text));
        }

        return (
            MediaResult {
                kind: "json".to_string(),
                mimetype: Some("application/json".to_string()),
                data_url: None,
                text: Some(text.clone()),
                base64: None,
                is_valid_base64: false,
                file_size_bytes: Some(text.len()),
                html_title: None,
                html_author: None,
            },
            json,
            Some(text),
        );
    }

    if let Some(media) = parse_data_url(&text) {
        return (media, None, Some(text));
    }

    let mimetype = sniff_mimetype(payload);
    let kind = media_kind(&mimetype);
    let has_renderable_data_url = matches!(kind.as_str(), "image" | "html");
    let encoded_media = if has_renderable_data_url {
        Some(base64_encode(payload))
    } else {
        None
    };
    let html_metadata = if kind == "html" {
        extract_html_metadata(&text)
    } else {
        HtmlMetadata::empty()
    };

    (
        MediaResult {
            kind,
            mimetype: Some(mimetype),
            data_url: encoded_media
                .as_ref()
                .map(|base64| format!("data:{};base64,{}", sniff_mimetype(payload), base64)),
            text: if has_renderable_data_url {
                None
            } else {
                Some(text.clone())
            },
            base64: encoded_media,
            is_valid_base64: has_renderable_data_url,
            file_size_bytes: Some(payload.len()),
            html_title: html_metadata.title,
            html_author: html_metadata.author,
        },
        None,
        Some(text),
    )
}

fn parse_data_url(description: &str) -> Option<MediaResult> {
    let data_start = description.find("data:")?;
    let data = &description[data_start..];
    let comma = data.find(',')?;
    let header = &data[..comma];
    let body = data[comma + 1..]
        .split(['"', '\'', ')', '<', '>'])
        .next()
        .unwrap_or("")
        .trim();

    if !header.contains(";base64") || body.is_empty() || !is_valid_base64(body) {
        return None;
    }

    let mimetype = header
        .strip_prefix("data:")
        .and_then(|value| value.split(';').next())
        .filter(|value| !value.is_empty())
        .unwrap_or("application/octet-stream")
        .to_string();

    let html_metadata = if media_kind(&mimetype) == "html" {
        base64_decode(body)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .map(|html| extract_html_metadata(&html))
            .unwrap_or_else(HtmlMetadata::empty)
    } else {
        HtmlMetadata::empty()
    };

    Some(MediaResult {
        kind: media_kind(&mimetype),
        mimetype: Some(mimetype.clone()),
        data_url: Some(format!("data:{mimetype};base64,{body}")),
        text: None,
        base64: Some(body.to_string()),
        is_valid_base64: true,
        file_size_bytes: decoded_base64_size(body),
        html_title: html_metadata.title,
        html_author: html_metadata.author,
    })
}

fn parse_src_protocol_media(value: &Value) -> Option<MediaResult> {
    let protocol = value
        .get("p")
        .or_else(|| value.get("P"))
        .and_then(Value::as_str)?
        .to_ascii_uppercase();

    if !matches!(protocol.as_str(), "SRC-20" | "SRC-721" | "SRC-101") {
        return None;
    }

    Some(MediaResult {
        kind: "json".to_string(),
        mimetype: Some("application/json".to_string()),
        data_url: None,
        text: Some(value.to_string()),
        base64: None,
        is_valid_base64: false,
        file_size_bytes: Some(value.to_string().len()),
        html_title: None,
        html_author: None,
    })
}

struct HtmlMetadata {
    title: Option<String>,
    author: Option<String>,
}

impl HtmlMetadata {
    fn empty() -> Self {
        Self {
            title: None,
            author: None,
        }
    }
}

fn extract_html_metadata(html: &str) -> HtmlMetadata {
    HtmlMetadata {
        title: extract_between_case_insensitive(html, "<title", "</title>").and_then(|value| {
            value
                .split_once('>')
                .map(|(_, title)| html_unescape(title.trim()))
                .filter(|title| !title.is_empty())
        }),
        author: extract_meta_author(html),
    }
}

fn extract_meta_author(html: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let mut cursor = 0;

    while let Some(relative_start) = lower[cursor..].find("<meta") {
        let start = cursor + relative_start;
        let end = lower[start..]
            .find('>')
            .map(|relative_end| start + relative_end)
            .unwrap_or(html.len());
        let tag = &html[start..end];
        let tag_lower = &lower[start..end];

        if tag_lower.contains("name=\"author\"")
            || tag_lower.contains("name='author'")
            || tag_lower.contains("name=author")
        {
            if let Some(author) = extract_attribute(tag, "content") {
                return Some(author);
            }
        }

        cursor = end.saturating_add(1);
    }

    None
}

fn extract_between_case_insensitive(
    value: &str,
    start_pattern: &str,
    end_pattern: &str,
) -> Option<String> {
    let lower = value.to_ascii_lowercase();
    let start = lower.find(start_pattern)?;
    let content_start = start + start_pattern.len();
    let end = lower[content_start..].find(end_pattern)? + content_start;
    Some(value[start..end].to_string())
}

fn extract_attribute(tag: &str, attr_name: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let pattern = format!("{attr_name}=");
    let start = lower.find(&pattern)? + pattern.len();
    let mut chars = tag[start..].chars();
    let quote = chars.next()?;

    let raw_value = if quote == '"' || quote == '\'' {
        let rest = &tag[start + quote.len_utf8()..];
        rest.split(quote).next()?.trim()
    } else {
        tag[start..]
            .split(|character: char| character.is_ascii_whitespace() || character == '>')
            .next()?
            .trim()
    };

    let value = html_unescape(raw_value);
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn html_unescape(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

fn sniff_mimetype(payload: &[u8]) -> String {
    if payload.starts_with(b"\x89PNG\r\n\x1a\n") {
        "image/png".to_string()
    } else if payload.starts_with(&[0xff, 0xd8, 0xff]) {
        "image/jpeg".to_string()
    } else if payload.starts_with(b"GIF87a") || payload.starts_with(b"GIF89a") {
        "image/gif".to_string()
    } else if looks_like_html(payload) {
        "text/html".to_string()
    } else if payload
        .iter()
        .take(128)
        .all(|byte| byte.is_ascii_graphic() || byte.is_ascii_whitespace())
    {
        "text/plain".to_string()
    } else {
        "application/octet-stream".to_string()
    }
}

fn looks_like_html(payload: &[u8]) -> bool {
    let text = String::from_utf8_lossy(payload);
    let trimmed = text.trim_start().to_ascii_lowercase();

    trimmed.starts_with("<!doctype html")
        || trimmed.starts_with("<html")
        || trimmed.starts_with("<head")
        || trimmed.starts_with("<body")
}

fn media_kind(mimetype: &str) -> String {
    if mimetype.starts_with("image/") {
        "image".to_string()
    } else if mimetype == "text/html" {
        "html".to_string()
    } else if mimetype == "application/json" {
        "json".to_string()
    } else if mimetype.starts_with("text/") {
        "text".to_string()
    } else {
        "binary".to_string()
    }
}

fn is_valid_base64(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'='))
}

fn decoded_base64_size(value: &str) -> Option<usize> {
    if !is_valid_base64(value) {
        return None;
    }

    let padding = value
        .as_bytes()
        .iter()
        .rev()
        .take_while(|byte| **byte == b'=')
        .count();
    Some((value.len() / 4) * 3 - padding)
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(bytes.len().div_ceil(3) * 4);

    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = *chunk.get(1).unwrap_or(&0);
        let third = *chunk.get(2).unwrap_or(&0);

        output.push(TABLE[(first >> 2) as usize] as char);
        output.push(TABLE[(((first & 0b0000_0011) << 4) | (second >> 4)) as usize] as char);

        if chunk.len() > 1 {
            output.push(TABLE[(((second & 0b0000_1111) << 2) | (third >> 6)) as usize] as char);
        } else {
            output.push('=');
        }

        if chunk.len() > 2 {
            output.push(TABLE[(third & 0b0011_1111) as usize] as char);
        } else {
            output.push('=');
        }
    }

    output
}

fn base64_decode(value: &str) -> Result<Vec<u8>, String> {
    if !is_valid_base64(value) || value.len() % 4 != 0 {
        return Err("Invalid base64 input.".to_string());
    }

    let mut output = Vec::with_capacity((value.len() / 4) * 3);
    for chunk in value.as_bytes().chunks(4) {
        let first = base64_value(chunk[0])?;
        let second = base64_value(chunk[1])?;
        let third = if chunk[2] == b'=' {
            0
        } else {
            base64_value(chunk[2])?
        };
        let fourth = if chunk[3] == b'=' {
            0
        } else {
            base64_value(chunk[3])?
        };
        let triple = ((first as u32) << 18)
            | ((second as u32) << 12)
            | ((third as u32) << 6)
            | fourth as u32;

        output.push(((triple >> 16) & 0xff) as u8);
        if chunk[2] != b'=' {
            output.push(((triple >> 8) & 0xff) as u8);
        }
        if chunk[3] != b'=' {
            output.push((triple & 0xff) as u8);
        }
    }

    Ok(output)
}

fn base64_value(byte: u8) -> Result<u8, String> {
    match byte {
        b'A'..=b'Z' => Ok(byte - b'A'),
        b'a'..=b'z' => Ok(byte - b'a' + 26),
        b'0'..=b'9' => Ok(byte - b'0' + 52),
        b'+' => Ok(62),
        b'/' => Ok(63),
        _ => Err("Invalid base64 character.".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_base64_data_url() {
        let media = parse_data_url("data:image/png;base64,QUJD").unwrap();
        assert_eq!(media.mimetype.as_deref(), Some("image/png"));
        assert_eq!(media.file_size_bytes, Some(3));
    }

    #[test]
    fn base64_encodes_raw_media() {
        assert_eq!(base64_encode(b"ABC"), "QUJD");
        assert_eq!(base64_encode(b"AB"), "QUI=");
        assert_eq!(base64_encode(b"A"), "QQ==");
    }

    #[test]
    fn renders_raw_html_payloads_as_iframe_media() {
        let (media, json, source_text) =
            media_from_payload(b"<!DOCTYPE html><html><head><title>Stamp Title</title><meta name=\"author\" content=\"Satoshi\"></head><body>Stamp</body></html>");

        assert_eq!(media.kind, "html");
        assert_eq!(media.mimetype.as_deref(), Some("text/html"));
        assert_eq!(media.html_title.as_deref(), Some("Stamp Title"));
        assert_eq!(media.html_author.as_deref(), Some("Satoshi"));
        assert!(media
            .data_url
            .as_deref()
            .unwrap_or("")
            .starts_with("data:text/html;base64,"));
        assert!(media.text.is_none());
        assert!(media.is_valid_base64);
        assert!(json.is_none());
        assert!(source_text.is_some());
    }

    #[test]
    fn extracts_html_metadata_from_base64_data_url() {
        let html = "<html><head><title>Encoded Stamp</title><meta content='Ada' name='author'></head></html>";
        let encoded = base64_encode(html.as_bytes());
        let media = parse_data_url(&format!("data:text/html;base64,{encoded}")).unwrap();

        assert_eq!(media.kind, "html");
        assert_eq!(media.html_title.as_deref(), Some("Encoded Stamp"));
        assert_eq!(media.html_author.as_deref(), Some("Ada"));
    }
}
