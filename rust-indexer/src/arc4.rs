pub fn decrypt(data: &[u8], seed: &[u8]) -> Vec<u8> {
    let mut state = init(seed);
    apply(data, &mut state)
}

fn init(seed: &[u8]) -> [u8; 256] {
    let mut state = [0_u8; 256];
    for (index, item) in state.iter_mut().enumerate() {
        *item = index as u8;
    }

    if seed.is_empty() {
        return state;
    }

    let mut j = 0_usize;
    for i in 0..256 {
        j = (j + state[i] as usize + seed[i % seed.len()] as usize) & 0xff;
        state.swap(i, j);
    }

    state
}

fn apply(data: &[u8], state: &mut [u8; 256]) -> Vec<u8> {
    let mut i = 0_usize;
    let mut j = 0_usize;
    let mut output = Vec::with_capacity(data.len());

    for byte in data {
        i = (i + 1) & 0xff;
        j = (j + state[i] as usize) & 0xff;
        state.swap(i, j);
        let k = state[(state[i] as usize + state[j] as usize) & 0xff];
        output.push(byte ^ k);
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decrypts_known_rc4_vector() {
        let cipher = [0xbb, 0xf3, 0x16, 0xe8, 0xd9, 0x40, 0xaf, 0x0a, 0xd3];
        assert_eq!(decrypt(&cipher, b"Key"), b"Plaintext");
    }
}
