use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

// ═══════════════════════════════════════════════════════════════════════════════
// ADVANCED UNICODE FILTERS — Security normalization, confusable detection
// ═══════════════════════════════════════════════════════════════════════════════

/// Removes invisible/zero-width characters that can be used for attacks.
#[derive(Clone, Debug)]
pub struct InvisibleCharRemoveTokenFilter;
impl InvisibleCharRemoveTokenFilter { pub fn new() -> Self { Self } }
impl Default for InvisibleCharRemoveTokenFilter { fn default() -> Self { Self } }

impl TokenFilter for InvisibleCharRemoveTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let cleaned: String = text.chars().filter(|c| !is_invisible_char(*c)).collect();
        if cleaned.len() != text.len() {
            if cleaned.is_empty() {
                return (true, None);
            }
            token.term = Cow::Owned(cleaned);
        }
        (false, None)
    }
}

/// Removes zero-width characters specifically.
#[derive(Clone, Debug)]
pub struct ZeroWidthRemoveTokenFilter;
impl ZeroWidthRemoveTokenFilter { pub fn new() -> Self { Self } }
impl Default for ZeroWidthRemoveTokenFilter { fn default() -> Self { Self } }

impl TokenFilter for ZeroWidthRemoveTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let cleaned: String = text.chars().filter(|c| !is_zero_width(*c)).collect();
        if cleaned.len() != text.len() {
            if cleaned.is_empty() {
                return (true, None);
            }
            token.term = Cow::Owned(cleaned);
        }
        (false, None)
    }
}

/// Strips BiDi (bidirectional) control characters used for text spoofing.
#[derive(Clone, Debug)]
pub struct BiDiStripTokenFilter;
impl BiDiStripTokenFilter { pub fn new() -> Self { Self } }
impl Default for BiDiStripTokenFilter { fn default() -> Self { Self } }

impl TokenFilter for BiDiStripTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let cleaned: String = text.chars().filter(|c| !is_bidi_control(*c)).collect();
        if cleaned.len() != text.len() {
            if cleaned.is_empty() {
                return (true, None);
            }
            token.term = Cow::Owned(cleaned);
        }
        (false, None)
    }
}

/// Normalizes confusable/lookalike characters to ASCII equivalents.
/// e.g. Cyrillic 'а' → Latin 'a', fullwidth 'Ａ' → 'A'
#[derive(Clone, Debug)]
pub struct ConfusableNormTokenFilter;
impl ConfusableNormTokenFilter { pub fn new() -> Self { Self } }
impl Default for ConfusableNormTokenFilter { fn default() -> Self { Self } }

impl TokenFilter for ConfusableNormTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let normalized: String = text.chars().map(|c| normalize_confusable(c)).collect();
        if normalized != text {
            token.term = Cow::Owned(normalized);
        }
        (false, None)
    }
}

/// Normalizes common homoglyphs (visual lookalikes) to canonical ASCII.
#[derive(Clone, Debug)]
pub struct HomoglyphNormTokenFilter;
impl HomoglyphNormTokenFilter { pub fn new() -> Self { Self } }
impl Default for HomoglyphNormTokenFilter { fn default() -> Self { Self } }

impl TokenFilter for HomoglyphNormTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let normalized: String = text.chars().map(|c| normalize_homoglyph(c)).collect();
        if normalized != text {
            token.term = Cow::Owned(normalized);
        }
        (false, None)
    }
}

/// Detects mixed scripts in a single token (potential spoofing).
#[derive(Clone, Debug)]
pub struct MixedScriptDetectTokenFilter;
impl MixedScriptDetectTokenFilter { pub fn new() -> Self { Self } }
impl Default for MixedScriptDetectTokenFilter { fn default() -> Self { Self } }

impl TokenFilter for MixedScriptDetectTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if has_mixed_scripts(text) {
            let tag = Token {
                term: Cow::Owned(String::from("_warning:mixed_script")),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            };
            return (false, Some(alloc::vec![tag]));
        }
        (false, None)
    }
}

/// Normalizes fullwidth characters to ASCII equivalents: "Ｈｅｌｌｏ" → "Hello"
#[derive(Clone, Debug)]
pub struct FullwidthNormTokenFilter;
impl FullwidthNormTokenFilter { pub fn new() -> Self { Self } }
impl Default for FullwidthNormTokenFilter { fn default() -> Self { Self } }

impl TokenFilter for FullwidthNormTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let normalized: String = text.chars().map(|c| {
            let cp = c as u32;
            // Fullwidth ASCII variants: U+FF01 to U+FF5E → U+0021 to U+007E
            if (0xFF01..=0xFF5E).contains(&cp) {
                char::from_u32(cp - 0xFF01 + 0x0021).unwrap_or(c)
            } else if cp == 0x3000 { // Ideographic space → regular space
                ' '
            } else {
                c
            }
        }).collect();
        if normalized != text {
            token.term = Cow::Owned(normalized);
        }
        (false, None)
    }
}

/// Strips diacritical marks/combining characters from text.
#[derive(Clone, Debug)]
pub struct DiacriticStripTokenFilter;
impl DiacriticStripTokenFilter { pub fn new() -> Self { Self } }
impl Default for DiacriticStripTokenFilter { fn default() -> Self { Self } }

