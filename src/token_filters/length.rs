use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Removes tokens that fall outside a min/max length range.
#[derive(Clone, Debug)]
pub struct LengthTokenFilter {
    pub min: usize,
    pub max: usize,
}

impl LengthTokenFilter {
    pub fn new(min: usize, max: usize) -> Self {
        Self { min, max }
    }
}

impl Default for LengthTokenFilter {
    fn default() -> Self {
        Self {
            min: 0,
            max: usize::MAX,
        }
    }
}

impl TokenFilter for LengthTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let len = token.term.chars().count();
        if len < self.min || len > self.max {
            return (true, None);
        }
        (false, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length_filter() {
        let f = LengthTokenFilter::new(3, 5);
        let mut short = Token::new("ab", 0, 2, 0);
        let mut ok = Token::new("abc", 0, 3, 1);
        let mut long = Token::new("abcdef", 0, 6, 2);

        assert_eq!(f.filter(&mut short).0, true);
        assert_eq!(f.filter(&mut ok).0, false);
        assert_eq!(f.filter(&mut long).0, true);
    }
}
