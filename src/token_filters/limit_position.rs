use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Removes tokens whose position exceeds a configured maximum.
/// Limits how many token positions are indexed.
#[derive(Clone, Debug)]
pub struct LimitTokenPositionFilter {
    pub max_token_position: u32,
}

impl LimitTokenPositionFilter {
    pub fn new(max_token_position: u32) -> Self {
        Self { max_token_position }
    }
}

impl TokenFilter for LimitTokenPositionFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        if token.position > self.max_token_position {
            return (true, None);
        }
        (false, None)
    }
}
