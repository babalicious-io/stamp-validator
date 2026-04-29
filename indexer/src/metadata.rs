//! Stamp metadata assembly, validation, and JSON serialisation.
//!
//! This module is the orchestration layer of the indexer. It validates the
//! transaction hash, calls the transaction parser and media detector, then
//! assembles every piece of known information into a flat [`MetadataField`]
//! list that JavaScript can render directly without further processing.
//!
//! The "stamp hash" produced here is a 20-character base62 string that matches
//! the btc_stamps indexer's own hash algorithm: SHA-256 of
//! `"<txhash>|<block_index>"`, big-integer-divided into base62.

use crate::media::{media_from_payload, MediaResult};
use crate::tx::{parse_transaction, ParsedTransaction};
use md5::Md5;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

// ===================================================================
//   PUBLIC TYPES
// ===================================================================

/// JSON input sent from JavaScript when calling `index_stamp`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LookupInput {
    /// 64-character hex transaction hash (normalised to lowercase internally).
    pub tx_hash: String,
    /// Name of the data provider that supplied the raw transaction.
    pub provider: String,
    /// Hex-encoded raw transaction bytes.
    pub raw_tx_hex: String,
    /// Optional provider context (block height, fee, vin/vout data, etc.).
    #[serde(default)]
    pub context: Value,
}

/// The top-level JSON response returned to JavaScript after indexing a stamp.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StampResult {
    /// `true` if the lookup completed without an error (stamp may still be absent).
    pub ok: bool,
    /// Human-readable summary of the result.
    pub message: String,
    /// Name of the data provider used for this lookup.
    pub provider: Option<String>,
    /// `true` when the transaction structure matches a known stamp encoding.
    pub has_valid_pattern: bool,
    /// `true` when a valid stamp payload was successfully extracted.
    pub has_valid_data: bool,
    /// Ordered list of metadata fields for display.
    pub metadata: Vec<MetadataField>,
    /// Media content derived from the stamp payload.
    pub media: MediaResult,
    /// Parsed SRC protocol object (SRC-20 / SRC-721 / SRC-101), if present.
    pub src_protocol: Option<Value>,
}

/// A single labelled metadata field with its value and provenance.
#[derive(Debug, Serialize)]
pub struct MetadataField {
    /// Machine-readable identifier (e.g. `"block_index"`, `"fee_sats"`).
    pub key: String,
    /// Human-readable display label.
    pub label: String,
    /// Field value, or `null` when the data is unavailable.
    pub value: Value,
    /// Where the value came from (e.g. `"provider context"`, `"payload"`).
    pub source: String,
}

impl StampResult {
    /// Constructs a failed [`StampResult`] with all fields empty except the
    /// error message. Used when the lookup cannot proceed.
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

    /// Serialises `self` to a JSON string. Returns a minimal error JSON if
    /// serialisation itself fails (should never happen in practice).
    pub fn to_json(self) -> String {
        serde_json::to_string(&self).unwrap_or_else(|_| {
            "{\"ok\":false,\"message\":\"Failed to serialize stamp result.\"}".to_string()
        })
    }
}

// ===================================================================
//   ENTRY POINT
// ===================================================================

