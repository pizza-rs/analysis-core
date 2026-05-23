use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Normalizes Scandinavian characters by transforming interchangeable
/// Scandinavian letters to a canonical form (åÅæÆøØ).
///
/// Equivalent to Lucene's `ScandinavianNormalizationFilter`.
///
/// Rules:
/// - ä → æ, Ä → Æ
/// - ö → ø, Ö → Ø
/// - aa/aA → å, AA/Aa → Å
/// - ao/aO → å, Ao/AO → Å
/// - ae/aE → æ, Ae/AE → Æ
/// - oe/oE/oo/oO → ø, Oe/OE/Oo/OO → Ø
#[derive(Clone, Debug, Default)]
pub struct ScandinavianNormalizationTokenFilter;

impl ScandinavianNormalizationTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

const AA: char = '\u{00C5}'; // Å
const AA_LOWER: char = '\u{00E5}'; // å
const AE: char = '\u{00C6}'; // Æ
const AE_LOWER: char = '\u{00E6}'; // æ
const AE_SE: char = '\u{00C4}'; // Ä
const AE_SE_LOWER: char = '\u{00E4}'; // ä
const OE: char = '\u{00D8}'; // Ø
const OE_LOWER: char = '\u{00F8}'; // ø
const OE_SE: char = '\u{00D6}'; // Ö
const OE_SE_LOWER: char = '\u{00F6}'; // ö

impl TokenFilter for ScandinavianNormalizationTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.is_empty() {
            return (false, None);
        }

        let chars: Vec<char> = text.chars().collect();
        let mut result = Vec::with_capacity(chars.len());
        let mut changed = false;
        let len = chars.len();
        let mut i = 0;

        while i < len {
            let c = chars[i];
            match c {
                c if c == AE_SE_LOWER => {
                    result.push(AE_LOWER);
                    changed = true;
                }
                c if c == AE_SE => {
                    result.push(AE);
                    changed = true;
                }
                c if c == OE_SE_LOWER => {
                    result.push(OE_LOWER);
                    changed = true;
                }
                c if c == OE_SE => {
                    result.push(OE);
                    changed = true;
                }
                'a' if i + 1 < len => {
                    let next = chars[i + 1];
                    match next {
                        'a' | 'A' | 'o' | 'O' => {
                            result.push(AA_LOWER);
                            i += 1; // skip next
                            changed = true;
                        }
                        'e' | 'E' => {
                            result.push(AE_LOWER);
                            i += 1;
                            changed = true;
                        }
                        _ => result.push(c),
                    }
                }
                'A' if i + 1 < len => {
                    let next = chars[i + 1];
                    match next {
                        'a' | 'A' | 'o' | 'O' => {
                            result.push(AA);
                            i += 1;
                            changed = true;
                        }
                        'e' | 'E' => {
                            result.push(AE);
                            i += 1;
                            changed = true;
                        }
                        _ => result.push(c),
                    }
                }
                'o' if i + 1 < len => {
                    let next = chars[i + 1];
                    match next {
                        'e' | 'E' | 'o' | 'O' => {
                            result.push(OE_LOWER);
                            i += 1;
                            changed = true;
                        }
                        _ => result.push(c),
                    }
                }
                'O' if i + 1 < len => {
                    let next = chars[i + 1];
                    match next {
                        'e' | 'E' | 'o' | 'O' => {
                            result.push(OE);
                            i += 1;
                            changed = true;
                        }
                        _ => result.push(c),
                    }
                }
                _ => result.push(c),
            }
            i += 1;
        }

        if changed {
            let s: String = result.into_iter().collect();
            token.term = Cow::Owned(s);
        }
        (false, None)
    }
}

/// Folds Scandinavian characters åÅäæÄÆ→a/A and öÖøØ→o/O, and discriminates
/// against double vowels (aa, ae, ao, oe, oo) by keeping just the first one.
///
/// Equivalent to Lucene's `ScandinavianFoldingFilter`.
///
/// This is more aggressive than `ScandinavianNormalizationTokenFilter`.
#[derive(Clone, Debug, Default)]
pub struct ScandinavianFoldingTokenFilter;

