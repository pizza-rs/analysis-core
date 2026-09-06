use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

// ═══════════════════════════════════════════════════════════════════════════════
// TEXT METRICS FILTERS — Compute and emit text statistics as tokens
// ═══════════════════════════════════════════════════════════════════════════════

/// Emits the character count as a synonym: "hello" → also "_len:5"
#[derive(Clone, Debug)]
pub struct CharCountTokenFilter {
    pub prefix: String,
}
impl CharCountTokenFilter {
    pub fn new() -> Self {
        Self {
            prefix: String::from("_len:"),
        }
    }
}
impl Default for CharCountTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for CharCountTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let count = token.term.chars().count();
        let tag = Token {
            term: Cow::Owned(format!("{}{}", self.prefix, count)),
            start_offset: token.start_offset,
            end_offset: token.end_offset,
            position: token.position,
        };
        (false, Some(alloc::vec![tag]))
    }
}

/// Emits the byte length as a synonym.
#[derive(Clone, Debug)]
pub struct ByteLengthTokenFilter;
impl ByteLengthTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for ByteLengthTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for ByteLengthTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let len = token.term.len();
        let tag = Token {
            term: Cow::Owned(format!("_bytes:{}", len)),
            start_offset: token.start_offset,
            end_offset: token.end_offset,
            position: token.position,
        };
        (false, Some(alloc::vec![tag]))
    }
}

/// Classifies token length into bands: short (1-3), medium (4-8), long (9+).
#[derive(Clone, Debug)]
pub struct LengthBandTokenFilter {
    pub short_max: usize,
    pub medium_max: usize,
}
impl LengthBandTokenFilter {
    pub fn new() -> Self {
        Self {
            short_max: 3,
            medium_max: 8,
        }
    }
}
impl Default for LengthBandTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for LengthBandTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let len = token.term.chars().count();
        let band = if len <= self.short_max {
            "short"
        } else if len <= self.medium_max {
            "medium"
        } else {
            "long"
        };
        let tag = Token {
            term: Cow::Owned(format!("_length_band:{}", band)),
            start_offset: token.start_offset,
            end_offset: token.end_offset,
            position: token.position,
        };
        (false, Some(alloc::vec![tag]))
    }
}

/// Estimates syllable count and emits it.
#[derive(Clone, Debug)]
pub struct SyllableCountTokenFilter;
impl SyllableCountTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for SyllableCountTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for SyllableCountTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let syllables = count_syllables(token.term.as_ref());
        let tag = Token {
            term: Cow::Owned(format!("_syllables:{}", syllables)),
            start_offset: token.start_offset,
            end_offset: token.end_offset,
            position: token.position,
        };
        (false, Some(alloc::vec![tag]))
    }
}

/// Computes Shannon entropy of the token and emits as tag.
/// High entropy → likely random/hashed, low entropy → likely natural text.
#[derive(Clone, Debug)]
pub struct EntropyTokenFilter;
impl EntropyTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for EntropyTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for EntropyTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let entropy = shannon_entropy(token.term.as_ref());
        let category = if entropy < 2.0 {
            "low"
        } else if entropy < 4.0 {
            "medium"
        } else {
            "high"
        };
        let tag = Token {
            term: Cow::Owned(format!("_entropy:{}", category)),
            start_offset: token.start_offset,
            end_offset: token.end_offset,
            position: token.position,
        };
        (false, Some(alloc::vec![tag]))
    }
}

/// Detects the predominant script of the token text.
#[derive(Clone, Debug)]
pub struct ScriptTagTokenFilter;
impl ScriptTagTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for ScriptTagTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for ScriptTagTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let script = detect_script_name(token.term.as_ref());
        let tag = Token {
            term: Cow::Owned(format!("_script:{}", script)),
            start_offset: token.start_offset,
            end_offset: token.end_offset,
            position: token.position,
        };
        (false, Some(alloc::vec![tag]))
    }
}

/// Detects the likely language of a token (simple heuristic).
#[derive(Clone, Debug)]
pub struct LanguageTagTokenFilter;
impl LanguageTagTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for LanguageTagTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for LanguageTagTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let lang = detect_language_heuristic(token.term.as_ref());
        let tag = Token {
            term: Cow::Owned(format!("_lang:{}", lang)),
            start_offset: token.start_offset,
            end_offset: token.end_offset,
            position: token.position,
        };
        (false, Some(alloc::vec![tag]))
    }
}