/// Validates the transaction hash, parses the raw transaction, detects the
/// stamp payload media, assembles all metadata fields, and serialises the
/// result to JSON.
pub fn index_stamp_from_input(input: &LookupInput) -> Result<String, String> {
    validate_tx_hash(&input.tx_hash)?;
    let parsed = parse_transaction(&input.raw_tx_hex)?;
    let (media, src_protocol) = parsed
        .payload
        .as_deref()
        .map(media_from_payload)
        .unwrap_or_else(|| (MediaResult::empty(), None));

    let metadata = build_metadata(
        input,
        &parsed,
        &media,
        parsed.payload.as_deref(),
        src_protocol.as_ref(),
    );
    let message = if parsed.has_valid_data {
        "Transaction hash has been processed and a Bitcoin Stamp was found.".to_string()
    } else if parsed.has_valid_pattern {
        "Stamp-like transaction pattern found, but no valid Bitcoin Stamp metadata was extracted."
            .to_string()
    } else {
        "Transaction hash does not contain a valid Bitcoin Stamp.".to_string()
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

// ===================================================================
//   VALIDATION
// ===================================================================

/// Validates that `value` is exactly 64 hexadecimal characters (trimmed).
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

// ===================================================================
//   METADATA FIELDS
// ===================================================================

/// Assembles the ordered list of [`MetadataField`] entries for the result card.
/// Fields are always present in the list; unavailable values are `null`.
fn build_metadata(
    input: &LookupInput,
    parsed: &ParsedTransaction,
    media: &MediaResult,
    payload: Option<&[u8]>,
    src_protocol: Option<&Value>,
) -> Vec<MetadataField> {
    let mut fields = Vec::new();
    let block_index = value_from_path(&input.context, &["status", "block_height"]);
    let block_time = value_from_path(&input.context, &["status", "block_time"]);
    let fee_sats = value_from_path(&input.context, &["localTxStats", "fee_sats"]);
    let provider_vsize = value_from_path(&input.context, &["localTxStats", "vsize"]);
    let confirmations = confirmations_from_context(&input.context);
    let stamp_hash = stamp_hash_from_context(&input.tx_hash, &block_index);
    let file_hash = payload.map(md5_hex).map(Value::from).unwrap_or(Value::Null);

    push(&mut fields, "block_index", "Block Index", block_index, "provider context");
    push(&mut fields, "tick", "Token Ticker", tick_from_protocol(src_protocol), "payload or chain database");
    push(&mut fields, "src20_operation", "Transaction Type", src20_operation_from_protocol(src_protocol), "payload");
    push(&mut fields, "creator", creator_label_from_protocol(src_protocol), creator_from_context(&input.context), "provider context");
    push(&mut fields, "receiver", "Receiver Addy", receiver_from_context(src_protocol, &input.context), "provider context");
    push(&mut fields, "keyburn", "Keyburn", Value::from(parsed.keyburn), "transaction parser");
    push(&mut fields, "block_time", "Block Time", block_time, "provider context");
    push(&mut fields, "fee_sats", "Fee", fee_sats, "provider context");
    push(
        &mut fields,
        "vsize",
        "Virtual Size",
        if provider_vsize.is_null() {
            Value::from(parsed.vsize)
        } else {
            provider_vsize
        },
        "transaction parser or provider context",
    );
    push(&mut fields, "tx_hash", "Transaction Hash", Value::from(input.tx_hash.to_ascii_lowercase()), "user input");
    push(&mut fields, "confirmations", "Confirmations", confirmations, "provider context");
    push(&mut fields, "ident", "Identifier", ident_from_media_or_protocol(media, src_protocol), "payload");
    push(&mut fields, "stamp_hash", "Stamp Hash", stamp_hash, "local hash rule from btc_stamps indexer");
    push(&mut fields, "stamp_mimetype", "File Type", opt_string(media.mimetype.clone()), "payload");
    push(&mut fields, "html_title", "Title", opt_string(media.html_title.clone()), "payload HTML");
    push(&mut fields, "html_author", "Artist", opt_string(media.html_author.clone()), "payload HTML");
    push(&mut fields, "file_hash", "File Hash", file_hash, "payload md5");
    push(&mut fields, "file_size_bytes", "File Size", media.file_size_bytes.map(Value::from).unwrap_or(Value::Null), "payload");
    push(&mut fields, "encoding_method", "Encoding Method", opt_string(parsed.encoding_method.clone()), "transaction parser");
    push(&mut fields, "is_btc_stamp", "Valid Bitcoin Stamp", Value::from(parsed.has_valid_data), "transaction parser");
    push(&mut fields, "is_valid_base64", "Valid Base64 code", Value::from(media.is_valid_base64), "payload");
    push(&mut fields, "stamp_base64", "Base64 Image", opt_string(media.base64.clone()), "payload");

    fields
}

/// Appends a single [`MetadataField`] to `fields`.
fn push(fields: &mut Vec<MetadataField>, key: &str, label: &str, value: Value, source: &str) {
    fields.push(MetadataField {
        key: key.to_string(),
        label: label.to_string(),
        value,
        source: source.to_string(),
    });
}

// ===================================================================
//   CONTEXT HELPERS
// ===================================================================

/// Traverses `value` along `path` keys and returns the nested value, or
/// `Value::Null` if any key is missing.
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

/// Resolves the confirmation count from the provider context. Tries several
/// known field paths and, as a fallback, computes it from block height and
/// chain tip height.
fn confirmations_from_context(context: &Value) -> Value {
    if let Some(confirmations) = [
        &["status", "confirmations"][..],
        &["localTxStats", "confirmations"][..],
        &["confirmations"][..],
    ]
    .iter()
    .find_map(|path| {
        let value = value_from_path(context, path);
        if value.is_null() { None } else { Some(value) }
    }) {
        return confirmations;
    }

    let block_height = value_from_path(context, &["status", "block_height"]);
    let chain_tip_height = value_from_path(context, &["localTxStats", "chain_tip_height"]);

    match (value_to_u64(&block_height), value_to_u64(&chain_tip_height)) {
        (Some(block_height), Some(chain_tip_height)) if chain_tip_height >= block_height => {
            Value::from(chain_tip_height - block_height + 1)
        }
        _ => Value::Null,
    }
}

/// Extracts a `u64` from a JSON value that may be a number, a negative integer,
/// or a string representation.
fn value_to_u64(value: &Value) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| value.as_i64().and_then(|number| u64::try_from(number).ok()))
        .or_else(|| value.as_str().and_then(|text| text.parse::<u64>().ok()))
}

