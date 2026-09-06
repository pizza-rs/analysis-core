use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Extracts string values from JSON text as tokens.
///
/// Parses simple JSON and emits string field values as tokens.
/// Optionally includes field names as tokens too.
///
/// Example: `{"name":"John","city":"NYC"}` → `"John"`, `"NYC"`
/// With include_keys: → `"name"`, `"John"`, `"city"`, `"NYC"`
#[derive(Clone, Debug)]
pub struct JsonFieldTokenizer {
    /// Include JSON keys as tokens
    include_keys: bool,
    /// Include numeric values
    include_numbers: bool,
}

impl JsonFieldTokenizer {
    pub fn new() -> Self {
        Self {
            include_keys: false,
            include_numbers: false,
        }
    }

    pub fn with_include_keys(mut self, v: bool) -> Self {
        self.include_keys = v;
        self
    }

    pub fn with_include_numbers(mut self, v: bool) -> Self {
        self.include_numbers = v;
        self
    }

    /// Lightweight JSON string/number extractor.
    ///
    /// Walks the input maintaining a tiny stack of contexts (`Object` vs
    /// `Array`) and a flag for whether the next string should be treated as
    /// a key. Inside objects, the string that appears before `:` is a key and
    /// the one after is a value. Inside arrays, every string is a value.
    ///
    /// This avoids the previous bug where `[` blindly set `is_key = true`,
    /// causing array elements like `["a","b"]` to be classified as keys.
    fn extract_strings<'a>(&self, text: &'a str) -> Vec<(&'a str, usize, bool)> {
        #[derive(Copy, Clone)]
        enum Ctx {
            Object { expect_key: bool },
            Array,
        }

        let mut results = Vec::new();
        let bytes = text.as_bytes();
        let mut stack: Vec<Ctx> = Vec::new();
        let mut i = 0;

        let current_is_key = |stack: &Vec<Ctx>| -> bool {
            match stack.last() {
                Some(Ctx::Object { expect_key }) => *expect_key,
                Some(Ctx::Array) => false,
                None => false,
            }
        };

        while i < bytes.len() {
            let b = bytes[i];
            match b {
                b'"' => {
                    let start = i + 1;
                    i += 1;
                    while i < bytes.len() {
                        match bytes[i] {
                            b'\\' => {
                                // Skip the escape introducer plus its argument.
                                // `\uXXXX` needs to skip 5 more bytes (4 hex + 1 already accounted).
                                if i + 1 < bytes.len() && bytes[i + 1] == b'u' {
                                    i += core::cmp::min(6, bytes.len() - i);
                                } else {
                                    i += core::cmp::min(2, bytes.len() - i);
                                }
                            }
                            b'"' => break,
                            _ => i += 1,
                        }
                    }
                    if i < bytes.len() {
                        let s = &text[start..i];
                        let is_key = current_is_key(&stack);
                        results.push((s, start, is_key));
                        i += 1;
                        // Toggle key/value expectation within objects.
                        if let Some(Ctx::Object { expect_key }) = stack.last_mut() {
                            *expect_key = !*expect_key;
                        }
                    }
                }
                b'{' => {
                    stack.push(Ctx::Object { expect_key: true });
                    i += 1;
                }
                b'[' => {
                    stack.push(Ctx::Array);
                    i += 1;
                }
                b'}' | b']' => {
                    stack.pop();
                    i += 1;
                }
                b':' => {
                    if let Some(Ctx::Object { expect_key }) = stack.last_mut() {
                        *expect_key = false;
                    }
                    i += 1;
                }
                b',' => {
                    if let Some(Ctx::Object { expect_key }) = stack.last_mut() {
                        *expect_key = true;
                    }
                    i += 1;
                }
                _ if self.include_numbers
                    && (b.is_ascii_digit() || b == b'-')
                    && !current_is_key(&stack) =>
                {
                    let start = i;
                    i += 1;
                    while i < bytes.len()
                        && (bytes[i].is_ascii_digit()
                            || bytes[i] == b'.'
                            || bytes[i] == b'e'
                            || bytes[i] == b'E'
                            || bytes[i] == b'+'
                            || bytes[i] == b'-')
                    {
                        i += 1;
                    }
                    let s = &text[start..i];
                    results.push((s, start, false));
                    // After a numeric value inside an object, next thing is a key.
                    if let Some(Ctx::Object { expect_key }) = stack.last_mut() {
                        *expect_key = true;
                    }
                }
                _ => i += 1,
            }
        }

        results
    }
}

impl Default for JsonFieldTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for JsonFieldTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut position = 0u32;

        let strings = self.extract_strings(text);

        for (s, offset, is_key) in strings {
            if s.is_empty() {
                continue;
            }
            if is_key && !self.include_keys {
                continue;
            }
            tokens.push(Token {
                term: Cow::Borrowed(s),
                start_offset: offset as u32,
                end_offset: (offset + s.len()) as u32,
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
    fn test_json_values_only() {
        let tok = JsonFieldTokenizer::new();
        let tokens = tok.tokenize(r#"{"name":"John","city":"NYC"}"#);
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["John", "NYC"]);
    }

    #[test]
    fn test_json_with_keys() {
        let tok = JsonFieldTokenizer::new().with_include_keys(true);
        let tokens = tok.tokenize(r#"{"name":"John"}"#);
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"name"));
        assert!(terms.contains(&"John"));
    }

    #[test]
    fn test_json_with_numbers() {
        let tok = JsonFieldTokenizer::new().with_include_numbers(true);
        let tokens = tok.tokenize(r#"{"age":42,"score":3.14}"#);
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"42"));
        assert!(terms.contains(&"3.14"));
    }

    #[test]
    fn test_empty_json() {
        let tok = JsonFieldTokenizer::new();
        let tokens = tok.tokenize("{}");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_escaped_quotes() {
        let tok = JsonFieldTokenizer::new();
        let tokens = tok.tokenize(r#"{"msg":"hello \"world\""}"#);
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"hello \\\"world\\\""));
    }

    #[test]
    fn test_array_elements_are_values() {
        // Array elements must NOT be classified as keys.
        let tok = JsonFieldTokenizer::new();
        let tokens = tok.tokenize(r#"{"tags":["red","green","blue"]}"#);
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"red"));
        assert!(terms.contains(&"green"));
        assert!(terms.contains(&"blue"));
        // "tags" is a key — should be excluded by default.
        assert!(!terms.contains(&"tags"));
    }

    #[test]
    fn test_nested_object() {
        let tok = JsonFieldTokenizer::new();
        let tokens = tok.tokenize(r#"{"user":{"name":"alice","roles":["admin"]},"active":"yes"}"#);
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"alice"));
        assert!(terms.contains(&"admin"));
        assert!(terms.contains(&"yes"));
        // Keys must not appear.
        assert!(!terms.contains(&"user"));
        assert!(!terms.contains(&"name"));
        assert!(!terms.contains(&"roles"));
    }

    #[test]
    fn test_unicode_escape_in_string() {
        // \uXXXX must not terminate string scanning prematurely.
        let tok = JsonFieldTokenizer::new();
        let tokens = tok.tokenize(r#"{"a":"x\u0022y","b":"z"}"#);
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"z"));
    }
}
