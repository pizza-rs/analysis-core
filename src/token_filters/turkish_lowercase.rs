use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Lowercases Turkish characters with proper handling of dotted/dotless `I`.
///
/// Equivalent to Lucene's `TurkishLowerCaseFilter`.
///
/// Rules:
/// - Capital I (\u0049) + COMBINING DOT ABOVE (\u0307) → small i (\u0069), delete the dot
/// - Capital I (\u0049) without following dot → dotless ı (\u0131)
/// - All other characters: standard lowercase
#[derive(Clone, Debug, Default)]
pub struct TurkishLowercaseTokenFilter;

impl TurkishLowercaseTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

const LATIN_CAPITAL_I: char = '\u{0049}';
const LATIN_SMALL_I: char = '\u{0069}';
const LATIN_SMALL_DOTLESS_I: char = '\u{0131}';
const COMBINING_DOT_ABOVE: char = '\u{0307}';

impl TokenFilter for TurkishLowercaseTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.is_empty() {
            return (false, None);
        }

        let mut result = String::with_capacity(text.len());
        let mut changed = false;

        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];
            if c == LATIN_CAPITAL_I {
                // Check if next char is COMBINING_DOT_ABOVE
                if i + 1 < chars.len() && chars[i + 1] == COMBINING_DOT_ABOVE {
                    // I + dot above → i (and skip the dot)
                    result.push(LATIN_SMALL_I);
                    i += 2; // skip the combining dot
                } else {
                    // I without dot → dotless ı
                    result.push(LATIN_SMALL_DOTLESS_I);
                    i += 1;
                }
                changed = true;
            } else if c == COMBINING_DOT_ABOVE {
                // Remove stray combining dot above after I handling
                changed = true;
                i += 1;
            } else {
                // Standard Turkish lowercase
                let lower: char = c.to_lowercase().next().unwrap_or(c);
                if lower != c {
                    changed = true;
                }
                result.push(lower);
                i += 1;
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
    fn test_capital_i_to_dotless() {
        let f = TurkishLowercaseTokenFilter::new();
        let mut token = Token::new("ISTANBUL", 0, 8, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "\u{0131}stanbul");
    }

    #[test]
    fn test_capital_i_with_dot() {
        let f = TurkishLowercaseTokenFilter::new();
        // I followed by COMBINING DOT ABOVE → i
        let input = format!("{}{}stanbul", LATIN_CAPITAL_I, COMBINING_DOT_ABOVE);
        let mut token = Token::new(&input, 0, input.len() as u32, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "istanbul");
    }

    #[test]
    fn test_other_chars() {
        let f = TurkishLowercaseTokenFilter::new();
        let mut token = Token::new("HELLO", 0, 5, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "hello");
    }
}
