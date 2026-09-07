use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;
use regex::Regex;

/// Emits a type-tag token at the same position based on regex pattern matching.
/// If the token matches a pattern, a tag token (e.g. "_type:email") is emitted as synonym.
#[derive(Clone, Debug)]
pub struct PatternTypingTokenFilter {
    rules: Vec<(Regex, String)>,
}

impl PatternTypingTokenFilter {
    pub fn new(rules: &[(&str, &str)]) -> Self {
        Self {
            rules: rules
                .iter()
                .filter_map(|(pat, tag)| Regex::new(pat).ok().map(|r| (r, String::from(*tag))))
                .collect(),
        }
    }

    pub fn single(pattern: &str, tag: &str) -> Self {
        Self::new(&[(pattern, tag)])
    }
}

impl Default for PatternTypingTokenFilter {
    fn default() -> Self {
        Self { rules: Vec::new() }
    }
}

impl TokenFilter for PatternTypingTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        if token.term.starts_with("_type:") {
            return (false, None); // idempotent: never re-tag an emitted tag
        }
        for (regex, tag) in &self.rules {
            if regex.is_match(token.term.as_ref()) {
                let type_token = Token {
                    term: Cow::Owned(format!("_type:{}", tag)),
                    start_offset: token.start_offset,
                    end_offset: token.end_offset,
                    position: token.position,
                };
                return (false, Some(alloc::vec![type_token]));
            }
        }
        (false, None)
    }
}
