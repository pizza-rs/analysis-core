use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Extracts a boost/payload value from a delimited token.
/// E.g. "term|1.5" → keeps "term", emits "_boost:1.5" as synonym.
#[derive(Clone, Debug)]
pub struct DelimitedBoostTokenFilter {
    pub delimiter: char,
}

impl DelimitedBoostTokenFilter {
    pub fn new(delimiter: char) -> Self {
        Self { delimiter }
    }
}

impl Default for DelimitedBoostTokenFilter {
    fn default() -> Self {
        Self::new('|')
    }
}

impl TokenFilter for DelimitedBoostTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        if token.term.starts_with("_boost:") {
            return (false, None); // idempotent: never re-tag an emitted tag
        }
        if let Some(pos) = token.term.find(self.delimiter) {
            let text = &token.term.as_ref()[..pos];
            let payload = &token.term.as_ref()[pos + self.delimiter.len_utf8()..];
            let boost_token = Token {
                term: Cow::Owned(format!("_boost:{}", payload)),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            };
            token.term = Cow::Owned(String::from(text));
            return (false, Some(alloc::vec![boost_token]));
        }
        (false, None)
    }
}
