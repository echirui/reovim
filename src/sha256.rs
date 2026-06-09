use std::ffi::c_char;

pub const SHA256_BUFFER_SIZE: usize = 64;
pub const SHA256_SUM_SIZE: usize = 32;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct context_sha256_T {
    pub total: [u32; 2],
    pub state: [u32; 8],
    pub buffer: [u8; SHA256_BUFFER_SIZE],
}

#[inline(always)]
fn get_uint32(b: &[u8], i: usize) -> u32 {
    let bytes = [b[i], b[i + 1], b[i + 2], b[i + 3]];
    u32::from_be_bytes(bytes)
}

#[inline(always)]
fn put_uint32(n: u32, b: &mut [u8], i: usize) {
    let bytes = n.to_be_bytes();
    b[i..i + 4].copy_from_slice(&bytes);
}

fn sha256_process(ctx: &mut context_sha256_T, data: &[u8; SHA256_BUFFER_SIZE]) {
    let mut w = [0u32; 64];
    for i in 0..16 {
        w[i] = get_uint32(data, i * 4);
    }

    #[inline(always)]
    fn shr(x: u32, n: u32) -> u32 {
        x >> n
    }
    #[inline(always)]
    fn rotr(x: u32, n: u32) -> u32 {
        (x >> n) | (x << (32 - n))
    }
    #[inline(always)]
    fn s0(x: u32) -> u32 {
        rotr(x, 7) ^ rotr(x, 18) ^ shr(x, 3)
    }
    #[inline(always)]
    fn s1(x: u32) -> u32 {
        rotr(x, 17) ^ rotr(x, 19) ^ shr(x, 10)
    }
    #[inline(always)]
    fn s2(x: u32) -> u32 {
        rotr(x, 2) ^ rotr(x, 13) ^ rotr(x, 22)
    }
    #[inline(always)]
    fn s3(x: u32) -> u32 {
        rotr(x, 6) ^ rotr(x, 11) ^ rotr(x, 25)
    }
    #[inline(always)]
    fn f0(x: u32, y: u32, z: u32) -> u32 {
        (x & y) | (z & (x | y))
    }
    #[inline(always)]
    fn f1(x: u32, y: u32, z: u32) -> u32 {
        z ^ (x & (y ^ z))
    }

    for t in 16..64 {
        w[t] = s1(w[t - 2])
            .wrapping_add(w[t - 7])
            .wrapping_add(s0(w[t - 15]))
            .wrapping_add(w[t - 16]);
    }

    let mut a = ctx.state[0];
    let mut b = ctx.state[1];
    let mut c = ctx.state[2];
    let mut d = ctx.state[3];
    let mut e = ctx.state[4];
    let mut f = ctx.state[5];
    let mut g = ctx.state[6];
    let mut h = ctx.state[7];

    const K256: [u32; 64] = [
        0x428A2F98, 0x71374491, 0xB5C0FBCF, 0xE9B5DBA5, 0x3956C25B, 0x59F111F1, 0x923F82A4, 0xAB1C5ED5,
        0xD807AA98, 0x12835B01, 0x243185BE, 0x550C7DC3, 0x72BE5D74, 0x80DEB1FE, 0x9BDC06A7, 0xC19BF174,
        0xE49B69C1, 0xEFBE4786, 0x0FC19DC6, 0x240CA1CC, 0x2DE92C6F, 0x4A7484AA, 0x5CB0A9DC, 0x76F988DA,
        0x983E5152, 0xA831C66D, 0xB00327C8, 0xBF597FC7, 0xC6E00BF3, 0xD5A79147, 0x06CA6351, 0x14292967,
        0x27B70A85, 0x2E1B2138, 0x4D2C6DFC, 0x53380D13, 0x650A7354, 0x766A0ABB, 0x81C2C92E, 0x92722C85,
        0xA2BFE8A1, 0xA81A664B, 0xC24B8B70, 0xC76C51A3, 0xD192E819, 0xD6990624, 0xF40E3585, 0x106AA070,
        0x19A4C116, 0x1E376C08, 0x2748774C, 0x34B0BCB5, 0x391C0CB3, 0x4ED8AA4A, 0x5B9CCA4F, 0x682E6FF3,
        0x748F82EE, 0x78A5636F, 0x84C87814, 0x8CC70208, 0x90BEFFFA, 0xA4506CEB, 0xBEF9A3F7, 0xC67178F2,
    ];

    for t in 0..64 {
        let temp1 = h
            .wrapping_add(s3(e))
            .wrapping_add(f1(e, f, g))
            .wrapping_add(K256[t])
            .wrapping_add(w[t]);
        let temp2 = s2(a).wrapping_add(f0(a, b, c));
        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(temp1);
        d = c;
        c = b;
        b = a;
        a = temp1.wrapping_add(temp2);
    }

    ctx.state[0] = ctx.state[0].wrapping_add(a);
    ctx.state[1] = ctx.state[1].wrapping_add(b);
    ctx.state[2] = ctx.state[2].wrapping_add(c);
    ctx.state[3] = ctx.state[3].wrapping_add(d);
    ctx.state[4] = ctx.state[4].wrapping_add(e);
    ctx.state[5] = ctx.state[5].wrapping_add(f);
    ctx.state[6] = ctx.state[6].wrapping_add(g);
    ctx.state[7] = ctx.state[7].wrapping_add(h);
}

