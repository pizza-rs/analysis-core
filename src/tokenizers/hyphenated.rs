use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Tokenizes hyphenated words by emitting both the compound and individual parts.
///
/// `"well-known"` → `"well-known"`, `"well"`, `"known"`
/// `"state-of-the-art"` → `"state-of-the-art"`, `"state"`, `"of"`, `"the"`, `"art"`
///
/// Useful for English text where hyphenated compounds should be searchable
/// both as the full compound and as individual words.
#[derive(Clone, Debug)]
pub struct HyphenatedTokenizer {
    /// Also emit the full hyphenated form
    preserve_original: bool,
    /// Minimum part length to emit (filters short connectors like "of")
    min_part_length: usize,
}

impl HyphenatedTokenizer {
    pub fn new() -> Self {
        Self {
            preserve_original: true,
            min_part_length: 1,
        }
    }

    pub fn with_preserve_original(mut self, v: bool) -> Self {
        self.preserve_original = v;
        self
    }

    pub fn with_min_part_length(mut self, len: usize) -> Self {
        self.min_part_length = len;
        self
    }
}

impl Default for HyphenatedTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for HyphenatedTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut position = 0u32;
        let mut i = 0;

        while i < text.len() {
            let c = text[i..].chars().next().unwrap();
            let c_len = c.len_utf8();

            if c.is_whitespace() {
                i += c_len;
                continue;
            }

            // Collect a word (possibly hyphenated)
            if c.is_alphanumeric() || c == '-' {
                let start = i;
                while i < text.len() {
                    let nc = text[i..].chars().next().unwrap();
                    if nc.is_alphanumeric() || nc == '-' || nc == '\'' {
                        i += nc.len_utf8();
                    } else {
                        break;
                    }
                }
                // Trim trailing hyphens
                while i > start && text.as_bytes()[i - 1] == b'-' {
                    i -= 1;
                }
                // Trim leading hyphens
                let mut adj_start = start;
                while adj_start < i && text.as_bytes()[adj_start] == b'-' {
                    adj_start += 1;
                }
                if adj_start >= i {
                    continue;
                }

                let word = &text[adj_start..i];

                // Check if it contains hyphens
                if word.contains('-') {
                    if self.preserve_original {
                        tokens.push(Token {
                            term: Cow::Borrowed(word),
                            start_offset: adj_start as u32,
                            end_offset: i as u32,
                            position,
                        });
                    }

                    // Split on hyphens
                    for part in word.split('-') {
                        // `min_part_length` is documented in characters; use a
                        // char count rather than byte length so non-ASCII
                        // parts are measured correctly.
                        if part.chars().count() >= self.min_part_length {
                            let part_offset = part.as_ptr() as usize - text.as_ptr() as usize;
                            tokens.push(Token {
                                term: Cow::Borrowed(part),
                                start_offset: part_offset as u32,
                                end_offset: (part_offset + part.len()) as u32,
                                position,
                            });
                        }
                    }
                } else {
                    // No hyphens, emit as-is
                    tokens.push(Token {
                        term: Cow::Borrowed(word),
                        start_offset: adj_start as u32,
                        end_offset: i as u32,
                        position,
                    });
                }
                position += 1;
                continue;
            }

            i += c_len;
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hyphenated_word() {
        let tok = HyphenatedTokenizer::new();
        let tokens = tok.tokenize("well-known");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"well-known"));
        assert!(terms.contains(&"well"));
        assert!(terms.contains(&"known"));
    }

    #[test]
    fn test_multi_hyphen() {
        let tok = HyphenatedTokenizer::new();
        let tokens = tok.tokenize("state-of-the-art");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"state-of-the-art"));
        assert!(terms.contains(&"state"));
        assert!(terms.contains(&"art"));
    }

    #[test]
    fn test_no_hyphen() {
        let tok = HyphenatedTokenizer::new();
        let tokens = tok.tokenize("hello world");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["hello", "world"]);
    }

    #[test]
    fn test_min_part_length() {
        let tok = HyphenatedTokenizer::new().with_min_part_length(3);
        let tokens = tok.tokenize("state-of-the-art");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"state"));
        assert!(terms.contains(&"the"));
        assert!(terms.contains(&"art"));
        assert!(!terms.contains(&"of")); // too short
    }

    #[test]
    fn test_no_preserve() {
        let tok = HyphenatedTokenizer::new().with_preserve_original(false);
        let tokens = tok.tokenize("well-known");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(!terms.contains(&"well-known"));
        assert!(terms.contains(&"well"));
        assert!(terms.contains(&"known"));
    }

    #[test]
    fn test_min_part_length_counts_chars_not_bytes() {
        // CJK chars are 3 bytes each. With min_part_length=2 we want to keep
        // 2-character parts ("中文") even though their byte length is 6.
        let tok = HyphenatedTokenizer::new().with_min_part_length(2);
        let tokens = tok.tokenize("中文-a-中文");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        // "a" (1 char) filtered out, both "中文" (2 chars) kept.
        assert!(terms.contains(&"中文"));
        assert!(!terms.contains(&"a"));
    }
}
