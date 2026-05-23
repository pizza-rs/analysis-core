use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Truncating tokenizer that limits token length.
///
/// Splits text on whitespace and truncates tokens that exceed `max_length`.
/// Optionally emits both the truncated and original forms.
///
/// Useful for preventing excessively long tokens from consuming index space,
/// or for creating prefix-based search indices.
#[derive(Clone, Debug)]
pub struct TruncateTokenizer {
    /// Maximum length of a token (in bytes)
    max_length: usize,
    /// Also emit the original (untruncated) token
    preserve_original: bool,
}

impl TruncateTokenizer {
    pub fn new(max_length: usize) -> Self {
        Self {
            max_length,
            preserve_original: false,
        }
    }

    pub fn with_preserve_original(mut self, v: bool) -> Self {
        self.preserve_original = v;
        self
    }

    fn truncate_str<'a>(s: &'a str, max_len: usize) -> &'a str {
        if s.len() <= max_len {
            return s;
        }
        // Find the last valid UTF-8 boundary within max_len
        let mut end = max_len;
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        &s[..end]
    }
}

impl Default for TruncateTokenizer {
    fn default() -> Self {
        Self::new(256)
    }
}

impl Tokenizer for TruncateTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut position = 0u32;

        for word in text.split_whitespace() {
            let offset = word.as_ptr() as usize - text.as_ptr() as usize;

            if word.len() <= self.max_length {
                tokens.push(Token {
                    term: Cow::Borrowed(word),
                    start_offset: offset as u32,
                    end_offset: (offset + word.len()) as u32,
                    position,
                });
            } else {
                if self.preserve_original {
                    tokens.push(Token {
                        term: Cow::Borrowed(word),
                        start_offset: offset as u32,
                        end_offset: (offset + word.len()) as u32,
                        position,
                    });
                }
                let truncated = Self::truncate_str(word, self.max_length);
                tokens.push(Token {
                    term: Cow::Borrowed(truncated),
                    start_offset: offset as u32,
                    end_offset: (offset + truncated.len()) as u32,
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
    fn test_no_truncation_needed() {
        let tok = TruncateTokenizer::new(10);
        let tokens = tok.tokenize("hello world");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["hello", "world"]);
    }

    #[test]
    fn test_truncation() {
        let tok = TruncateTokenizer::new(3);
        let tokens = tok.tokenize("hello world");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["hel", "wor"]);
    }

    #[test]
    fn test_preserve_original() {
        let tok = TruncateTokenizer::new(3).with_preserve_original(true);
        let tokens = tok.tokenize("hello");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"hello"));
        assert!(terms.contains(&"hel"));
    }

    #[test]
    fn test_utf8_boundary() {
        let tok = TruncateTokenizer::new(4);
        let tokens = tok.tokenize("café");
        // 'é' is 2 bytes, so "café" = 5 bytes, truncate to 4 = "caf"
        assert_eq!(tokens.len(), 1);
        let term = tokens[0].term.as_ref();
        assert!(term.len() <= 4);
    }

    #[test]
    fn test_mixed_lengths() {
        let tok = TruncateTokenizer::new(5);
        let tokens = tok.tokenize("hi longword ok");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["hi", "longw", "ok"]);
    }
}
