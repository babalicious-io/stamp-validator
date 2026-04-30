//! Bitcoin raw transaction parser.
//!
//! Decodes a hex-encoded raw transaction binary and extracts the Bitcoin Stamp
//! payload from two possible encoding methods:
//!
//! - **MULTISIG**: Data is hidden inside fake public keys in one or more
//!   `OP_CHECKMULTISIG` output scripts. Per output, only the **first two**
//!   pubkey pushes are used (`asm[1:3]` in btc_stamps); the third push in
//!   standard 1-of-3 stamp multisig is the dummy burn key and must be omitted.
//!   Inner bytes (`[1..len-1]`) from those keys are concatenated across outputs
//!   (vout order), then ARC4-decrypted once with the first input’s previous
//!   txid as the key — matching
//!   [btc_stamps](https://github.com/stampchain-io/btc_stamps)
//!   (`pubkeys_compiled` + `decode_checkmultisig`).
//! - **OLGA / P2WSH**: Data is spread across consecutive P2WSH
//!   (pay-to-witness-script-hash) output scripts starting at index 1 and
//!   concatenated in order.
//!
//! Also computes the transaction's virtual size (vsize) following BIP 141.

use crate::arc4;

const STAMP_PREFIX: &[u8] = b"stamp:";
const OP_CHECKMULTISIG: u8 = 0xae;

// ===================================================================
//   PUBLIC TYPES
// ===================================================================

/// The result of parsing a raw Bitcoin transaction.
#[derive(Debug, Clone)]
pub struct ParsedTransaction {
    /// Raw stamp payload bytes, if any were found.
    pub payload: Option<Vec<u8>>,
    /// `true` when the transaction structure matches a known stamp encoding
    /// (MULTISIG or OLGA/P2WSH), even if no valid `stamp:` payload was found.
    pub has_valid_pattern: bool,
    /// `true` when a valid stamp payload was successfully extracted.
    pub has_valid_data: bool,
    /// Number of keyburn outputs (1 when a stamp is confirmed, 0 otherwise).
    ///
    /// Follows the btc_stamps indexer convention: a stamp transaction burns
    /// exactly one output to an unspendable key as proof-of-immutability.
    pub keyburn: u32,
    /// Which encoding method was detected (`"MULTISIG"` or `"OLGA/P2WSH"`).
    pub encoding_method: Option<String>,
    /// Virtual size in vbytes, calculated per BIP 141.
    pub vsize: usize,
}

// ===================================================================
//   CURSOR (STATEFUL BYTE READER)
// ===================================================================

