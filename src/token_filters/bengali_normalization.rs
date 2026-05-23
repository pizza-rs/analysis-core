use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Normalizes Bengali/Bangla text. Equivalent to Lucene's `BengaliNormalizationFilter`.
///
/// Rules:
/// - Chandrabindu (\u0981) → Anusvara (\u0982)
/// - Nukta (\u09BC) → deleted
/// - Long vowels → short equivalents:
///   - \u09C7 → \u09C7 (no change, E vowel sign)
///   - \u09C8 → \u09C7 (AI → E)
///   - \u09CB → \u09CB (no change, O vowel sign)
///   - \u09CC → \u09CB (AU → O)
///   - \u09E1 → \u09E0 (Vocalic RR → Vocalic R)
///   - \u09E3 → \u09E2 (Vocalic LL → Vocalic L)
/// - \u09DC → \u09A1 (RRA → DDA)
/// - \u09DD → \u09A2 (RHA → DDHA)
/// - \u09DF → \u09AF (YYA → YA)
#[derive(Clone, Debug, Default)]
pub struct BengaliNormalizationTokenFilter;

impl BengaliNormalizationTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for BengaliNormalizationTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.is_empty() {
            return (false, None);
        }

        let mut result = String::with_capacity(text.len());
        let mut changed = false;

        for c in text.chars() {
            match c {
                // Chandrabindu → Anusvara
                '\u{0981}' => {
                    result.push('\u{0982}');
                    changed = true;
                }
                // Nukta → delete
                '\u{09BC}' => {
                    changed = true;
                }
                // Nukta-composed consonants → base
                '\u{09DC}' => {
                    result.push('\u{09A1}');
                    changed = true;
                } // RRA → DDA
                '\u{09DD}' => {
                    result.push('\u{09A2}');
                    changed = true;
                } // RHA → DDHA
                '\u{09DF}' => {
                    result.push('\u{09AF}');
                    changed = true;
                } // YYA → YA
                // Vowel sign normalization (long → short)
                '\u{09C8}' => {
                    result.push('\u{09C7}');
                    changed = true;
                } // AI → E
                '\u{09CC}' => {
                    result.push('\u{09CB}');
                    changed = true;
                } // AU → O
                '\u{09E1}' => {
                    result.push('\u{09E0}');
                    changed = true;
                } // Vocalic RR → R
                '\u{09E3}' => {
                    result.push('\u{09E2}');
                    changed = true;
                } // Vocalic LL → L
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
    fn test_chandrabindu_to_anusvara() {
        let f = BengaliNormalizationTokenFilter::new();
        let mut token = Token::new("\u{0981}", 0, 3, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "\u{0982}");
    }

    #[test]
    fn test_nukta_deletion() {
        let f = BengaliNormalizationTokenFilter::new();
        let mut token = Token::new("\u{0995}\u{09BC}", 0, 6, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "\u{0995}");
    }

    #[test]
    fn test_rra_to_dda() {
        let f = BengaliNormalizationTokenFilter::new();
        let mut token = Token::new("\u{09DC}", 0, 3, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "\u{09A1}");
    }
}
