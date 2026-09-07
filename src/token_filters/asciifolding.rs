use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Folds Unicode characters to their ASCII equivalents.
///
/// For example: ü→u, é→e, ñ→n, ö→o, ß→ss
#[derive(Clone, Debug)]
pub struct AsciiFoldingTokenFilter {
    pub preserve_original: bool,
}

impl AsciiFoldingTokenFilter {
    pub fn new() -> Self {
        Self {
            preserve_original: false,
        }
    }

    pub fn preserving_original() -> Self {
        Self {
            preserve_original: true,
        }
    }
}

impl Default for AsciiFoldingTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Fold a single character to its ASCII equivalent(s).
fn fold_char(ch: char) -> Option<&'static str> {
    match ch {
        'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' | 'Ā' | 'Ă' | 'Ą' => Some("A"),
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' => Some("a"),
        'Æ' => Some("AE"),
        'æ' => Some("ae"),
        'Ç' | 'Ć' | 'Ĉ' | 'Ċ' | 'Č' => Some("C"),
        'ç' | 'ć' | 'ĉ' | 'ċ' | 'č' => Some("c"),
        'Ð' | 'Ď' | 'Đ' => Some("D"),
        'ð' | 'ď' | 'đ' => Some("d"),
        'È' | 'É' | 'Ê' | 'Ë' | 'Ē' | 'Ĕ' | 'Ė' | 'Ę' | 'Ě' => Some("E"),
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' => Some("e"),
        'Ĝ' | 'Ğ' | 'Ġ' | 'Ģ' => Some("G"),
        'ĝ' | 'ğ' | 'ġ' | 'ģ' => Some("g"),
        'Ĥ' | 'Ħ' => Some("H"),
        'ĥ' | 'ħ' => Some("h"),
        'Ì' | 'Í' | 'Î' | 'Ï' | 'Ĩ' | 'Ī' | 'Ĭ' | 'Į' | 'İ' => Some("I"),
        'ì' | 'í' | 'î' | 'ï' | 'ĩ' | 'ī' | 'ĭ' | 'į' => Some("i"),
        'Ĳ' => Some("IJ"),
        'ĳ' => Some("ij"),
        'Ĵ' => Some("J"),
        'ĵ' => Some("j"),
        'Ķ' => Some("K"),
        'ķ' => Some("k"),
        'Ĺ' | 'Ļ' | 'Ľ' | 'Ŀ' | 'Ł' => Some("L"),
        'ĺ' | 'ļ' | 'ľ' | 'ŀ' | 'ł' => Some("l"),
        'Ñ' | 'Ń' | 'Ņ' | 'Ň' => Some("N"),
        'ñ' | 'ń' | 'ņ' | 'ň' | 'ŉ' => Some("n"),
        'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'Ø' | 'Ō' | 'Ŏ' | 'Ő' => Some("O"),
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' | 'ŏ' | 'ő' => Some("o"),
        'Œ' => Some("OE"),
        'œ' => Some("oe"),
        'Ŕ' | 'Ŗ' | 'Ř' => Some("R"),
        'ŕ' | 'ŗ' | 'ř' => Some("r"),
        'Ś' | 'Ŝ' | 'Ş' | 'Š' => Some("S"),
        'ś' | 'ŝ' | 'ş' | 'š' => Some("s"),
        'ß' => Some("ss"),
        'Ţ' | 'Ť' | 'Ŧ' => Some("T"),
        'ţ' | 'ť' | 'ŧ' => Some("t"),
        'Þ' => Some("TH"),
        'þ' => Some("th"),
        'Ù' | 'Ú' | 'Û' | 'Ü' | 'Ũ' | 'Ū' | 'Ŭ' | 'Ů' | 'Ű' | 'Ų' => Some("U"),
        'ù' | 'ú' | 'û' | 'ü' | 'ũ' | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' => Some("u"),
        'Ŵ' => Some("W"),
        'ŵ' => Some("w"),
        'Ý' | 'Ŷ' | 'Ÿ' => Some("Y"),
        'ý' | 'ÿ' | 'ŷ' => Some("y"),
        'Ź' | 'Ż' | 'Ž' => Some("Z"),
        'ź' | 'ż' | 'ž' => Some("z"),
        'ƒ' => Some("f"),
        // Latin ligatures (U+FB00..U+FB06), matching Lucene's ASCIIFoldingFilter
        'ﬀ' => Some("ff"),
        'ﬁ' => Some("fi"),
        'ﬂ' => Some("fl"),
        'ﬃ' => Some("ffi"),
        'ﬄ' => Some("ffl"),
        'ﬆ' => Some("st"),
        // Greek
        'Α' => Some("A"),
        'Β' => Some("B"),
        'Γ' => Some("G"),
        'Δ' => Some("D"),
        'Ε' => Some("E"),
        'Ζ' => Some("Z"),
        'Η' => Some("H"),
        'Θ' => Some("Th"),
        'Ι' => Some("I"),
        'Κ' => Some("K"),
        'Λ' => Some("L"),
        'Μ' => Some("M"),
        'Ν' => Some("N"),
        'Ξ' => Some("X"),
        'Ο' => Some("O"),
        'Π' => Some("P"),
        'Ρ' => Some("R"),
        'Σ' => Some("S"),
        'Τ' => Some("T"),
        'Υ' => Some("U"),
        'Φ' => Some("Ph"),
        'Χ' => Some("Ch"),
        'Ψ' => Some("Ps"),
        'Ω' => Some("O"),
        'α' => Some("a"),
        'β' => Some("b"),
        'γ' => Some("g"),
        'δ' => Some("d"),
        'ε' => Some("e"),
        'ζ' => Some("z"),
        'η' => Some("h"),
        'θ' => Some("th"),
        'ι' => Some("i"),
        'κ' => Some("k"),
        'λ' => Some("l"),
        'μ' => Some("m"),
        'ν' => Some("n"),
        'ξ' => Some("x"),
        'ο' => Some("o"),
        'π' => Some("p"),
        'ρ' => Some("r"),
        'σ' | 'ς' => Some("s"),
        'τ' => Some("t"),
        'υ' => Some("u"),
        'φ' => Some("ph"),
        'χ' => Some("ch"),
        'ψ' => Some("ps"),
        'ω' => Some("o"),
        // Cyrillic (basic)
        'А' => Some("A"),
        'Б' => Some("B"),
        'В' => Some("V"),
        'Г' => Some("G"),
        'Д' => Some("D"),
        'Е' => Some("E"),
        'Ж' => Some("Zh"),
        'З' => Some("Z"),
        'И' => Some("I"),
        'Й' => Some("J"),
        'К' => Some("K"),
        'Л' => Some("L"),
        'М' => Some("M"),
        'Н' => Some("N"),
        'О' => Some("O"),
        'П' => Some("P"),
        'Р' => Some("R"),
        'С' => Some("S"),
        'Т' => Some("T"),
        'У' => Some("U"),
        'Ф' => Some("F"),
        'Х' => Some("H"),
        'Ц' => Some("Ts"),
        'Ч' => Some("Ch"),
        'Ш' => Some("Sh"),
        'Щ' => Some("Shch"),
        'Ъ' => Some(""),
        'Ы' => Some("Y"),
        'Ь' => Some(""),
        'Э' => Some("E"),
        'Ю' => Some("Yu"),
        'Я' => Some("Ya"),
        'а' => Some("a"),
        'б' => Some("b"),
        'в' => Some("v"),
        'г' => Some("g"),
        'д' => Some("d"),
        'е' => Some("e"),
        'ж' => Some("zh"),
        'з' => Some("z"),
        'и' => Some("i"),
        'й' => Some("j"),
        'к' => Some("k"),
        'л' => Some("l"),
        'м' => Some("m"),
        'н' => Some("n"),
        'о' => Some("o"),
        'п' => Some("p"),
        'р' => Some("r"),
        'с' => Some("s"),
        'т' => Some("t"),
        'у' => Some("u"),
        'ф' => Some("f"),
        'х' => Some("h"),
        'ц' => Some("ts"),
        'ч' => Some("ch"),
        'ш' => Some("sh"),
        'щ' => Some("shch"),
        'ъ' => Some(""),
        'ы' => Some("y"),
        'ь' => Some(""),
        'э' => Some("e"),
        'ю' => Some("yu"),
        'я' => Some("ya"),
        _ => None,
    }
}

