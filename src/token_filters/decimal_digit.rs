use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Folds all Unicode decimal digits (General_Category=Decimal_Number) to
/// Basic Latin digits (`0-9`).
///
/// This is equivalent to Lucene's `DecimalDigitFilter`.
///
/// For example, Arabic-Indic digits `١٢٣٤` become `1234`.
#[derive(Clone, Debug, Default)]
pub struct DecimalDigitTokenFilter;

impl DecimalDigitTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for DecimalDigitTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        // Fast path: check if any non-ASCII digit exists
        let needs_folding = text
            .chars()
            .any(|c| c > '\x7F' && c.is_ascii_digit() == false && c.is_numeric());
        if !needs_folding {
            // More precise check using Character numeric value
            let needs = text.chars().any(|c| c > '\x7F' && char_is_decimal_digit(c));
            if !needs {
                return (false, None);
            }
        }

        let mut result = String::with_capacity(text.len());
        for c in text.chars() {
            if c > '\x7F' && char_is_decimal_digit(c) {
                if let Some(digit) = char_decimal_value(c) {
                    result.push(char::from(b'0' + digit));
                } else {
                    result.push(c);
                }
            } else {
                result.push(c);
            }
        }
        if result != text {
            token.term = Cow::Owned(result);
        }
        (false, None)
    }
}

/// Returns true if the character is a Unicode decimal digit.
#[inline]
fn char_is_decimal_digit(c: char) -> bool {
    // Unicode General_Category = Nd (Decimal Number)
    c.is_numeric() && char_decimal_value(c).is_some()
}

/// Returns the decimal digit value (0-9) for a Unicode decimal digit character,
/// or None if it's not a decimal digit.
#[inline]
fn char_decimal_value(c: char) -> Option<u8> {
    // Unicode decimal digits come in contiguous blocks of 10
    let cp = c as u32;
    // Check known decimal digit ranges
    let base = match cp {
        0x0030..=0x0039 => 0x0030, // ASCII
        0x0660..=0x0669 => 0x0660, // Arabic-Indic
        0x06F0..=0x06F9 => 0x06F0, // Extended Arabic-Indic
        0x07C0..=0x07C9 => 0x07C0, // NKo
        0x0966..=0x096F => 0x0966, // Devanagari
        0x09E6..=0x09EF => 0x09E6, // Bengali
        0x0A66..=0x0A6F => 0x0A66, // Gurmukhi
        0x0AE6..=0x0AEF => 0x0AE6, // Gujarati
        0x0B66..=0x0B6F => 0x0B66, // Oriya
        0x0BE6..=0x0BEF => 0x0BE6, // Tamil
        0x0C66..=0x0C6F => 0x0C66, // Telugu
        0x0CE6..=0x0CEF => 0x0CE6, // Kannada
        0x0D66..=0x0D6F => 0x0D66, // Malayalam
        0x0DE6..=0x0DEF => 0x0DE6, // Sinhala
        0x0E50..=0x0E59 => 0x0E50, // Thai
        0x0ED0..=0x0ED9 => 0x0ED0, // Lao
        0x0F20..=0x0F29 => 0x0F20, // Tibetan
        0x1040..=0x1049 => 0x1040, // Myanmar
        0x1090..=0x1099 => 0x1090, // Myanmar Shan
        0x17E0..=0x17E9 => 0x17E0, // Khmer
        0x1810..=0x1819 => 0x1810, // Mongolian
        0x1946..=0x194F => 0x1946, // Limbu
        0x19D0..=0x19D9 => 0x19D0, // New Tai Lue
        0x1A80..=0x1A89 => 0x1A80, // Tai Tham Hora
        0x1A90..=0x1A99 => 0x1A90, // Tai Tham Tham
        0x1B50..=0x1B59 => 0x1B50, // Balinese
        0x1BB0..=0x1BB9 => 0x1BB0, // Sundanese
        0x1C40..=0x1C49 => 0x1C40, // Lepcha
        0x1C50..=0x1C59 => 0x1C50, // Ol Chiki
        0xA620..=0xA629 => 0xA620, // Vai
        0xA8D0..=0xA8D9 => 0xA8D0, // Saurashtra
        0xA900..=0xA909 => 0xA900, // Kayah Li
        0xA9D0..=0xA9D9 => 0xA9D0, // Javanese
        0xA9F0..=0xA9F9 => 0xA9F0, // Myanmar Tai Laing
        0xAA50..=0xAA59 => 0xAA50, // Cham
        0xABF0..=0xABF9 => 0xABF0, // Meetei Mayek
        0xFF10..=0xFF19 => 0xFF10, // Fullwidth
        _ => return None,
    };
    Some((cp - base) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arabic_indic_digits() {
        let f = DecimalDigitTokenFilter::new();
        let mut token = Token::new("١٢٣٤", 0, 8, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "1234");
    }

    #[test]
    fn test_devanagari_digits() {
        let f = DecimalDigitTokenFilter::new();
        let mut token = Token::new("०१२३", 0, 12, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "0123");
    }

    #[test]
    fn test_fullwidth_digits() {
        let f = DecimalDigitTokenFilter::new();
        let mut token = Token::new("０１２３", 0, 12, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "0123");
    }

    #[test]
    fn test_ascii_digits_unchanged() {
        let f = DecimalDigitTokenFilter::new();
        let mut token = Token::new("1234", 0, 4, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "1234");
    }

    #[test]
    fn test_mixed_text() {
        let f = DecimalDigitTokenFilter::new();
        let mut token = Token::new("hello١٢world", 0, 15, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "hello12world");
    }
}
