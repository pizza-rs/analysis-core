use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// A predicate that determines whether a token should be processed by a sub-filter.
pub trait TokenPredicate: TokenPredicateClone + Send + Sync {
    fn matches(&self, token: &Token<'_>) -> bool;
}

/// Object-safe clone helper for [`TokenPredicate`].
pub trait TokenPredicateClone {
    fn clone_predicate(&self) -> Box<dyn TokenPredicate>;
}

impl<T: 'static + TokenPredicate + Clone> TokenPredicateClone for T {
    fn clone_predicate(&self) -> Box<dyn TokenPredicate> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn TokenPredicate> {
    fn clone(&self) -> Box<dyn TokenPredicate> {
        self.clone_predicate()
    }
}

/// A predicate that matches tokens by minimum length.
#[derive(Clone, Debug)]
pub struct MinLengthPredicate(pub usize);

impl TokenPredicate for MinLengthPredicate {
    fn matches(&self, token: &Token<'_>) -> bool {
        token.term.len() >= self.0
    }
}

/// A predicate that matches tokens by maximum length.
#[derive(Clone, Debug)]
pub struct MaxLengthPredicate(pub usize);

impl TokenPredicate for MaxLengthPredicate {
    fn matches(&self, token: &Token<'_>) -> bool {
        token.term.len() <= self.0
    }
}

/// A predicate based on a regex pattern.
#[derive(Clone, Debug)]
pub struct PatternPredicate {
    pub pattern: regex::Regex,
}

impl PatternPredicate {
    pub fn new(pattern: &str) -> Result<Self, regex::Error> {
        Ok(Self {
            pattern: regex::Regex::new(pattern)?,
        })
    }
}

impl TokenPredicate for PatternPredicate {
    fn matches(&self, token: &Token<'_>) -> bool {
        self.pattern.is_match(token.term.as_ref())
    }
}

/// Condition token filter that applies a sub-filter only when a predicate matches.
///
/// Tokens that don't match the predicate pass through unchanged.
/// Tokens that match are processed by the inner filter.
///
/// This enables conditional processing, e.g. only stem tokens longer than 4 chars,
/// or only lowercase tokens matching a specific pattern.
#[derive(Clone)]
pub struct ConditionalTokenFilter {
    predicate: Box<dyn TokenPredicate>,
    filter: Box<dyn TokenFilter>,
}

impl ConditionalTokenFilter {
    pub fn new(predicate: Box<dyn TokenPredicate>, filter: Box<dyn TokenFilter>) -> Self {
        Self { predicate, filter }
    }
}

impl TokenFilter for ConditionalTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        if self.predicate.matches(token) {
            self.filter.filter(token)
        } else {
            (false, None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token_filters::LowercaseTokenFilter;

    fn make_token(term: &str) -> Token<'_> {
        Token {
            term: Cow::Borrowed(term),
            start_offset: 0,
            end_offset: term.len() as u32,
            position: 0,
        }
    }

    #[test]
    fn test_condition_matches() {
        let filter = ConditionalTokenFilter::new(
            Box::new(MinLengthPredicate(4)),
            Box::new(LowercaseTokenFilter),
        );

        let mut token = make_token("HELLO");
        let (remove, _) = filter.filter(&mut token);
        assert!(!remove);
        assert_eq!(token.term.as_ref(), "hello"); // >= 4 chars, so lowercased
    }

    #[test]
    fn test_condition_no_match() {
        let filter = ConditionalTokenFilter::new(
            Box::new(MinLengthPredicate(4)),
            Box::new(LowercaseTokenFilter),
        );

        let mut token = make_token("HI");
        let (remove, _) = filter.filter(&mut token);
        assert!(!remove);
        assert_eq!(token.term.as_ref(), "HI"); // < 4 chars, unchanged
    }

    #[test]
    fn test_pattern_predicate() {
        let pred = PatternPredicate::new(r"^[A-Z]").unwrap();
        let filter = ConditionalTokenFilter::new(Box::new(pred), Box::new(LowercaseTokenFilter));

        let mut t1 = make_token("Hello");
        filter.filter(&mut t1);
        assert_eq!(t1.term.as_ref(), "hello"); // starts with uppercase

        let mut t2 = make_token("hello");
        filter.filter(&mut t2);
        assert_eq!(t2.term.as_ref(), "hello"); // already lowercase, no match
    }
}
