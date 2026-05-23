use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Normalizes Sorani Kurdish text. Equivalent to Lucene's `SoraniNormalizationFilter`.
///
/// Rules:
/// - YEH (\u064A), DOTLESS_YEH (\u0649) → FARSI_YEH (\u06CC)
/// - KAF (\u0643) → KEHEH (\u06A9)
/// - HEH + ZWNJ / word-final HEH / TEH_MARBUTA → AE (\u06D5)
/// - HEH_DOACHASHMEE (\u06BE) → HEH (\u0647)
/// - Removes harakat (diacritics) and TATWEEL
#[derive(Clone, Debug, Default)]
pub struct SoraniNormalizationTokenFilter;

impl SoraniNormalizationTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for SoraniNormalizationTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.is_empty() {
            return (false, None);
        }

        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();
        let mut result = String::with_capacity(text.len());
        let mut changed = false;
        let mut i = 0;

        while i < len {
            let c = chars[i];
            match c {
                // YEH / DOTLESS_YEH → FARSI_YEH
                '\u{064A}' | '\u{0649}' => {
                    result.push('\u{06CC}');
                    changed = true;
                }
                // KAF → KEHEH
                '\u{0643}' => {
                    result.push('\u{06A9}');
                    changed = true;
                }
                // HEH_DOACHASHMEE → HEH
                '\u{06BE}' => {
                    result.push('\u{0647}');
                    changed = true;
                }
                // TEH_MARBUTA → AE
                '\u{0629}' => {
                    result.push('\u{06D5}');
                    changed = true;
                }
                // HEH with ZWNJ following → AE
                '\u{0647}' => {
                    if i + 1 < len && chars[i + 1] == '\u{200C}' {
                        // HEH + ZWNJ → AE
                        result.push('\u{06D5}');
                        i += 1; // skip ZWNJ
                        changed = true;
                    } else if i == len - 1 {
                        // Word-final HEH → AE
                        result.push('\u{06D5}');
                        changed = true;
                    } else {
                        result.push(c);
                    }
                }
                // Remove harakat and TATWEEL
                '\u{0640}' | '\u{064B}' | '\u{064C}' | '\u{064D}' | '\u{064E}' | '\u{064F}'
                | '\u{0650}' | '\u{0651}' | '\u{0652}' => {
                    changed = true;
                }
                // ZWNJ
                '\u{200C}' => {
                    changed = true;
                }
                _ => {
                    result.push(c);
                }
            }
            i += 1;
        }

        if changed {
            token.term = Cow::Owned(result);
        }
        (false, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yeh_normalization() {
        let f = SoraniNormalizationTokenFilter::new();
        let mut token = Token::new("\u{064A}", 0, 2, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "\u{06CC}");
    }

    #[test]
    fn test_kaf_to_keheh() {
        let f = SoraniNormalizationTokenFilter::new();
        let mut token = Token::new("\u{0643}", 0, 2, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "\u{06A9}");
    }

    #[test]
    fn test_heh_final_to_ae() {
        let f = SoraniNormalizationTokenFilter::new();
        // Word-final HEH → AE
        let mut token = Token::new("\u{0647}", 0, 2, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "\u{06D5}");
    }

    #[test]
    fn test_heh_doachashmee() {
        let f = SoraniNormalizationTokenFilter::new();
        let mut token = Token::new("\u{06BE}", 0, 2, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "\u{0647}");
    }
}