/// Returns the creator address from the first input's previous output, as
/// supplied by the provider context `vin[0].prevout.scriptpubkey_address`.
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

/// For SRC-20 TRANSFER transactions, returns the first output address that
/// differs from the sender. Returns `null` for all other operation types.
fn receiver_from_context(src_protocol: Option<&Value>, context: &Value) -> Value {
    if !is_src20_transfer(src_protocol) {
        return Value::Null;
    }

    let sender = creator_from_context(context)
        .as_str()
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();

    context
        .get("vout")
        .and_then(Value::as_array)
        .and_then(|outputs| {
            outputs.iter().find_map(|output| {
                let address = output_address(output)?;
                if !sender.is_empty() && address.eq_ignore_ascii_case(&sender) {
                    return None;
                }
                Some(Value::from(address.to_string()))
            })
        })
        .unwrap_or(Value::Null)
}

/// Resolves an output's Bitcoin address from several provider-specific field
/// names (`scriptpubkey_address`, `address`, `addr`, `addresses[0]`,
/// `scriptPubKey.addresses[0]`).
fn output_address(output: &Value) -> Option<&str> {
    output
        .get("scriptpubkey_address")
        .or_else(|| output.get("address"))
        .or_else(|| output.get("addr"))
        .and_then(Value::as_str)
        .or_else(|| {
            output
                .get("addresses")
                .and_then(Value::as_array)
                .and_then(|addresses| addresses.first())
                .and_then(Value::as_str)
        })
        .or_else(|| {
            output
                .get("scriptPubKey")
                .and_then(|script| script.get("addresses"))
                .and_then(Value::as_array)
                .and_then(|addresses| addresses.first())
                .and_then(Value::as_str)
        })
}

// ===================================================================
//   PROTOCOL HELPERS
// ===================================================================

