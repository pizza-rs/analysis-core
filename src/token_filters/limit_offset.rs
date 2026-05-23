use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Removes tokens whose end offset exceeds a configured maximum.
/// Useful for truncating indexing at a certain character position.
#[derive(Clone, Debug)]
pub struct LimitTokenOffsetFilter {
    pub max_start_offset: usize,
}

impl LimitTokenOffsetFilter {
    pub fn new(max_start_offset: usize) -> Self {
        Self { max_start_offset }
    }
}

impl TokenFilter for LimitTokenOffsetFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        if token.start_offset as usize > self.max_start_offset {
            return (true, None);
        }
        (false, None)
    }
}
