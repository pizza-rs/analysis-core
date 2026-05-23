use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

// ═══════════════════════════════════════════════════════════════════════════════
// CHEROKEE ANALYSIS — Syllabary normalization (U+13A0–U+13FF, U+AB70–U+ABBF)
// Cherokee has 85 syllable characters. Each represents a full syllable.
// ═══════════════════════════════════════════════════════════════════════════════

/// Normalizes Cherokee text by converting lowercase syllabary to uppercase.
/// Cherokee was unicameral until Unicode 8.0 added lowercase forms.
/// Most Cherokee text uses uppercase; this normalizes both to uppercase.
#[derive(Clone, Debug)]
pub struct CherokeeNormalizationTokenFilter;
impl CherokeeNormalizationTokenFilter {
    pub fn new() -> Self { Self }
}
impl Default for CherokeeNormalizationTokenFilter {
    fn default() -> Self { Self }
}

impl TokenFilter for CherokeeNormalizationTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let mut changed = false;
        let normalized: String = text.chars().map(|c| {
            let cp = c as u32;
            // Cherokee Supplement lowercase (U+AB70–U+ABBF) → uppercase (U+13A0–U+13EF)
            if (0xAB70..=0xABBF).contains(&cp) {
                changed = true;
                char::from_u32(cp - 0xAB70 + 0x13A0).unwrap_or(c)
            }
            // Cherokee block lowercase (U+13F8–U+13FD) → uppercase (U+13F0–U+13F5)
            else if (0x13F8..=0x13FD).contains(&cp) {
                changed = true;
                char::from_u32(cp - 8).unwrap_or(c)
            } else {
                c
            }
        }).collect();

        if changed {
            token.term = Cow::Owned(normalized);
        }
        (false, None)
    }
}

/// Cherokee transliteration normalizer — normalizes common transliteration
/// variants to a canonical form for Latin-script Cherokee search.
/// Cherokee has standard Latin transliterations using the Worcester system.
#[derive(Clone, Debug)]
pub struct CherokeeTranslitNormTokenFilter;
impl CherokeeTranslitNormTokenFilter {
    pub fn new() -> Self { Self }
}
impl Default for CherokeeTranslitNormTokenFilter {
    fn default() -> Self { Self }
}

impl TokenFilter for CherokeeTranslitNormTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let lower = text.to_lowercase();

        // Normalize common Cherokee transliteration variants
        let normalized = lower
            .replace("ts", "j")     // ts/j variation
            .replace("qu", "kw")    // qu→kw normalization
            .replace("tl", "hl");   // tl→hl variant

        if normalized != lower {
            token.term = Cow::Owned(normalized);
        } else if lower != text {
            token.term = Cow::Owned(lower);
        }
        (false, None)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// KHMER ANALYSIS — Word boundary detection for Khmer script (U+1780–U+17FF)
// Khmer (Cambodian) has no word spaces. Uses zero-width space (U+200B)
// as optional word boundary hint in modern digital text.
// ═══════════════════════════════════════════════════════════════════════════════

/// Khmer word boundary segmentation filter.
/// Splits Khmer text on zero-width spaces (ZWS, U+200B) which are used
/// as invisible word boundary markers in properly prepared Khmer text.
/// Also handles ZWSP and word-joiner characters.
#[derive(Clone, Debug)]
pub struct KhmerWordBoundaryTokenFilter;
impl KhmerWordBoundaryTokenFilter {
    pub fn new() -> Self { Self }
}
impl Default for KhmerWordBoundaryTokenFilter {
    fn default() -> Self { Self }
}

const ZERO_WIDTH_SPACE: char = '\u{200B}';
const ZERO_WIDTH_NON_JOINER: char = '\u{200C}';

impl TokenFilter for KhmerWordBoundaryTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.to_string();

        // Only process text containing Khmer characters
        if !text.chars().any(|c| (c as u32) >= 0x1780 && (c as u32) <= 0x17FF) {
            return (false, None);
        }

        // Split on ZWS and ZWNJ
        let words: Vec<&str> = text.split(|c| c == ZERO_WIDTH_SPACE || c == ZERO_WIDTH_NON_JOINER)
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if words.len() <= 1 {
            // Remove any remaining ZWS
            let cleaned: String = text.chars()
                .filter(|c| *c != ZERO_WIDTH_SPACE && *c != ZERO_WIDTH_NON_JOINER)
                .collect();
            if cleaned != text && !cleaned.is_empty() {
                token.term = Cow::Owned(cleaned);
            }
            return (false, None);
        }

        token.term = Cow::Owned(words[0].to_string());

        let mut extra: Vec<Token<'a>> = Vec::with_capacity(words.len() - 1);
        for (i, word) in words[1..].iter().enumerate() {
            extra.push(Token {
                term: Cow::Owned(word.to_string()),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position + (i as u32) + 1,
            });
        }
        (false, Some(extra))
    }
}

/// Removes Khmer diacritics and sign marks that are optional/redundant
/// for search purposes (e.g., nikahit, reahmuk, etc.)
#[derive(Clone, Debug)]
pub struct KhmerSignRemoveTokenFilter;
impl KhmerSignRemoveTokenFilter {
    pub fn new() -> Self { Self }
}
impl Default for KhmerSignRemoveTokenFilter {
    fn default() -> Self { Self }
}

impl TokenFilter for KhmerSignRemoveTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let cleaned: String = text.chars().filter(|c| {
            let cp = *c as u32;
            // Remove Khmer signs that are redundant for search
            !matches!(cp,
                0x17B4..=0x17B5 | // Inherent vowel markers (invisible)
                0x17D4..=0x17DA | // Punctuation (khan, bariyoosan, etc.)
                0x17DC |          // Avakrahasanya
                0x17DD           // Atthacan
            )
        }).collect();
        if cleaned != text {
            if cleaned.is_empty() {
                return (true, None);
            }
            token.term = Cow::Owned(cleaned);
        }
        (false, None)
    }
}
