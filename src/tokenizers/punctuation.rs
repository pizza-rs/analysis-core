use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Splits text on punctuation boundaries while preserving numbers with dots/commas.
///
/// - `"hello,world"` → `"hello"`, `"world"`
/// - `"price: $3.99"` → `"price"`, `"3.99"` (number preserved)
/// - `"Dr. Smith"` → `"Dr"`, `"Smith"` (abbreviation dot splits)
///
/// Useful when you need finer-grained splitting than whitespace but want
/// to keep numeric values intact.
#[derive(Clone, Debug)]
pub struct PunctuationTokenizer {
    /// Keep numbers with decimal points intact
    preserve_numbers: bool,
}

impl PunctuationTokenizer {
    pub fn new() -> Self {
        Self {
            preserve_numbers: true,
        }
    }

    pub fn with_preserve_numbers(mut self, v: bool) -> Self {
        self.preserve_numbers = v;
        self
    }

    fn is_number_char(c: char) -> bool {
        c.is_ascii_digit() || c == '.' || c == ','
    }

    fn is_valid_number(s: &str) -> bool {
        // Accept integers, decimals, and grouped numbers like "1,234.56".
        //
        // Rules:
        //   * must contain at least one digit
        //   * at most one '.' (decimal point)
        //   * ',' allowed only when followed by exactly three digits (thousands grouping)
        //   * '.' must be preceded and followed by at least one digit
        //   * may not start or end with '.' or ','
        let bytes = s.as_bytes();
        if bytes.is_empty() {
            return false;
        }
        let first = bytes[0];
        let last = bytes[bytes.len() - 1];
        if first == b'.' || first == b',' || last == b'.' || last == b',' {
            return false;
        }

        let mut has_digit = false;
        let mut dot_seen = false;
        let mut i = 0;
        while i < bytes.len() {
            let b = bytes[i];
            if b.is_ascii_digit() {
                has_digit = true;
                i += 1;
            } else if b == b'.' {
                if dot_seen {
                    return false; // multiple decimal points
                }
                dot_seen = true;
                i += 1;
            } else if b == b',' {
                // Thousands separator: must be followed by exactly three digits.
                // Grouping is not allowed after a decimal point.
                if dot_seen {
                    return false;
                }
                for j in 1..=3 {
                    if i + j >= bytes.len() || !bytes[i + j].is_ascii_digit() {
                        return false;
                    }
                }
                // The next char after the 3 digits, if any, must NOT be a digit
                // (otherwise the grouping is wrong, e.g. "1,2345").
                if i + 4 < bytes.len() && bytes[i + 4].is_ascii_digit() {
                    return false;
                }
                i += 4; // skip ',' + 3 digits
            } else {
                return false;
            }
        }
        has_digit
    }
}

impl Default for PunctuationTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for PunctuationTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut position = 0u32;
        let mut i = 0;

        while i < text.len() {
            let c = text[i..].chars().next().unwrap();
            let c_len = c.len_utf8();

            // Skip whitespace
            if c.is_whitespace() {
                i += c_len;
                continue;
            }

            // Start of a token
            let start = i;

            // Number handling
            if self.preserve_numbers && c.is_ascii_digit() {
                while i < text.len() {
                    let nc = text[i..].chars().next().unwrap();
                    if Self::is_number_char(nc) {
                        i += nc.len_utf8();
                    } else {
                        break;
                    }
                }
                // Trim trailing dots/commas
                while i > start && matches!(text.as_bytes()[i - 1], b'.' | b',') {
                    i -= 1;
                }
                let candidate = &text[start..i];
                if Self::is_valid_number(candidate) {
                    tokens.push(Token {
                        term: Cow::Borrowed(candidate),
                        start_offset: start as u32,
                        end_offset: i as u32,
                        position,
                    });
                    position += 1;
                    continue;
                }
                // Reset and fall through
                i = start;
            }

            // Alphanumeric run
            if c.is_alphanumeric() {
                while i < text.len() {
                    let nc = text[i..].chars().next().unwrap();
                    if nc.is_alphanumeric() {
                        i += nc.len_utf8();
                    } else {
                        break;
                    }
                }
                tokens.push(Token {
                    term: Cow::Borrowed(&text[start..i]),
                    start_offset: start as u32,
                    end_offset: i as u32,
                    position,
                });
                position += 1;
                continue;
            }

            // Skip punctuation
            i += c_len;
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comma_separated() {
        let tok = PunctuationTokenizer::new();
        let tokens = tok.tokenize("hello,world");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["hello", "world"]);
    }

    #[test]
    fn test_preserve_numbers() {
        let tok = PunctuationTokenizer::new();
        let tokens = tok.tokenize("price 3.99 items");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["price", "3.99", "items"]);
    }

    #[test]
    fn test_comma_number() {
        let tok = PunctuationTokenizer::new();
        let tokens = tok.tokenize("total: 1,234,567");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"1,234,567"));
        assert!(terms.contains(&"total"));
    }

    #[test]
    fn test_no_preserve_numbers() {
        let tok = PunctuationTokenizer::new().with_preserve_numbers(false);
        let tokens = tok.tokenize("3.99");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["3", "99"]);
    }

    #[test]
    fn test_mixed_punctuation() {
        let tok = PunctuationTokenizer::new();
        let tokens = tok.tokenize("hello! world? foo-bar");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["hello", "world", "foo", "bar"]);
    }

    #[test]
    fn test_rejects_malformed_numbers() {
        // "1.2.3" must NOT be classified as a number (multiple dots).
        let tok = PunctuationTokenizer::new();
        let tokens = tok.tokenize("v1.2.3 release");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        // Should be split into individual digit runs, not joined.
        assert!(!terms.contains(&"1.2.3"));
    }

    #[test]
    fn test_rejects_malformed_grouped_numbers() {
        // "1,23" is malformed grouping (only 2 digits after comma).
        let tok = PunctuationTokenizer::new();
        let tokens = tok.tokenize("price 1,23 only");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(!terms.contains(&"1,23"));
    }
}
