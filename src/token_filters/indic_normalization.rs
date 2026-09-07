use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Normalizes the Unicode representation of text in Indian languages —
/// faithful port of Lucene's `IndicNormalizer`.
///
/// Follows guidelines from Unicode 5.2, chapter 6, South Asian Scripts I and
/// graphical decompositions from
/// <http://ldc.upenn.edu/myl/IndianScriptsUnicode.html>. The normalizer
/// *composes* decomposed sequences (e.g. `अ` + `ा` → `आ`, consonant + nukta →
/// precomposed nukta form, Bengali `ত` + virama + ZWJ → khanda-ta `ৎ`) into
/// their standard precomposed forms across nine scripts: Devanagari, Bengali,
/// Gurmukhi, Gujarati, Oriya, Tamil, Telugu, Kannada and Malayalam.
///
/// Unlike the previous approximation, zero-width joiners are NOT stripped
/// unconditionally: a ZWJ is only consumed as the third element of a
/// three-character composition (khanda-ta / Malayalam chillu), exactly as in
/// Lucene.
#[derive(Clone, Debug, Default)]
pub struct IndicNormalizationTokenFilter;

impl IndicNormalizationTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

// Script flag bits and base codepoints, mirroring the Java ScriptData table.
const DEVANAGARI: u16 = 1;
const BENGALI: u16 = 2;
const GURMUKHI: u16 = 4;
const GUJARATI: u16 = 8;
const ORIYA: u16 = 16;
const TAMIL: u16 = 32;
const TELUGU: u16 = 64;
const KANNADA: u16 = 128;
const MALAYALAM: u16 = 256;