/// Fold an entire string, returning None if no changes were made.
fn fold_to_ascii(s: &str) -> Option<alloc::string::String> {
    let mut result = alloc::string::String::new();
    let mut changed = false;

    for ch in s.chars() {
        if ch.is_ascii() {
            result.push(ch);
        } else if let Some(folded) = fold_char(ch) {
            result.push_str(folded);
            changed = true;
        } else if let Some(folded) = fold_programmatic(ch) {
            result.push_str(&folded);
            changed = true;
        } else {
            result.push(ch);
        }
    }

    if changed {
        Some(result)
    } else {
        None
    }
}

/// Programmatic fallback covering large, regular Unicode ranges that would
/// bloat the static match table:
///
/// - **Fullwidth Forms** (U+FF01-U+FF5E) — wide ASCII variants commonly
///   used in CJK text.
/// - **Latin Extended Additional** (U+1E00-U+1EFF) — precomposed Latin
///   letters with combining diacritics, the source of most Vietnamese
///   characters Lucene's `ASCIIFoldingFilter` handles but our static
///   table previously missed.
/// - **Mathematical Alphanumeric Symbols** (U+1D400-U+1D7FF) — styled
///   ASCII alphabets (bold, italic, script, fraktur, …) and digits.
/// - **Halfwidth Katakana / Symbols** (U+FF61-U+FF9F) is intentionally
///   NOT folded (these are CJK characters, not Latin).
///
/// Returns `None` if the codepoint is outside these ranges.
fn fold_programmatic(ch: char) -> Option<alloc::string::String> {
    use alloc::string::ToString;
    let cp = ch as u32;

    // Fullwidth ASCII: U+FF01 (!) .. U+FF5E (~). Map back to U+0021..U+007E.
    if (0xFF01..=0xFF5E).contains(&cp) {
        let ascii = (cp - 0xFF00 + 0x20) as u8;
        return Some((ascii as char).to_string());
    }
    // Ideographic space → ASCII space.
    if cp == 0x3000 {
        return Some(" ".to_string());
    }

    // Latin Extended Additional (U+1E00..U+1EFF): each pair is upper/lower
    // of a precomposed Latin letter with diacritic. Decompose by base letter.
    if (0x1E00..=0x1EFF).contains(&cp) {
        if let Some(base) = latin_extended_additional_base(cp) {
            return Some(base.to_string());
        }
    }

    // Mathematical Alphanumeric Symbols (U+1D400..U+1D7FF):
    // 13 stylistic alphabets of 52 letters each (A-Z then a-z), followed
    // by digit ranges. We strip the style and recover the base ASCII.
    if (0x1D400..=0x1D7FF).contains(&cp) {
        if let Some(base) = math_alphanumeric_base(cp) {
            return Some(base.to_string());
        }
    }

    None
}

