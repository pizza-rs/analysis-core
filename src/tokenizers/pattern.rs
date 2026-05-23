use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;
use regex::Regex;

/// Splits text on a regex pattern or extracts regex matches as tokens.
///
/// Two modes:
/// - Split mode (default): uses the pattern as a delimiter to split text
/// - Match mode: each regex match becomes a token
#[derive(Clone, Debug)]
pub struct PatternTokenizer {
    pattern: Regex,
    group: i32, // -1 = split mode, 0+ = capture group
}

impl PatternTokenizer {
    /// Create a pattern tokenizer in split mode (pattern is the delimiter).
    pub fn new(pattern: &str) -> Self {
        Self {
            pattern: Regex::new(pattern).expect("invalid regex pattern"),
            group: -1,
        }
    }

    /// Create a pattern tokenizer that extracts matches.
    /// `group` specifies which capture group to extract (0 = whole match).
    pub fn with_group(pattern: &str, group: i32) -> Self {
        Self {
            pattern: Regex::new(pattern).expect("invalid regex pattern"),
            group,
        }
    }

    /// Create from a pre-compiled regex in split mode.
    pub fn from_regex(pattern: Regex) -> Self {
        Self { pattern, group: -1 }
    }
}

impl Default for PatternTokenizer {
    fn default() -> Self {
        // Default: split on non-word characters (like ES default)
        Self::new(r"\W+")
    }
}

impl Tokenizer for PatternTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut position = 0u32;

        if self.group < 0 {
            // Split mode: pattern is delimiter
            let mut last_end = 0;
            for mat in self.pattern.find_iter(text) {
                let start = last_end;
                let end = mat.start();
                if start < end {
                    tokens.push(Token {
                        term: Cow::Borrowed(&text[start..end]),
                        start_offset: start as u32,
                        end_offset: end as u32,
                        position,
                    });
                    position += 1;
                }
                last_end = mat.end();
            }
            // Remaining text after last match
            if last_end < text.len() {
                tokens.push(Token {
                    term: Cow::Borrowed(&text[last_end..]),
                    start_offset: last_end as u32,
                    end_offset: text.len() as u32,
                    position,
                });
            }
        } else {
            // Match mode: each match/group becomes a token
            let group = self.group as usize;
            for caps in self.pattern.captures_iter(text) {
                if let Some(mat) = caps.get(group) {
                    tokens.push(Token {
                        term: Cow::Borrowed(mat.as_str()),
                        start_offset: mat.start() as u32,
                        end_offset: mat.end() as u32,
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
    fn test_split_mode_default() {
        let t = PatternTokenizer::default();
        let tokens = t.tokenize("hello, world! foo");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].term, "hello");
        assert_eq!(tokens[1].term, "world");
        assert_eq!(tokens[2].term, "foo");
    }

    #[test]
    fn test_split_on_comma() {
        let t = PatternTokenizer::new(r",\s*");
        let tokens = t.tokenize("one, two,three");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].term, "one");
        assert_eq!(tokens[1].term, "two");
        assert_eq!(tokens[2].term, "three");
    }

    #[test]
    fn test_match_mode() {
        let t = PatternTokenizer::with_group(r"\w+", 0);
        let tokens = t.tokenize("hello, world!");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].term, "hello");
        assert_eq!(tokens[1].term, "world");
    }

    #[test]
    fn test_offsets() {
        let t = PatternTokenizer::new(r"\s+");
        let tokens = t.tokenize("hello world");
        assert_eq!(tokens[0].start_offset, 0);
        assert_eq!(tokens[0].end_offset, 5);
        assert_eq!(tokens[1].start_offset, 6);
        assert_eq!(tokens[1].end_offset, 11);
    }

    #[test]
    fn test_capture_group() {
        let t = PatternTokenizer::with_group(r#""([^"]+)""#, 1);
        let tokens = t.tokenize(r#"say "hello" and "world""#);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].term, "hello");
        assert_eq!(tokens[1].term, "world");
    }
}
