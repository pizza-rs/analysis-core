use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Log line tokenizer for structured log parsing.
///
/// Handles common log formats:
/// - Syslog: `<timestamp> <host> <process>[pid]: message`
/// - JSON logs: extracts values
/// - Key=value pairs: `key1=val1 key2=val2`
/// - Standard formats with timestamps, levels, etc.
///
/// Emits meaningful tokens: timestamp, level, message words, key-value pairs.
#[derive(Clone, Debug)]
pub struct LogTokenizer {
    /// Extract key=value pairs as tokens
    extract_kv_pairs: bool,
    /// Include log level as a token
    include_level: bool,
}

impl LogTokenizer {
    pub fn new() -> Self {
        Self {
            extract_kv_pairs: true,
            include_level: true,
        }
    }

    pub fn with_extract_kv_pairs(mut self, v: bool) -> Self {
        self.extract_kv_pairs = v;
        self
    }

    fn is_log_level(word: &str) -> bool {
        matches!(
            word.to_uppercase().as_str(),
            "TRACE" | "DEBUG" | "INFO" | "WARN" | "WARNING" | "ERROR" | "FATAL" | "CRITICAL"
        )
    }

    fn is_kv_pair(word: &str) -> bool {
        // Contains = and has content on both sides
        if let Some(eq_pos) = word.find('=') {
            eq_pos > 0 && eq_pos < word.len() - 1
        } else {
            false
        }
    }
}

impl Default for LogTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for LogTokenizer {
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

            // Quoted strings
            if c == '"' || c == '\'' {
                let quote = c;
                let start = i;
                i += c_len;
                while i < text.len() {
                    let nc = text[i..].chars().next().unwrap();
                    i += nc.len_utf8();
                    if nc == quote {
                        break;
                    }
                    if nc == '\\' && i < text.len() {
                        let esc = text[i..].chars().next().unwrap();
                        i += esc.len_utf8();
                    }
                }
                // Emit content without quotes
                let content_start = start + c_len;
                let content_end = if i > start + c_len && text.as_bytes()[i - 1] == quote as u8 {
                    i - 1
                } else {
                    i
                };
                if content_start < content_end {
                    tokens.push(Token {
                        term: Cow::Borrowed(&text[content_start..content_end]),
                        start_offset: content_start as u32,
                        end_offset: content_end as u32,
                        position,
                    });
                    position += 1;
                }
                continue;
            }

            // Bracketed content like [INFO] or [2024-01-01]
            if c == '[' {
                let start = i;
                i += c_len;
                while i < text.len() {
                    let nc = text[i..].chars().next().unwrap();
                    i += nc.len_utf8();
                    if nc == ']' {
                        break;
                    }
                }
                let content_start = start + 1;
                let content_end = if i > start + 1 && text.as_bytes()[i - 1] == b']' {
                    i - 1
                } else {
                    i
                };
                if content_start < content_end {
                    let content = &text[content_start..content_end];
                    // Check if it's a log level
                    if self.include_level && Self::is_log_level(content) {
                        tokens.push(Token {
                            term: Cow::Borrowed(content),
                            start_offset: content_start as u32,
                            end_offset: content_end as u32,
                            position,
                        });
                        position += 1;
                    } else {
                        tokens.push(Token {
                            term: Cow::Borrowed(content),
                            start_offset: content_start as u32,
                            end_offset: content_end as u32,
                            position,
                        });
                        position += 1;
                    }
                }
                continue;
            }

            // Regular word/token
            let start = i;
            while i < text.len() {
                let nc = text[i..].chars().next().unwrap();
                if nc.is_whitespace() || nc == '[' || nc == '"' {
                    break;
                }
                i += nc.len_utf8();
            }

            let word = &text[start..i];
            if word.is_empty() {
                continue;
            }

            // Key=value handling
            if self.extract_kv_pairs && Self::is_kv_pair(word) {
                // Emit the full kv pair
                tokens.push(Token {
                    term: Cow::Borrowed(word),
                    start_offset: start as u32,
                    end_offset: i as u32,
                    position,
                });
                position += 1;

                // Also emit key and value separately
                if let Some(eq_pos) = word.find('=') {
                    let key = &word[..eq_pos];
                    let val = &word[eq_pos + 1..];
                    let key_offset = start;
                    let val_offset = start + eq_pos + 1;
                    if !key.is_empty() {
                        tokens.push(Token {
                            term: Cow::Borrowed(key),
                            start_offset: key_offset as u32,
                            end_offset: (key_offset + key.len()) as u32,
                            position,
                        });
                    }
                    if !val.is_empty() {
                        tokens.push(Token {
                            term: Cow::Borrowed(val),
                            start_offset: val_offset as u32,
                            end_offset: (val_offset + val.len()) as u32,
                            position,
                        });
                    }
                }
            } else {
                // Check if it's a standalone log level
                if self.include_level && Self::is_log_level(word) {
                    tokens.push(Token {
                        term: Cow::Borrowed(word),
                        start_offset: start as u32,
                        end_offset: i as u32,
                        position,
                    });
                    position += 1;
                } else {
                    tokens.push(Token {
                        term: Cow::Borrowed(word),
                        start_offset: start as u32,
                        end_offset: i as u32,
                        position,
                    });
                    position += 1;
                }
            }
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bracketed_level() {
        let tok = LogTokenizer::new();
        let tokens = tok.tokenize("[INFO] Server started on port 8080");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"INFO"));
        assert!(terms.contains(&"Server"));
        assert!(terms.contains(&"8080"));
    }

    #[test]
    fn test_kv_pairs() {
        let tok = LogTokenizer::new();
        let tokens = tok.tokenize("status=200 method=GET path=/api/users");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"status=200"));
        assert!(terms.contains(&"status"));
        assert!(terms.contains(&"200"));
        assert!(terms.contains(&"method=GET"));
        assert!(terms.contains(&"GET"));
    }

    #[test]
    fn test_quoted_string() {
        let tok = LogTokenizer::new();
        let tokens = tok.tokenize("msg=\"hello world\"");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"hello world"));
    }

    #[test]
    fn test_standard_log_line() {
        let tok = LogTokenizer::new();
        let tokens = tok.tokenize("2024-01-15 ERROR Database connection failed");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"ERROR"));
        assert!(terms.contains(&"Database"));
        assert!(terms.contains(&"connection"));
    }
}