/// A stateful cursor over a borrowed byte slice. Advances an internal position
/// on each read and returns typed values decoded from the Bitcoin transaction
/// binary format (little-endian integers and compact-size varints).
struct Cursor<'a> {
    bytes: &'a [u8],
    index: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, index: 0 }
    }

    fn read_u8(&mut self) -> Result<u8, String> {
        let byte = *self
            .bytes
            .get(self.index)
            .ok_or_else(|| "Unexpected end of transaction.".to_string())?;
        self.index += 1;
        Ok(byte)
    }

    fn read_exact(&mut self, len: usize) -> Result<&'a [u8], String> {
        let end = self.index + len;
        let slice = self
            .bytes
            .get(self.index..end)
            .ok_or_else(|| "Unexpected end of transaction.".to_string())?;
        self.index = end;
        Ok(slice)
    }

    fn read_u32_le(&mut self) -> Result<u32, String> {
        let bytes = self.read_exact(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_u64_le(&mut self) -> Result<u64, String> {
        let bytes = self.read_exact(8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    /// Reads a Bitcoin compact-size (varint) value.
    fn read_varint(&mut self) -> Result<u64, String> {
        match self.read_u8()? {
            value @ 0x00..=0xfc => Ok(value as u64),
            0xfd => {
                let bytes = self.read_exact(2)?;
                Ok(u16::from_le_bytes([bytes[0], bytes[1]]) as u64)
            }
            0xfe => Ok(self.read_u32_le()? as u64),
            0xff => self.read_u64_le(),
        }
    }
}

// ===================================================================
//   PUBLIC API
// ===================================================================

/// Parses a hex-encoded raw Bitcoin transaction and extracts any Bitcoin Stamp
/// payload along with transaction metadata (encoding method and vsize).
pub fn parse_transaction(raw_tx_hex: &str) -> Result<ParsedTransaction, String> {
    let bytes = hex_to_bytes(raw_tx_hex)?;
    let total_size = bytes.len();
    let mut cursor = Cursor::new(&bytes);

    cursor.read_u32_le()?; // version
    let marker_or_count = cursor.read_u8()?;
    let input_count = if marker_or_count == 0 {
        let flag = cursor.read_u8()?;
        if flag == 0 {
            return Err("Invalid segwit marker/flag.".to_string());
        }
        cursor.read_varint()?
    } else {
        marker_or_count as u64
    };
    let has_witness = marker_or_count == 0;
    // Accounts for the 4-byte version and, for legacy transactions, the varint
    // input count byte that is included in the non-witness portion.
    let stripped_prefix_size = if has_witness { 4 } else { 5 };

    // Collect the previous txid from each input. Only the first is used as the
    // ARC4 decryption seed for MULTISIG-encoded stamps.
    let mut prev_txids: Vec<String> = Vec::new();
    for _ in 0..input_count {
        let prev_txid_bytes = cursor.read_exact(32)?;
        let prev_txid = reverse_hex(prev_txid_bytes);
        cursor.read_u32_le()?; // vout index
        let script_len = cursor.read_varint()? as usize;
        cursor.read_exact(script_len)?; // scriptSig
        cursor.read_u32_le()?; // sequence
        prev_txids.push(prev_txid);
    }

    let output_count = cursor.read_varint()?;
    let mut p2wsh_chunks = Vec::new();
    // Per-output ciphertext chunks for MULTISIG decryption (see decrypt_multisig_per_output).
    let mut multisig_output_ciphertexts: Vec<Vec<u8>> = Vec::new();
    let mut saw_multisig_output = false;
    let mut payload = None;
    let mut has_valid_pattern = false;
    let mut has_valid_data = false;
    let mut keyburn = 0;
    let mut encoding_method = None;

    for output_index in 0..output_count {
        cursor.read_u64_le()?; // value in satoshis
        let script_len = cursor.read_varint()? as usize;
        let script = cursor.read_exact(script_len)?;
        let has_op_checkmultisig = script.last() == Some(&OP_CHECKMULTISIG);

        // OLGA/P2WSH: collect 32-byte data chunks from outputs after index 0.
        // Output 0 is always the keyburn (unspendable) output.
        if output_index > 0 && script.len() == 34 && script[0] == 0x00 && script[1] == 0x20 {
            has_valid_pattern = true;
            p2wsh_chunks.extend_from_slice(&script[2..34]);
            encoding_method = Some("OLGA/P2WSH".to_string());
        }

        if has_op_checkmultisig {
            saw_multisig_output = true;
            let mut chunk = Vec::new();
            append_multisig_inner_ciphertext(script, &mut chunk);
            multisig_output_ciphertexts.push(chunk);
        }
    }

    if saw_multisig_output {
        has_valid_pattern = true;
    }

    // Consume witness data to reach the locktime field and measure witness size.
    let witness_start = cursor.index;
    let mut witness_size = 0;
    if has_witness {
        for _ in 0..input_count {
            let witness_count = cursor.read_varint()?;
            for _ in 0..witness_count {
                let item_len = cursor.read_varint()? as usize;
                cursor.read_exact(item_len)?;
            }
        }
        witness_size = cursor.index - witness_start;
    }

    cursor.read_u32_le()?; // locktime
    let stripped_size = if has_witness {
        stripped_prefix_size + (total_size - witness_size - 6)
    } else {
        total_size
    };
    let weight = stripped_size * 4 + witness_size;
    let vsize = weight.div_ceil(4);

    // Finalise OLGA/P2WSH payload: strip trailing zero-padding then extract
    // the length-prefixed stamp data.
    if !p2wsh_chunks.is_empty() {
        while p2wsh_chunks.last() == Some(&0) {
            p2wsh_chunks.pop();
        }

        if let Some(extracted) = extract_length_prefixed_payload(&p2wsh_chunks) {
            payload = Some(extracted);
            has_valid_data = true;
            keyburn = 1;
        }
    }

    // MULTISIG: independently decrypt each output's 62-byte ciphertext block,
    // strip the per-block Counterparty magic prefix, then concatenate.
    if !has_valid_data && !multisig_output_ciphertexts.is_empty() {
        if let Some(decrypted_payload) = decrypt_multisig_per_output(
            &multisig_output_ciphertexts,
            prev_txids.first().map(String::as_str),
        ) {
            payload = Some(decrypted_payload);
            has_valid_data = true;
            keyburn = 1;
            encoding_method = Some("MULTISIG".to_string());
        }
    }

    Ok(ParsedTransaction {
        payload,
        has_valid_pattern,
        has_valid_data,
        keyburn,
        encoding_method,
        vsize,
    })
}

// ===================================================================
//   PAYLOAD EXTRACTION
// ===================================================================

/// Appends inner "pubkey" slices from a single `OP_CHECKMULTISIG` script.
///
/// Matches btc_stamps [`get_checkmultisig`](https://github.com/stampchain-io/btc_stamps/blob/main/indexer/src/index_core/script.py):
/// only **`asm[1:3]`** (the first two fake pubkeys) are used per output. The
/// third push in a standard 1-of-3 stamp multisig is the dummy / burn key
/// (`0222…22`); including its inner bytes corrupts the ARC4 ciphertext.
fn append_multisig_inner_ciphertext(script: &[u8], out: &mut Vec<u8>) {
    let pubkey_like: Vec<Vec<u8>> = push_data_items(script)
        .into_iter()
        .filter(|p| p.len() > 2)
        .collect();

    // Classic stamp multisig is 1-of-3 with two data keys + one dummy key.
    let take = if pubkey_like.len() >= 3 {
        2
    } else {
        pubkey_like.len()
    };

    for pubkey in pubkey_like.iter().take(take) {
        out.extend_from_slice(&pubkey[1..pubkey.len() - 1]);
    }
}

fn decrypt_concatenated_multisig(ciphertext: &[u8], prev_txid: Option<&str>) -> Option<Vec<u8>> {
    let seed = hex_to_bytes(prev_txid?).ok()?;
    let decrypted = arc4::decrypt(ciphertext, &seed);
    extract_length_prefixed_payload(&decrypted)
}

/// Decodes a Counterparty MULTISIG stamp using the correct per-output
/// independent decryption scheme.
///
/// The Counterparty protocol ARC4-encrypts each output block **independently**
/// with the same key (the first input's previous txid).  After decrypting,
/// every block begins with a Counterparty magic prefix — `=CNTRPRTY` (9 bytes)
/// for the classic format, or `CNTRPRTY` (8 bytes) for an older variant.
/// That prefix is stripped from each chunk and the remaining bytes are
/// concatenated to rebuild the full Counterparty message payload.
///
/// If the first decrypted chunk does not start with a recognised prefix, the
/// function falls back to treating all blocks as a single concatenated
/// ciphertext (legacy single-block ARC4) for compatibility with non-CP stamps.
fn decrypt_multisig_per_output(
    output_ciphertexts: &[Vec<u8>],
    prev_txid: Option<&str>,
) -> Option<Vec<u8>> {
    let seed = hex_to_bytes(prev_txid?).ok()?;
    if output_ciphertexts.is_empty() {
        return None;
    }

    // Detect magic prefix length from the first decrypted output chunk.
    let first_decrypted = arc4::decrypt(&output_ciphertexts[0], &seed);
    let magic_len: Option<usize> = if first_decrypted.starts_with(b"=CNTRPRTY") {
        Some(9)
    } else if first_decrypted.starts_with(b"CNTRPRTY") {
        Some(8)
    } else {
        None
    };

    if let Some(plen) = magic_len {
        // Per-output Counterparty mode: strip the magic prefix from every
        // independently decrypted block and concatenate the payload bytes.
        let mut assembled: Vec<u8> = Vec::new();
        assembled.extend_from_slice(&first_decrypted[plen.min(first_decrypted.len())..]);
        for chunk in &output_ciphertexts[1..] {
            if chunk.is_empty() {
                continue;
            }
            let decrypted = arc4::decrypt(chunk, &seed);
            let strip = plen.min(decrypted.len());
            assembled.extend_from_slice(&decrypted[strip..]);
        }
        // Remove trailing null-byte padding added by the last chunk.
        while assembled.last() == Some(&0) {
            assembled.pop();
        }
        extract_length_prefixed_payload(&assembled)
    } else {
        // Legacy fallback: concatenate all raw ciphertext and decrypt as one block.
        let concatenated: Vec<u8> = output_ciphertexts.iter().flatten().copied().collect();
        decrypt_concatenated_multisig(&concatenated, prev_txid)
    }
}

/// Locates the stamp payload within a byte slice.
///
/// Tries the length-prefix format first (`[u16 big-endian length][data]`). If
/// the data region starts with `stamp:` (case-insensitive), that prefix is
/// stripped and the bounded chunk is returned verbatim.  OLGA/P2WSH stamps
/// store raw media bytes directly after the length prefix (e.g. GIF magic
/// bytes), which pass through unchanged.
///
/// Falls back to a **case-insensitive** linear scan for `stamp:` when the
/// length-prefix check fails (e.g. old Counterparty-encoded stamps where the
/// first two bytes are not a valid length).  In the fallback path the data
/// after the prefix is **trimmed to the first non-base64 byte** so that
/// binary Counterparty overhead that follows the description does not pollute
/// the payload.  Data-URL stamps (`data:…`) that happen to be found via the
/// fallback scan are returned verbatim.
fn extract_length_prefixed_payload(bytes: &[u8]) -> Option<Vec<u8>> {
    if bytes.len() >= 2 {
        let chunk_len = ((bytes[0] as usize) << 8) | bytes[1] as usize;
        let end = 2 + chunk_len;
        if end <= bytes.len() {
            let chunk = &bytes[2..end];
            // Case-insensitive: handles both lowercase `stamp:` (direct stamps)
            // and the rare uppercase variant.
            if chunk.len() >= STAMP_PREFIX.len()
                && chunk[..STAMP_PREFIX.len()].eq_ignore_ascii_case(STAMP_PREFIX)
            {
                return Some(chunk[STAMP_PREFIX.len()..].to_vec());
            }
            return Some(chunk.to_vec());
        }
    }

    // Fallback: case-insensitive scan — needed for old Counterparty stamps
    // where the encrypted data starts with `=CNTRPRTY…STAMP:<base64>`.
    find_stamp_prefix(bytes).map(|pos| {
        trim_non_base64_suffix(&bytes[pos + STAMP_PREFIX.len()..])
    })
}

/// Finds the byte offset of the `stamp:` prefix (case-insensitive) inside
/// `bytes`, or `None` if absent.
fn find_stamp_prefix(bytes: &[u8]) -> Option<usize> {
    bytes.windows(STAMP_PREFIX.len()).position(|window| {
        window
            .iter()
            .zip(STAMP_PREFIX.iter())
            .all(|(a, b)| a.eq_ignore_ascii_case(b))
    })
}

/// Returns a copy of `after_prefix` trimmed to just the valid base64
/// characters at its start.  If the content begins with `data:` (a data-URL
/// stamp) it is returned unchanged — trimming would destroy the URL.
fn trim_non_base64_suffix(after_prefix: &[u8]) -> Vec<u8> {
    if after_prefix.starts_with(b"data:") || after_prefix.starts_with(b"DATA:") {
        return after_prefix.to_vec();
    }
    let end = after_prefix
        .iter()
        .position(|b| !matches!(b, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'+' | b'/' | b'='))
        .unwrap_or(after_prefix.len());
    after_prefix[..end].to_vec()
}

// ===================================================================
//   SCRIPT PARSING
// ===================================================================

/// Extracts all push-data items from a Bitcoin script. Handles opcodes 1–75
/// (direct push) and OP_PUSHDATA1 (opcode 76). Items from higher-value push
/// opcodes or non-push opcodes are skipped.
fn push_data_items(script: &[u8]) -> Vec<Vec<u8>> {
    let mut cursor = 0;
    let mut items = Vec::new();

    while cursor < script.len() {
        let opcode = script[cursor];
        cursor += 1;

        let len = match opcode {
            1..=75 => opcode as usize,
            76 => {
                if cursor >= script.len() {
                    break;
                }
                let len = script[cursor] as usize;
                cursor += 1;
                len
            }
            _ => continue,
        };

        if cursor + len > script.len() {
            break;
        }

        items.push(script[cursor..cursor + len].to_vec());
        cursor += len;
    }

    items
}

// ===================================================================
//   HEX UTILITIES
// ===================================================================

/// Decodes a hex string into a byte vector. Trims surrounding whitespace and
/// rejects odd-length or non-hex input.
pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    let trimmed = hex.trim();
    if trimmed.len() % 2 != 0 {
        return Err("Hex input must contain an even number of characters.".to_string());
    }

    let mut bytes = Vec::with_capacity(trimmed.len() / 2);
    for chunk in trimmed.as_bytes().chunks(2) {
        let high = hex_value(chunk[0])?;
        let low = hex_value(chunk[1])?;
        bytes.push((high << 4) | low);
    }
    Ok(bytes)
}

/// Encodes a byte slice as a lowercase hex string.
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

fn hex_value(byte: u8) -> Result<u8, String> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err("Invalid hexadecimal transaction data.".to_string()),
    }
}

