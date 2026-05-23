use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Trims leading and trailing whitespace from each token.
#[derive(Clone, Debug)]
pub struct TrimTokenFilter;

impl TrimTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TrimTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for TrimTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let trimmed = token.term.trim();
        if trimmed.len() != token.term.len() {
            if trimmed.is_empty() {
                return (true, None);
            }
            token.term = Cow::Owned(trimmed.to_string());
        }
        (false, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim() {
        let f = TrimTokenFilter::new();
        let mut token = Token::new("  hello  ", 0, 9, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
        assert_eq!(token.term.as_ref(), "hello");
    }
}
