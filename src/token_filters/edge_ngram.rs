use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Generates edge n-grams (prefix-anchored) from each token.
///
/// For example, with min_gram=2, max_gram=4, the token "pizza" produces:
/// `["pi", "piz", "pizz"]`
#[derive(Clone, Debug)]
pub struct EdgeNgramTokenFilter {
    min_gram: usize,
    max_gram: usize,
    preserve_original: bool,
}

impl EdgeNgramTokenFilter {
    pub fn new(min_gram: usize, max_gram: usize) -> Self {
        Self {
            min_gram: min_gram.max(1),
            max_gram: max_gram.max(min_gram),
            preserve_original: false,
        }
    }

    pub fn with_preserve_original(mut self, preserve: bool) -> Self {
        self.preserve_original = preserve;
        self
    }
}

impl Default for EdgeNgramTokenFilter {
    fn default() -> Self {
        Self::new(1, 1)
    }
}

impl TokenFilter for EdgeNgramTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.to_string();
        let chars: Vec<char> = text.chars().collect();
        let char_count = chars.len();

        if char_count == 0 {
            return (true, None);
        }

        // If token is shorter than min_gram
        if char_count < self.min_gram {
            if self.preserve_original {
                return (false, None);
            }
            return (true, None);
        }

        let effective_max = self.max_gram.min(char_count);
        let mut extra_tokens: Vec<Token<'a>> = Vec::new();
        let mut first = true;

        for n in self.min_gram..=effective_max {
            let gram: String = chars[..n].iter().collect();
            if first {
                token.term = Cow::Owned(gram);
                first = false;
            } else {
                extra_tokens.push(Token {
                    term: Cow::Owned(gram),
                    start_offset: token.start_offset,
                    end_offset: token.end_offset,
                    position: token.position,
                });
            }
        }

        if self.preserve_original && effective_max < char_count {
            extra_tokens.push(Token {
                term: Cow::Owned(text.to_string()),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            });
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
    fn test_edge_ngrams() {
        let filter = EdgeNgramTokenFilter::new(1, 4);
        let mut token = make_token("pizza");
        let (remove, extra) = filter.filter(&mut token);
        assert!(!remove);
        assert_eq!(token.term.as_ref(), "p");
        let extra = extra.unwrap();
        let terms: Vec<&str> = extra.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["pi", "piz", "pizz"]);
    }

    #[test]
    fn test_max_exceeds_length() {
        let filter = EdgeNgramTokenFilter::new(2, 10);
        let mut token = make_token("abc");
        let (remove, extra) = filter.filter(&mut token);
        assert!(!remove);
        assert_eq!(token.term.as_ref(), "ab");
        let extra = extra.unwrap();
        assert_eq!(extra[0].term.as_ref(), "abc");
    }

    #[test]
    fn test_too_short() {
        let filter = EdgeNgramTokenFilter::new(3, 5);
        let mut token = make_token("ab");
        let (remove, _) = filter.filter(&mut token);
        assert!(remove);
    }

    #[test]
    fn test_preserve_original() {
        let filter = EdgeNgramTokenFilter::new(1, 2).with_preserve_original(true);
        let mut token = make_token("pizza");
        let (_, extra) = filter.filter(&mut token);
        let extra = extra.unwrap();
        assert_eq!(extra.last().unwrap().term.as_ref(), "pizza");
    }
}
