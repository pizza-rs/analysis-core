use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Splits text on non-letter characters.
#[derive(Clone, Debug)]
pub struct LetterTokenizer;

impl LetterTokenizer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LetterTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for LetterTokenizer {
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
                tokens.push(Token::new(&text[s..i], s as u32, i as u32, position));
                position += 1;
                start = None;
            }
        }

        // Handle trailing token
        if let Some(s) = start {
            tokens.push(Token::new(
                &text[s..],
                s as u32,
                text.len() as u32,
                position,
            ));
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_letter_tokenizer() {
        let t = LetterTokenizer::new();
        let tokens = t.tokenize("Hello, World! 123");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].term.as_ref(), "Hello");
        assert_eq!(tokens[1].term.as_ref(), "World");
    }

    #[test]
    fn test_letter_tokenizer_unicode() {
        let t = LetterTokenizer::new();
        let tokens = t.tokenize("café résumé");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].term.as_ref(), "café");
        assert_eq!(tokens[1].term.as_ref(), "résumé");
    }
}