/// Reverses a byte slice and returns it as a lowercase hex string. Used to
/// convert Bitcoin's internal little-endian txid byte order to the standard
/// big-endian display order.
fn reverse_hex(bytes: &[u8]) -> String {
    let mut reversed = bytes.to_vec();
    reversed.reverse();
    bytes_to_hex(&reversed)
}

// ===================================================================
//   TESTS
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_hex() {
        assert_eq!(hex_to_bytes("0aFF").unwrap(), vec![10, 255]);
    }

    #[test]
    fn rejects_odd_hex() {
        assert!(hex_to_bytes("abc").is_err());
    }

    #[test]
    fn extracts_stamp_payload_with_length_prefix() {
        let mut data = vec![0, 11];
        data.extend_from_slice(b"stamp:hello");
        assert_eq!(extract_length_prefixed_payload(&data).unwrap(), b"hello");
    }

    #[test]
    fn extracts_olga_media_payload_with_length_prefix() {
        let mut data = vec![0, 6];
        data.extend_from_slice(b"GIF87a");
        assert_eq!(extract_length_prefixed_payload(&data).unwrap(), b"GIF87a");
    }

    // --- extract_length_prefixed_payload: case-insensitive / trim tests ---

    #[test]
    fn finds_uppercase_stamp_prefix_in_fallback_scan() {
        // Counterparty-encoded stamps have uppercase `STAMP:` at a non-zero offset.
        let mut data = b"=CNTRPRTY\x14binary_header\x00\x21\x00\x01\x00".to_vec();
        data.extend_from_slice(b"STAMP:");
        data.extend_from_slice(b"QUJD"); // base64 for "ABC"
        data.extend_from_slice(b"\xa0\x82"); // binary Counterparty overhead after description
        let payload = extract_length_prefixed_payload(&data).unwrap();
        assert_eq!(payload, b"QUJD"); // trimmed to only valid base64
    }

    #[test]
    fn finds_lowercase_stamp_prefix_in_fallback_scan() {
        let mut data = vec![0xffu8, 0xff]; // makes length prefix out-of-range
        data.extend_from_slice(b"stamp:QUJD\xa0");
        let payload = extract_length_prefixed_payload(&data).unwrap();
        assert_eq!(payload, b"QUJD");
    }

    #[test]
    fn preserves_data_url_after_stamp_prefix() {
        let mut data = vec![0xffu8, 0xff];
        data.extend_from_slice(b"STAMP:data:image/png;base64,QUJD");
        let payload = extract_length_prefixed_payload(&data).unwrap();
        assert_eq!(payload, b"data:image/png;base64,QUJD");
    }

    #[test]
    fn multisig_skips_third_dummy_pubkey_per_output() {
        let mut script = Vec::new();
        for body in [[1u8, 2, 3, 4], [5, 6, 7, 8], [0xee, 0xee, 0xee, 0xee]] {
            script.push(body.len() as u8);
            script.extend_from_slice(&body);
        }
        script.push(OP_CHECKMULTISIG);

        let mut ciphertext = Vec::new();
        append_multisig_inner_ciphertext(&script, &mut ciphertext);
        // Inner [1..len-1]: first push → [2,3], second → [6,7]; third dummy excluded.
        assert_eq!(ciphertext, vec![2, 3, 6, 7]);
    }

    /// Legacy single-block decryption still works (non-Counterparty stamp).
    #[test]
    fn multisig_concatenated_decrypt_matches_single_output_inner() {
        let prev_txid = "9f501abcfd91488c94be1e5d576220f1a6c4928cecb538e882eb0a8bc0fd2d80";
        let seed = hex_to_bytes(prev_txid).unwrap();

        let mut plain = vec![0u8, 11];
        plain.extend_from_slice(b"stamp:hello");
        let on_chain = crate::arc4::decrypt(&plain, &seed);

        // One OP_CHECKMULTISIG-shaped script: single push of 15 bytes; inner [1..14] = 13 = on_chain.
        let mut script = Vec::new();
        script.push(0x0f); // push 15 bytes
        script.push(0x02);
        script.extend_from_slice(&on_chain);
        script.push(0x02);
        script.push(OP_CHECKMULTISIG);

        let mut ciphertext = Vec::new();
        append_multisig_inner_ciphertext(&script, &mut ciphertext);
        assert_eq!(ciphertext, on_chain);

        // Via legacy helper.
        let out = decrypt_concatenated_multisig(&ciphertext, Some(prev_txid)).unwrap();
        assert_eq!(out, b"hello");

        // Via per-output function: no Counterparty prefix detected → falls back
        // to single-block decryption and still produces the correct result.
        let out2 = decrypt_multisig_per_output(&[ciphertext], Some(prev_txid)).unwrap();
        assert_eq!(out2, b"hello");
    }

    /// Counterparty MULTISIG stamps use per-output independent ARC4: each
    /// decrypted block starts with `=CNTRPRTY` which is stripped before
    /// concatenating the payload bytes across outputs.
    #[test]
    fn multisig_per_output_strips_cntrprty_prefix_and_assembles() {
        let prev_txid = "0000000000000000000000000000000000000000000000000000000000000001";
        let seed = hex_to_bytes(prev_txid).unwrap();
        let magic = b"=CNTRPRTY";

        // Counterparty message split across two output blocks (magic prefix repeated).
        // Assembled payload after stripping prefix:
        //   block0[9..] = b"\x00\x0bstamp:hell"
        //   block1[9..] = b"o"
        // → b"\x00\x0bstamp:hello" → extract_length_prefixed_payload → b"hello"
        let mut plain0 = Vec::new();
        plain0.extend_from_slice(magic);
        plain0.extend_from_slice(b"\x00\x0bstamp:hell");
        let cipher0 = crate::arc4::decrypt(&plain0, &seed);

        let mut plain1 = Vec::new();
        plain1.extend_from_slice(magic);
        plain1.push(b'o');
        let cipher1 = crate::arc4::decrypt(&plain1, &seed);

        let out = decrypt_multisig_per_output(&[cipher0, cipher1], Some(prev_txid)).unwrap();
        assert_eq!(out, b"hello");
    }

    #[test]
    fn multisig_inner_bytes_concatenate_in_output_order() {
        let mut ciphertext = Vec::new();
        let s1 = vec![3u8, 0xaa, 0xbb, 0xaa, OP_CHECKMULTISIG];
        let s2 = vec![3u8, 0xcc, 0xdd, 0xcc, OP_CHECKMULTISIG];
        append_multisig_inner_ciphertext(&s1, &mut ciphertext);
        append_multisig_inner_ciphertext(&s2, &mut ciphertext);
        assert_eq!(ciphertext, vec![0xbb, 0xdd]);
    }
}
