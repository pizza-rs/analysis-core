use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Multiplexer token filter that applies multiple sub-filters to each token,
/// emitting all variants at the same position.
///
/// This is useful when you want to index multiple analyzed forms of each token.
/// For example, you might want both the stemmed and unstemmed forms, or both
/// ASCII-folded and original forms.
///
/// Each token passes through every registered sub-filter independently, and
/// all results are emitted at the same position.
///
/// Configuration:
/// - `filters`: list of token filters to apply in parallel
/// - `preserve_original`: whether to keep the original token (default: true)
#[derive(Clone)]
pub struct MultiplexerTokenFilter {
    filters: Vec<Box<dyn TokenFilter>>,
    preserve_original: bool,
}

impl MultiplexerTokenFilter {
    pub fn new(filters: Vec<Box<dyn TokenFilter>>) -> Self {
        Self {
            filters,
            preserve_original: true,
        }
    }

    pub fn with_preserve_original(mut self, preserve: bool) -> Self {
        self.preserve_original = preserve;
        self
    }
}

impl TokenFilter for MultiplexerTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        if self.filters.is_empty() {
            return (false, None);
        }

        let mut extra = Vec::new();

        for sub_filter in &self.filters {
            let mut cloned = Token {
                term: Cow::Owned(token.term.as_ref().to_owned()),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            };

            let (remove, sub_extra) = sub_filter.filter(&mut cloned);

            if !remove {
                // Only add if it's different from the original
                if cloned.term.as_ref() != token.term.as_ref() {
                    extra.push(cloned);
                }
            }

            if let Some(sub_tokens) = sub_extra {
                extra.extend(sub_tokens);
            }
        }

        if !self.preserve_original && !extra.is_empty() {
            // Replace original with first variant
            let first = extra.remove(0);
            token.term = first.term;
        }

        if extra.is_empty() {
            (false, None)
        } else {
            (false, Some(extra))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token_filters::LowercaseTokenFilter;
    use crate::token_filters::ReverseTokenFilter;

    fn make_token(term: &str) -> Token<'_> {
        Token {
            term: Cow::Borrowed(term),
            start_offset: 0,
            end_offset: term.len() as u32,
            position: 0,
        }
    }

    #[test]
    fn test_multiplex_preserves_original() {
        let filter = MultiplexerTokenFilter::new(vec![
            Box::new(LowercaseTokenFilter),
            Box::new(ReverseTokenFilter),
        ]);

        let mut token = make_token("Hello");
        let (remove, extra) = filter.filter(&mut token);
        assert!(!remove);
        assert_eq!(token.term.as_ref(), "Hello"); // original preserved
        let extra = extra.unwrap();
        let terms: Vec<&str> = extra.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"hello")); // lowercase variant
        assert!(terms.contains(&"olleH")); // reversed variant
    }

    #[test]
    fn test_multiplex_no_duplicate() {
        // If a filter doesn't change the token, it shouldn't produce an extra
        let filter = MultiplexerTokenFilter::new(vec![Box::new(LowercaseTokenFilter)]);

        let mut token = make_token("hello"); // already lowercase
        let (_, extra) = filter.filter(&mut token);
        assert!(extra.is_none()); // no change, no extra
    }
}
