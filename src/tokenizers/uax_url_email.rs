use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;
use regex::Regex;

/// URL and email-aware tokenizer that treats URLs and email addresses
/// as single tokens, while applying standard word-break rules to other text.
///
/// Without this tokenizer, URLs like `https://example.com/path` would be
/// split into multiple tokens. This tokenizer keeps them intact.
#[derive(Clone, Debug)]
pub struct UaxUrlEmailTokenizer {
    max_token_length: usize,
    url_pattern: Regex,
    email_pattern: Regex,
}

impl UaxUrlEmailTokenizer {
    pub fn new() -> Self {
        Self {
            max_token_length: 255,
            url_pattern: Regex::new(r"(?i)(?:https?|ftp|file)://[^\s<>\[\]{}|\\^`\x00-\x1f\x7f]+")
                .unwrap(),
            email_pattern: Regex::new(r"[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}").unwrap(),
        }
    }

    pub fn with_max_token_length(mut self, max: usize) -> Self {
        self.max_token_length = max;
        self
    }
}

impl Default for UaxUrlEmailTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for UaxUrlEmailTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens: Vec<Token<'a>> = Vec::new();
        let mut position: u32 = 0;

        // Find all URLs and emails first
        let mut special_ranges: Vec<(usize, usize)> = Vec::new();

        for mat in self.url_pattern.find_iter(text) {
            special_ranges.push((mat.start(), mat.end()));
        }
        for mat in self.email_pattern.find_iter(text) {
            // Don't add if it overlaps with a URL match
            let start = mat.start();
            let end = mat.end();
            let overlaps = special_ranges.iter().any(|(s, e)| start < *e && end > *s);
            if !overlaps {
                special_ranges.push((start, end));
            }
        }

        special_ranges.sort_by_key(|r| r.0);

        let mut last_end = 0;

        for (start, end) in &special_ranges {
            // Tokenize text before the special token
            if *start > last_end {
                let before = &text[last_end..*start];
                tokenize_standard(
                    before,
                    last_end,
                    &mut position,
                    self.max_token_length,
                    &mut tokens,
                );
            }

            // Emit the URL/email as a single token
            let term = &text[*start..*end];
            if term.len() <= self.max_token_length {
                tokens.push(Token {
                    term: Cow::Borrowed(term),
                    start_offset: *start as u32,
                    end_offset: *end as u32,
                    position,
                });
                position += 1;
            }
            last_end = *end;
        }

        // Tokenize remaining text
        if last_end < text.len() {
            let remaining = &text[last_end..];
            tokenize_standard(
                remaining,
                last_end,
                &mut position,
                self.max_token_length,
                &mut tokens,
            );
        }

        tokens
    }
}

/// Standard word-break tokenization for text between URLs/emails.
fn tokenize_standard<'a>(
    text: &'a str,
    base_offset: usize,
    position: &mut u32,
    max_len: usize,
    tokens: &mut Vec<Token<'a>>,
) {
    let mut start = None;

    for (i, c) in text.char_indices() {
        if c.is_alphanumeric() {
            if start.is_none() {
                start = Some(i);
            }
        } else if let Some(s) = start {
            let term = &text[s..i];
            if term.len() <= max_len {
                tokens.push(Token {
                    term: Cow::Borrowed(term),
                    start_offset: (base_offset + s) as u32,
                    end_offset: (base_offset + i) as u32,
                    position: *position,
                });
                *position += 1;
            }
            start = None;
        }
    }

    // Handle last token
    if let Some(s) = start {
        let term = &text[s..];
        if term.len() <= max_len {
            tokens.push(Token {
                term: Cow::Borrowed(term),
                start_offset: (base_offset + s) as u32,
                end_offset: (base_offset + text.len()) as u32,
                position: *position,
            });
            *position += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_preserved() {
        let tokenizer = UaxUrlEmailTokenizer::new();
        let tokens = tokenizer.tokenize("visit https://example.com/path today");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"https://example.com/path"));
        assert!(terms.contains(&"visit"));
        assert!(terms.contains(&"today"));
    }

    #[test]
    fn test_email_preserved() {
        let tokenizer = UaxUrlEmailTokenizer::new();
        let tokens = tokenizer.tokenize("email user@example.com please");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"user@example.com"));
        assert!(terms.contains(&"email"));
        assert!(terms.contains(&"please"));
    }

    #[test]
    fn test_mixed_content() {
        let tokenizer = UaxUrlEmailTokenizer::new();
        let tokens = tokenizer.tokenize("Check http://foo.bar and admin@test.org now");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"http://foo.bar"));
        assert!(terms.contains(&"admin@test.org"));
        assert!(terms.contains(&"Check"));
        assert!(terms.contains(&"now"));
    }

    #[test]
    fn test_no_special_tokens() {
        let tokenizer = UaxUrlEmailTokenizer::new();
        let tokens = tokenizer.tokenize("hello world");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["hello", "world"]);
    }
}