/// Returns the `tick` field from a SRC protocol object, or `null`.
fn tick_from_protocol(src_protocol: Option<&Value>) -> Value {
    src_protocol
        .and_then(|value| value.get("tick"))
        .cloned()
        .unwrap_or(Value::Null)
}

/// Returns the SRC-20 operation type (`"deploy"`, `"mint"`, `"transfer"`) as a
/// JSON string, or `null` for non-SRC-20 stamps.
fn src20_operation_from_protocol(src_protocol: Option<&Value>) -> Value {
    src20_operation(src_protocol)
        .map(Value::from)
        .unwrap_or(Value::Null)
}

/// Chooses the creator field label based on the SRC-20 operation type.
fn creator_label_from_protocol(src_protocol: Option<&Value>) -> &'static str {
    if !is_src20_protocol(src_protocol) {
        return "Artist Addy";
    }

    match src20_operation(src_protocol) {
        Some(operation) if operation.eq_ignore_ascii_case("transfer") => "Sender Addy",
        Some(operation) if operation.eq_ignore_ascii_case("deploy") => "Creator Addy",
        Some(operation) if operation.eq_ignore_ascii_case("mint") => "Mint Addy",
        _ => "Artist Addy",
    }
}

/// Returns the identifier string: the SRC protocol name (uppercased) if
/// present, `"STAMP"` for media stamps, or `null` if no payload was found.
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

/// Returns `true` when `src_protocol` is a SRC-20 TRANSFER operation.
fn is_src20_transfer(src_protocol: Option<&Value>) -> bool {
    is_src20_protocol(src_protocol)
        && src20_operation(src_protocol)
            .is_some_and(|operation| operation.eq_ignore_ascii_case("transfer"))
}

/// Returns `true` when `src_protocol` has a `"p"` or `"P"` field equal to
/// `"SRC-20"` (case-insensitive).
fn is_src20_protocol(src_protocol: Option<&Value>) -> bool {
    src_protocol
        .and_then(|value| value.get("p").or_else(|| value.get("P")))
        .and_then(Value::as_str)
        .is_some_and(|protocol| protocol.eq_ignore_ascii_case("SRC-20"))
}

/// Returns the `"op"` or `"OP"` field value from a SRC protocol object.
fn src20_operation(src_protocol: Option<&Value>) -> Option<&str> {
    src_protocol
        .and_then(|value| value.get("op").or_else(|| value.get("OP")))
        .and_then(Value::as_str)
}

// ===================================================================
//   HASHING
// ===================================================================

/// Computes the stamp hash: a 20-character base62 string derived from
/// SHA-256(`"<tx_hash>|<block_index>"`). Matches the btc_stamps indexer
/// algorithm. Returns `null` when `block_index` is unavailable.
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

/// SHA-256 hashes `"<first>|<second>"`, then big-integer-divides the 32-byte
/// digest into base62, and returns the first `length` characters. This mirrors
/// the btc_stamps indexer's stamp hash derivation exactly.
fn create_base62_hash(first: &str, second: &str, length: usize) -> String {
    let combined = format!("{first}|{second}");
    let hash = Sha256::digest(combined.as_bytes());
    let encoded = base62_encode_bytes(&hash);
    encoded.chars().take(length).collect()
}

/// Encodes an arbitrary byte slice as a base62 string using repeated big-integer
/// division. The 256-possible-values-per-byte input cannot be mapped to base62
/// with a simple lookup table, so each character is derived by dividing the
/// entire remaining value and taking the remainder.
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

/// Returns the MD5 digest of `bytes` as a lowercase hex string.
fn md5_hex(bytes: &[u8]) -> String {
    let digest = Md5::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

// ===================================================================
//   SERIALISATION UTILITIES
// ===================================================================

/// Converts `Option<String>` to a JSON `Value::String` or `Value::Null`.
fn opt_string(value: Option<String>) -> Value {
    value.map(Value::from).unwrap_or(Value::Null)
}

// ===================================================================
//   TESTS
// ===================================================================

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
