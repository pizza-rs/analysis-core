use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

// ═══════════════════════════════════════════════════════════════════════════════
// ENCODING & HASH FILTERS — Data transformation beyond text
// ═══════════════════════════════════════════════════════════════════════════════

/// Base64 encodes the token term.
#[derive(Clone, Debug)]
pub struct Base64EncodeTokenFilter;
impl Base64EncodeTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for Base64EncodeTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for Base64EncodeTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let encoded = base64_encode(token.term.as_ref().as_bytes());
        token.term = Cow::Owned(encoded);
        (false, None)
    }
}

/// Base64 decodes the token term (if valid base64).
#[derive(Clone, Debug)]
pub struct Base64DecodeTokenFilter;
impl Base64DecodeTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for Base64DecodeTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for Base64DecodeTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        if let Some(decoded) = base64_decode(token.term.as_ref()) {
            if let Ok(s) = String::from_utf8(decoded) {
                token.term = Cow::Owned(s);
            }
        }
        (false, None)
    }
}

/// Hex encodes the token bytes.
#[derive(Clone, Debug)]
pub struct HexEncodeTokenFilter;
impl HexEncodeTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for HexEncodeTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for HexEncodeTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let hex: String = token
            .term
            .as_ref()
            .bytes()
            .map(|b| format!("{:02x}", b))
            .collect();
        token.term = Cow::Owned(hex);
        (false, None)
    }
}

/// Hex decodes the token (if valid hex string).
#[derive(Clone, Debug)]
pub struct HexDecodeTokenFilter;
impl HexDecodeTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for HexDecodeTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for HexDecodeTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.len() % 2 != 0 {
            return (false, None);
        }
        let bytes: Option<Vec<u8>> = (0..text.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&text[i..i + 2], 16).ok())
            .collect();
        if let Some(b) = bytes {
            if let Ok(s) = String::from_utf8(b) {
                token.term = Cow::Owned(s);
            }
        }
        (false, None)
    }
}

/// URL-encodes (percent-encodes) the token.
#[derive(Clone, Debug)]
pub struct UrlEncodeTokenFilter;
impl UrlEncodeTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for UrlEncodeTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for UrlEncodeTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let encoded = percent_encode(token.term.as_ref());
        if encoded != token.term.as_ref() {
            token.term = Cow::Owned(encoded);
        }
        (false, None)
    }
}

/// URL-decodes (percent-decodes) the token.
#[derive(Clone, Debug)]
pub struct UrlDecodeTokenFilter;
impl UrlDecodeTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for UrlDecodeTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for UrlDecodeTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let decoded = percent_decode(token.term.as_ref());
        if decoded != token.term.as_ref() {
            token.term = Cow::Owned(decoded);
        }
        (false, None)
    }
}

/// ROT13 cipher transformation (reversible).
#[derive(Clone, Debug)]
pub struct Rot13TokenFilter;
impl Rot13TokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for Rot13TokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for Rot13TokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let rotated: String = token
            .term
            .as_ref()
            .chars()
            .map(|c| match c {
                'a'..='m' | 'A'..='M' => char::from(c as u8 + 13),
                'n'..='z' | 'N'..='Z' => char::from(c as u8 - 13),
                _ => c,
            })
            .collect();
        token.term = Cow::Owned(rotated);
        (false, None)
    }
}

/// FNV-1a 32-bit hash of the token, output as hex string.
#[derive(Clone, Debug)]
pub struct FnvHashTokenFilter;
impl FnvHashTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for FnvHashTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for FnvHashTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let hash = fnv1a_32(token.term.as_ref().as_bytes());
        token.term = Cow::Owned(format!("{:08x}", hash));
        (false, None)
    }
}

/// CRC32 hash of the token.
#[derive(Clone, Debug)]
pub struct Crc32TokenFilter;
impl Crc32TokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for Crc32TokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for Crc32TokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let hash = crc32(token.term.as_ref().as_bytes());
        token.term = Cow::Owned(format!("{:08x}", hash));
        (false, None)
    }
}

/// MurmurHash3 (32-bit) of the token.
#[derive(Clone, Debug)]
pub struct MurmurHash3TokenFilter {
    pub seed: u32,
}
impl MurmurHash3TokenFilter {
    pub fn new() -> Self {
        Self { seed: 0 }
    }

