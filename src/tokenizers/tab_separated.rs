use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Tokenizes text on tab characters, useful for TSV (Tab-Separated Values) data.
///
/// Each field between tabs becomes a token. Optionally trims whitespace from fields.
///
/// Example: `"name\tage\tcity"` → `"name"`, `"age"`, `"city"`
#[derive(Clone, Debug)]
pub struct TabSeparatedTokenizer {
    /// Trim leading/trailing whitespace from each field
    trim_fields: bool,
    /// Skip empty fields
    skip_empty: bool,
}

impl TabSeparatedTokenizer {
    pub fn new() -> Self {
        Self {
            trim_fields: true,
            skip_empty: true,
        }
    }

    pub fn with_trim_fields(mut self, v: bool) -> Self {
        self.trim_fields = v;
        self
    }

    pub fn with_skip_empty(mut self, v: bool) -> Self {
        self.skip_empty = v;
        self
    }
}

impl Default for TabSeparatedTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for TabSeparatedTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut position = 0u32;

        for field in text.split('\t') {
            let trimmed = if self.trim_fields {
                field.trim()
            } else {
                field
            };

            if self.skip_empty && trimmed.is_empty() {
                continue;
            }

            let offset = trimmed.as_ptr() as usize - text.as_ptr() as usize;
            tokens.push(Token {
                term: Cow::Borrowed(trimmed),
                start_offset: offset as u32,
                end_offset: (offset + trimmed.len()) as u32,
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
    fn test_basic_tsv() {
        let tok = TabSeparatedTokenizer::new();
        let tokens = tok.tokenize("name\tage\tcity");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["name", "age", "city"]);
    }

    #[test]
    fn test_trim_whitespace() {
        let tok = TabSeparatedTokenizer::new();
        let tokens = tok.tokenize(" hello \t world \t foo ");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["hello", "world", "foo"]);
    }

    #[test]
    fn test_empty_fields() {
        let tok = TabSeparatedTokenizer::new();
        let tokens = tok.tokenize("a\t\tb");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["a", "b"]); // empty field skipped
    }

    #[test]
    fn test_keep_empty() {
        let tok = TabSeparatedTokenizer::new().with_skip_empty(false).with_trim_fields(false);
        let tokens = tok.tokenize("a\t\tb");
        assert_eq!(tokens.len(), 3); // includes empty field
    }

    #[test]
    fn test_no_tabs() {
        let tok = TabSeparatedTokenizer::new();
        let tokens = tok.tokenize("hello world");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].term.as_ref(), "hello world");
    }
}
