//! Russian normalization token filter.
//!
//! Normalizes Russian text by replacing ё (yo) with е (ye), which is
//! critical for Russian search quality since users interchangeably use
//! both characters.

use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Normalizes Russian ё/Ё to е/Е.
///
/// In Russian, the letters ё and е are frequently used interchangeably in
/// written text. This filter ensures that both variants match during search
/// by normalizing ё→е and Ё→Е.
#[derive(Clone, Debug, Default)]
pub struct RussianYoNormalizationTokenFilter;

impl RussianYoNormalizationTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for RussianYoNormalizationTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        // Fast path: no ё/Ё present
        if !text.contains('ё') && !text.contains('Ё') {
            return (false, None);
        }
        let normalized: String = text
            .chars()
            .map(|c| match c {
                'ё' => 'е',
                'Ё' => 'Е',
                _ => c,
            })
            .collect();
        token.term = Cow::Owned(normalized);
        (false, None)
    }
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
    fn test_yo_normalization() {
        let filter = RussianYoNormalizationTokenFilter::new();

        let mut token = make_token("ёлка");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "елка");

        let mut token = make_token("всё");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "все");

        // No ё — should not allocate
        let mut token = make_token("привет");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "привет");
    }

    #[test]
    fn test_uppercase_yo() {
        let filter = RussianYoNormalizationTokenFilter::new();
        let mut token = make_token("Ёж");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "Еж");
    }
}
