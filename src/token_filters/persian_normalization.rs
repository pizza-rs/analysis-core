use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Normalizes Persian text. Equivalent to Lucene's `PersianNormalizationFilter`.
///
/// Rules (matching `PersianNormalizer` exactly):
/// - FARSI_YEH (U+06CC), YEH_BARREE (U+06D2) → YEH (U+064A)
/// - HEH_YEH (U+06C0), HEH_GOAL (U+06C1) → HEH (U+0647)
/// - KEHEH (U+06A9) → KAF (U+0643)
/// - HAMZA_ABOVE (U+0654) is deleted (necessary for HEH + HAMZA)
///
/// Diacritics (fatha, damma, …) are deliberately NOT removed: Lucene's
/// normalizer leaves them untouched.
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
                // Farsi yeh / yeh barree → Arabic yeh
                '\u{06CC}' | '\u{06D2}' => {
                    result.push('\u{064A}');
                    changed = true;
                }
                // Keheh → Arabic kaf
                '\u{06A9}' => {
                    result.push('\u{0643}');
                    changed = true;
                }
                // Heh+yeh / heh goal → heh
                '\u{06C0}' | '\u{06C1}' => {
                    result.push('\u{0647}');
                    changed = true;
                }
                // Hamza above is removed (heh + hamza → heh)
                '\u{0654}' => {
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

    // Vectors from Lucene's TestPersianNormalizationFilter.
    #[test]
    fn test_lucene_vectors() {
        let f = PersianNormalizationTokenFilter::new();
        // farsi yeh → arabic yeh
        let mut token = Token::new("های", 0, 6, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "هاي");
        // yeh barree → arabic yeh
        let mut token = Token::new("هاے", 0, 6, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "هاي");
        // keheh → kaf
        let mut token = Token::new("کشاندن", 0, 10, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "كشاندن");
        // heh+yeh → heh
        let mut token = Token::new("كتابۀ", 0, 8, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "كتابه");
        // heh + hamza above → heh (hamza deleted)
        let mut token = Token::new("كتابهٔ", 0, 8, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "كتابه");
        // heh goal → heh
        let mut token = Token::new("زادہ", 0, 6, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "زاده");
    }

    #[test]
    fn test_diacritics_kept() {
        let f = PersianNormalizationTokenFilter::new();
        // Lucene leaves diacritics (fatha here) untouched.
        let mut token = Token::new("بَ", 0, 4, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "بَ");
    }
}
