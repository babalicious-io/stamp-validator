mod arc4;
mod media;
mod metadata;
mod tx;

use metadata::{index_stamp_from_input, LookupInput};

#[no_mangle]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    let mut buffer = vec![0_u8; len];
    let ptr = buffer.as_mut_ptr();
    std::mem::forget(buffer);
    ptr
}

#[no_mangle]
pub extern "C" fn dealloc(ptr: *mut u8, len: usize) {
    if ptr.is_null() {
        return;
    }

    // Wasm boundary invariant: JS passes a pointer and length previously
    // returned by `alloc` or by `index_stamp`.
    unsafe {
        drop(Vec::from_raw_parts(ptr, len, len));
    }
}

#[no_mangle]
pub extern "C" fn index_stamp(input_ptr: *const u8, input_len: usize) -> u64 {
    let response = read_input(input_ptr, input_len)
        .and_then(|input| index_stamp_from_input(&input))
        .unwrap_or_else(|message| metadata::StampResult::error(message).to_json());

    leak_response(response)
}

fn read_input(input_ptr: *const u8, input_len: usize) -> Result<LookupInput, String> {
    if input_ptr.is_null() {
        return Err("Missing lookup input.".to_string());
    }

    // Wasm boundary invariant: JS writes `input_len` initialized bytes at
    // `input_ptr` before calling this function.
    let bytes = unsafe { std::slice::from_raw_parts(input_ptr, input_len) };
    let input = std::str::from_utf8(bytes).map_err(|_| "Lookup input is not valid UTF-8.")?;
    serde_json::from_str(input).map_err(|_| "Lookup input is not valid JSON.".to_string())
}

fn leak_response(response: String) -> u64 {
    let bytes = response.into_bytes();
    let len = bytes.len();
    let ptr = alloc(len);

    // Wasm boundary invariant: `alloc` returns a writable buffer of `len`
    // bytes, and `bytes` remains alive for the duration of this copy.
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, len);
    }

    ((ptr as u64) << 32) | len as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_pointer_and_length() {
        let packed = leak_response("{}".to_string());
        assert!(packed > 0);
    }
}
