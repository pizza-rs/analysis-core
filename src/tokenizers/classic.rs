use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Classic tokenizer that uses English grammar rules to produce tokens.
///
/// It recognizes:
/// - Email addresses as single tokens
/// - Internet hostnames as single tokens
/// - Acronyms (U.S.A.) as single tokens
/// - Company names with apostrophes (O'Reilly) as single tokens
/// - Regular words split at non-alphanumeric boundaries
///
/// This is a legacy tokenizer; for new applications prefer the standard or
/// UaxUrlEmail tokenizer.
#[derive(Clone, Debug)]
pub struct ClassicTokenizer {
    max_token_length: usize,
}

impl ClassicTokenizer {
    pub fn new() -> Self {
        Self {
            max_token_length: 255,
        }
    }

    pub fn with_max_token_length(mut self, max: usize) -> Self {
        self.max_token_length = max;
        self
    }
}

impl Default for ClassicTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for ClassicTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens: Vec<Token<'a>> = Vec::new();
        let mut position: u32 = 0;
        // Pre-compute (byte_offset, char) pairs so byte↔char index lookups
        // are O(1). Using `text.chars().collect()` + `char_indices().nth(idx)`
        // per token would be O(n) per emit, making the whole tokenize call
        // quadratic on long inputs.
        let char_indices: Vec<(usize, char)> = text.char_indices().collect();
        let chars: Vec<char> = char_indices.iter().map(|(_, c)| *c).collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            // Skip non-token chars
            if !is_token_char(chars[i]) {
                i += 1;
                continue;
            }

            let start = i;

            // Try to match special patterns
            // 1. Email: contains @ in the middle
            // 2. Acronym: X.X.X. pattern
            // 3. Company with apostrophe: O'Reilly
            // 4. Numbers with separators: 3.14, 1,000

            while i < len && is_token_char_extended(chars[i], i > start, &chars, i) {
                i += 1;
            }

            if i > start {
                let byte_start = char_indices[start].0;
                let byte_end = if i < len {
                    char_indices[i].0
                } else {
                    text.len()
                };
                let term = &text[byte_start..byte_end];

                // Strip trailing dots from acronyms
                let term = term.trim_end_matches('.');

                if !term.is_empty() && term.len() <= self.max_token_length {
                    tokens.push(Token {
                        term: Cow::Borrowed(term),
                        start_offset: byte_start as u32,
                        end_offset: byte_end as u32,
                        position,
                    });
                    position += 1;
                }
            }
        }

        tokens
    }
}

#[inline]
fn is_token_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

#[inline]
fn is_token_char_extended(c: char, in_token: bool, chars: &[char], pos: usize) -> bool {
    if c.is_alphanumeric() || c == '_' {
        return true;
    }
    if !in_token {
        return false;
    }

    // Allow dots in acronyms (U.S.A) — dot followed by an alpha
    if c == '.' && pos + 1 < chars.len() && chars[pos + 1].is_alphabetic() {
        return true;
    }

    // Allow apostrophes mid-word (O'Reilly, don't)
    if (c == '\'' || c == '\u{2019}') && pos + 1 < chars.len() && chars[pos + 1].is_alphabetic() {
        return true;
    }

    // Allow @ in emails
    if c == '@' && pos + 1 < chars.len() && chars[pos + 1].is_alphabetic() {
        return true;
    }

    // Allow hyphens mid-word
    if c == '-' && pos + 1 < chars.len() && chars[pos + 1].is_alphanumeric() {
        return true;
    }

    // Allow dots and commas in numbers
    if (c == '.' || c == ',') && pos + 1 < chars.len() && chars[pos + 1].is_ascii_digit() {
        // Check that previous char is also a digit
        if pos > 0 && chars[pos - 1].is_ascii_digit() {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_words() {
        let tokenizer = ClassicTokenizer::new();
        let tokens = tokenizer.tokenize("Hello World");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["Hello", "World"]);
    }

    #[test]
    fn test_apostrophe() {
        let tokenizer = ClassicTokenizer::new();
        let tokens = tokenizer.tokenize("O'Reilly");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["O'Reilly"]);
    }

    #[test]
    fn test_acronym() {
        let tokenizer = ClassicTokenizer::new();
        let tokens = tokenizer.tokenize("U.S.A.");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["U.S.A"]);
    }

    #[test]
    fn test_number() {
        let tokenizer = ClassicTokenizer::new();
        let tokens = tokenizer.tokenize("price is 3.14");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"3.14"));
        assert!(terms.contains(&"price"));
    }
}