/// Map a Latin Extended Additional codepoint to its ASCII base letter.
/// The block (U+1E00..U+1EFF) lays out precomposed Latin letters with
/// combining diacritics in alternating Upper/lower pairs by base letter,
/// but the *base letter* changes at irregular boundaries, so we encode
/// the actual per-letter ranges from the Unicode chart.
fn latin_extended_additional_base(cp: u32) -> Option<char> {
    // Main block U+1E00..U+1E95 — pairs Upper/lower per base letter.
    // Each row: (start, end_inclusive, Upper, lower).
    let table: &[(u32, u32, char, char)] = &[
        (0x1E00, 0x1E01, 'A', 'a'), // A ring below
        (0x1E02, 0x1E07, 'B', 'b'), // B dot above / below / line below
        (0x1E08, 0x1E09, 'C', 'c'), // C cedilla acute
        (0x1E0A, 0x1E13, 'D', 'd'), // D dot above / below / line / cedilla / circumflex below
        (0x1E14, 0x1E1D, 'E', 'e'), // E macron grave / acute / circumflex below / tilde below / cedilla breve
        (0x1E1E, 0x1E1F, 'F', 'f'), // F dot above
        (0x1E20, 0x1E21, 'G', 'g'), // G macron
        (0x1E22, 0x1E2B, 'H', 'h'), // H dot above / below / diaeresis / cedilla / breve below
        (0x1E2C, 0x1E2F, 'I', 'i'), // I tilde below / diaeresis acute
        (0x1E30, 0x1E35, 'K', 'k'), // K acute / dot below / line below
        (0x1E36, 0x1E3D, 'L', 'l'), // L dot below / dot-below macron / line below / circumflex below
        (0x1E3E, 0x1E43, 'M', 'm'), // M acute / dot above / dot below
        (0x1E44, 0x1E4B, 'N', 'n'), // N dot above / below / line below / circumflex below
        (0x1E4C, 0x1E53, 'O', 'o'), // O tilde acute / diaeresis / macron grave / acute
        (0x1E54, 0x1E57, 'P', 'p'), // P acute / dot above
        (0x1E58, 0x1E5F, 'R', 'r'), // R dot above / below / dot-below macron / line below
        (0x1E60, 0x1E69, 'S', 's'), // S dot above / below / acute / caron / dot-below dot-above
        (0x1E6A, 0x1E71, 'T', 't'), // T dot above / below / line below / circumflex below
        (0x1E72, 0x1E7B, 'U', 'u'), // U diaeresis below / tilde below / circumflex below / tilde acute / macron diaeresis
        (0x1E7C, 0x1E7F, 'V', 'v'), // V tilde / dot below
        (0x1E80, 0x1E89, 'W', 'w'), // W grave / acute / diaeresis / dot above / below
        (0x1E8A, 0x1E8D, 'X', 'x'), // X dot above / diaeresis
        (0x1E8E, 0x1E8F, 'Y', 'y'), // Y dot above
        (0x1E90, 0x1E95, 'Z', 'z'), // Z circumflex / dot below / line below
    ];
    for &(start, end, upper, lower) in table {
        if cp >= start && cp <= end {
            return Some(if (cp - start) % 2 == 0 { upper } else { lower });
        }
    }

    // U+1E96..U+1E9F: a few stragglers we map individually.
    match cp {
        0x1E96 => return Some('h'), // h with line below
        0x1E97 => return Some('t'), // t with diaeresis
        0x1E98 => return Some('w'), // w with ring above
        0x1E99 => return Some('y'), // y with ring above
        0x1E9A => return Some('a'), // a with right half ring
        0x1E9B => return Some('s'), // long s with dot above
        0x1E9E => return Some('S'), // capital sharp S (uppercase ß)
        _ => {}
    }

    // Vietnamese precomposed block (U+1EA0..U+1EF9): pairs Upper/lower
    // for A, E, I, O, U, Y with various tone marks.
    let vn_table: &[(u32, u32, char, char)] = &[
        (0x1EA0, 0x1EB7, 'A', 'a'),
        (0x1EB8, 0x1EC7, 'E', 'e'),
        (0x1EC8, 0x1ECB, 'I', 'i'),
        (0x1ECC, 0x1EE3, 'O', 'o'),
        (0x1EE4, 0x1EF1, 'U', 'u'),
        (0x1EF2, 0x1EF9, 'Y', 'y'),
    ];
    for &(start, end, upper, lower) in vn_table {
        if cp >= start && cp <= end {
            return Some(if (cp - start) % 2 == 0 { upper } else { lower });
        }
    }
    None
}

