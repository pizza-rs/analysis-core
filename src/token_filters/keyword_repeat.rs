use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Emits each token twice — once as keyword-protected (won't be stemmed),
/// once as normal. Used in combination with a stemmer and
/// `RemoveDuplicatesTokenFilter` to index both stemmed and unstemmed forms.
///
/// First emission: keyword=true (original form preserved through stemming)
/// Second emission: keyword=false (will be stemmed by subsequent filter)
#[derive(Clone, Debug, Default)]
pub struct KeywordRepeatTokenFilter;

impl KeywordRepeatTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for KeywordRepeatTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        // Emit a duplicate of the current token at the same position
        let duplicate = Token {
            term: token.term.clone(),
            start_offset: token.start_offset,
            end_offset: token.end_offset,
            position: token.position,
        };
        (false, Some(alloc::vec![duplicate]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::borrow::Cow;

    #[test]
    fn test_keyword_repeat() {
        let filter = KeywordRepeatTokenFilter::new();
        let mut token = Token {
            term: Cow::Borrowed("running"),
            start_offset: 0,
            end_offset: 7,
            position: 0,
        };
        let (remove, extra) = filter.filter(&mut token);
        assert!(!remove);
        assert_eq!(token.term.as_ref(), "running");
        let extra = extra.unwrap();
        assert_eq!(extra.len(), 1);
        assert_eq!(extra[0].term.as_ref(), "running");
        assert_eq!(extra[0].position, token.position);
    }
}
