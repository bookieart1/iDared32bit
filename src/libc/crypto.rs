/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! CommonCrypto and friends

use crate::dyld::FunctionExports;
use crate::mem::{ConstVoidPtr, GuestUSize, MutPtr, MutVoidPtr, SafeRead};
use crate::{export_c_func, Environment};
use aes::cipher::{generic_array::GenericArray as CipherArray, BlockDecrypt, BlockEncrypt, KeyInit};
use aes::{Aes128, Aes192, Aes256};
use digest::Digest;
use md5::Md5;
use sha1::Sha1;
use sha2::digest::generic_array::GenericArray as DigestArray;
use sha2::{Sha224, Sha256, Sha384, Sha512};
use std::collections::HashMap;

// TODO: struct definition
#[allow(non_camel_case_types)]
struct CC_MD5_CTX {}

#[derive(Default)]
pub struct State {
    md5_contexts: HashMap<MutPtr<CC_MD5_CTX>, Md5>,
}
impl State {
    fn get_mut(env: &mut Environment) -> &mut Self {
        &mut env.libc_state.crypto
    }
}

fn CC_MD5_Init(env: &mut Environment, ctx: MutPtr<CC_MD5_CTX>) -> i32 {
    log_once!("Warning: CC_MD5_Init doesn't update side effects! (internal changes to CC_MD5_CTX are not done)");
    assert!(!State::get_mut(env).md5_contexts.contains_key(&ctx));
    State::get_mut(env).md5_contexts.insert(ctx, Md5::new());
    1 // success
}

fn CC_MD5_Update(
    env: &mut Environment,
    ctx: MutPtr<CC_MD5_CTX>,
    data: ConstVoidPtr,
    len: u32,
) -> i32 {
    log_once!("Warning: CC_MD5_Update doesn't update side effects! (internal changes to CC_MD5_CTX are not done)");
    let hasher = env.libc_state.crypto.md5_contexts.get_mut(&ctx).unwrap();
    hasher.update(env.mem.bytes_at(data.cast(), len));
    1 // success
}

fn CC_MD5_Final(env: &mut Environment, md: MutPtr<u8>, ctx: MutPtr<CC_MD5_CTX>) -> i32 {
    log_once!("Warning: CC_MD5_Final doesn't update side effects! (internal changes to CC_MD5_CTX are not done)");
    let hasher = State::get_mut(env).md5_contexts.remove(&ctx).unwrap();
    let digest = hasher.finalize();
    env.mem.bytes_at_mut(md, 16).copy_from_slice(&digest[..]);
    1 // success
}

fn CC_MD5(env: &mut Environment, data: ConstVoidPtr, len: u32, md: MutPtr<u8>) -> MutPtr<u8> {
    let mut hasher = Md5::new();
    hasher.update(env.mem.bytes_at(data.cast(), len));
    let digest = hasher.finalize();
    env.mem.bytes_at_mut(md, 16).copy_from_slice(&digest[..]);
    md
}

fn CC_SHA1(env: &mut Environment, data: ConstVoidPtr, len: u32, md: MutPtr<u8>) -> MutPtr<u8> {
    let mut hasher = Sha1::new();
    hasher.update(env.mem.bytes_at(data.cast(), len));
    let digest = hasher.finalize();
    env.mem.bytes_at_mut(md, 20).copy_from_slice(&digest[..]);
    md
}

fn CC_SHA256(env: &mut Environment, data: ConstVoidPtr, len: u32, md: MutPtr<u8>) -> MutPtr<u8> {
    let mut hasher = Sha256::new();
    hasher.update(env.mem.bytes_at(data.cast(), len));
    let digest = hasher.finalize();
    env.mem.bytes_at_mut(md, 32).copy_from_slice(&digest[..]);
    md
}

/// All state lives in the app's context, so apps can copy it.
#[allow(non_camel_case_types)]
#[repr(C, packed)]
struct CC_SHA256_CTX {
    /// Length in bits, low word first.
    count: [u32; 2],
    hash: [u32; 8],
    wbuf: [u8; 64],
}
unsafe impl SafeRead for CC_SHA256_CTX {}