/// Decompositions according to Unicode 5.2. Offsets are relative to the
/// script base; `ch2` is `-1` for two-character rules and `0xFF` marks a
/// zero-width joiner. Columns: (ch1, ch2, ch3, res, script flags).
const DECOMPOSITIONS: &[(u8, u8, i16, u8, u16)] = &[
    // devanagari, gujarati vowel candra O
    (0x05, 0x3E, 0x45, 0x11, DEVANAGARI | GUJARATI),
    // devanagari short O
    (0x05, 0x3E, 0x46, 0x12, DEVANAGARI),
    // devanagari, gujarati letter O
    (0x05, 0x3E, 0x47, 0x13, DEVANAGARI | GUJARATI),
    // devanagari letter AI, gujarati letter AU
    (0x05, 0x3E, 0x48, 0x14, DEVANAGARI | GUJARATI),
    // devanagari, bengali, gurmukhi, gujarati, oriya AA
    (
        0x05,
        0x3E,
        -1,
        0x06,
        DEVANAGARI | BENGALI | GURMUKHI | GUJARATI | ORIYA,
    ),
    // devanagari letter candra A
    (0x05, 0x45, -1, 0x72, DEVANAGARI),
    // gujarati vowel candra E
    (0x05, 0x45, -1, 0x0D, GUJARATI),
    // devanagari letter short A
    (0x05, 0x46, -1, 0x04, DEVANAGARI),
    // gujarati letter E
    (0x05, 0x47, -1, 0x0F, GUJARATI),
    // gurmukhi, gujarati letter AI
    (0x05, 0x48, -1, 0x10, GURMUKHI | GUJARATI),
    // devanagari, gujarati vowel candra O
    (0x05, 0x49, -1, 0x11, DEVANAGARI | GUJARATI),
    // devanagari short O
    (0x05, 0x4A, -1, 0x12, DEVANAGARI),
    // devanagari, gujarati letter O
    (0x05, 0x4B, -1, 0x13, DEVANAGARI | GUJARATI),
    // devanagari letter AI, gurmukhi letter AU, gujarati letter AU
    (0x05, 0x4C, -1, 0x14, DEVANAGARI | GURMUKHI | GUJARATI),
    // devanagari, gujarati vowel candra O
    (0x06, 0x45, -1, 0x11, DEVANAGARI | GUJARATI),
    // devanagari short O
    (0x06, 0x46, -1, 0x12, DEVANAGARI),
    // devanagari, gujarati letter O
    (0x06, 0x47, -1, 0x13, DEVANAGARI | GUJARATI),
    // devanagari letter AI, gujarati letter AU
    (0x06, 0x48, -1, 0x14, DEVANAGARI | GUJARATI),
    // malayalam letter II
    (0x07, 0x57, -1, 0x08, MALAYALAM),
    // devanagari letter UU
    (0x09, 0x41, -1, 0x0A, DEVANAGARI),
    // tamil, malayalam letter UU (some styles)
    (0x09, 0x57, -1, 0x0A, TAMIL | MALAYALAM),
    // malayalam letter AI
    (0x0E, 0x46, -1, 0x10, MALAYALAM),
    // devanagari candra E
    (0x0F, 0x45, -1, 0x0D, DEVANAGARI),
    // devanagari short E
    (0x0F, 0x46, -1, 0x0E, DEVANAGARI),
    // devanagari AI
    (0x0F, 0x47, -1, 0x10, DEVANAGARI),
    // oriya AI
    (0x0F, 0x57, -1, 0x10, ORIYA),
    // malayalam letter OO
    (0x12, 0x3E, -1, 0x13, MALAYALAM),
    // telugu, kannada letter AU
    (0x12, 0x4C, -1, 0x14, TELUGU | KANNADA),
    // telugu letter OO
    (0x12, 0x55, -1, 0x13, TELUGU),
    // tamil, malayalam letter AU
    (0x12, 0x57, -1, 0x14, TAMIL | MALAYALAM),
    // oriya letter AU
    (0x13, 0x57, -1, 0x14, ORIYA),
    // devanagari qa
    (0x15, 0x3C, -1, 0x58, DEVANAGARI),
    // devanagari, gurmukhi khha
    (0x16, 0x3C, -1, 0x59, DEVANAGARI | GURMUKHI),
    // devanagari, gurmukhi ghha
    (0x17, 0x3C, -1, 0x5A, DEVANAGARI | GURMUKHI),
    // devanagari, gurmukhi za
    (0x1C, 0x3C, -1, 0x5B, DEVANAGARI | GURMUKHI),
    // devanagari dddha, bengali, oriya rra
    (0x21, 0x3C, -1, 0x5C, DEVANAGARI | BENGALI | ORIYA),
    // devanagari, bengali, oriya rha
    (0x22, 0x3C, -1, 0x5D, DEVANAGARI | BENGALI | ORIYA),
    // malayalam chillu nn
    (0x23, 0x4D, 0xFF, 0x7A, MALAYALAM),
    // bengali khanda ta
    (0x24, 0x4D, 0xFF, 0x4E, BENGALI),
    // devanagari nnna
    (0x28, 0x3C, -1, 0x29, DEVANAGARI),
    // malayalam chillu n
    (0x28, 0x4D, 0xFF, 0x7B, MALAYALAM),
    // devanagari, gurmukhi fa
    (0x2B, 0x3C, -1, 0x5E, DEVANAGARI | GURMUKHI),
    // devanagari, bengali yya
    (0x2F, 0x3C, -1, 0x5F, DEVANAGARI | BENGALI),
    // telugu letter vocalic R
    (0x2C, 0x41, 0x41, 0x0B, TELUGU),
    // devanagari rra
    (0x30, 0x3C, -1, 0x31, DEVANAGARI),
    // malayalam chillu rr
    (0x30, 0x4D, 0xFF, 0x7C, MALAYALAM),
    // malayalam chillu l
    (0x32, 0x4D, 0xFF, 0x7D, MALAYALAM),
    // devanagari llla
    (0x33, 0x3C, -1, 0x34, DEVANAGARI),
    // malayalam chillu ll
    (0x33, 0x4D, 0xFF, 0x7E, MALAYALAM),
    // telugu letter MA
    (0x35, 0x41, -1, 0x2E, TELUGU),
    // devanagari, gujarati vowel sign candra O
    (0x3E, 0x45, -1, 0x49, DEVANAGARI | GUJARATI),
    // devanagari vowel sign short O
    (0x3E, 0x46, -1, 0x4A, DEVANAGARI),
    // devanagari, gujarati vowel sign O
    (0x3E, 0x47, -1, 0x4B, DEVANAGARI | GUJARATI),
    // devanagari, gujarati vowel sign AU
    (0x3E, 0x48, -1, 0x4C, DEVANAGARI | GUJARATI),
    // kannada vowel sign II
    (0x3F, 0x55, -1, 0x40, KANNADA),
    // gurmukhi vowel sign UU (when stacking)
    (0x41, 0x41, -1, 0x42, GURMUKHI),
    // tamil, malayalam vowel sign O
    (0x46, 0x3E, -1, 0x4A, TAMIL | MALAYALAM),
    // kannada vowel sign OO
    (0x46, 0x42, 0x55, 0x4B, KANNADA),
    // kannada vowel sign O
    (0x46, 0x42, -1, 0x4A, KANNADA),
    // malayalam vowel sign AI (if reordered twice)
    (0x46, 0x46, -1, 0x48, MALAYALAM),
    // telugu, kannada vowel sign EE
    (0x46, 0x55, -1, 0x47, TELUGU | KANNADA),
    // telugu, kannada vowel sign AI
    (0x46, 0x56, -1, 0x48, TELUGU | KANNADA),
    // tamil, malayalam vowel sign AU
    (0x46, 0x57, -1, 0x4C, TAMIL | MALAYALAM),
    // bengali, oriya vowel sign O, tamil, malayalam vowel sign OO
    (0x47, 0x3E, -1, 0x4B, BENGALI | ORIYA | TAMIL | MALAYALAM),
    // bengali, oriya vowel sign AU
    (0x47, 0x57, -1, 0x4C, BENGALI | ORIYA),
    // kannada vowel sign OO
    (0x4A, 0x55, -1, 0x4B, KANNADA),
    // gurmukhi letter I
    (0x72, 0x3F, -1, 0x07, GURMUKHI),
    // gurmukhi letter II
    (0x72, 0x40, -1, 0x08, GURMUKHI),
    // gurmukhi letter EE
    (0x72, 0x47, -1, 0x0F, GURMUKHI),
    // gurmukhi letter U
    (0x73, 0x41, -1, 0x09, GURMUKHI),
    // gurmukhi letter UU
    (0x73, 0x42, -1, 0x0A, GURMUKHI),
    // gurmukhi letter OO
    (0x73, 0x4B, -1, 0x13, GURMUKHI),
];

