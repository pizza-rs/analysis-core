use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Normalizes Arabic text by removing diacritics and normalizing letter forms.
///
/// Equivalent to Lucene's `ArabicNormalizationFilter`.
///
/// Rules:
/// - ALEF_MADDA (\u0622), ALEF_HAMZA_ABOVE (\u0623), ALEF_HAMZA_BELOW (\u0625) → ALEF (\u0627)
/// - DOTLESS_YEH (\u0649) → YEH (\u064A)
/// - TEH_MARBUTA (\u0629) → HEH (\u0647)
/// - Removes: TATWEEL (\u0640), FATHATAN (\u064B), DAMMATAN (\u064C), KASRATAN (\u064D),
///   FATHA (\u064E), DAMMA (\u064F), KASRA (\u0650), SHADDA (\u0651), SUKUN (\u0652)
#[derive(Clone, Debug, Default)]
pub struct ArabicNormalizationTokenFilter;

impl ArabicNormalizationTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for ArabicNormalizationTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.is_empty() {
            return (false, None);
        }

        let mut result = String::with_capacity(text.len());
        let mut changed = false;

        for c in text.chars() {
            match c {
                // Normalize ALEF variants
                '\u{0622}' | '\u{0623}' | '\u{0625}' => {
                    result.push('\u{0627}'); // ALEF
                    changed = true;
                }
                // Normalize DOTLESS_YEH
                '\u{0649}' => {
                    result.push('\u{064A}'); // YEH
                    changed = true;
                }
                // Normalize TEH_MARBUTA
                '\u{0629}' => {
                    result.push('\u{0647}'); // HEH
                    changed = true;
                }
                // Remove diacritics and TATWEEL
                '\u{0640}' | // TATWEEL
                '\u{064B}' | // FATHATAN
                '\u{064C}' | // DAMMATAN
                '\u{064D}' | // KASRATAN
                '\u{064E}' | // FATHA
                '\u{064F}' | // DAMMA
                '\u{0650}' | // KASRA
                '\u{0651}' | // SHADDA
                '\u{0652}' => { // SUKUN
                    changed = true;
                    // character deleted
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
    fn test_alef_normalization() {
        let f = ArabicNormalizationTokenFilter::new();
        let mut token = Token::new("\u{0622}\u{0623}\u{0625}", 0, 6, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "\u{0627}\u{0627}\u{0627}");
    }

    #[test]
    fn test_yeh_normalization() {
        let f = ArabicNormalizationTokenFilter::new();
        let mut token = Token::new("\u{0649}", 0, 2, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "\u{064A}");
    }

    #[test]
    fn test_teh_marbuta() {
        let f = ArabicNormalizationTokenFilter::new();
        let mut token = Token::new("\u{0629}", 0, 2, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "\u{0647}");
    }

    #[test]
    fn test_remove_diacritics() {
        let f = ArabicNormalizationTokenFilter::new();
        let mut token = Token::new("\u{0627}\u{064E}\u{0644}\u{0650}", 0, 8, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "\u{0627}\u{0644}");
    }
}