impl CC_SHA256_CTX {
    fn len_bits(&self) -> u64 {
        let count = self.count;
        (u64::from(count[1]) << 32) | u64::from(count[0])
    }

    fn update(&mut self, mut data: &[u8]) {
        let len_bits = self.len_bits();
        let mut used = (len_bits / 8 % 64) as usize;
        let len_bits = len_bits.wrapping_add(data.len() as u64 * 8);
        self.count = [len_bits as u32, (len_bits >> 32) as u32];

        let mut hash = self.hash;
        let mut wbuf = self.wbuf;
        while !data.is_empty() {
            let n = (64 - used).min(data.len());
            wbuf[used..used + n].copy_from_slice(&data[..n]);
            used += n;
            data = &data[n..];
            if used == 64 {
                sha2::compress256(&mut hash, &[DigestArray::clone_from_slice(&wbuf)]);
                used = 0;
            }
        }
        self.hash = hash;
        self.wbuf = wbuf;
    }

    fn finish(mut self) -> [u8; 32] {
        let len_bits = self.len_bits();
        let used = (len_bits / 8 % 64) as usize;
        let mut padding = vec![0x80];
        padding.resize(1 + (119 - used) % 64, 0);
        padding.extend_from_slice(&len_bits.to_be_bytes());
        self.update(&padding);
        self.hash
            .map(u32::to_be_bytes)
            .as_flattened()
            .try_into()
            .unwrap()
    }
}

fn CC_SHA256_Init(env: &mut Environment, c: MutPtr<CC_SHA256_CTX>) -> i32 {
    let ctx = CC_SHA256_CTX {
        count: [0; 2],
        hash: [
            0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
            0x5be0cd19,
        ],
        wbuf: [0; 64],
    };
    env.mem.write(c, ctx);
    1
}

fn CC_SHA256_Update(
    env: &mut Environment,
    c: MutPtr<CC_SHA256_CTX>,
    data: ConstVoidPtr,
    len: u32,
) -> i32 {
    let mut ctx = env.mem.read(c);
    ctx.update(env.mem.bytes_at(data.cast(), len));
    env.mem.write(c, ctx);
    1
}

fn CC_SHA256_Final(env: &mut Environment, md: MutPtr<u8>, c: MutPtr<CC_SHA256_CTX>) -> i32 {
    let digest = env.mem.read(c).finish();
    env.mem.bytes_at_mut(md, 32).copy_from_slice(&digest);
    1
}

fn hmac_digest<D: Digest + Default>(block_size: usize, key: &[u8], data: &[u8]) -> Vec<u8> {
    let normalized_key = if key.len() > block_size {
        D::digest(key).to_vec()
    } else {
        key.to_vec()
    };

    let mut inner_pad = vec![0x36; block_size];
    let mut outer_pad = vec![0x5c; block_size];
    for (idx, byte) in normalized_key.iter().enumerate() {
        inner_pad[idx] ^= byte;
        outer_pad[idx] ^= byte;
    }

    let mut inner = D::new();
    inner.update(&inner_pad);
    inner.update(data);
    let inner_digest = inner.finalize();

    let mut outer = D::new();
    outer.update(&outer_pad);
    outer.update(inner_digest);
    outer.finalize().to_vec()
}

