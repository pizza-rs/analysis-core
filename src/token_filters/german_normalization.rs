use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Normalizes German characters according to the heuristics of the German
/// snowball algorithm.
///
/// Equivalent to Lucene's `GermanNormalizationFilter`.
///
/// Rules:
/// - `ß` → `ss`
/// - `ä`, `ö`, `ü` → `a`, `o`, `u`
/// - `ae` → `a` (delete the `e`)
/// - `oe` → `o` (delete the `e`)
/// - `ue` → `u` (delete the `e`, only when not preceded by a vowel or `q`)
#[derive(Clone, Debug, Default)]
pub struct GermanNormalizationTokenFilter;

impl GermanNormalizationTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

// FSM states
const N: u8 = 0; // ordinary state
const V: u8 = 1; // stops 'u' from entering umlaut state
const U: u8 = 2; // umlaut state, allows e-deletion

impl TokenFilter for GermanNormalizationTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.is_empty() {
            return (false, None);
        }

        let mut result = String::with_capacity(text.len());
        let mut state = N;

        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            match c {
                'a' | 'o' => {
                    state = U;
                    result.push(c);
                }
                'u' => {
                    state = if state == N { U } else { V };
                    result.push(c);
                }
                'e' => {
                    if state == U {
                        // Delete 'e' (don't push it) - this implements ae->a, oe->o, ue->u
                    } else {
                        result.push(c);
                    }
                    state = V;
                }
                'i' | 'q' | 'y' => {
                    state = V;
                    result.push(c);
                }
                'ä' => {
                    result.push('a');
                    state = V;
                }
                'ö' => {
                    result.push('o');
                    state = V;
                }
                'ü' => {
                    result.push('u');
                    state = V;
                }
                'ß' => {
                    result.push('s');
                    result.push('s');
                    state = N;
                }
                _ => {
                    state = N;
                    result.push(c);
                }
            }
            i += 1;
        }

        if result != text {
            token.term = Cow::Owned(result);
        }
        (false, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_umlaut_folding() {
        let f = GermanNormalizationTokenFilter::new();
        let mut token = Token::new("Schaltflächen", 0, 16, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "Schaltflachen");
    }

    #[test]
    fn test_ae_folding() {
        let f = GermanNormalizationTokenFilter::new();
        let mut token = Token::new("Schaltflaechen", 0, 14, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "Schaltflachen");
    }

    #[test]
    fn test_sharp_s() {
        let f = GermanNormalizationTokenFilter::new();
        let mut token = Token::new("weißbier", 0, 10, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "weissbier");
    }

    #[test]
    fn test_ue_after_vowel() {
        let f = GermanNormalizationTokenFilter::new();
        // ue after a vowel should NOT be folded
        let mut token = Token::new("dauer", 0, 5, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "dauer");
    }

    #[test]
    fn test_ue_after_consonant() {
        let f = GermanNormalizationTokenFilter::new();
        // ue after a consonant SHOULD be folded
        let mut token = Token::new("ueber", 0, 5, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "uber");
    }
}
