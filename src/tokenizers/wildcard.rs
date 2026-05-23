use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Preserves wildcard characters (`*`, `?`) as part of tokens for wildcard query support.
///
/// Regular tokenizers strip wildcards. This tokenizer keeps them intact so that
/// wildcard queries like `hel*` or `te?t` can be indexed or searched properly.
///
/// Splits on whitespace and common delimiters but preserves `*` and `?` within tokens.
#[derive(Clone, Debug)]
pub struct WildcardTokenizer {
    /// Convert to lowercase
    lowercase: bool,
}

impl WildcardTokenizer {
    pub fn new() -> Self {
        Self { lowercase: true }
    }

    pub fn with_lowercase(mut self, v: bool) -> Self {
        self.lowercase = v;
        self
    }

    fn is_token_char(c: char) -> bool {
        c.is_alphanumeric() || c == '*' || c == '?' || c == '_'
    }
}

impl Default for WildcardTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for WildcardTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut position = 0u32;
        let mut i = 0;

        while i < text.len() {
            let c = text[i..].chars().next().unwrap();
            let c_len = c.len_utf8();

            if !Self::is_token_char(c) {
                i += c_len;
                continue;
            }

            let start = i;
            while i < text.len() {
                let nc = text[i..].chars().next().unwrap();
                if Self::is_token_char(nc) {
                    i += nc.len_utf8();
                } else {
                    break;
                }
            }

            let word = &text[start..i];
            if self.lowercase {
                let lower = word.to_lowercase();
                tokens.push(Token {
                    term: Cow::Owned(lower),
                    start_offset: start as u32,
                    end_offset: i as u32,
                    position,
                });
            } else {
                tokens.push(Token {
                    term: Cow::Borrowed(word),
                    start_offset: start as u32,
                    end_offset: i as u32,
                    position,
                });
            }
            position += 1;
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wildcard_star() {
        let tok = WildcardTokenizer::new();
        let tokens = tok.tokenize("hel* world");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["hel*", "world"]);
    }

    #[test]
    fn test_wildcard_question() {
        let tok = WildcardTokenizer::new();
        let tokens = tok.tokenize("te?t pattern");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["te?t", "pattern"]);
    }

    #[test]
    fn test_multiple_wildcards() {
        let tok = WildcardTokenizer::new();
        let tokens = tok.tokenize("h*ll? w*rld");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["h*ll?", "w*rld"]);
    }

    #[test]
    fn test_no_wildcards() {
        let tok = WildcardTokenizer::new();
        let tokens = tok.tokenize("hello world");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["hello", "world"]);
    }

    #[test]
    fn test_uppercase_lowered() {
        let tok = WildcardTokenizer::new();
        let tokens = tok.tokenize("HEL* World");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["hel*", "world"]);
    }

    #[test]
    fn test_no_lowercase() {
        let tok = WildcardTokenizer::new().with_lowercase(false);
        let tokens = tok.tokenize("HEL* World");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["HEL*", "World"]);
    }
}