/// Detects if a token looks like a UUID/GUID.
#[derive(Clone, Debug)]
pub struct UuidDetectTokenFilter;
impl UuidDetectTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for UuidDetectTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for UuidDetectTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if is_uuid(text) {
            let tag = Token {
                term: Cow::Owned(String::from("_type:uuid")),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            };
            return (false, Some(alloc::vec![tag]));
        }
        (false, None)
    }
}

/// Filters tokens by their entropy — removes high-entropy (random/hash) tokens.
#[derive(Clone, Debug)]
pub struct EntropyFilterTokenFilter {
    pub max_entropy: f64,
}
impl EntropyFilterTokenFilter {
    pub fn new(max_entropy: f64) -> Self {
        Self { max_entropy }
    }
}
impl Default for EntropyFilterTokenFilter {
    fn default() -> Self {
        Self::new(4.5)
    }
}

impl TokenFilter for EntropyFilterTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let entropy = shannon_entropy(token.term.as_ref());
        if entropy > self.max_entropy {
            return (true, None);
        }
        (false, None)
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────

fn count_syllables(word: &str) -> usize {
    let lower = word.to_lowercase();
    let chars: Vec<char> = lower.chars().collect();
    if chars.is_empty() {
        return 0;
    }
    let mut count = 0usize;
    let mut prev_vowel = false;
    for (i, &c) in chars.iter().enumerate() {
        let is_v = matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y');
        if is_v && !prev_vowel {
            count += 1;
        }
        prev_vowel = is_v;
    }
    // Silent 'e' at end
    if chars.len() > 2
        && chars[chars.len() - 1] == 'e'
        && !matches!(chars[chars.len() - 2], 'a' | 'e' | 'i' | 'o' | 'u')
    {
        if count > 1 {
            count -= 1;
        }
    }
    if count == 0 {
        1
    } else {
        count
    }
}

fn shannon_entropy(s: &str) -> f64 {
    if s.is_empty() {
        return 0.0;
    }
    let mut freq = [0u32; 256];
    let len = s.len() as f64;
    for b in s.bytes() {
        freq[b as usize] += 1;
    }
    let mut entropy = 0.0f64;
    for &count in freq.iter() {
        if count > 0 {
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
    }
    entropy
}

fn detect_script_name(s: &str) -> &'static str {
    let first_significant = s.chars().find(|c| c.is_alphabetic());
    match first_significant {
        None => "unknown",
        Some(c) => {
            let cp = c as u32;
            match cp {
                0x0000..=0x007F => "latin",
                0x0400..=0x04FF => "cyrillic",
                0x0370..=0x03FF => "greek",
                0x0600..=0x06FF => "arabic",
                0x0590..=0x05FF => "hebrew",
                0x0900..=0x097F => "devanagari",
                0x4E00..=0x9FFF => "cjk",
                0x3040..=0x309F => "hiragana",
                0x30A0..=0x30FF => "katakana",
                0xAC00..=0xD7AF => "hangul",
                0x0E00..=0x0E7F => "thai",
                _ => "other",
            }
        }
    }
}

fn detect_language_heuristic(s: &str) -> &'static str {
    let first = s.chars().find(|c| c.is_alphabetic());
    match first {
        None => "und",
        Some(c) => {
            let cp = c as u32;
            match cp {
                0x4E00..=0x9FFF => "zh",
                0x3040..=0x309F | 0x30A0..=0x30FF => "ja",
                0xAC00..=0xD7AF => "ko",
                0x0600..=0x06FF => "ar",
                0x0400..=0x04FF => "ru",
                0x0900..=0x097F => "hi",
                0x0E00..=0x0E7F => "th",
                _ => "en", // default assumption for latin
            }
        }
    }
}

fn is_uuid(s: &str) -> bool {
    if s.len() != 36 {
        return false;
    }
    let bytes = s.as_bytes();
    bytes[8] == b'-'
        && bytes[13] == b'-'
        && bytes[18] == b'-'
        && bytes[23] == b'-'
        && s.chars()
            .filter(|c| *c != '-')
            .all(|c| c.is_ascii_hexdigit())
}