fn CCHmac(
    env: &mut Environment,
    algorithm: u32,
    key: ConstVoidPtr,
    key_length: GuestUSize,
    data: ConstVoidPtr,
    data_length: GuestUSize,
    mac_out: MutVoidPtr,
) {
    // Values from CommonHMAC.h's CCHmacAlgorithm enum.
    const K_CC_HMAC_ALG_SHA1: u32 = 0;
    const K_CC_HMAC_ALG_MD5: u32 = 1;
    const K_CC_HMAC_ALG_SHA256: u32 = 2;
    const K_CC_HMAC_ALG_SHA384: u32 = 3;
    const K_CC_HMAC_ALG_SHA512: u32 = 4;
    const K_CC_HMAC_ALG_SHA224: u32 = 5;

    assert!(!mac_out.is_null());
    assert!(!key.is_null() || key_length == 0);
    assert!(!data.is_null() || data_length == 0);

    let key = if key_length == 0 {
        Vec::new()
    } else {
        env.mem.bytes_at(key.cast(), key_length).to_vec()
    };
    let data = if data_length == 0 {
        Vec::new()
    } else {
        env.mem.bytes_at(data.cast(), data_length).to_vec()
    };
    let digest = match algorithm {
        K_CC_HMAC_ALG_SHA1 => hmac_digest::<Sha1>(64, &key, &data),
        K_CC_HMAC_ALG_MD5 => hmac_digest::<Md5>(64, &key, &data),
        K_CC_HMAC_ALG_SHA256 => hmac_digest::<Sha256>(64, &key, &data),
        K_CC_HMAC_ALG_SHA384 => hmac_digest::<Sha384>(128, &key, &data),
        K_CC_HMAC_ALG_SHA512 => hmac_digest::<Sha512>(128, &key, &data),
        K_CC_HMAC_ALG_SHA224 => hmac_digest::<Sha224>(64, &key, &data),
        _ => panic!("Unsupported CCHmac algorithm {algorithm}"),
    };
    env.mem
        .bytes_at_mut(mac_out.cast(), digest.len().try_into().unwrap())
        .copy_from_slice(&digest);
}

type CCCryptorStatus = i32;
const K_CC_SUCCESS: CCCryptorStatus = 0;
const K_CC_PARAM_ERROR: CCCryptorStatus = -4300;
const K_CC_BUFFER_TOO_SMALL: CCCryptorStatus = -4301;
const K_CC_ALIGNMENT_ERROR: CCCryptorStatus = -4303;
const K_CC_DECODE_ERROR: CCCryptorStatus = -4304;
const K_CC_UNIMPLEMENTED: CCCryptorStatus = -4305;

const K_CC_ENCRYPT: u32 = 0;
const K_CC_DECRYPT: u32 = 1;
const K_CC_ALGORITHM_AES128: u32 = 0;
const K_CC_OPTION_PKCS7_PADDING: u32 = 1;
const K_CC_OPTION_ECB_MODE: u32 = 2;
const AES_BLOCK_SIZE: usize = 16;

fn aes_crypt<C>(
    key: &[u8],
    iv: &[u8; AES_BLOCK_SIZE],
    input: &[u8],
    encrypt: bool,
    ecb: bool,
    pkcs7: bool,
) -> Result<Vec<u8>, CCCryptorStatus>
where
    C: BlockEncrypt + BlockDecrypt + KeyInit,
{
    let cipher = C::new_from_slice(key).map_err(|_| K_CC_PARAM_ERROR)?;
    if !encrypt && (input.is_empty() || !input.len().is_multiple_of(AES_BLOCK_SIZE)) {
        return Err(K_CC_ALIGNMENT_ERROR);
    }

    let mut output = input.to_vec();
    if encrypt {
        if pkcs7 {
            let padding = AES_BLOCK_SIZE - output.len() % AES_BLOCK_SIZE;
            output.resize(output.len() + padding, padding as u8);
        } else if !output.len().is_multiple_of(AES_BLOCK_SIZE) {
            return Err(K_CC_ALIGNMENT_ERROR);
        }

        let mut previous = *iv;
        for chunk in output.chunks_exact_mut(AES_BLOCK_SIZE) {
            if !ecb {
                for (byte, previous_byte) in chunk.iter_mut().zip(previous) {
                    *byte ^= previous_byte;
                }
            }
            let block = CipherArray::from_mut_slice(chunk);
            cipher.encrypt_block(block);
            if !ecb {
                previous.copy_from_slice(chunk);
            }
        }
    } else {
        let mut previous = *iv;
        for chunk in output.chunks_exact_mut(AES_BLOCK_SIZE) {
            let mut encrypted_block = [0u8; AES_BLOCK_SIZE];
            encrypted_block.copy_from_slice(chunk);
            let block = CipherArray::from_mut_slice(chunk);
            cipher.decrypt_block(block);
            if !ecb {
                for (byte, previous_byte) in chunk.iter_mut().zip(previous) {
                    *byte ^= previous_byte;
                }
                previous = encrypted_block;
            }
        }
        if pkcs7 {
            let Some(&padding) = output.last() else {
                return Err(K_CC_DECODE_ERROR);
            };
            let padding = padding as usize;
            if padding == 0
                || padding > AES_BLOCK_SIZE
                || output.len() < padding
                || !output[output.len() - padding..]
                    .iter()
                    .all(|&byte| byte as usize == padding)
            {
                return Err(K_CC_DECODE_ERROR);
            }
            output.truncate(output.len() - padding);
        }
    }
    Ok(output)
}

