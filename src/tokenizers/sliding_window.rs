use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Decompounding tokenizer using a sliding window approach.
///
/// Unlike the dictionary-based CompoundWordTokenizer, this uses character n-grams
/// within words to generate overlapping sub-tokens. Good for fuzzy matching
/// and finding partial matches within compound words.
///
/// Example with min=3, max=5:
/// `"testing"` → `"tes"`, `"test"`, `"testi"`, `"est"`, `"esti"`, `"estin"`, ...
#[derive(Clone, Debug)]
pub struct SlidingWindowTokenizer {
    /// Minimum window size
    min_size: usize,
    /// Maximum window size
    max_size: usize,
    /// Also emit the original full token
    preserve_original: bool,
    /// Hard cap on total tokens emitted. Defends against pathological inputs
    /// where the cross-product of word length and (max-min) window range
    /// would otherwise emit an unbounded number of tokens.
    max_tokens: usize,
}

impl SlidingWindowTokenizer {
    pub fn new(min_size: usize, max_size: usize) -> Self {
        Self {
            min_size,
            max_size,
            preserve_original: true,
            max_tokens: 10_000,
        }
    }

    pub fn with_preserve_original(mut self, v: bool) -> Self {
        self.preserve_original = v;
        self
    }

    pub fn with_max_tokens(mut self, n: usize) -> Self {
        self.max_tokens = n;
        self
    }
}

impl Default for SlidingWindowTokenizer {
    fn default() -> Self {
        Self::new(3, 5)
    }
}

impl Tokenizer for SlidingWindowTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut position = 0u32;

        for word in text.split_whitespace() {
            if tokens.len() >= self.max_tokens {
                break;
            }
            let word_offset = word.as_ptr() as usize - text.as_ptr() as usize;

            if self.preserve_original {
                tokens.push(Token {
                    term: Cow::Borrowed(word),
                    start_offset: word_offset as u32,
                    end_offset: (word_offset + word.len()) as u32,
                    position,
                });
            }

            // Generate character-based sliding windows.
            let chars: Vec<(usize, char)> = word.char_indices().collect();
            let char_count = chars.len();

            // Use `>=` so that a word whose length exactly equals `min_size`
            // still produces the single window covering the whole word.
            // Previously `>` silently dropped the only valid window in that
            // case (e.g. `SlidingWindowTokenizer::new(3, 3)` on `"abc"` emitted
            // nothing but the original).
            if char_count >= self.min_size {
                'outer: for window_size in self.min_size..=self.max_size.min(char_count) {
                    for start_idx in 0..=(char_count - window_size) {
                        if tokens.len() >= self.max_tokens {
                            break 'outer;
                        }
                        let byte_start = chars[start_idx].0;
                        let byte_end = if start_idx + window_size < char_count {
                            chars[start_idx + window_size].0
                        } else {
                            word.len()
                        };

                        let sub = &word[byte_start..byte_end];
                        let abs_offset = word_offset + byte_start;
                        tokens.push(Token {
                            term: Cow::Borrowed(sub),
                            start_offset: abs_offset as u32,
                            end_offset: (abs_offset + sub.len()) as u32,
                            position,
                        });
                    }
                }
            }

            position += 1;
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sliding_window() {
        let tok = SlidingWindowTokenizer::new(3, 3);
        let tokens = tok.tokenize("hello");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        // Original + 3-char windows: hel, ell, llo
        assert!(terms.contains(&"hello")); // original
        assert!(terms.contains(&"hel"));
        assert!(terms.contains(&"ell"));
        assert!(terms.contains(&"llo"));
    }

    #[test]
    fn test_variable_window() {
        let tok = SlidingWindowTokenizer::new(2, 3);
        let tokens = tok.tokenize("test");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        // 2-char: te, es, st
        // 3-char: tes, est
        assert!(terms.contains(&"te"));
        assert!(terms.contains(&"es"));
        assert!(terms.contains(&"st"));
        assert!(terms.contains(&"tes"));
        assert!(terms.contains(&"est"));
    }

    #[test]
    fn test_short_word() {
        let tok = SlidingWindowTokenizer::new(3, 5);
        let tokens = tok.tokenize("ab");
        // Word is shorter than min_size, only original emitted
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].term.as_ref(), "ab");
    }

    #[test]
    fn test_no_preserve() {
        let tok = SlidingWindowTokenizer::new(3, 3).with_preserve_original(false);
        let tokens = tok.tokenize("hello");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(!terms.contains(&"hello"));
        assert!(terms.contains(&"hel"));
    }

    #[test]
    fn test_multiple_words() {
        let tok = SlidingWindowTokenizer::new(3, 3);
        let tokens = tok.tokenize("hi there");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"the"));
        assert!(terms.contains(&"her"));
        assert!(terms.contains(&"ere"));
    }

    #[test]
    fn test_word_exactly_min_size_emits_window() {
        // Regression: `char_count > min_size` previously dropped the only
        // valid window when the word length equalled min_size.
        let tok = SlidingWindowTokenizer::new(3, 5).with_preserve_original(false);
        let tokens = tok.tokenize("abc");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(
            terms.contains(&"abc"),
            "expected `abc` window, got {:?}",
            terms
        );
    }
}
