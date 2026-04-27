use crate::arc4;

const STAMP_PREFIX: &[u8] = b"stamp:";
const OP_CHECKMULTISIG: u8 = 0xae;

#[derive(Debug, Clone)]
pub struct ParsedTransaction {
    pub txid: String,
    pub inputs: Vec<InputInfo>,
    pub outputs: Vec<OutputInfo>,
    pub payload: Option<Vec<u8>>,
    pub has_valid_pattern: bool,
    pub has_valid_data: bool,
    pub keyburn: u32,
    pub encoding_method: Option<String>,
}

#[derive(Debug, Clone)]
pub struct InputInfo {
    pub prev_txid: String,
    pub prev_vout: u32,
    pub sequence: u32,
}

#[derive(Debug, Clone)]
pub struct OutputInfo {
    pub value: u64,
    pub script_hex: String,
    pub index: u32,
    pub has_op_checkmultisig: bool,
    pub keyburn: u32,
}

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

pub fn parse_transaction(raw_tx_hex: &str) -> Result<ParsedTransaction, String> {
    let bytes = hex_to_bytes(raw_tx_hex)?;
    let txid = double_sha256_txid_placeholder(&bytes);
    let mut cursor = Cursor::new(&bytes);

    cursor.read_u32_le()?;
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

    let mut inputs = Vec::new();
    for _ in 0..input_count {
        let prev_txid_bytes = cursor.read_exact(32)?;
        let prev_txid = reverse_hex(prev_txid_bytes);
        let prev_vout = cursor.read_u32_le()?;
        let script_len = cursor.read_varint()? as usize;
        cursor.read_exact(script_len)?;
        let sequence = cursor.read_u32_le()?;
        inputs.push(InputInfo {
            prev_txid,
            prev_vout,
            sequence,
        });
    }

    let output_count = cursor.read_varint()?;
    let mut outputs = Vec::new();
    let mut p2wsh_chunks = Vec::new();
    let mut payload = None;
    let mut has_valid_pattern = false;
    let mut has_valid_data = false;
    let mut keyburn = 0;
    let mut encoding_method = None;

    for output_index in 0..output_count {
        let value = cursor.read_u64_le()?;
        let script_len = cursor.read_varint()? as usize;
        let script = cursor.read_exact(script_len)?;
        let has_op_checkmultisig = script.last() == Some(&OP_CHECKMULTISIG);
        let mut output_keyburn = 0;

        if output_index > 0 && script.len() == 34 && script[0] == 0x00 && script[1] == 0x20 {
            has_valid_pattern = true;
            p2wsh_chunks.extend_from_slice(&script[2..34]);
            encoding_method = Some("OLGA/P2WSH".to_string());
        }

        if has_op_checkmultisig {
            if let Some(decrypted_payload) = extract_multisig_payload(script, inputs.first()) {
                payload = Some(decrypted_payload);
                has_valid_pattern = true;
                has_valid_data = true;
                keyburn = 1;
                output_keyburn = 1;
                encoding_method = Some("MULTISIG".to_string());
            }
        }

        outputs.push(OutputInfo {
            value,
            script_hex: bytes_to_hex(script),
            index: output_index as u32,
            has_op_checkmultisig,
            keyburn: output_keyburn,
        });
    }

    if has_witness {
        for _ in 0..input_count {
            let witness_count = cursor.read_varint()?;
            for _ in 0..witness_count {
                let item_len = cursor.read_varint()? as usize;
                cursor.read_exact(item_len)?;
            }
        }
    }

    cursor.read_u32_le()?;

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

    Ok(ParsedTransaction {
        txid,
        inputs,
        outputs,
        payload,
        has_valid_pattern,
        has_valid_data,
        keyburn,
        encoding_method,
    })
}

fn extract_multisig_payload(script: &[u8], first_input: Option<&InputInfo>) -> Option<Vec<u8>> {
    let pubkeys = push_data_items(script);
    if pubkeys.len() < 2 {
        return None;
    }

    let mut encrypted = Vec::new();
    for pubkey in pubkeys.iter().take(2) {
        if pubkey.len() > 2 {
            encrypted.extend_from_slice(&pubkey[1..pubkey.len() - 1]);
        }
    }

    let seed = hex_to_bytes(&first_input?.prev_txid).ok()?;
    let decrypted = arc4::decrypt(&encrypted, &seed);
    extract_length_prefixed_payload(&decrypted)
}

fn extract_length_prefixed_payload(bytes: &[u8]) -> Option<Vec<u8>> {
    if bytes.len() >= 2 {
        let chunk_len = ((bytes[0] as usize) << 8) | bytes[1] as usize;
        let end = 2 + chunk_len;
        if end <= bytes.len() {
            let chunk = &bytes[2..end];
            if chunk.len() >= STAMP_PREFIX.len() && &chunk[..STAMP_PREFIX.len()] == STAMP_PREFIX {
                return Some(chunk[STAMP_PREFIX.len()..].to_vec());
            }

            // OLGA/P2WSH stamps store the media bytes directly after the
            // length prefix. For example, GIF stamps begin with GIF87a/GIF89a
            // here rather than a text "stamp:" prefix.
            return Some(chunk.to_vec());
        }
    }

    bytes
        .windows(STAMP_PREFIX.len())
        .position(|window| window == STAMP_PREFIX)
        .map(|position| bytes[position + STAMP_PREFIX.len()..].to_vec())
}

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

fn hex_value(byte: u8) -> Result<u8, String> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err("Invalid hexadecimal transaction data.".to_string()),
    }
}

pub fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

fn reverse_hex(bytes: &[u8]) -> String {
    let mut reversed = bytes.to_vec();
    reversed.reverse();
    bytes_to_hex(&reversed)
}

fn double_sha256_txid_placeholder(bytes: &[u8]) -> String {
    // This app receives the canonical tx hash from user input/context. The
    // parser keeps a deterministic local identifier for fixture tests without
    // pulling in a hashing crate just for display.
    format!("local-{}-bytes", bytes.len())
}

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
}
