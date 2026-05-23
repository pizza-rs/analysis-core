use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Marks specified tokens as keywords to protect them from stemming.
#[derive(Clone, Debug)]
pub struct KeywordMarkerTokenFilter {
    keywords: Vec<alloc::string::String>,
}

impl KeywordMarkerTokenFilter {
    pub fn new(keywords: Vec<alloc::string::String>) -> Self {
        Self { keywords }
    }
}

impl TokenFilter for KeywordMarkerTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        // Marks the token so downstream filters (e.g. stemmer) can skip it.
        // Since Pizza's Token struct doesn't have keyword flags yet,
        // this is a no-op placeholder that will be wired when keyword
        // attribute support is added to Token.
        let _ = self.keywords.contains(&token.term.to_string());
        (false, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_marker_no_deletion() {
        let f = KeywordMarkerTokenFilter::new(vec!["test".into()]);
        let mut token = Token::new("test", 0, 4, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
    }
}