    pub fn with_seed(seed: u32) -> Self {
        Self { seed }
    }
}
impl Default for MurmurHash3TokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for MurmurHash3TokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let hash = murmur3_32(token.term.as_ref().as_bytes(), self.seed);
        token.term = Cow::Owned(format!("{:08x}", hash));
        (false, None)
    }
}

/// Emits both original and hashed version (hash as synonym).
#[derive(Clone, Debug)]
pub struct HashSynonymTokenFilter {
    pub algorithm: HashAlgorithm,
}

#[derive(Clone, Debug, Copy)]
pub enum HashAlgorithm {
    Fnv1a,
    Crc32,
    Murmur3,
}

impl HashSynonymTokenFilter {
    pub fn new(algorithm: HashAlgorithm) -> Self {
        Self { algorithm }
    }
}

impl Default for HashSynonymTokenFilter {
    fn default() -> Self {
        Self::new(HashAlgorithm::Fnv1a)
    }
}

impl TokenFilter for HashSynonymTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let hash = match self.algorithm {
            HashAlgorithm::Fnv1a => fnv1a_32(token.term.as_ref().as_bytes()),
            HashAlgorithm::Crc32 => crc32(token.term.as_ref().as_bytes()),
            HashAlgorithm::Murmur3 => murmur3_32(token.term.as_ref().as_bytes(), 0),
        };
        let synonym = Token {
            term: Cow::Owned(format!("{:08x}", hash)),
            start_offset: token.start_offset,
            end_offset: token.end_offset,
            position: token.position,
        };
        (false, Some(alloc::vec![synonym]))
    }
}

// ─── Helper implementations ────────────────────────────────────────────────

const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(input: &[u8]) -> String {
    let mut result = String::with_capacity((input.len() + 2) / 3 * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(BASE64_CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(BASE64_CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(BASE64_CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(BASE64_CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

fn base64_decode(input: &str) -> Option<Vec<u8>> {
    let input = input.trim_end_matches('=');
    let mut result = Vec::with_capacity(input.len() * 3 / 4);
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;
    for c in input.bytes() {
        let val = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        } as u32;
        buf = (buf << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            result.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }
    Some(result)
}

fn percent_encode(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for b in input.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(b as char);
            }
            _ => {
                result.push('%');
                result.push(char::from(HEX_CHARS[(b >> 4) as usize]));
                result.push(char::from(HEX_CHARS[(b & 0xF) as usize]));
            }
        }
    }
    result
}

const HEX_CHARS: &[u8] = b"0123456789ABCDEF";

fn percent_decode(input: &str) -> String {
    let mut result = Vec::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(val) = u8::from_str_radix(&input[i + 1..i + 3], 16) {
                result.push(val);
                i += 3;
                continue;
            }
        }
        result.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(result).unwrap_or_else(|_| String::from(input))
}

fn fnv1a_32(data: &[u8]) -> u32 {
    let mut hash: u32 = 0x811c9dc5;
    for &byte in data {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(0x01000193);
    }
    hash
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

fn murmur3_32(data: &[u8], seed: u32) -> u32 {
    let mut h: u32 = seed;
    let len = data.len();
    let nblocks = len / 4;

    for i in 0..nblocks {
        let idx = i * 4;
        let mut k = u32::from_le_bytes([data[idx], data[idx + 1], data[idx + 2], data[idx + 3]]);
        k = k.wrapping_mul(0xcc9e2d51);
        k = k.rotate_left(15);
        k = k.wrapping_mul(0x1b873593);
        h ^= k;
        h = h.rotate_left(13);
        h = h.wrapping_mul(5).wrapping_add(0xe6546b64);
    }

    let tail = &data[nblocks * 4..];
    let mut k1: u32 = 0;
    if tail.len() >= 3 {
        k1 ^= (tail[2] as u32) << 16;
    }
    if tail.len() >= 2 {
        k1 ^= (tail[1] as u32) << 8;
    }
    if !tail.is_empty() {
        k1 ^= tail[0] as u32;
        k1 = k1.wrapping_mul(0xcc9e2d51);
        k1 = k1.rotate_left(15);
        k1 = k1.wrapping_mul(0x1b873593);
        h ^= k1;
    }

    h ^= len as u32;
    h ^= h >> 16;
    h = h.wrapping_mul(0x85ebca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2ae35);
    h ^= h >> 16;
    h
}
