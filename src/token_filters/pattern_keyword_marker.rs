use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;
use regex::Regex;

/// Marks tokens as keywords by prepending a configurable prefix if they match a regex.
/// Downstream stemmers can check for this prefix and skip the token.
#[derive(Clone, Debug)]
pub struct PatternKeywordMarkerTokenFilter {
    pattern: Regex,
    pub prefix: String,
}

impl PatternKeywordMarkerTokenFilter {
    pub fn new(pattern: &str) -> Self {
        Self {
            pattern: Regex::new(pattern).unwrap_or_else(|_| Regex::new(".*").unwrap()),
            prefix: String::from("__KW__"),
        }
    }

    pub fn with_prefix(pattern: &str, prefix: &str) -> Self {
        Self {
            pattern: Regex::new(pattern).unwrap_or_else(|_| Regex::new(".*").unwrap()),
            prefix: String::from(prefix),
        }
    }
}

impl TokenFilter for PatternKeywordMarkerTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        // Emit the keyword-marked version as a synonym at same position
        if self.pattern.is_match(token.term.as_ref()) {
            let marked = format!("{}{}", self.prefix, token.term);
            let synonym = Token {
                term: Cow::Owned(marked),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            };
            return (false, Some(alloc::vec![synonym]));
        }
        (false, None)
    }
}
