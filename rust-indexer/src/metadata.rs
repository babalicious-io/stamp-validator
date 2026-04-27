use crate::media::{media_from_payload, MediaResult};
use crate::tx::{parse_transaction, ParsedTransaction};
use md5::Md5;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LookupInput {
    pub tx_hash: String,
    pub provider: String,
    pub raw_tx_hex: String,
    #[serde(default)]
    pub context: Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StampResult {
    pub ok: bool,
    pub message: String,
    pub provider: Option<String>,
    pub has_valid_pattern: bool,
    pub has_valid_data: bool,
    pub metadata: Vec<MetadataField>,
    pub media: MediaResult,
    pub src_protocol: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct MetadataField {
    pub key: String,
    pub label: String,
    pub value: Value,
    pub source: String,
}

impl StampResult {
    pub fn error(message: String) -> Self {
        Self {
            ok: false,
            message,
            provider: None,
            has_valid_pattern: false,
            has_valid_data: false,
            metadata: Vec::new(),
            media: MediaResult::empty(),
            src_protocol: None,
        }
    }

    pub fn to_json(self) -> String {
        serde_json::to_string(&self).unwrap_or_else(|_| {
            "{\"ok\":false,\"message\":\"Failed to serialize stamp result.\"}".to_string()
        })
    }
}

pub fn index_stamp_from_input(input: &LookupInput) -> Result<String, String> {
    validate_tx_hash(&input.tx_hash)?;
    let parsed = parse_transaction(&input.raw_tx_hex)?;
    let (media, src_protocol, src_data) = parsed
        .payload
        .as_deref()
        .map(media_from_payload)
        .unwrap_or_else(|| (MediaResult::empty(), None, None));

    let metadata = build_metadata(
        input,
        &parsed,
        &media,
        parsed.payload.as_deref(),
        src_protocol.as_ref(),
        src_data.as_deref(),
    );
    let message = if parsed.has_valid_data {
        "Stamp payload processed locally in Rust/Wasm.".to_string()
    } else if parsed.has_valid_pattern {
        "Stamp-like transaction pattern found, but no valid stamp payload was extracted."
            .to_string()
    } else {
        "No Bitcoin Stamps payload was found in this transaction.".to_string()
    };

    Ok(StampResult {
        ok: true,
        message,
        provider: Some(input.provider.clone()),
        has_valid_pattern: parsed.has_valid_pattern,
        has_valid_data: parsed.has_valid_data,
        metadata,
        media,
        src_protocol,
    }
    .to_json())
}

fn validate_tx_hash(value: &str) -> Result<(), String> {
    let normalized = value.trim();
    if normalized.len() != 64 {
        return Err("Transaction hash must be exactly 64 hex characters.".to_string());
    }
    if !normalized.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("Transaction hash can only contain hexadecimal characters.".to_string());
    }
    Ok(())
}

fn build_metadata(
    input: &LookupInput,
    parsed: &ParsedTransaction,
    media: &MediaResult,
    payload: Option<&[u8]>,
    src_protocol: Option<&Value>,
    src_data: Option<&str>,
) -> Vec<MetadataField> {
    let mut fields = Vec::new();
    let block_index = value_from_path(&input.context, &["status", "block_height"]);
    let block_time = value_from_path(&input.context, &["status", "block_time"]);
    let stamp_hash = stamp_hash_from_context(&input.tx_hash, &block_index);
    let file_hash = payload.map(md5_hex).map(Value::from).unwrap_or(Value::Null);

    push(
        &mut fields,
        "stamp",
        "Stamp",
        Value::Null,
        "chain database required",
    );
    push(
        &mut fields,
        "block_index",
        "Block Index",
        block_index,
        "provider context",
    );
    push(
        &mut fields,
        "cpid",
        "CPID",
        cpid_from_protocol(src_protocol),
        "payload or chain database",
    );
    push(
        &mut fields,
        "asset_longname",
        "Asset Longname",
        string_from_protocol(src_protocol, "asset_longname"),
        "payload",
    );
    push(
        &mut fields,
        "creator",
        "Creator",
        creator_from_context(&input.context),
        "provider context",
    );
    push(
        &mut fields,
        "creator_name",
        "Creator Name",
        Value::Null,
        "chain database required",
    );
    push(
        &mut fields,
        "divisible",
        "Divisible",
        bool_from_protocol(src_protocol, "divisible"),
        "payload",
    );
    push(
        &mut fields,
        "keyburn",
        "Keyburn",
        Value::from(parsed.keyburn),
        "transaction parser",
    );
    push(
        &mut fields,
        "locked",
        "Locked",
        Value::Null,
        "chain database required",
    );
    push(
        &mut fields,
        "supply",
        "Supply",
        value_from_protocol(src_protocol, "quantity"),
        "payload",
    );
    push(
        &mut fields,
        "block_time",
        "Block Time",
        block_time,
        "provider context",
    );
    push(
        &mut fields,
        "tx_hash",
        "Transaction Hash",
        Value::from(input.tx_hash.to_ascii_lowercase()),
        "user input",
    );
    push(
        &mut fields,
        "tx_index",
        "Transaction Index",
        Value::Null,
        "chain database required",
    );
    push(
        &mut fields,
        "ident",
        "Identifier",
        ident_from_media_or_protocol(media, src_protocol),
        "payload",
    );
    push(
        &mut fields,
        "stamp_hash",
        "Stamp Hash",
        stamp_hash,
        "local hash rule from btc_stamps indexer",
    );
    push(
        &mut fields,
        "stamp_mimetype",
        "Stamp MIME Type",
        opt_string(media.mimetype.clone()),
        "payload",
    );
    push(
        &mut fields,
        "stamp_url",
        "Stamp URL",
        opt_string(media.data_url.clone()),
        "local embedded payload data URL",
    );
    push(
        &mut fields,
        "file_hash",
        "File Hash",
        file_hash,
        "payload md5",
    );
    push(
        &mut fields,
        "file_size_bytes",
        "File Size Bytes",
        media
            .file_size_bytes
            .map(Value::from)
            .unwrap_or(Value::Null),
        "payload",
    );
    push(
        &mut fields,
        "encoding_method",
        "Encoding Method",
        opt_string(parsed.encoding_method.clone()),
        "transaction parser",
    );
    push(
        &mut fields,
        "is_btc_stamp",
        "Is BTC Stamp",
        Value::from(parsed.has_valid_data),
        "transaction parser",
    );
    push(
        &mut fields,
        "is_cursed",
        "Is Cursed",
        Value::Null,
        "chain database required",
    );
    push(
        &mut fields,
        "is_valid_base64",
        "Is Valid Base64",
        Value::from(media.is_valid_base64),
        "payload",
    );
    push(
        &mut fields,
        "is_posh",
        "Is POSH",
        Value::Null,
        "chain database required",
    );
    push(
        &mut fields,
        "stamp_base64",
        "Stamp Base64",
        opt_string(media.base64.clone()),
        "payload",
    );
    push(
        &mut fields,
        "src_data",
        "SRC Data",
        src_data.map(Value::from).unwrap_or(Value::Null),
        "payload",
    );
    push(
        &mut fields,
        "local_txid",
        "Local Parser Transaction ID",
        Value::from(parsed.txid.clone()),
        "transaction parser",
    );
    push(
        &mut fields,
        "input_count",
        "Input Count",
        Value::from(parsed.inputs.len()),
        "transaction parser",
    );
    push(
        &mut fields,
        "output_count",
        "Output Count",
        Value::from(parsed.outputs.len()),
        "transaction parser",
    );
    push(
        &mut fields,
        "first_prev_txid",
        "First Previous TXID",
        parsed
            .inputs
            .first()
            .map(|input| Value::from(input.prev_txid.clone()))
            .unwrap_or(Value::Null),
        "transaction parser",
    );

    fields
}

fn push(fields: &mut Vec<MetadataField>, key: &str, label: &str, value: Value, source: &str) {
    fields.push(MetadataField {
        key: key.to_string(),
        label: label.to_string(),
        value,
        source: source.to_string(),
    });
}

fn value_from_path(value: &Value, path: &[&str]) -> Value {
    let mut current = value;
    for key in path {
        match current.get(*key) {
            Some(next) => current = next,
            None => return Value::Null,
        }
    }
    current.clone()
}

fn value_from_protocol(src_protocol: Option<&Value>, key: &str) -> Value {
    src_protocol
        .and_then(|value| value.get(key))
        .cloned()
        .unwrap_or(Value::Null)
}

fn string_from_protocol(src_protocol: Option<&Value>, key: &str) -> Value {
    src_protocol
        .and_then(|value| value.get(key))
        .and_then(Value::as_str)
        .map(Value::from)
        .unwrap_or(Value::Null)
}

fn bool_from_protocol(src_protocol: Option<&Value>, key: &str) -> Value {
    src_protocol
        .and_then(|value| value.get(key))
        .and_then(Value::as_bool)
        .map(Value::from)
        .unwrap_or(Value::Null)
}

fn cpid_from_protocol(src_protocol: Option<&Value>) -> Value {
    src_protocol
        .and_then(|value| value.get("cpid").or_else(|| value.get("tick")))
        .cloned()
        .unwrap_or(Value::Null)
}

fn creator_from_context(context: &Value) -> Value {
    context
        .get("vin")
        .and_then(Value::as_array)
        .and_then(|vin| vin.first())
        .and_then(|input| input.get("prevout"))
        .and_then(|prevout| prevout.get("scriptpubkey_address"))
        .cloned()
        .unwrap_or(Value::Null)
}

fn ident_from_media_or_protocol(media: &MediaResult, src_protocol: Option<&Value>) -> Value {
    if let Some(protocol) = src_protocol
        .and_then(|value| value.get("p").or_else(|| value.get("P")))
        .and_then(Value::as_str)
    {
        return Value::from(protocol.to_ascii_uppercase());
    }

    if media.kind != "none" {
        return Value::from("STAMP");
    }

    Value::Null
}

fn opt_string(value: Option<String>) -> Value {
    value.map(Value::from).unwrap_or(Value::Null)
}

fn stamp_hash_from_context(tx_hash: &str, block_index: &Value) -> Value {
    block_index_to_string(block_index)
        .map(|block| create_base62_hash(tx_hash, &block, 20))
        .map(Value::from)
        .unwrap_or(Value::Null)
}

fn block_index_to_string(value: &Value) -> Option<String> {
    if let Some(number) = value.as_u64() {
        return Some(number.to_string());
    }
    if let Some(number) = value.as_i64() {
        return Some(number.to_string());
    }
    value.as_str().map(ToString::to_string)
}

fn create_base62_hash(first: &str, second: &str, length: usize) -> String {
    let combined = format!("{first}|{second}");
    let hash = Sha256::digest(combined.as_bytes());
    let encoded = base62_encode_bytes(&hash);
    encoded.chars().take(length).collect()
}

fn base62_encode_bytes(bytes: &[u8]) -> String {
    const CHARS: &[u8; 62] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut number = bytes.to_vec();
    let mut result = Vec::new();

    while number.iter().any(|byte| *byte != 0) {
        let mut remainder = 0_u16;
        for byte in &mut number {
            let value = (remainder << 8) + *byte as u16;
            *byte = (value / 62) as u8;
            remainder = value % 62;
        }
        result.push(CHARS[remainder as usize] as char);
        while number.first() == Some(&0) {
            number.remove(0);
        }
    }

    if result.is_empty() {
        return "0".to_string();
    }

    result.iter().rev().collect()
}

fn md5_hex(bytes: &[u8]) -> String {
    let digest = Md5::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_tx_hash() {
        assert!(validate_tx_hash("a".repeat(64).as_str()).is_ok());
        assert!(validate_tx_hash("a".repeat(63).as_str()).is_err());
        assert!(validate_tx_hash("z".repeat(64).as_str()).is_err());
    }

    #[test]
    fn creates_indexer_compatible_base62_hash() {
        assert_eq!(
            create_base62_hash(
                "3619829bb3c92aad8ded6840bf544d441d14372c2bc8194b3b25d2a181821fcc",
                "915158",
                20
            ),
            "2n2JEV6Jj4mFm7mzl8JB"
        );
    }

    #[test]
    fn creates_md5_file_hash() {
        assert_eq!(md5_hex(b"ABC"), "902fbdd2b1df0c4f70b4a5d23525e932");
    }
}