/// Map a Mathematical Alphanumeric Symbols codepoint (U+1D400..U+1D7FF) to
/// its ASCII base. The block is organized as 13 styles × 52 letters
/// (A-Z then a-z) starting at U+1D400, with sporadic gaps for letters
/// pre-existing in earlier blocks (e.g. ℎ at U+210E replaces U+1D455).
/// We use the dense layout and accept that gap codepoints simply don't
/// match this branch (they are handled by the static table if present).
fn math_alphanumeric_base(cp: u32) -> Option<char> {
    // Letters: U+1D400..U+1D6A3 (52 letters × 13 styles = 676 slots).
    if (0x1D400..=0x1D6A3).contains(&cp) {
        let idx = (cp - 0x1D400) % 52;
        let base = if idx < 26 {
            b'A' + idx as u8
        } else {
            b'a' + (idx - 26) as u8
        };
        return Some(base as char);
    }
    // Greek letters block (U+1D6A4..U+1D7C9) — out of scope; rely on
    // the static Greek mapping in `fold_char` instead.

    // Digits: U+1D7CE..U+1D7FF (10 digits × 5 styles = 50 slots).
    if (0x1D7CE..=0x1D7FF).contains(&cp) {
        let idx = (cp - 0x1D7CE) % 10;
        return Some((b'0' + idx as u8) as char);
    }
    None
}

