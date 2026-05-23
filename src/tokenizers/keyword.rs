use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// No-op tokenizer — emits the entire input as a single token.
#[derive(Clone, Debug)]
pub struct KeywordTokenizer;

impl KeywordTokenizer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for KeywordTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for KeywordTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        if text.is_empty() {
            return Vec::new();
        }
        vec![Token::new(text, 0, text.len() as u32, 0)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_tokenizer() {
        let t = KeywordTokenizer::new();
        let tokens = t.tokenize("Hello World");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].term.as_ref(), "Hello World");
        assert_eq!(tokens[0].start_offset, 0);
        assert_eq!(tokens[0].end_offset, 11);
    }

    #[test]
    fn test_keyword_tokenizer_empty() {
        let t = KeywordTokenizer::new();
        let tokens = t.tokenize("");
        assert_eq!(tokens.len(), 0);
    }
}