#[unsafe(no_mangle)]
pub extern "C" fn sha256_start(ctx: *mut context_sha256_T) {
    if ctx.is_null() {
        return;
    }
    unsafe {
        (*ctx).total[0] = 0;
        (*ctx).total[1] = 0;

        (*ctx).state[0] = 0x6A09E667;
        (*ctx).state[1] = 0xBB67AE85;
        (*ctx).state[2] = 0x3C6EF372;
        (*ctx).state[3] = 0xA54FF53A;
        (*ctx).state[4] = 0x510E527F;
        (*ctx).state[5] = 0x9B05688C;
        (*ctx).state[6] = 0x1F83D9AB;
        (*ctx).state[7] = 0x5BE0CD19;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn sha256_update(ctx: *mut context_sha256_T, input: *const u8, mut length: usize) {
    if ctx.is_null() || input.is_null() || length == 0 {
        return;
    }
    unsafe {
        let mut input_slice = std::slice::from_raw_parts(input, length);
        let mut left = ((*ctx).total[0] & 63) as usize;

        (*ctx).total[0] = (*ctx).total[0].wrapping_add(length as u32);
        if (*ctx).total[0] < length as u32 {
            (*ctx).total[1] += 1;
        }

        let fill = 64 - left;

        if left > 0 && length >= fill {
            let buffer_ptr = ((*ctx).buffer.as_mut_ptr()).add(left);
            std::ptr::copy_nonoverlapping(input_slice.as_ptr(), buffer_ptr, fill);
            sha256_process(&mut *ctx, &(*ctx).buffer);
            input_slice = &input_slice[fill..];
            length -= fill;
            left = 0;
        }

        while length >= 64 {
            let chunk = &input_slice[..64];
            let mut arr = [0u8; 64];
            arr.copy_from_slice(chunk);
            sha256_process(&mut *ctx, &arr);
            input_slice = &input_slice[64..];
            length -= 64;
        }

        if length > 0 {
            let buffer_ptr = ((*ctx).buffer.as_mut_ptr()).add(left);
            std::ptr::copy_nonoverlapping(input_slice.as_ptr(), buffer_ptr, length);
        }
    }
}

static SHA256_PADDING: [u8; 64] = [
    0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
];

#[unsafe(no_mangle)]
pub extern "C" fn sha256_finish(ctx: *mut context_sha256_T, digest: *mut u8) {
    if ctx.is_null() || digest.is_null() {
        return;
    }
    unsafe {
        let high = ((*ctx).total[0] >> 29) | ((*ctx).total[1] << 3);
        let low = (*ctx).total[0] << 3;

        let mut msglen = [0u8; 8];
        put_uint32(high, &mut msglen, 0);
        put_uint32(low, &mut msglen, 4);

        let last = (*ctx).total[0] & 0x3F;
        let padn = if last < 56 { 56 - last } else { 120 - last } as usize;

        sha256_update(ctx, SHA256_PADDING.as_ptr(), padn);
        sha256_update(ctx, msglen.as_ptr(), 8);

        let digest_slice = std::slice::from_raw_parts_mut(digest, 32);
        put_uint32((*ctx).state[0], digest_slice, 0);
        put_uint32((*ctx).state[1], digest_slice, 4);
        put_uint32((*ctx).state[2], digest_slice, 8);
        put_uint32((*ctx).state[3], digest_slice, 12);
        put_uint32((*ctx).state[4], digest_slice, 16);
        put_uint32((*ctx).state[5], digest_slice, 20);
        put_uint32((*ctx).state[6], digest_slice, 24);
        put_uint32((*ctx).state[7], digest_slice, 28);
    }
}

#[inline(always)]
fn to_hex_char(val: u8) -> c_char {
    if val < 10 {
        (b'0' + val) as c_char
    } else {
        (b'a' + (val - 10)) as c_char
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn sha256_bytes(
    buf: *const u8,
    buf_len: usize,
    salt: *const u8,
    salt_len: usize,
) -> *const c_char {
    static mut HEXIT: [c_char; 65] = [0; 65];

    sha256_self_test();

    let mut ctx = context_sha256_T {
        total: [0; 2],
        state: [0; 8],
        buffer: [0; 64],
    };
    sha256_start(&mut ctx);
    sha256_update(&mut ctx, buf, buf_len);

    if !salt.is_null() {
        sha256_update(&mut ctx, salt, salt_len);
    }
    let mut sha256sum = [0u8; 32];
    sha256_finish(&mut ctx, sha256sum.as_mut_ptr());

    unsafe {
        let hexit_ptr = std::ptr::addr_of_mut!(HEXIT) as *mut c_char;
        for j in 0..32 {
            let val = sha256sum[j];
            let high = (val >> 4) & 0x0F;
            let low = val & 0x0F;
            *hexit_ptr.add(j * 2) = to_hex_char(high);
            *hexit_ptr.add(j * 2 + 1) = to_hex_char(low);
        }
        *hexit_ptr.add(64) = 0; // NUL terminator
        hexit_ptr as *const c_char
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn sha256_self_test() -> bool {
    static mut SHA256_SELF_TESTED: bool = false;
    static mut FAILURES: bool = false;

    unsafe {
        if SHA256_SELF_TESTED {
            return !FAILURES;
        }
        SHA256_SELF_TESTED = true;
    }

    let sha_self_test_msg = [
        "abc",
        "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
    ];

    let sha_self_test_vector = [
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
        "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0",
    ];

    let mut output = [0u8; 65];

    for i in 0..3 {
        if i < 2 {
            let msg = sha_self_test_msg[i];
            let hexit_ptr = sha256_bytes(msg.as_ptr(), msg.len(), std::ptr::null(), 0);
            unsafe {
                let len = 64; // SHA256 size in hex
                std::ptr::copy_nonoverlapping(hexit_ptr as *const u8, output.as_mut_ptr(), len);
            }
        } else {
            let mut ctx = context_sha256_T {
                total: [0; 2],
                state: [0; 8],
                buffer: [0; 64],
            };
            sha256_start(&mut ctx);
            let buf = [b'a'; 1000];
            for _ in 0..1000 {
                sha256_update(&mut ctx, buf.as_ptr(), 1000);
            }
            let mut sha256sum = [0u8; 32];
            sha256_finish(&mut ctx, sha256sum.as_mut_ptr());

            for j in 0..32 {
                let val = sha256sum[j];
                let high = (val >> 4) & 0x0F;
                let low = val & 0x0F;
                output[j * 2] = to_hex_char(high) as u8;
                output[j * 2 + 1] = to_hex_char(low) as u8;
            }
        }

        let expected = sha_self_test_vector[i].as_bytes();
        if output[..64] != expected[..64] {
            unsafe {
                FAILURES = true;
            }
        }
    }

    unsafe { !FAILURES }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    #[test]
    fn test_sha256_self_test() {
        assert!(sha256_self_test());
    }

    #[test]
    fn test_sha256_bytes_abc() {
        let msg = b"abc";
        let res_ptr = sha256_bytes(msg.as_ptr(), msg.len(), std::ptr::null(), 0);
        let c_str = unsafe { CStr::from_ptr(res_ptr) };
        let str_slice = c_str.to_str().unwrap();
        assert_eq!(str_slice, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }

    #[test]
    fn test_sha256_bytes_with_salt() {
        let msg = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
        let res_ptr = sha256_bytes(msg.as_ptr(), msg.len(), std::ptr::null(), 0);
        let c_str = unsafe { CStr::from_ptr(res_ptr) };
        let str_slice = c_str.to_str().unwrap();
        assert_eq!(str_slice, "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1");
    }
}