/// Script base codepoint for a character, or None when the character does not
/// belong to one of the nine supported Indic blocks.
fn script_base(c: char) -> Option<(u32, u16)> {
    let cp = c as u32;
    match cp {
        0x0900..=0x097F => Some((0x0900, DEVANAGARI)),
        0x0980..=0x09FF => Some((0x0980, BENGALI)),
        0x0A00..=0x0A7F => Some((0x0A00, GURMUKHI)),
        0x0A80..=0x0AFF => Some((0x0A80, GUJARATI)),
        0x0B00..=0x0B7F => Some((0x0B00, ORIYA)),
        0x0B80..=0x0BFF => Some((0x0B80, TAMIL)),
        0x0C00..=0x0C7F => Some((0x0C00, TELUGU)),
        0x0C80..=0x0CFF => Some((0x0C80, KANNADA)),
        0x0D00..=0x0D7F => Some((0x0D00, MALAYALAM)),
        _ => None,
    }
}

impl TokenFilter for IndicNormalizationTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if !text.chars().any(|c| script_base(c).is_some()) {
            return (false, None);
        }

        let mut chars: Vec<char> = text.chars().collect();
        let len = normalize(&mut chars);
        let result: String = chars[..len].iter().collect();
        if result == text {
            return (false, None);
        }
        token.term = Cow::Owned(result);
        (false, None)
    }
}

/// Port of `IndicNormalizer.normalize`: composes decomposed sequences
/// in-place, returning the new length (always <= input length).
fn normalize(text: &mut [char]) -> usize {
    let mut len = text.len();
    let mut i = 0;
    while i < len {
        let Some((base, flag)) = script_base(text[i]) else {
            i += 1;
            continue;
        };
        let ch = (text[i] as u32 - base) as u8;
        if is_decomposable(ch, flag) {
            len = compose(ch, base, flag, text, i, len);
        }
        i += 1;
    }
    len
}

