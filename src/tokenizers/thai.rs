use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Thai tokenizer that segments Thai text at script boundaries.
///
/// Thai script doesn't use spaces between words. This tokenizer provides
/// basic segmentation by:
/// 1. Splitting at Thai/non-Thai script boundaries
/// 2. Splitting at whitespace/punctuation within non-Thai text
///
/// For full dictionary-based Thai word segmentation, use the ICU tokenizer
/// or a dedicated Thai NLP library.
#[derive(Clone, Debug, Default)]
pub struct ThaiTokenizer;

impl ThaiTokenizer {
    pub fn new() -> Self {
        Self
    }
}

impl Tokenizer for ThaiTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens: Vec<Token<'a>> = Vec::new();
        let mut position: u32 = 0;

        if text.is_empty() {
            return tokens;
        }

        let mut start = 0;
        let mut in_thai = false;
        let mut last_boundary = 0;

        for (i, c) in text.char_indices() {
            let is_thai = is_thai_char(c);
            let is_whitespace = c.is_whitespace();
            let is_punct = !c.is_alphanumeric() && !is_whitespace;

            if i == 0 {
                in_thai = is_thai;
                start = 0;
                continue;
            }

            // Emit token at script boundaries or whitespace
            let should_split = is_whitespace
                || is_punct
                || (is_thai && !in_thai)
                || (!is_thai && in_thai && c.is_alphanumeric());

            if should_split {
                let term = &text[start..i];
                let trimmed = term.trim();
                if !trimmed.is_empty() {
                    let actual_start = start + term.len() - term.trim_start().len();
                    tokens.push(Token {
                        term: Cow::Borrowed(trimmed),
                        start_offset: actual_start as u32,
                        end_offset: (actual_start + trimmed.len()) as u32,
                        position,
                    });
                    position += 1;
                }
                start = i;
                if is_whitespace || is_punct {
                    start = i + c.len_utf8();
                }
                in_thai = is_thai;
            }

            last_boundary = i;
        }

        // Handle final segment
        if start < text.len() {
            let term = &text[start..];
            let trimmed = term.trim();
            if !trimmed.is_empty() {
                let actual_start = start + term.len() - term.trim_start().len();
                tokens.push(Token {
                    term: Cow::Borrowed(trimmed),
                    start_offset: actual_start as u32,
                    end_offset: (actual_start + trimmed.len()) as u32,
                    position,
                });
            }
        }

        let _ = last_boundary;
        tokens
    }
}

/// Check if a character is in the Thai Unicode block (U+0E00–U+0E7F).
#[inline]
fn is_thai_char(c: char) -> bool {
    let cp = c as u32;
    (0x0E00..=0x0E7F).contains(&cp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thai_only() {
        let tokenizer = ThaiTokenizer::new();
        let tokens = tokenizer.tokenize("สวัสดี");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].term.as_ref(), "สวัสดี");
    }

    #[test]
    fn test_thai_english_mixed() {
        let tokenizer = ThaiTokenizer::new();
        let tokens = tokenizer.tokenize("สวัสดีhello");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].term.as_ref(), "สวัสดี");
        assert_eq!(tokens[1].term.as_ref(), "hello");
    }

    #[test]
    fn test_english_only() {
        let tokenizer = ThaiTokenizer::new();
        let tokens = tokenizer.tokenize("hello world");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["hello", "world"]);
    }
}
