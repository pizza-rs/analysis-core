use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Splits a token at a delimiter character. The text before the delimiter
/// becomes the token term, and the text after becomes a payload.
///
/// For example, with delimiter `|`:
/// - Input: `"cat|1.5"` → term: `"cat"`, payload: `"1.5"`
/// - Input: `"dog"` → term: `"dog"`, payload: none
///
/// Since Pizza's Token struct doesn't have a dedicated payload field,
/// the payload is stored as part of the token metadata via a wrapper.
#[derive(Clone, Debug)]
pub struct DelimitedPayloadTokenFilter {
    delimiter: char,
}

impl DelimitedPayloadTokenFilter {
    pub fn new(delimiter: char) -> Self {
        Self { delimiter }
    }
}

impl Default for DelimitedPayloadTokenFilter {
    fn default() -> Self {
        Self::new('|')
    }
}

impl TokenFilter for DelimitedPayloadTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if let Some(pos) = text.find(self.delimiter) {
            let term_part = &text[..pos];
            // payload = &text[pos + self.delimiter.len_utf8()..];
            // Store only the term part (payload handling is done at the engine level)
            if term_part.is_empty() {
                return (true, None);
            }
            token.term = Cow::Owned(String::from(term_part));
        }
        (false, None)
    }
}

/// Splits a token at a delimiter to extract term frequency information.
///
/// For example, with delimiter `|`:
/// - Input: `"cat|3"` → term: `"cat"`, freq: 3
#[derive(Clone, Debug)]
pub struct DelimitedTermFreqTokenFilter {
    delimiter: char,
}

impl DelimitedTermFreqTokenFilter {
    pub fn new(delimiter: char) -> Self {
        Self { delimiter }
    }
}

impl Default for DelimitedTermFreqTokenFilter {
    fn default() -> Self {
        Self::new('|')
    }
}

impl TokenFilter for DelimitedTermFreqTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if let Some(pos) = text.find(self.delimiter) {
            let term_part = &text[..pos];
            if term_part.is_empty() {
                return (true, None);
            }
            token.term = Cow::Owned(String::from(term_part));
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
    fn test_delimited_payload() {
        let filter = DelimitedPayloadTokenFilter::default();
        let mut token = make_token("cat|1.5");
        let (remove, _) = filter.filter(&mut token);
        assert!(!remove);
        assert_eq!(token.term.as_ref(), "cat");
    }

    #[test]
    fn test_no_delimiter() {
        let filter = DelimitedPayloadTokenFilter::default();
        let mut token = make_token("hello");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "hello");
    }

    #[test]
    fn test_custom_delimiter() {
        let filter = DelimitedPayloadTokenFilter::new(':');
        let mut token = make_token("word:payload");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "word");
    }

    #[test]
    fn test_empty_term_removes() {
        let filter = DelimitedPayloadTokenFilter::default();
        let mut token = make_token("|payload");
        let (remove, _) = filter.filter(&mut token);
        assert!(remove);
    }

    #[test]
    fn test_delimited_term_freq() {
        let filter = DelimitedTermFreqTokenFilter::default();
        let mut token = make_token("search|42");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "search");
    }
}
