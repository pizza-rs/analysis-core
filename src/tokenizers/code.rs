use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Tokenizer for source code: splits identifiers, operators, strings, and numbers.
///
/// Handles:
/// - Identifiers (variables, function names)
/// - Numbers (integer and float literals, hex `0x...`)
/// - String literals (content between quotes)
/// - Operators collapsed as single tokens
/// - Preserves dot-separated identifiers (e.g., `obj.method`)
#[derive(Clone, Debug)]
pub struct CodeTokenizer {
    /// Whether to split dot-notation (e.g., "obj.method" → ["obj", "method"])
    split_dots: bool,
    /// Whether to include operator tokens in output
    include_operators: bool,
}

impl CodeTokenizer {
    pub fn new() -> Self {
        Self {
            split_dots: true,
            include_operators: false,
        }
    }

    pub fn with_split_dots(mut self, split: bool) -> Self {
        self.split_dots = split;
        self
    }

    pub fn with_operators(mut self, include: bool) -> Self {
        self.include_operators = include;
        self
    }

    fn is_ident_char(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }

    fn is_operator_char(c: char) -> bool {
        matches!(
            c,
            '+' | '-' | '*' | '/' | '%' | '=' | '!' | '<' | '>' | '&' | '|' | '^' | '~'
        )
    }
}

impl Default for CodeTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for CodeTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut position = 0u32;
        let bytes = text.as_bytes();
        let mut i = 0;

        while i < text.len() {
            let c = text[i..].chars().next().unwrap();

            if c.is_whitespace() || matches!(c, '(' | ')' | '{' | '}' | '[' | ']' | ',' | ';') {
                i += c.len_utf8();
                continue;
            }

            // String literals
            if c == '"' || c == '\'' || c == '`' {
                let quote = c;
                let start = i;
                i += c.len_utf8();
                let mut closed = false;
                while i < text.len() {
                    let nc = text[i..].chars().next().unwrap();
                    if nc == '\\' {
                        i += nc.len_utf8();
                        if i < text.len() {
                            i += text[i..].chars().next().unwrap().len_utf8();
                        }
                    } else if nc == quote {
                        i += nc.len_utf8();
                        closed = true;
                        break;
                    } else {
                        i += nc.len_utf8();
                    }
                }
                // Extract content without quotes. Only strip the trailing
                // closing quote if it was actually consumed — otherwise the
                // last character of an unterminated string would be lost.
                let content_start = start + quote.len_utf8();
                let content_end = if closed {
                    i - quote.len_utf8()
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

            // Operators
            if Self::is_operator_char(c) {
                let start = i;
                while i < text.len() {
                    let nc = text[i..].chars().next().unwrap();
                    if Self::is_operator_char(nc) {
                        i += nc.len_utf8();
                    } else {
                        break;
                    }
                }
                if self.include_operators {
                    tokens.push(Token {
                        term: Cow::Borrowed(&text[start..i]),
                        start_offset: start as u32,
                        end_offset: i as u32,
                        position,
                    });
                    position += 1;
                }
                continue;
            }

            // Numbers (including hex 0x...)
            if c.is_ascii_digit() {
                let start = i;
                if c == '0' && i + 1 < text.len() && (bytes[i + 1] == b'x' || bytes[i + 1] == b'X')
                {
                    i += 2;
                    while i < text.len() && text[i..].chars().next().unwrap().is_ascii_hexdigit() {
                        i += 1;
                    }
                } else {
                    while i < text.len() {
                        let nc = text[i..].chars().next().unwrap();
                        if nc.is_ascii_digit() || nc == '.' || nc == '_' {
                            i += nc.len_utf8();
                        } else {
                            break;
                        }
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

            // Identifiers (with optional dot-notation)
            if Self::is_ident_char(c) || c == '.' && !self.split_dots {
                let start = i;
                while i < text.len() {
                    let nc = text[i..].chars().next().unwrap();
                    if Self::is_ident_char(nc) || (!self.split_dots && nc == '.') {
                        i += nc.len_utf8();
                    } else {
                        break;
                    }
                }
                // When `split_dots == true` the loop above stops at every '.',
                // so the collected `ident` never contains a dot — the outer
                // loop's fall-through then skips the dot and the next ident
                // is collected separately. So we never need a dot-split branch
                // here.
                let ident = &text[start..i];
                tokens.push(Token {
                    term: Cow::Borrowed(ident),
                    start_offset: start as u32,
                    end_offset: i as u32,
                    position,
                });
                position += 1;
                continue;
            }

            // Skip other characters (punctuation like #, @, etc.)
            i += c.len_utf8();
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_code() {
        let tok = CodeTokenizer::new();
        let tokens = tok.tokenize("let x = foo.bar(42)");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["let", "x", "foo", "bar", "42"]);
    }

    #[test]
    fn test_string_literal() {
        let tok = CodeTokenizer::new();
        let tokens = tok.tokenize(r#"println("hello world")"#);
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"hello world"));
        assert!(terms.contains(&"println"));
    }

    #[test]
    fn test_hex_numbers() {
        let tok = CodeTokenizer::new();
        let tokens = tok.tokenize("color = 0xFF00AA");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"0xFF00AA"));
    }

    #[test]
    fn test_no_split_dots() {
        let tok = CodeTokenizer::new().with_split_dots(false);
        let tokens = tok.tokenize("obj.method.call");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["obj.method.call"]);
    }

    #[test]
    fn test_with_operators() {
        let tok = CodeTokenizer::new().with_operators(true);
        let tokens = tok.tokenize("a += b");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"+="));
    }

    #[test]
    fn test_unterminated_string_keeps_last_char() {
        // Without a closing quote, the previous implementation chopped off
        // the final character. Verify the full content survives.
        let tok = CodeTokenizer::new();
        let tokens = tok.tokenize(r#"x = "hello"#);
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"hello"), "got {:?}", terms);
    }
}
