use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Truncates each token to a maximum number of characters.
#[derive(Clone, Debug)]
pub struct TruncateTokenFilter {
    pub length: usize,
}

impl TruncateTokenFilter {
    pub fn new(length: usize) -> Self {
        Self { length }
    }
}

impl Default for TruncateTokenFilter {
    fn default() -> Self {
        Self { length: 10 }
    }
}

impl TokenFilter for TruncateTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        if token.term.chars().count() > self.length {
            let truncated: alloc::string::String = token.term.chars().take(self.length).collect();
            token.term = Cow::Owned(truncated);
        }
        (false, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        let f = TruncateTokenFilter::new(3);
        let mut token = Token::new("hello", 0, 5, 0);
        f.filter(&mut token);
        assert_eq!(token.term.as_ref(), "hel");
    }

    #[test]
    fn test_truncate_short() {
        let f = TruncateTokenFilter::new(10);
        let mut token = Token::new("hi", 0, 2, 0);
        f.filter(&mut token);
        assert_eq!(token.term.as_ref(), "hi");
    }
}
