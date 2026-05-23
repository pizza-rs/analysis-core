use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Removes tokens whose Unicode codepoint count falls outside [min, max].
/// This explicitly documents Unicode codepoint semantics for Lucene API compatibility.
#[derive(Clone, Debug)]
pub struct CodepointCountTokenFilter {
    pub min: usize,
    pub max: usize,
}

impl CodepointCountTokenFilter {
    pub fn new(min: usize, max: usize) -> Self {
        Self { min, max }
    }
}

impl TokenFilter for CodepointCountTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let count = token.term.chars().count();
        if count < self.min || count > self.max {
            return (true, None);
        }
        (false, None)
    }
}