/// Whether any decomposition rule starts at `ch` for this script (the Java
/// BitSet mask). Computed per call over the static table — the table is small
/// and this keeps the port simple.
fn is_decomposable(ch: u8, flag: u16) -> bool {
    DECOMPOSITIONS
        .iter()
        .any(|&(c1, _, _, _, flags)| c1 == ch && (flags & flag) != 0)
}

/// Port of `IndicNormalizer.compose`.
fn compose(ch0: u8, base: u32, flag: u16, text: &mut [char], pos: usize, len: usize) -> usize {
    if pos + 1 >= len {
        // need at least 2 chars
        return len;
    }
    // the second character must belong to the same writing system
    let next = text[pos + 1];
    if script_base(next) != Some((base, flag)) {
        return len;
    }
    let ch1 = (next as u32 - base) as u8;

    let mut ch2: i16 = -1;
    if pos + 2 < len {
        let third = text[pos + 2];
        if third == '\u{200D}' {
            // ZWJ
            ch2 = 0xFF;
        } else if script_base(third) != Some((base, flag)) {
            // still allow a 2-char match
            ch2 = -1;
        } else {
            ch2 = (third as u32 - base) as i16;
        }
    }

    for &(c1, c2, c3, res, flags) in DECOMPOSITIONS {
        if c1 == ch0 && (flags & flag) != 0 && c2 == ch1 && (c3 < 0 || c3 == ch2) {
            text[pos] = char::from_u32(base + res as u32).unwrap_or(text[pos]);
            let mut new_len = delete_at(text, pos + 1, len);
            if c3 >= 0 {
                new_len = delete_at(text, pos + 1, new_len);
            }
            return new_len;
        }
    }
    len
}

/// Port of `StemmerUtil.delete`: shift the tail left by one.
fn delete_at(text: &mut [char], pos: usize, len: usize) -> usize {
    if pos >= len {
        return len;
    }
    text.copy_within(pos + 1..len, pos);
    len - 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use pizza_engine::analysis::TokenFilter;

    fn make_token(term: &str) -> Token<'_> {
        Token {
            term: Cow::Borrowed(term),
            start_offset: 0,
            end_offset: term.len() as u32,
            position: 0,
        }
    }

    #[test]
    fn test_devanagari_nukta() {
        let filter = IndicNormalizationTokenFilter::new();
        // क + ़ → क़
        let mut token = make_token("\u{0915}\u{093C}");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "\u{0958}");
    }

    #[test]
    fn test_bengali_nukta() {
        let filter = IndicNormalizationTokenFilter::new();
        // ড + ় → ড়
        let mut token = make_token("\u{09A1}\u{09BC}");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "\u{09DC}");
    }

    // Vectors from Lucene's TestIndicNormalizer.testBasics.
    #[test]
    fn test_lucene_vectors() {
        let filter = IndicNormalizationTokenFilter::new();
        // अ + ॉ (candra O) → ऑ
        let mut token = make_token("अाॅअाॅ");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "ऑऑ");
        // अ + ॆ → ऒ
        let mut token = make_token("अाॆअाॆ");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "ऒऒ");
        // अ + े → ओ
        let mut token = make_token("अाेअाे");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "ओओ");
        // अ + ै → औ
        let mut token = make_token("अाैअाै");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "औऔ");
        // अ + ा → आ
        let mut token = make_token("अाअा");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "आआ");
        let mut token = make_token("अाैर");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "और");
        // khanda-ta: ত + ্ + ZWJ → ৎ
        let mut token = make_token("ত্\u{200D}");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "ৎ");
    }

    #[test]
    fn test_zwj_kept_outside_composition() {
        let filter = IndicNormalizationTokenFilter::new();
        // Lucene only consumes ZWJ inside khanda-ta/chillu compositions;
        // elsewhere it stays in the term.
        let mut token = make_token("क\u{200D}ष");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "क\u{200D}ष");
    }

    #[test]
    fn test_non_indic_passthrough() {
        let filter = IndicNormalizationTokenFilter::new();
        let mut token = make_token("hello");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "hello");
    }
}
