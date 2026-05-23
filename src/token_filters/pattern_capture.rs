use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;
use regex::Regex;

/// Emits tokens for each regex capture group match against the input token.
///
/// Multiple patterns can be specified. For each pattern that matches the token,
/// each non-empty capture group is emitted as a separate token at the same position.
///
/// # Example
/// Pattern `"(\\d+)-(\\w+)"` on token `"123-abc"` produces: `["123", "abc"]`
#[derive(Clone, Debug)]
pub struct PatternCaptureTokenFilter {
    patterns: Vec<Regex>,
    preserve_original: bool,
}

impl PatternCaptureTokenFilter {
    pub fn new(patterns: Vec<&str>, preserve_original: bool) -> Self {
        let compiled: Vec<Regex> = patterns
            .into_iter()
            .filter_map(|p| Regex::new(p).ok())
            .collect();
        Self {
            patterns: compiled,
            preserve_original,
        }
    }
}

impl TokenFilter for PatternCaptureTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let mut captures: Vec<String> = Vec::new();

        for pattern in &self.patterns {
            for cap in pattern.captures_iter(text) {
                // Skip group 0 (the whole match), emit groups 1..n
                for i in 1..cap.len() {
                    if let Some(m) = cap.get(i) {
                        let s = m.as_str();
                        if !s.is_empty() && !captures.contains(&String::from(s)) {
                            captures.push(String::from(s));
                        }
                    }
                }
            }
        }

        if captures.is_empty() {
            return (false, None);
        }

        let mut extra_tokens: Vec<Token<'a>> = Vec::new();

        if self.preserve_original {
            // Keep original, emit captures as extras
            for cap in captures {
                extra_tokens.push(Token {
                    term: Cow::Owned(cap),
                    start_offset: token.start_offset,
                    end_offset: token.end_offset,
                    position: token.position,
                });
            }
        } else {
            // Replace original with first capture, rest as extras
            let mut iter = captures.into_iter();
            if let Some(first) = iter.next() {
                token.term = Cow::Owned(first);
            }
            for cap in iter {
                extra_tokens.push(Token {
                    term: Cow::Owned(cap),
                    start_offset: token.start_offset,
                    end_offset: token.end_offset,
                    position: token.position,
                });
            }
        }

        if extra_tokens.is_empty() {
            (false, None)
        } else {
            (false, Some(extra_tokens))
        }
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
    fn test_pattern_capture() {
        let filter = PatternCaptureTokenFilter::new(vec![r"(\d+)-(\w+)"], false);
        let mut token = make_token("123-abc");
        let (remove, extra) = filter.filter(&mut token);
        assert!(!remove);
        assert_eq!(token.term.as_ref(), "123");
        let extra = extra.unwrap();
        assert_eq!(extra[0].term.as_ref(), "abc");
    }

    #[test]
    fn test_preserve_original() {
        let filter = PatternCaptureTokenFilter::new(vec![r"(\d+)"], true);
        let mut token = make_token("item123");
        let (remove, extra) = filter.filter(&mut token);
        assert!(!remove);
        assert_eq!(token.term.as_ref(), "item123"); // preserved
        let extra = extra.unwrap();
        assert_eq!(extra[0].term.as_ref(), "123");
    }

    #[test]
    fn test_no_match() {
        let filter = PatternCaptureTokenFilter::new(vec![r"(\d+)"], false);
        let mut token = make_token("hello");
        let (remove, extra) = filter.filter(&mut token);
        assert!(!remove);
        assert!(extra.is_none());
    }

    #[test]
    fn test_multiple_patterns() {
        let filter = PatternCaptureTokenFilter::new(vec![r"(\d+)", r"([a-z]+)"], false);
        let mut token = make_token("abc123");
        let (_, extra) = filter.filter(&mut token);
        // Should capture "123" from first pattern and "abc" from second
        assert!(extra.is_some());
    }
}
