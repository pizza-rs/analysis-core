use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Drops tokens matching a configurable prefix pattern.
/// Useful for removing tokens marked by upstream filters.
#[derive(Clone, Debug)]
pub struct DropIfFlaggedTokenFilter {
    pub flag_prefix: String,
}

impl DropIfFlaggedTokenFilter {
    pub fn new(prefix: &str) -> Self {
        Self {
            flag_prefix: String::from(prefix),
        }
    }
}

impl Default for DropIfFlaggedTokenFilter {
    fn default() -> Self {
        Self::new("__DROP__")
    }
}

impl TokenFilter for DropIfFlaggedTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        if !self.flag_prefix.is_empty() && token.term.starts_with(self.flag_prefix.as_str()) {
            return (true, None);
        }
        (false, None)
    }
}