impl TokenFilter for DiacriticStripTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let stripped: String = text.chars().filter(|c| {
            // Combining diacritical marks: U+0300 to U+036F
            let cp = *c as u32;
            !(0x0300..=0x036F).contains(&cp)
        }).collect();
        if stripped.len() != text.len() {
            token.term = Cow::Owned(stripped);
        }
        (false, None)
    }
}

/// Detects and tags tokens that contain emoji.
#[derive(Clone, Debug)]
pub struct EmojiPresenceTokenFilter;
impl EmojiPresenceTokenFilter { pub fn new() -> Self { Self } }
impl Default for EmojiPresenceTokenFilter { fn default() -> Self { Self } }

impl TokenFilter for EmojiPresenceTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.chars().any(|c| is_emoji(c)) {
            let tag = Token {
                term: Cow::Owned(String::from("_has_emoji:true")),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            };
            return (false, Some(alloc::vec![tag]));
        }
        (false, None)
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────

fn is_invisible_char(c: char) -> bool {
    let cp = c as u32;
    matches!(cp,
        0x200B | 0x200C | 0x200D | 0x200E | 0x200F |  // Zero-width, LRM, RLM
        0x2028 | 0x2029 |  // Line/paragraph separator
        0x202A..=0x202E |   // BiDi embedding
        0x2060..=0x2064 |   // Word joiner, invisible chars
        0x2066..=0x2069 |   // BiDi isolates
        0xFEFF |            // BOM/ZWNBSP
        0xFFF9..=0xFFFB |   // Interlinear annotations
        0x00AD |            // Soft hyphen
        0x034F |            // Combining grapheme joiner
        0x061C |            // Arabic letter mark
        0x180E              // Mongolian vowel separator
    )
}

fn is_zero_width(c: char) -> bool {
    let cp = c as u32;
    matches!(cp, 0x200B | 0x200C | 0x200D | 0xFEFF | 0x2060)
}

fn is_bidi_control(c: char) -> bool {
    let cp = c as u32;
    matches!(cp,
        0x200E | 0x200F |   // LRM, RLM
        0x202A..=0x202E |   // LRE, RLE, PDF, LRO, RLO
        0x2066..=0x2069 |   // LRI, RLI, FSI, PDI
        0x061C              // ALM
    )
}

fn normalize_confusable(c: char) -> char {
    let cp = c as u32;
    match cp {
        // Cyrillic lookalikes → Latin
        0x0430 => 'a', // а
        0x0435 => 'e', // е
        0x043E => 'o', // о
        0x0440 => 'p', // р
        0x0441 => 'c', // с
        0x0443 => 'y', // у
        0x0445 => 'x', // х
        0x0410 => 'A', // А
        0x0412 => 'B', // В
        0x0415 => 'E', // Е
        0x041A => 'K', // К
        0x041C => 'M', // М
        0x041D => 'H', // Н
        0x041E => 'O', // О
        0x0420 => 'P', // Р
        0x0421 => 'C', // С
        0x0422 => 'T', // Т
        0x0425 => 'X', // Х
        // Fullwidth
        0xFF01..=0xFF5E => char::from_u32(cp - 0xFF01 + 0x0021).unwrap_or(c),
        // Greek lookalikes
        0x0391 => 'A', // Α
        0x0392 => 'B', // Β
        0x0395 => 'E', // Ε
        0x0396 => 'Z', // Ζ
        0x0397 => 'H', // Η
        0x0399 => 'I', // Ι
        0x039A => 'K', // Κ
        0x039C => 'M', // Μ
        0x039D => 'N', // Ν
        0x039F => 'O', // Ο
        0x03A1 => 'P', // Ρ
        0x03A4 => 'T', // Τ
        0x03A5 => 'Y', // Υ
        0x03A7 => 'X', // Χ
        _ => c,
    }
}

fn normalize_homoglyph(c: char) -> char {
    match c {
        '０'..='９' => char::from_u32(c as u32 - '０' as u32 + '0' as u32).unwrap_or(c),
        'ⅰ' => 'i',
        'ⅱ' => 'i', // Simplified
        'ℓ' => 'l',
        'ℐ' | 'ℑ' => 'I',
        'ℝ' => 'R',
        'ℕ' => 'N',
        'ℤ' => 'Z',
        'ℚ' => 'Q',
        'ℂ' => 'C',
        _ => normalize_confusable(c),
    }
}

fn has_mixed_scripts(s: &str) -> bool {
    let mut has_latin = false;
    let mut has_cyrillic = false;
    let mut has_greek = false;
    for c in s.chars() {
        if !c.is_alphabetic() { continue; }
        let cp = c as u32;
        match cp {
            0x0041..=0x007A | 0x00C0..=0x024F => has_latin = true,
            0x0400..=0x04FF => has_cyrillic = true,
            0x0370..=0x03FF => has_greek = true,
            _ => {}
        }
    }
    (has_latin as u8 + has_cyrillic as u8 + has_greek as u8) > 1
}

fn is_emoji(c: char) -> bool {
    let cp = c as u32;
    matches!(cp,
        0x1F600..=0x1F64F | 0x1F300..=0x1F5FF | 0x1F680..=0x1F6FF |
        0x1F1E0..=0x1F1FF | 0x2702..=0x27B0 | 0x24C2..=0x1F251 |
        0x1F900..=0x1F9FF | 0x1FA00..=0x1FA6F | 0x1FA70..=0x1FAFF |
        0x2600..=0x26FF | 0x2700..=0x27BF | 0xFE00..=0xFE0F |
        0x1F000..=0x1F02F
    )
}
