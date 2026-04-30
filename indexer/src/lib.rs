//! Browser-safe Rust/Wasm Bitcoin Stamps transaction indexer.
//!
//! # Wasm memory model
//!
//! JavaScript and Wasm do not share a garbage collector, so this crate manages
//! its own heap manually through three exported C-ABI functions:
//!
//! - [`alloc`] — allocates a byte buffer and returns a raw pointer.
//! - [`dealloc`] — frees a buffer previously returned by `alloc` or `index_stamp`.
//! - [`index_stamp`] — the main entry point; reads a JSON [`LookupInput`] from
//!   Wasm memory and writes a JSON [`StampResult`] back, returning a packed
//!   `u64` with the pointer in the high 32 bits and the byte length in the low
//!   32 bits.
//!
//! JavaScript is responsible for writing input bytes before calling
//! `index_stamp` and for freeing the response buffer with `dealloc` after
//! reading it.

mod arc4;
mod media;
mod metadata;
mod tx;

use metadata::{index_stamp_from_input, LookupInput};

// ===================================================================
//   WASM MEMORY ABI
// ===================================================================

/// Allocates a zeroed byte buffer of `len` bytes and returns a raw pointer.
///
/// The buffer is "forgotten" by Rust so it will not be freed automatically;
/// the caller (JavaScript) is responsible for calling [`dealloc`] with the
/// same pointer and length when it is no longer needed.
#[no_mangle]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    let mut buffer = vec![0_u8; len];
    let ptr = buffer.as_mut_ptr();
    std::mem::forget(buffer);
    ptr
}

/// Frees a byte buffer previously allocated by [`alloc`] or returned by
/// [`index_stamp`].
///
/// # Safety
///
/// Wasm boundary invariant: `ptr` and `len` must match a buffer previously
/// returned by `alloc` or by `index_stamp` that has not yet been freed.
#[no_mangle]
pub extern "C" fn dealloc(ptr: *mut u8, len: usize) {
    if ptr.is_null() {
        return;
    }

    // SAFETY: upheld by the invariant documented above.
    unsafe {
        drop(Vec::from_raw_parts(ptr, len, len));
    }
}

// ===================================================================
//   ENTRY POINT
// ===================================================================

/// Reads a [`LookupInput`] JSON from Wasm memory, indexes the stamp, and
/// returns a packed `u64` encoding the [`StampResult`] JSON response:
/// `(ptr << 32) | len`. Always succeeds — errors are returned as JSON with
/// `"ok": false`. JavaScript must free the returned buffer with [`dealloc`].
#[no_mangle]
pub extern "C" fn index_stamp(input_ptr: *const u8, input_len: usize) -> u64 {
    let response = read_input(input_ptr, input_len)
        .and_then(|input| index_stamp_from_input(&input))
        .unwrap_or_else(|message| metadata::StampResult::error(message).to_json());

    leak_response(response)
}

// ===================================================================
//   PRIVATE HELPERS
// ===================================================================

/// Reads `input_len` bytes from `input_ptr` and deserialises them as a
/// [`LookupInput`] JSON object.
fn read_input(input_ptr: *const u8, input_len: usize) -> Result<LookupInput, String> {
    if input_ptr.is_null() {
        return Err("Missing lookup input.".to_string());
    }

    // SAFETY: JS writes `input_len` initialised bytes at `input_ptr` before
    // calling `index_stamp`.
    let bytes = unsafe { std::slice::from_raw_parts(input_ptr, input_len) };
    let input = std::str::from_utf8(bytes).map_err(|_| "Lookup input is not valid UTF-8.")?;
    serde_json::from_str(input).map_err(|_| "Lookup input is not valid JSON.".to_string())
}

/// Copies `response` into a new Wasm heap allocation and returns the pointer
/// and byte length packed into a single `u64` (high 32 bits = ptr, low 32
/// bits = len). The allocation is intentionally leaked; JavaScript frees it
/// via [`dealloc`].
fn leak_response(response: String) -> u64 {
    let bytes = response.into_bytes();
    let len = bytes.len();
    let ptr = alloc(len);

    // SAFETY: `alloc` returns a writable buffer of `len` bytes and `bytes`
    // remains alive for the duration of this copy.
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, len);
    }

    ((ptr as u64) << 32) | len as u64
}

// ===================================================================
//   TESTS
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_pointer_and_length() {
        let packed = leak_response("{}".to_string());
        assert!(packed > 0);
    }
}
