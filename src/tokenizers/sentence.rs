use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Splits text into sentences based on sentence-ending punctuation.
///
/// Recognizes `.`, `!`, `?`, `。`, `！`, `？` followed by whitespace or end of string.
/// Each sentence becomes a single token.
#[derive(Clone, Debug)]
pub struct SentenceTokenizer {
    /// Minimum sentence length to emit (in characters)
    min_length: usize,
}

impl SentenceTokenizer {
    pub fn new() -> Self {
        Self { min_length: 1 }
    }

    pub fn with_min_length(mut self, min: usize) -> Self {
        self.min_length = min;
        self
    }

    fn is_sentence_end(c: char) -> bool {
        matches!(c, '.' | '!' | '?' | '。' | '！' | '？' | '⁈' | '⁉')
    }
}

impl Default for SentenceTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for SentenceTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut position = 0u32;
        let mut start = 0usize;
        let mut chars = text.char_indices().peekable();

        while let Some((i, c)) = chars.next() {
            if Self::is_sentence_end(c) {
                // Check if next is whitespace, EOF, or another sentence-end
                let at_boundary = match chars.peek() {
                    None => true,
                    Some(&(_, next_c)) => next_c.is_whitespace() || next_c == '"' || next_c == '\'',
                };

                if at_boundary {
                    let end = i + c.len_utf8();
                    let sentence = &text[start..end];
                    let trimmed = sentence.trim();
                    // `min_length` is documented in characters; count chars
                    // not bytes so CJK/non-ASCII sentences are sized correctly.
                    if trimmed.chars().count() >= self.min_length {
                        let trim_start = start + sentence.len() - sentence.trim_start().len();
                        let trim_end = end - (sentence.len() - sentence.trim_end().len());
                        tokens.push(Token {
                            term: Cow::Borrowed(trimmed),
                            start_offset: trim_start as u32,
                            end_offset: trim_end as u32,
                            position,
                        });
                        position += 1;
                    }
                    // Skip whitespace after sentence end
                    while let Some(&(_, nc)) = chars.peek() {
                        if nc.is_whitespace() {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    start = chars.peek().map(|&(i, _)| i).unwrap_or(text.len());
                }
            }
        }

        // Remaining text as last sentence
        if start < text.len() {
            let remaining = text[start..].trim();
            if remaining.chars().count() >= self.min_length {
                let trim_start = start + text[start..].len() - text[start..].trim_start().len();
                tokens.push(Token {
                    term: Cow::Borrowed(remaining),
                    start_offset: trim_start as u32,
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
    fn test_basic_sentences() {
        let tok = SentenceTokenizer::new();
        let tokens = tok.tokenize("Hello world. How are you? I'm fine!");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].term.as_ref(), "Hello world.");
        assert_eq!(tokens[1].term.as_ref(), "How are you?");
        assert_eq!(tokens[2].term.as_ref(), "I'm fine!");
    }

    #[test]
    fn test_chinese_sentence_end() {
        let tok = SentenceTokenizer::new();
        let tokens = tok.tokenize("你好世界。今天天气好吗？很好！");
        assert_eq!(tokens.len(), 3);
    }

    #[test]
    fn test_no_sentence_end() {
        let tok = SentenceTokenizer::new();
        let tokens = tok.tokenize("no punctuation here");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].term.as_ref(), "no punctuation here");
    }
}
