use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Generates n-grams of configurable length from the input text.
#[derive(Clone, Debug)]
pub struct NgramTokenizer {
    pub min_gram: usize,
    pub max_gram: usize,
    pub token_chars: Vec<TokenCharKind>,
}

/// Character classes that tokens may contain.
#[derive(Clone, Debug, PartialEq)]
pub enum TokenCharKind {
    Letter,
    Digit,
    Whitespace,
    Punctuation,
    Symbol,
}

impl NgramTokenizer {
    pub fn new(min_gram: usize, max_gram: usize) -> Self {
        Self {
            min_gram,
            max_gram,
            token_chars: Vec::new(), // empty = all chars accepted
        }
    }

    pub fn with_token_chars(mut self, token_chars: Vec<TokenCharKind>) -> Self {
        self.token_chars = token_chars;
        self
    }

    fn char_accepted(&self, ch: char) -> bool {
        if self.token_chars.is_empty() {
            return true;
        }
        for kind in &self.token_chars {
            let matches = match kind {
                TokenCharKind::Letter => ch.is_alphabetic(),
                TokenCharKind::Digit => ch.is_ascii_digit(),
                TokenCharKind::Whitespace => ch.is_whitespace(),
                TokenCharKind::Punctuation => ch.is_ascii_punctuation(),
                TokenCharKind::Symbol => {
                    !ch.is_alphanumeric() && !ch.is_whitespace() && !ch.is_ascii_punctuation()
                }
            };
            if matches {
                return true;
            }
        }
        false
    }
}

impl Default for NgramTokenizer {
    fn default() -> Self {
        Self::new(1, 2)
    }
}

impl Tokenizer for NgramTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut position = 0u32;

        // First, split into runs of accepted characters
        let chars: Vec<(usize, char)> = text.char_indices().collect();

        if self.token_chars.is_empty() {
            // No token_chars filter: generate n-grams from the entire text
            let char_count = chars.len();
            for n in self.min_gram..=self.max_gram {
                if n > char_count {
                    break;
                }
                for i in 0..=(char_count - n) {
                    let start_byte = chars[i].0;
                    let end_byte = if i + n < char_count {
                        chars[i + n].0
                    } else {
                        text.len()
                    };
                    tokens.push(Token::new(
                        &text[start_byte..end_byte],
                        start_byte as u32,
                        end_byte as u32,
                        position,
                    ));
                    position += 1;
                }
            }
        } else {
            // Split on non-accepted chars, generate n-grams per segment
            let mut seg_start = None;
            let mut segments: Vec<(usize, usize)> = Vec::new(); // (start_char_idx, end_char_idx)

            for (idx, &(_, ch)) in chars.iter().enumerate() {
                if self.char_accepted(ch) {
                    if seg_start.is_none() {
                        seg_start = Some(idx);
                    }
                } else if let Some(s) = seg_start {
                    segments.push((s, idx));
                    seg_start = None;
                }
            }
            if let Some(s) = seg_start {
                segments.push((s, chars.len()));
            }

            for (seg_s, seg_e) in segments {
                let seg_len = seg_e - seg_s;
                for n in self.min_gram..=self.max_gram {
                    if n > seg_len {
                        break;
                    }
                    for i in 0..=(seg_len - n) {
                        let ci = seg_s + i;
                        let start_byte = chars[ci].0;
                        let end_byte = if ci + n < chars.len() {
                            chars[ci + n].0
                        } else {
                            text.len()
                        };
                        tokens.push(Token::new(
                            &text[start_byte..end_byte],
                            start_byte as u32,
                            end_byte as u32,
                            position,
                        ));
                        position += 1;
                    }
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
    fn test_ngram_default() {
        let t = NgramTokenizer::new(1, 2);
        let tokens = t.tokenize("abc");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        // 1-grams: a, b, c  2-grams: ab, bc
        assert_eq!(terms, vec!["a", "b", "c", "ab", "bc"]);
    }

    #[test]
    fn test_ngram_with_token_chars() {
        let t = NgramTokenizer::new(1, 2).with_token_chars(vec![TokenCharKind::Letter]);
        let tokens = t.tokenize("a1b");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        // segments: "a", "b" → 1-grams only since each segment is length 1
        assert_eq!(terms, vec!["a", "b"]);
    }
}
