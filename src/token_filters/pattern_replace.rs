use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;
use regex::Regex;

/// Applies regex-based find/replace on each token's text.
///
/// Unlike the `PatternReplaceNormalizer` which works on the full input text,
/// this filter operates on individual token terms.
#[derive(Clone, Debug)]
pub struct PatternReplaceTokenFilter {
    pattern: Regex,
    replacement: String,
    replace_all: bool,
}

impl PatternReplaceTokenFilter {
    pub fn new(pattern: &str, replacement: &str) -> Option<Self> {
        Regex::new(pattern).ok().map(|re| Self {
            pattern: re,
            replacement: String::from(replacement),
            replace_all: true,
        })
    }

    pub fn with_replace_all(mut self, all: bool) -> Self {
        self.replace_all = all;
        self
    }
}

impl TokenFilter for PatternReplaceTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let result = if self.replace_all {
            self.pattern.replace_all(text, self.replacement.as_str())
        } else {
            self.pattern.replace(text, self.replacement.as_str())
        };

        match result {
            Cow::Borrowed(_) => {} // no change
            Cow::Owned(s) => {
                if s.is_empty() {
                    return (true, None); // remove empty tokens
                }
                token.term = Cow::Owned(s);
            }
        }
        (false, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_token(term: &str) -> Token<'_> {
        Token {
            term: Cow::Borrowed(term),
            start_offset: 0,
            end_offset: term.len() as u32,
            position: 0,
        }
    }

    #[test]
    fn test_pattern_replace() {
        let filter = PatternReplaceTokenFilter::new(r"\d+", "").unwrap();
        let mut token = make_token("abc123def456");
        let (remove, _) = filter.filter(&mut token);
        assert!(!remove);
        assert_eq!(token.term.as_ref(), "abcdef");
    }

    #[test]
    fn test_replace_first_only() {
        let filter = PatternReplaceTokenFilter::new(r"\d+", "X")
            .unwrap()
            .with_replace_all(false);
        let mut token = make_token("a1b2c3");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "aXb2c3");
    }

    #[test]
    fn test_no_match() {
        let filter = PatternReplaceTokenFilter::new(r"\d+", "X").unwrap();
        let mut token = make_token("hello");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "hello");
    }

    #[test]
    fn test_empty_result_removes() {
        let filter = PatternReplaceTokenFilter::new(r".+", "").unwrap();
        let mut token = make_token("hello");
        let (remove, _) = filter.filter(&mut token);
        assert!(remove);
    }
}
