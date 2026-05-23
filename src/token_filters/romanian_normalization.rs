use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Normalizes Romanian characters by converting cedilla forms to comma-below forms.
///
/// Equivalent to Lucene's `RomanianNormalizationFilter` (proposed).
///
/// Rules:
/// - S_WITH_CEDILLA (Ş/ş \u015E/\u015F) → S_WITH_COMMA_BELOW (Ș/ș \u0218/\u0219)
/// - T_WITH_CEDILLA (Ţ/ţ \u0162/\u0163) → T_WITH_COMMA_BELOW (Ț/ț \u021A/\u021B)
#[derive(Clone, Debug, Default)]
pub struct RomanianNormalizationTokenFilter;

impl RomanianNormalizationTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for RomanianNormalizationTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.is_empty() {
            return (false, None);
        }

        let mut result = String::with_capacity(text.len());
        let mut changed = false;

        for c in text.chars() {
            let mapped = match c {
                '\u{015E}' => '\u{0218}', // Ş → Ș
                '\u{015F}' => '\u{0219}', // ş → ș
                '\u{0162}' => '\u{021A}', // Ţ → Ț
                '\u{0163}' => '\u{021B}', // ţ → ț
                _ => {
                    result.push(c);
                    continue;
                }
            };
            result.push(mapped);
            changed = true;
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
    fn test_s_cedilla_to_comma() {
        let f = RomanianNormalizationTokenFilter::new();
        let mut token = Token::new("Şcoală", 0, 8, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "Școală");
    }

    #[test]
    fn test_t_cedilla_to_comma() {
        let f = RomanianNormalizationTokenFilter::new();
        let mut token = Token::new("ţară", 0, 6, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "țară");
    }

    #[test]
    fn test_no_change() {
        let f = RomanianNormalizationTokenFilter::new();
        let mut token = Token::new("hello", 0, 5, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "hello");
    }
}
