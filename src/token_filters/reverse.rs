use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Reverses each token string.
#[derive(Clone, Debug)]
pub struct ReverseTokenFilter;

impl ReverseTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ReverseTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for ReverseTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let reversed: alloc::string::String = token.term.chars().rev().collect();
        token.term = Cow::Owned(reversed);
        (false, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reverse() {
        let f = ReverseTokenFilter::new();
        let mut token = Token::new("hello", 0, 5, 0);
        f.filter(&mut token);
        assert_eq!(token.term.as_ref(), "olleh");
    }
}
