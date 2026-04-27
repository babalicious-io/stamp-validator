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

    Some(MediaResult {
        kind: media_kind(&mimetype),
        mimetype: Some(mimetype.clone()),
        data_url: Some(format!("data:{mimetype};base64,{body}")),
        text: None,
        base64: Some(body.to_string()),
        is_valid_base64: true,
        file_size_bytes: decoded_base64_size(body),
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
    })
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
            media_from_payload(b"<!DOCTYPE html><html><body>Stamp</body></html>");

        assert_eq!(media.kind, "html");
        assert_eq!(media.mimetype.as_deref(), Some("text/html"));
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
}
