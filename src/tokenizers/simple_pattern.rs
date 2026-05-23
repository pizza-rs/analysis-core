use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;
use regex::Regex;

/// Tokenizer that uses a regex pattern to match tokens.
///
/// Unlike `PatternTokenizer` which splits on the pattern, this tokenizer
/// emits text that matches the pattern as tokens. Only the portions of text
/// that match the pattern are emitted.
///
/// For example, with pattern `\w+`, text "hello, world!" produces ["hello", "world"].
#[derive(Clone, Debug)]
pub struct SimplePatternTokenizer {
    pattern: Regex,
}

impl SimplePatternTokenizer {
    pub fn new(pattern: &str) -> Option<Self> {
        Regex::new(pattern).ok().map(|re| Self { pattern: re })
    }
}

impl Tokenizer for SimplePatternTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut position: u32 = 0;

        for mat in self.pattern.find_iter(text) {
            let term = &text[mat.start()..mat.end()];
            if !term.is_empty() {
                tokens.push(Token {
                    term: Cow::Borrowed(term),
                    start_offset: mat.start() as u32,
                    end_offset: mat.end() as u32,
                    position,
                });
                position += 1;
            }
        }

        tokens
    }
}

/// Tokenizer that splits text at regex pattern matches.
///
/// Unlike `SimplePatternTokenizer` which emits matches as tokens, this
/// tokenizer uses the pattern as a delimiter and emits the text between matches.
///
/// For example, with pattern `\W+`, text "hello, world!" produces ["hello", "world"].
#[derive(Clone, Debug)]
pub struct SimplePatternSplitTokenizer {
    pattern: Regex,
}

impl SimplePatternSplitTokenizer {
    pub fn new(pattern: &str) -> Option<Self> {
        Regex::new(pattern).ok().map(|re| Self { pattern: re })
    }
}

impl Tokenizer for SimplePatternSplitTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut position: u32 = 0;
        let mut last_end = 0;

        for mat in self.pattern.find_iter(text) {
            if mat.start() > last_end {
                let term = &text[last_end..mat.start()];
                if !term.is_empty() {
                    tokens.push(Token {
                        term: Cow::Borrowed(term),
                        start_offset: last_end as u32,
                        end_offset: mat.start() as u32,
                        position,
                    });
                    position += 1;
                }
            }
            last_end = mat.end();
        }

        // Remaining text after last match
        if last_end < text.len() {
            let term = &text[last_end..];
            if !term.is_empty() {
                tokens.push(Token {
                    term: Cow::Borrowed(term),
                    start_offset: last_end as u32,
                    end_offset: text.len() as u32,
                    position,
                });
            }
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_pattern_match() {
        let tokenizer = SimplePatternTokenizer::new(r"\w+").unwrap();
        let tokens = tokenizer.tokenize("hello, world!");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["hello", "world"]);
    }

    #[test]
    fn test_simple_pattern_digits() {
        let tokenizer = SimplePatternTokenizer::new(r"\d+").unwrap();
        let tokens = tokenizer.tokenize("abc 123 def 456");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["123", "456"]);
    }

    #[test]
    fn test_simple_pattern_split() {
        let tokenizer = SimplePatternSplitTokenizer::new(r"[,\s]+").unwrap();
        let tokens = tokenizer.tokenize("hello, world, foo");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["hello", "world", "foo"]);
    }

    #[test]
    fn test_offsets() {
        let tokenizer = SimplePatternTokenizer::new(r"\w+").unwrap();
        let tokens = tokenizer.tokenize("hi there");
        assert_eq!(tokens[0].start_offset, 0);
        assert_eq!(tokens[0].end_offset, 2);
        assert_eq!(tokens[1].start_offset, 3);
        assert_eq!(tokens[1].end_offset, 8);
    }
}
