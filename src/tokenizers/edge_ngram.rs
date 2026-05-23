use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Generates edge n-grams (prefix-anchored) from the input text.
#[derive(Clone, Debug)]
pub struct EdgeNgramTokenizer {
    pub min_gram: usize,
    pub max_gram: usize,
}

impl EdgeNgramTokenizer {
    pub fn new(min_gram: usize, max_gram: usize) -> Self {
        Self { min_gram, max_gram }
    }
}

impl Default for EdgeNgramTokenizer {
    fn default() -> Self {
        Self::new(1, 2)
    }
}

impl Tokenizer for EdgeNgramTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let chars: Vec<(usize, char)> = text.char_indices().collect();
        let char_count = chars.len();

        for n in self.min_gram..=self.max_gram {
            if n > char_count {
                break;
            }
            let end_byte = if n < char_count {
                chars[n].0
            } else {
                text.len()
            };
            tokens.push(Token::new(
                &text[0..end_byte],
                0,
                end_byte as u32,
                (n - self.min_gram) as u32,
            ));
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_ngram() {
        let t = EdgeNgramTokenizer::new(1, 3);
        let tokens = t.tokenize("hello");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["h", "he", "hel"]);
    }

    #[test]
    fn test_edge_ngram_short_input() {
        let t = EdgeNgramTokenizer::new(1, 5);
        let tokens = t.tokenize("hi");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["h", "hi"]);
    }
}
