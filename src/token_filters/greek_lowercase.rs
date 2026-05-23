use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Lowercases Greek characters and removes tonos (accents).
///
/// Equivalent to Lucene's `GreekLowerCaseFilter`.
///
/// Rules:
/// - Final sigma (ς) → sigma (σ)
/// - Alpha with tonos (Ά/ά) → alpha (α)
/// - Epsilon with tonos (Έ/έ) → epsilon (ε)
/// - Eta with tonos (Ή/ή) → eta (η)
/// - Iota variants (Ί/Ϊ/ί/ϊ/ΐ) → iota (ι)
/// - Upsilon variants (Ύ/Ϋ/ύ/ϋ/ΰ) → upsilon (υ)
/// - Omicron with tonos (Ό/ό) → omicron (ο)
/// - Omega with tonos (Ώ/ώ) → omega (ω)
/// - All other characters: standard lowercase
#[derive(Clone, Debug, Default)]
pub struct GreekLowercaseTokenFilter;

impl GreekLowercaseTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for GreekLowercaseTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.is_empty() {
            return (false, None);
        }

        let mut result = String::with_capacity(text.len());
        let mut changed = false;

        for c in text.chars() {
            let mapped = match c {
                // Final sigma → sigma
                '\u{03C2}' => '\u{03C3}',
                // Alpha with tonos
                '\u{0386}' | '\u{03AC}' => '\u{03B1}',
                // Epsilon with tonos
                '\u{0388}' | '\u{03AD}' => '\u{03B5}',
                // Eta with tonos
                '\u{0389}' | '\u{03AE}' => '\u{03B7}',
                // Iota variants (with tonos, with diaeresis, with both)
                '\u{038A}' | '\u{03AA}' | '\u{03AF}' | '\u{03CA}' | '\u{0390}' => '\u{03B9}',
                // Upsilon variants
                '\u{038E}' | '\u{03AB}' | '\u{03CD}' | '\u{03CB}' | '\u{03B0}' => '\u{03C5}',
                // Omicron with tonos
                '\u{038C}' | '\u{03CC}' => '\u{03BF}',
                // Omega with tonos
                '\u{038F}' | '\u{03CE}' => '\u{03C9}',
                _ => {
                    // Standard lowercase
                    let lower: char = c.to_lowercase().next().unwrap_or(c);
                    if lower != c {
                        changed = true;
                    }
                    result.push(lower);
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
    fn test_tonos_removal() {
        let f = GreekLowercaseTokenFilter::new();
        let mut token = Token::new("ΜΆΪΟΣ", 0, 10, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "μαιοσ");
    }

    #[test]
    fn test_final_sigma() {
        let f = GreekLowercaseTokenFilter::new();
        let mut token = Token::new("λόγος", 0, 10, 0);
        f.filter(&mut token);
        // ό→ο, ς→σ
        assert_eq!(token.term, "λογοσ");
    }

    #[test]
    fn test_already_lowercase() {
        let f = GreekLowercaseTokenFilter::new();
        let mut token = Token::new("αβγ", 0, 6, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "αβγ");
    }
}
