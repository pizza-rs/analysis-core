use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Splits text on non-letter characters and lowercases all tokens.
#[derive(Clone, Debug)]
pub struct LowercaseTokenizer;

impl LowercaseTokenizer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LowercaseTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for LowercaseTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut position = 0u32;
        let mut start = None;

        for (i, ch) in text.char_indices() {
            if ch.is_alphabetic() {
                if start.is_none() {
                    start = Some(i);
                }
            } else if let Some(s) = start {
                let segment = &text[s..i];
                let lower = segment.to_lowercase();
                tokens.push(Token {
                    term: Cow::Owned(lower),
                    start_offset: s as u32,
                    end_offset: i as u32,
                    position,
                });
                position += 1;
                start = None;
            }
        }

        // Handle trailing token
        if let Some(s) = start {
            let segment = &text[s..];
            let lower = segment.to_lowercase();
            tokens.push(Token {
                term: Cow::Owned(lower),
                start_offset: s as u32,
                end_offset: text.len() as u32,
                position,
            });
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lowercase_tokenizer() {
        let t = LowercaseTokenizer::new();
        let tokens = t.tokenize("Hello, WORLD! Test123");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].term.as_ref(), "hello");
        assert_eq!(tokens[1].term.as_ref(), "world");
        assert_eq!(tokens[2].term.as_ref(), "test");
    }
}
