use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Splits structured identifiers into searchable components.
///
/// Handles:
/// - UUIDs: `550e8400-e29b-41d4-a716-446655440000` → segments + full
/// - Serial numbers: `SN-2024-ABC-001` → parts
/// - Version numbers: `v1.2.3-beta.1` → components
/// - SKU/product codes: `PROD-A-123-XL` → parts
/// - MAC addresses: `00:1B:44:11:3A:B7` → segments
/// - IP addresses: `192.168.1.1` → octets
#[derive(Clone, Debug)]
pub struct StructuredIdTokenizer {
    /// Also emit the original unsplit identifier
    preserve_original: bool,
    /// Delimiters to split on (in addition to auto-detected ones)
    delimiters: String,
}

impl StructuredIdTokenizer {
    pub fn new() -> Self {
        Self {
            preserve_original: true,
            delimiters: String::from("-_.:"),
        }
    }

    pub fn with_preserve_original(mut self, v: bool) -> Self {
        self.preserve_original = v;
        self
    }

    pub fn with_delimiters(mut self, d: &str) -> Self {
        self.delimiters = String::from(d);
        self
    }

    fn is_delimiter(&self, c: char) -> bool {
        self.delimiters.contains(c)
    }
}

impl Default for StructuredIdTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for StructuredIdTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut position = 0u32;

        for word in text.split_whitespace() {
            let word_offset = word.as_ptr() as usize - text.as_ptr() as usize;

            // Check if this looks like a structured ID (contains delimiters)
            let has_delimiters = word.chars().any(|c| self.is_delimiter(c));

            if !has_delimiters {
                // Regular word, emit as-is
                tokens.push(Token {
                    term: Cow::Borrowed(word),
                    start_offset: word_offset as u32,
                    end_offset: (word_offset + word.len()) as u32,
                    position,
                });
                position += 1;
                continue;
            }

            // Structured ID: emit original + parts
            if self.preserve_original {
                tokens.push(Token {
                    term: Cow::Borrowed(word),
                    start_offset: word_offset as u32,
                    end_offset: (word_offset + word.len()) as u32,
                    position,
                });
            }

            // Split on delimiters
            let mut part_start = 0;
            let mut chars_iter = word.char_indices().peekable();

            while let Some((i, c)) = chars_iter.next() {
                if self.is_delimiter(c) {
                    if part_start < i {
                        let part = &word[part_start..i];
                        let offset = word_offset + part_start;
                        tokens.push(Token {
                            term: Cow::Borrowed(part),
                            start_offset: offset as u32,
                            end_offset: (offset + part.len()) as u32,
                            position,
                        });
                        position += 1;
                    }
                    part_start = i + c.len_utf8();
                }
            }

            // Last part
            if part_start < word.len() {
                let part = &word[part_start..];
                let offset = word_offset + part_start;
                tokens.push(Token {
                    term: Cow::Borrowed(part),
                    start_offset: offset as u32,
                    end_offset: (offset + part.len()) as u32,
                    position,
                });
                position += 1;
            }
            // If the word ended with a delimiter, the last in-loop emit
            // already advanced `position`; we intentionally do NOT add
            // another increment here, which previously left a spurious
            // position gap between words like "A-" and the next word.
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid() {
        let tok = StructuredIdTokenizer::new();
        let tokens = tok.tokenize("550e8400-e29b-41d4-a716-446655440000");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        // Original + 5 segments
        assert!(terms.contains(&"550e8400-e29b-41d4-a716-446655440000"));
        assert!(terms.contains(&"550e8400"));
        assert!(terms.contains(&"e29b"));
        assert!(terms.contains(&"41d4"));
        assert!(terms.contains(&"a716"));
        assert!(terms.contains(&"446655440000"));
    }

    #[test]
    fn test_version_number() {
        let tok = StructuredIdTokenizer::new();
        let tokens = tok.tokenize("v1.2.3");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"v1.2.3"));
        assert!(terms.contains(&"v1"));
        assert!(terms.contains(&"2"));
        assert!(terms.contains(&"3"));
    }

    #[test]
    fn test_ip_address() {
        let tok = StructuredIdTokenizer::new();
        let tokens = tok.tokenize("192.168.1.1");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"192.168.1.1"));
        assert!(terms.contains(&"192"));
        assert!(terms.contains(&"168"));
    }

    #[test]
    fn test_regular_word() {
        let tok = StructuredIdTokenizer::new();
        let tokens = tok.tokenize("hello");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].term.as_ref(), "hello");
    }

    #[test]
    fn test_no_original() {
        let tok = StructuredIdTokenizer::new().with_preserve_original(false);
        let tokens = tok.tokenize("a-b-c");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_trailing_delimiter_no_position_gap() {
        // Previously, a word ending with a delimiter (e.g. "A-") would
        // double-increment `position`, leaving a phantom position slot
        // between successive words. Verify the position trail is contiguous.
        let tok = StructuredIdTokenizer::new().with_preserve_original(false);
        let tokens = tok.tokenize("a- b");
        let positions: Vec<u32> = tokens.iter().map(|t| t.position).collect();
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["a", "b"]);
        // "a" at pos 0, "b" should follow immediately at pos 1 (not pos 2).
        assert_eq!(positions, vec![0, 1]);
    }
}