impl ScandinavianFoldingTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for ScandinavianFoldingTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.is_empty() {
            return (false, None);
        }

        let chars: Vec<char> = text.chars().collect();
        let mut result = Vec::with_capacity(chars.len());
        let mut changed = false;
        let len = chars.len();
        let mut i = 0;

        while i < len {
            let c = chars[i];
            match c {
                // Fold single characters
                c if c == AA_LOWER || c == AE_SE_LOWER || c == AE_LOWER => {
                    result.push('a');
                    changed = true;
                }
                c if c == AA || c == AE_SE || c == AE => {
                    result.push('A');
                    changed = true;
                }
                c if c == OE_LOWER || c == OE_SE_LOWER => {
                    result.push('o');
                    changed = true;
                }
                c if c == OE || c == OE_SE => {
                    result.push('O');
                    changed = true;
                }
                // Fold digraphs starting with a/A
                'a' if i + 1 < len => {
                    let next = chars[i + 1];
                    match next {
                        'a' | 'A' | 'e' | 'E' | 'o' | 'O' => {
                            result.push('a');
                            i += 1; // skip second char
                            changed = true;
                        }
                        _ => result.push(c),
                    }
                }
                'A' if i + 1 < len => {
                    let next = chars[i + 1];
                    match next {
                        'a' | 'A' | 'e' | 'E' | 'o' | 'O' => {
                            result.push('A');
                            i += 1;
                            changed = true;
                        }
                        _ => result.push(c),
                    }
                }
                // Fold digraphs starting with o/O
                'o' if i + 1 < len => {
                    let next = chars[i + 1];
                    match next {
                        'e' | 'E' | 'o' | 'O' => {
                            result.push('o');
                            i += 1;
                            changed = true;
                        }
                        _ => result.push(c),
                    }
                }
                'O' if i + 1 < len => {
                    let next = chars[i + 1];
                    match next {
                        'e' | 'E' | 'o' | 'O' => {
                            result.push('O');
                            i += 1;
                            changed = true;
                        }
                        _ => result.push(c),
                    }
                }
                _ => result.push(c),
            }
            i += 1;
        }

        if changed {
            let s: String = result.into_iter().collect();
            token.term = Cow::Owned(s);
        }
        (false, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ScandinavianNormalizationFilter tests
    #[test]
    fn test_norm_ae_to_ae() {
        let f = ScandinavianNormalizationTokenFilter::new();
        let mut token = Token::new("ae", 0, 2, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "æ");
    }

    #[test]
    fn test_norm_oe_to_oe() {
        let f = ScandinavianNormalizationTokenFilter::new();
        let mut token = Token::new("oe", 0, 2, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "ø");
    }

    #[test]
    fn test_norm_aa_to_aa() {
        let f = ScandinavianNormalizationTokenFilter::new();
        let mut token = Token::new("aa", 0, 2, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "å");
    }

    #[test]
    fn test_norm_umlaut_a() {
        let f = ScandinavianNormalizationTokenFilter::new();
        let mut token = Token::new("ä", 0, 2, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "æ");
    }

    #[test]
    fn test_norm_umlaut_o() {
        let f = ScandinavianNormalizationTokenFilter::new();
        let mut token = Token::new("ö", 0, 2, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "ø");
    }

    #[test]
    fn test_norm_full_word() {
        let f = ScandinavianNormalizationTokenFilter::new();
        let mut token = Token::new("räksmörgås", 0, 13, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "ræksmørgås");
    }

    // ScandinavianFoldingFilter tests
    #[test]
    fn test_fold_aa_to_a() {
        let f = ScandinavianFoldingTokenFilter::new();
        let mut token = Token::new("å", 0, 2, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "a");
    }

    #[test]
    fn test_fold_ae_digraph() {
        let f = ScandinavianFoldingTokenFilter::new();
        let mut token = Token::new("ae", 0, 2, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "a");
    }

    #[test]
    fn test_fold_oe_char() {
        let f = ScandinavianFoldingTokenFilter::new();
        let mut token = Token::new("ø", 0, 2, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "o");
    }

    #[test]
    fn test_fold_full_word() {
        let f = ScandinavianFoldingTokenFilter::new();
        let mut token = Token::new("räksmörgås", 0, 13, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "raksmorgas");
    }
}
