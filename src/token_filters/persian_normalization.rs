use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Normalizes Persian text. Equivalent to Lucene's `PersianNormalizationFilter`.
///
/// Rules:
/// - YEH_BARREE (\u06D2), FARSI_YEH (\u06CC) → YEH (\u064A)
/// - HEH_YEH (\u06C0), HEH_GOAL (\u06C1) → HEH (\u0647)
/// - KEHEH (\u06A9) → KAF (\u0643)
/// - Removes: TATWEEL (\u0640), FATHATAN (\u064B), DAMMATAN (\u064C),
///   KASRATAN (\u064D), FATHA (\u064E), DAMMA (\u064F), KASRA (\u0650),
///   SHADDA (\u0651), SUKUN (\u0652)
#[derive(Clone, Debug, Default)]
pub struct PersianNormalizationTokenFilter;

impl PersianNormalizationTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for PersianNormalizationTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.is_empty() {
            return (false, None);
        }

        let mut result = String::with_capacity(text.len());
        let mut changed = false;

        for c in text.chars() {
            match c {
                // YEH variants → YEH
                '\u{06D2}' | '\u{06CC}' => {
                    result.push('\u{064A}');
                    changed = true;
                }
                // HEH variants → HEH
                '\u{06C0}' | '\u{06C1}' => {
                    result.push('\u{0647}');
                    changed = true;
                }
                // KEHEH → KAF
                '\u{06A9}' => {
                    result.push('\u{0643}');
                    changed = true;
                }
                // Remove diacritics and TATWEEL
                '\u{0640}' | '\u{064B}' | '\u{064C}' | '\u{064D}' | '\u{064E}' | '\u{064F}'
                | '\u{0650}' | '\u{0651}' | '\u{0652}' => {
                    changed = true;
                }
                _ => {
                    result.push(c);
                }
            }
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
        let f = PersianNormalizationTokenFilter::new();
        let mut token = Token::new("\u{06CC}", 0, 2, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "\u{064A}");
    }

    #[test]
    fn test_heh_goal() {
        let f = PersianNormalizationTokenFilter::new();
        let mut token = Token::new("\u{06C1}", 0, 2, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "\u{0647}");
    }

    #[test]
    fn test_keheh() {
        let f = PersianNormalizationTokenFilter::new();
        let mut token = Token::new("\u{06A9}", 0, 2, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "\u{0643}");
    }
}
