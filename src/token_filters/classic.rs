use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Removes possessive endings (`'s`, `'S`) and dots from acronyms.
///
/// Designed to work with `ClassicTokenizer` output:
/// - Tokens of type "apostrophe" ending in `'s` have the suffix removed
/// - Tokens of type "acronym" have dots removed (e.g., `U.S.A.` → `USA`)
///
/// In this generic implementation (without token type metadata), we apply both
/// rules unconditionally based on token content.
#[derive(Clone, Debug, Default)]
pub struct ClassicTokenFilter;

impl ClassicTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for ClassicTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();

        // Rule 1: Remove possessive endings ('s, 'S)
        for suffix in ["'s", "'S", "\u{2019}s", "\u{2019}S"] {
            if text.ends_with(suffix) {
                let new_text = &text[..text.len() - suffix.len()];
                if new_text.is_empty() {
                    return (true, None);
                }
                token.term = Cow::Owned(String::from(new_text));
                return (false, None);
            }
        }

        // Rule 2: Remove dots from acronyms (text with multiple dots)
        // Only apply if the text looks like an acronym (has dots between letters)
        if text.contains('.') && is_likely_acronym(text) {
            let cleaned: String = text.chars().filter(|&c| c != '.').collect();
            if cleaned.is_empty() {
                return (true, None);
            }
            token.term = Cow::Owned(cleaned);
            return (false, None);
        }

        (false, None)
    }
}

/// Heuristic: a token is likely an acronym if it contains dots and most
/// characters are uppercase letters or dots.
fn is_likely_acronym(s: &str) -> bool {
    if s.len() < 3 {
        return false;
    }
    let total = s.chars().count();
    let alpha_or_dot = s.chars().filter(|c| c.is_alphabetic() || *c == '.').count();
    // Most chars should be letters or dots
    alpha_or_dot == total && s.chars().filter(|c| *c == '.').count() >= 1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_token(term: &str) -> Token<'_> {
        Token {
            term: Cow::Borrowed(term),
            start_offset: 0,
            end_offset: term.len() as u32,
            position: 0,
        }
    }

    #[test]
    fn test_possessive_removal() {
        let filter = ClassicTokenFilter::new();
        let mut token = make_token("cat's");
        let (remove, _) = filter.filter(&mut token);
        assert!(!remove);
        assert_eq!(token.term.as_ref(), "cat");
    }

    #[test]
    fn test_acronym_dots() {
        let filter = ClassicTokenFilter::new();
        let mut token = make_token("U.S.A.");
        let (remove, _) = filter.filter(&mut token);
        assert!(!remove);
        assert_eq!(token.term.as_ref(), "USA");
    }

    #[test]
    fn test_no_change() {
        let filter = ClassicTokenFilter::new();
        let mut token = make_token("hello");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "hello");
    }

    #[test]
    fn test_not_acronym() {
        let filter = ClassicTokenFilter::new();
        let mut token = make_token("3.14");
        filter.filter(&mut token);
        // Has non-alpha chars, so not treated as acronym
        assert_eq!(token.term.as_ref(), "3.14");
    }
}