impl TokenFilter for AsciiFoldingTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        if let Some(folded) = fold_to_ascii(&token.term) {
            if self.preserve_original {
                let extra = Token {
                    term: Cow::Owned(folded),
                    start_offset: token.start_offset,
                    end_offset: token.end_offset,
                    position: token.position,
                };
                return (false, Some(alloc::vec![extra]));
            } else {
                token.term = Cow::Owned(folded);
            }
        }
        (false, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asciifolding() {
        let f = AsciiFoldingTokenFilter::new();
        let mut token = Token::new("café", 0, 5, 0);
        let (deleted, extra) = f.filter(&mut token);
        assert!(!deleted);
        assert!(extra.is_none());
        assert_eq!(token.term.as_ref(), "cafe");
    }

    #[test]
    fn test_asciifolding_german() {
        let f = AsciiFoldingTokenFilter::new();
        let mut token = Token::new("straße", 0, 7, 0);
        let (_, _) = f.filter(&mut token);
        assert_eq!(token.term.as_ref(), "strasse");
    }

    #[test]
    fn test_asciifolding_preserve() {
        let f = AsciiFoldingTokenFilter::preserving_original();
        let mut token = Token::new("résumé", 0, 8, 0);
        let (deleted, extra) = f.filter(&mut token);
        assert!(!deleted);
        assert_eq!(token.term.as_ref(), "résumé"); // Original preserved
        let extra = extra.unwrap();
        assert_eq!(extra[0].term.as_ref(), "resume"); // Folded emitted
    }

    #[test]
    fn test_asciifolding_no_change() {
        let f = AsciiFoldingTokenFilter::new();
        let mut token = Token::new("hello", 0, 5, 0);
        let (deleted, extra) = f.filter(&mut token);
        assert!(!deleted);
        assert!(extra.is_none());
        assert_eq!(token.term.as_ref(), "hello");
    }

    #[test]
    fn test_asciifolding_fullwidth() {
        // Fullwidth ASCII letters and digits (e.g., used in CJK contexts).
        let f = AsciiFoldingTokenFilter::new();
        let mut token = Token::new("ＡＢＣ１２３", 0, 0, 0);
        let _ = f.filter(&mut token);
        assert_eq!(token.term.as_ref(), "ABC123");
    }

    #[test]
    fn test_asciifolding_vietnamese() {
        // Latin Extended Additional / Vietnamese precomposed forms.
        let f = AsciiFoldingTokenFilter::new();
        let mut token = Token::new("Tiếng Việt", 0, 0, 0);
        let _ = f.filter(&mut token);
        assert_eq!(token.term.as_ref(), "Tieng Viet");
    }

    #[test]
    fn test_asciifolding_math_alphanumeric() {
        // Mathematical Alphanumeric Symbols: bold 'H', italic 'i', etc.
        let f = AsciiFoldingTokenFilter::new();
        // 𝐇 = U+1D407 (math bold capital H), 𝐢 = U+1D422 (math bold small i)
        let mut token = Token::new("𝐇𝐢", 0, 0, 0);
        let _ = f.filter(&mut token);
        assert_eq!(token.term.as_ref(), "Hi");
    }

    #[test]
    fn test_asciifolding_math_digits() {
        // Math bold digits 𝟎-𝟗 should fold to ASCII 0-9.
        let f = AsciiFoldingTokenFilter::new();
        // 𝟏 = U+1D7CF (math bold digit 1), 𝟐 = U+1D7D0
        let mut token = Token::new("𝟏𝟐", 0, 0, 0);
        let _ = f.filter(&mut token);
        assert_eq!(token.term.as_ref(), "12");
    }
}
