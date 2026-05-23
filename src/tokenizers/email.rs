use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Tokenizes email addresses into meaningful parts.
///
/// `john.doe@example.com` → `john.doe@example.com`, `john`, `doe`, `example`, `com`
///
/// Useful for email search where you want to match on local part, domain, or full address.
#[derive(Clone, Debug)]
pub struct EmailTokenizer {
    /// Emit the full email as a token
    preserve_original: bool,
    /// Split the local part on `.` and `+`
    split_local: bool,
    /// Split the domain on `.`
    split_domain: bool,
}

impl EmailTokenizer {
    pub fn new() -> Self {
        Self {
            preserve_original: true,
            split_local: true,
            split_domain: true,
        }
    }

    pub fn with_preserve_original(mut self, v: bool) -> Self {
        self.preserve_original = v;
        self
    }

    pub fn with_split_local(mut self, v: bool) -> Self {
        self.split_local = v;
        self
    }

    pub fn with_split_domain(mut self, v: bool) -> Self {
        self.split_domain = v;
        self
    }

    fn is_email_local_char(c: char) -> bool {
        c.is_alphanumeric() || matches!(c, '.' | '+' | '-' | '_' | '!' | '#' | '$' | '%' | '&' | '\'' | '*' | '/' | '=' | '?' | '^' | '`' | '{' | '|' | '}' | '~')
    }

    fn is_email_domain_char(c: char) -> bool {
        c.is_alphanumeric() || c == '.' || c == '-'
    }
}

impl Default for EmailTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for EmailTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut position = 0u32;
        let mut i = 0;

        while i < text.len() {
            let c = text[i..].chars().next().unwrap();

            if c.is_whitespace() {
                i += c.len_utf8();
                continue;
            }

            // Try to find an email address
            if Self::is_email_local_char(c) {
                let start = i;
                // Consume local part candidate
                while i < text.len() {
                    let nc = text[i..].chars().next().unwrap();
                    if Self::is_email_local_char(nc) {
                        i += nc.len_utf8();
                    } else {
                        break;
                    }
                }

                // Check for @
                if i < text.len() && text[i..].starts_with('@') {
                    let at_pos = i;
                    i += 1; // skip @
                    let domain_start = i;

                    // Consume domain
                    while i < text.len() {
                        let nc = text[i..].chars().next().unwrap();
                        if Self::is_email_domain_char(nc) {
                            i += nc.len_utf8();
                        } else {
                            break;
                        }
                    }

                    // Validate: domain must contain at least one dot
                    let domain = &text[domain_start..i];
                    if domain.contains('.') && domain.len() > 2 {
                        // It's an email!
                        let email = &text[start..i];

                        if self.preserve_original {
                            tokens.push(Token {
                                term: Cow::Borrowed(email),
                                start_offset: start as u32,
                                end_offset: i as u32,
                                position,
                            });
                        }

                        let local = &text[start..at_pos];
                        // Emit full local part
                        tokens.push(Token {
                            term: Cow::Borrowed(local),
                            start_offset: start as u32,
                            end_offset: at_pos as u32,
                            position,
                        });

                        // Split local part
                        if self.split_local {
                            for part in local.split(|c| c == '.' || c == '+') {
                                if !part.is_empty() {
                                    let part_offset = part.as_ptr() as usize - text.as_ptr() as usize;
                                    tokens.push(Token {
                                        term: Cow::Borrowed(part),
                                        start_offset: part_offset as u32,
                                        end_offset: (part_offset + part.len()) as u32,
                                        position,
                                    });
                                }
                            }
                        }

                        // Emit full domain
                        tokens.push(Token {
                            term: Cow::Borrowed(domain),
                            start_offset: domain_start as u32,
                            end_offset: i as u32,
                            position,
                        });

                        // Split domain
                        if self.split_domain {
                            for part in domain.split('.') {
                                if !part.is_empty() {
                                    let part_offset = part.as_ptr() as usize - text.as_ptr() as usize;
                                    tokens.push(Token {
                                        term: Cow::Borrowed(part),
                                        start_offset: part_offset as u32,
                                        end_offset: (part_offset + part.len()) as u32,
                                        position,
                                    });
                                }
                            }
                        }

                        position += 1;
                    } else {
                        // Not a valid email, emit as regular word
                        tokens.push(Token {
                            term: Cow::Borrowed(&text[start..i]),
                            start_offset: start as u32,
                            end_offset: i as u32,
                            position,
                        });
                        position += 1;
                    }
                } else {
                    // No @, just a regular word
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

            // Skip punctuation/other
            i += c.len_utf8();
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_decomposition() {
        let tok = EmailTokenizer::new();
        let tokens = tok.tokenize("john.doe@example.com");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"john.doe@example.com"));
        assert!(terms.contains(&"john.doe"));
        assert!(terms.contains(&"john"));
        assert!(terms.contains(&"doe"));
        assert!(terms.contains(&"example.com"));
        assert!(terms.contains(&"example"));
        assert!(terms.contains(&"com"));
    }

    #[test]
    fn test_email_plus_addressing() {
        let tok = EmailTokenizer::new();
        let tokens = tok.tokenize("user+tag@gmail.com");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"user+tag@gmail.com"));
        assert!(terms.contains(&"user"));
        assert!(terms.contains(&"tag"));
    }

    #[test]
    fn test_no_email() {
        let tok = EmailTokenizer::new();
        let tokens = tok.tokenize("hello world");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["hello", "world"]);
    }

    #[test]
    fn test_no_preserve() {
        let tok = EmailTokenizer::new().with_preserve_original(false);
        let tokens = tok.tokenize("a@b.com");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(!terms.contains(&"a@b.com"));
        assert!(terms.contains(&"a"));
    }
}
