use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Reverse tokenizer that emits tokens in reverse character order.
///
/// Useful for suffix-based searching (e.g., finding files by extension, or
/// words by suffix like "-tion", "-ment").
///
/// Example: `"hello world"` → `"olleh"`, `"dlrow"`
///
/// Combine with a regular tokenizer and prefix queries to achieve suffix search.
#[derive(Clone, Debug)]
pub struct ReverseTokenizer;

impl ReverseTokenizer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ReverseTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for ReverseTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut position = 0u32;

        for word in text.split_whitespace() {
            let offset = word.as_ptr() as usize - text.as_ptr() as usize;
            let reversed: String = word.chars().rev().collect();

            tokens.push(Token {
                term: Cow::Owned(reversed),
                start_offset: offset as u32,
                end_offset: (offset + word.len()) as u32,
                position,
            });
            position += 1;
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reverse_basic() {
        let tok = ReverseTokenizer::new();
        let tokens = tok.tokenize("hello world");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["olleh", "dlrow"]);
    }

    #[test]
    fn test_reverse_single_char() {
        let tok = ReverseTokenizer::new();
        let tokens = tok.tokenize("a b c");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_reverse_palindrome() {
        let tok = ReverseTokenizer::new();
        let tokens = tok.tokenize("racecar");
        assert_eq!(tokens[0].term.as_ref(), "racecar");
    }

    #[test]
    fn test_reverse_unicode() {
        let tok = ReverseTokenizer::new();
        let tokens = tok.tokenize("café");
        assert_eq!(tokens[0].term.as_ref(), "éfac");
    }

    #[test]
    fn test_positions() {
        let tok = ReverseTokenizer::new();
        let tokens = tok.tokenize("one two three");
        assert_eq!(tokens[0].position, 0);
        assert_eq!(tokens[1].position, 1);
        assert_eq!(tokens[2].position, 2);
    }
}
