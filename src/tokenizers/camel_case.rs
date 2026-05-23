use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Splits camelCase and PascalCase identifiers into individual words.
///
/// Also handles:
/// - ALLCAPS followed by lowercase (e.g., "XMLParser" → "XML", "Parser")
/// - Snake_case (splits on `_`)
/// - Numbers as separate tokens (e.g., "log4j" → "log", "4", "j")
#[derive(Clone, Debug)]
pub struct CamelCaseTokenizer {
    /// Also emit the original unsplit token
    preserve_original: bool,
}

impl CamelCaseTokenizer {
    pub fn new() -> Self {
        Self {
            preserve_original: false,
        }
    }

    pub fn with_preserve_original(mut self, preserve: bool) -> Self {
        self.preserve_original = preserve;
        self
    }

    fn split_identifier(text: &str) -> Vec<(usize, usize)> {
        let mut parts = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        if chars.is_empty() {
            return parts;
        }

        let mut start = 0;
        let mut byte_pos = 0;
        let mut byte_positions: Vec<usize> = Vec::with_capacity(chars.len() + 1);

        for c in &chars {
            byte_positions.push(byte_pos);
            byte_pos += c.len_utf8();
        }
        byte_positions.push(byte_pos); // end position

        let mut i = 1;
        while i < chars.len() {
            let prev = chars[i - 1];
            let curr = chars[i];
            let split = if curr == '_' || prev == '_' {
                // Split on underscores
                true
            } else if prev.is_lowercase() && curr.is_uppercase() {
                // camelCase boundary: aB
                true
            } else if prev.is_alphabetic() && curr.is_ascii_digit() {
                // letter→digit: log4
                true
            } else if prev.is_ascii_digit() && curr.is_alphabetic() {
                // digit→letter: 4j
                true
            } else if i + 1 < chars.len()
                && prev.is_uppercase()
                && curr.is_uppercase()
                && chars[i + 1].is_lowercase()
            {
                // ALLCAPS→lowercase: XMLParser → XML|Parser
                true
            } else {
                false
            };

            if split {
                if start < i && chars[start] != '_' {
                    parts.push((byte_positions[start], byte_positions[i]));
                }
                // Skip underscores
                if curr == '_' {
                    i += 1;
                    while i < chars.len() && chars[i] == '_' {
                        i += 1;
                    }
                    start = i;
                } else {
                    start = i;
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        // Last part
        if start < chars.len() && chars[start] != '_' {
            parts.push((byte_positions[start], byte_positions[chars.len()]));
        }

        parts
    }
}

impl Default for CamelCaseTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for CamelCaseTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut position = 0u32;

        // First split on whitespace to get individual identifiers
        let mut last_end = 0;
        for (i, c) in text.char_indices() {
            if c.is_whitespace() {
                if last_end < i {
                    let ident = &text[last_end..i];
                    self.emit_identifier(ident, last_end, &mut tokens, &mut position);
                }
                last_end = i + c.len_utf8();
            }
        }
        if last_end < text.len() {
            let ident = &text[last_end..];
            self.emit_identifier(ident, last_end, &mut tokens, &mut position);
        }

        tokens
    }
}

impl CamelCaseTokenizer {
    fn emit_identifier<'a>(
        &self,
        ident: &'a str,
        base_offset: usize,
        tokens: &mut Vec<Token<'a>>,
        position: &mut u32,
    ) {
        let parts = Self::split_identifier(ident);

        if self.preserve_original && parts.len() > 1 {
            tokens.push(Token {
                term: Cow::Borrowed(ident),
                start_offset: base_offset as u32,
                end_offset: (base_offset + ident.len()) as u32,
                position: *position,
            });
            // Don't increment position — sub-parts share position with original
        }

        if parts.len() <= 1 {
            // No split needed — emit as-is
            tokens.push(Token {
                term: Cow::Borrowed(ident),
                start_offset: base_offset as u32,
                end_offset: (base_offset + ident.len()) as u32,
                position: *position,
            });
            *position += 1;
        } else {
            for (s, e) in parts {
                tokens.push(Token {
                    term: Cow::Borrowed(&ident[s..e]),
                    start_offset: (base_offset + s) as u32,
                    end_offset: (base_offset + e) as u32,
                    position: *position,
                });
                *position += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camel_case() {
        let tok = CamelCaseTokenizer::new();
        let tokens = tok.tokenize("camelCase");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["camel", "Case"]);
    }

    #[test]
    fn test_pascal_case() {
        let tok = CamelCaseTokenizer::new();
        let tokens = tok.tokenize("PascalCase");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["Pascal", "Case"]);
    }

    #[test]
    fn test_all_caps_transition() {
        let tok = CamelCaseTokenizer::new();
        let tokens = tok.tokenize("XMLParser");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["XML", "Parser"]);
    }

    #[test]
    fn test_snake_case() {
        let tok = CamelCaseTokenizer::new();
        let tokens = tok.tokenize("hello_world_test");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["hello", "world", "test"]);
    }

    #[test]
    fn test_number_boundary() {
        let tok = CamelCaseTokenizer::new();
        let tokens = tok.tokenize("log4j");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["log", "4", "j"]);
    }

    #[test]
    fn test_preserve_original() {
        let tok = CamelCaseTokenizer::new().with_preserve_original(true);
        let tokens = tok.tokenize("camelCase");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["camelCase", "camel", "Case"]);
    }
}