#[allow(clippy::too_many_arguments)]
fn CCCrypt(
    env: &mut Environment,
    operation: u32,
    algorithm: u32,
    options: u32,
    key: ConstVoidPtr,
    key_length: u32,
    iv: ConstVoidPtr,
    data_in: ConstVoidPtr,
    data_in_length: u32,
    data_out: MutVoidPtr,
    data_out_available: u32,
    data_out_moved: MutPtr<u32>,
) -> CCCryptorStatus {
    if !data_out_moved.is_null() {
        env.mem.write(data_out_moved, 0);
    }
    if algorithm != K_CC_ALGORITHM_AES128
        || !matches!(operation, K_CC_ENCRYPT | K_CC_DECRYPT)
        || options & !(K_CC_OPTION_PKCS7_PADDING | K_CC_OPTION_ECB_MODE) != 0
    {
        return K_CC_UNIMPLEMENTED;
    }
    if key.is_null() || data_out.is_null() || (data_in.is_null() && data_in_length != 0) {
        return K_CC_PARAM_ERROR;
    }

    let key = env.mem.bytes_at(key.cast(), key_length).to_vec();
    let input = if data_in_length == 0 {
        Vec::new()
    } else {
        env.mem.bytes_at(data_in.cast(), data_in_length).to_vec()
    };
    let iv = if iv.is_null() {
        [0; AES_BLOCK_SIZE]
    } else {
        env.mem
            .bytes_at(iv.cast(), AES_BLOCK_SIZE as u32)
            .try_into()
            .unwrap()
    };
    let encrypt = operation == K_CC_ENCRYPT;
    let ecb = options & K_CC_OPTION_ECB_MODE != 0;
    let pkcs7 = options & K_CC_OPTION_PKCS7_PADDING != 0;
    let result = match key.len() {
        16 => aes_crypt::<Aes128>(&key, &iv, &input, encrypt, ecb, pkcs7),
        24 => aes_crypt::<Aes192>(&key, &iv, &input, encrypt, ecb, pkcs7),
        32 => aes_crypt::<Aes256>(&key, &iv, &input, encrypt, ecb, pkcs7),
        _ => return K_CC_PARAM_ERROR,
    };
    let output = match result {
        Ok(output) => output,
        Err(status) => return status,
    };
    if output.len() > data_out_available as usize {
        return K_CC_BUFFER_TOO_SMALL;
    }
    env.mem
        .bytes_at_mut(data_out.cast(), output.len().try_into().unwrap())
        .copy_from_slice(&output);
    if !data_out_moved.is_null() {
        env.mem.write(data_out_moved, output.len().try_into().unwrap());
    }
    K_CC_SUCCESS
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CC_MD5_Init(_)),
    export_c_func!(CC_MD5_Update(_, _, _)),
    export_c_func!(CC_MD5_Final(_, _)),
    export_c_func!(CC_MD5(_, _, _)),
    export_c_func!(CC_SHA1(_, _, _)),
    export_c_func!(CC_SHA256(_, _, _)),
    export_c_func!(CC_SHA256_Init(_)),
    export_c_func!(CC_SHA256_Update(_, _, _)),
    export_c_func!(CC_SHA256_Final(_, _)),
    export_c_func!(CCHmac(_, _, _, _, _, _)),
    export_c_func!(CCCrypt(_, _, _, _, _, _, _, _, _, _, _)),
];
