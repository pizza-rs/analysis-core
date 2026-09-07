use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Normalizes Persian text — Lucene's `PersianNormalizationFilter` as the
/// compatibility baseline, extended by pizza.
///
/// Baseline (matching `PersianNormalizer` exactly):
/// - FARSI_YEH (U+06CC), YEH_BARREE (U+06D2) → YEH (U+064A)
/// - HEH_YEH (U+06C0), HEH_GOAL (U+06C1) → HEH (U+0647)
/// - KEHEH (U+06A9) → KAF (U+0643)
/// - HAMZA_ABOVE (U+0654) is deleted (necessary for HEH + HAMZA)
///
/// Pizza extension (default): TATWEEL (U+0640) and the optional diacritics
/// (U+064B..U+0652) are also stripped. Queries are almost never typed with
/// these marks while documents often carry them, so folding them
/// consistently on the index and query sides improves recall at no
/// meaningful precision cost — the same convention ES applies to the Arabic
/// analyzer. Use [`PersianNormalizationTokenFilter::lucene_strict`] for
/// exact Lucene behavior.
#[derive(Clone, Debug, Default)]
pub struct PersianNormalizationTokenFilter {
    lucene_strict: bool,
}

impl PersianNormalizationTokenFilter {
    /// Extended normalizer (Lucene baseline + diacritics/tatweel folding).
    pub fn new() -> Self {
        Self {
            lucene_strict: false,
        }
    }

    /// Exact Lucene `PersianNormalizer` behavior: diacritics and tatweel
    /// are kept untouched.
    pub fn lucene_strict() -> Self {
        Self {
            lucene_strict: true,
        }
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
                // Pizza extension: strip tatweel and optional diacritics.
                '\u{0640}' | '\u{064B}'..='\u{0652}' if !self.lucene_strict => {
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

    // Vectors from Lucene's TestPersianNormalizationFilter (strict mode).
    #[test]
    fn test_lucene_vectors() {
        let f = PersianNormalizationTokenFilter::lucene_strict();
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
        // strict mode keeps diacritics and tatweel
        let mut token = Token::new("بَ", 0, 4, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "بَ");
    }

    // The pizza extension folds diacritics and tatweel for recall.
    #[test]
    fn test_extended_diacritics_folding() {
        let f = PersianNormalizationTokenFilter::new();
        // fatha on beh is stripped
        let mut token = Token::new("بَ", 0, 4, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "ب");
        // tatweel (kashida) is stripped
        let mut token = Token::new("كت\u{0640}اب", 0, 8, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "كتاب");
        // sukun/shadda likewise
        let mut token = Token::new("\u{0651}\u{0652}", 0, 4, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "");
        // baseline mappings still apply alongside the extension
        let mut token = Token::new("های", 0, 6, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "هاي");
    }
}
