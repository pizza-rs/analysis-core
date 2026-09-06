use alloc::borrow::Cow;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;
use std::sync::Mutex;

/// Buffered token info used to build shingles spanning multiple inputs.
///
/// We capture the original `start_offset` / `end_offset` / `position` of
/// every token that enters the filter so a shingle can faithfully span
/// from the first token's start to the last token's end and inherit the
/// first token's position — matching Lucene's `ShingleFilter` semantics.
#[derive(Clone, Debug)]
struct BufferedToken {
    term: String,
    start_offset: u32,
    end_offset: u32,
    position: u32,
}

/// Creates word-level n-grams (shingles) by concatenating adjacent tokens.
///
/// When used through the `TokenFilter` trait, this filter maintains an internal
/// buffer using interior mutability. Each incoming token is added to the sliding
/// window buffer, and shingles are emitted as additional tokens.
///
/// For example, with size=2, tokens ["the", "quick", "fox"] produce:
/// - Token "the" → no additional tokens (need 2 for bigram)
/// - Token "quick" → additional token "the quick"
/// - Token "fox" → additional token "quick fox"
///
/// If `output_unigrams` is false, the original token is removed and only
/// shingles are emitted.
#[derive(Clone, Debug)]
pub struct ShingleTokenFilter {
    min_size: usize,
    max_size: usize,
    separator: String,
    output_unigrams: bool,
    filler_token: String,
    buffer: Arc<Mutex<Vec<BufferedToken>>>,
}

impl ShingleTokenFilter {
    pub fn new(min_size: usize, max_size: usize) -> Self {
        Self {
            min_size: min_size.max(2),
            max_size: max_size.max(min_size),
            separator: String::from(" "),
            output_unigrams: true,
            filler_token: String::from("_"),
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn with_separator(mut self, sep: &str) -> Self {
        self.separator = String::from(sep);
        self
    }

    pub fn with_output_unigrams(mut self, output: bool) -> Self {
        self.output_unigrams = output;
        self
    }

    pub fn with_filler_token(mut self, filler: &str) -> Self {
        self.filler_token = String::from(filler);
        self
    }

    /// Reset the internal buffer (call between documents).
    pub fn reset(&self) {
        if let Ok(mut buf) = self.buffer.lock() {
            buf.clear();
        }
    }
}

impl Default for ShingleTokenFilter {
    fn default() -> Self {
        Self::new(2, 2)
    }
}

impl TokenFilter for ShingleTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let mut buf = match self.buffer.lock() {
            Ok(b) => b,
            Err(_) => return (false, None),
        };

        buf.push(BufferedToken {
            term: token.term.as_ref().to_owned(),
            start_offset: token.start_offset,
            end_offset: token.end_offset,
            position: token.position,
        });

        let buf_len = buf.len();
        let mut shingles = Vec::new();

        for size in self.min_size..=self.max_size {
            if buf_len >= size {
                let start = buf_len - size;
                let mut shingle = String::new();
                for (idx, bt) in buf[start..buf_len].iter().enumerate() {
                    if idx > 0 {
                        shingle.push_str(&self.separator);
                    }
                    shingle.push_str(&bt.term);
                }
                // Shingle spans from first token's start to last token's end,
                // anchored at the first token's position (Lucene semantics).
                let first = &buf[start];
                let last = &buf[buf_len - 1];
                shingles.push(Token {
                    term: Cow::Owned(shingle),
                    start_offset: first.start_offset,
                    end_offset: last.end_offset,
                    position: first.position,
                });
            }
        }

        // Sliding window: keep only max_size - 1 tokens
        if buf.len() > self.max_size {
            buf.remove(0);
        }

        // Remove the original token if unigrams are suppressed
        let remove = !self.output_unigrams;

        if shingles.is_empty() {
            (remove, None)
        } else if remove {
            // Replace the token with the first shingle when suppressing unigrams
            let first = shingles.remove(0);
            token.term = first.term;
            token.start_offset = first.start_offset;
            token.end_offset = first.end_offset;
            if shingles.is_empty() {
                (false, None)
            } else {
                (false, Some(shingles))
            }
        } else {
            (false, Some(shingles))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_token(term: &str, pos: u32) -> Token<'_> {
        Token {
            term: Cow::Borrowed(term),
            start_offset: 0,
            end_offset: term.len() as u32,
            position: pos,
        }
    }

    #[test]
    fn test_shingle_bigrams() {
        let filter = ShingleTokenFilter::new(2, 2);

        let mut t1 = make_token("the", 0);
        let (remove, extra) = filter.filter(&mut t1);
        assert!(!remove);
        assert!(extra.is_none()); // need 2 tokens for bigram

        let mut t2 = make_token("quick", 1);
        let (remove, extra) = filter.filter(&mut t2);
        assert!(!remove);
        let extra = extra.unwrap();
        assert_eq!(extra.len(), 1);
        assert_eq!(extra[0].term.as_ref(), "the quick");

        let mut t3 = make_token("fox", 2);
        let (_, extra) = filter.filter(&mut t3);
        let extra = extra.unwrap();
        assert_eq!(extra[0].term.as_ref(), "quick fox");
    }

    #[test]
    fn test_shingle_trigrams() {
        let filter = ShingleTokenFilter::new(2, 3);

        let mut t1 = make_token("the", 0);
        filter.filter(&mut t1);
        let mut t2 = make_token("quick", 1);
        filter.filter(&mut t2);

        let mut t3 = make_token("brown", 2);
        let (_, extra) = filter.filter(&mut t3);
        let extra = extra.unwrap();
        let terms: Vec<&str> = extra.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"quick brown"));
        assert!(terms.contains(&"the quick brown"));
    }

    #[test]
    fn test_shingle_no_unigrams() {
        let filter = ShingleTokenFilter::new(2, 2).with_output_unigrams(false);

        let mut t1 = make_token("hello", 0);
        let (remove1, _) = filter.filter(&mut t1);
        assert!(remove1); // no shingle yet, suppress unigram

        let mut t2 = make_token("world", 1);
        let (remove2, _) = filter.filter(&mut t2);
        assert!(!remove2); // shingle replaces the token term
        assert_eq!(t2.term.as_ref(), "hello world");
    }

    #[test]
    fn test_custom_separator() {
        let filter = ShingleTokenFilter::new(2, 2).with_separator("_");
        let mut t1 = make_token("hello", 0);
        filter.filter(&mut t1);
        let mut t2 = make_token("world", 1);
        let (_, extra) = filter.filter(&mut t2);
        assert_eq!(extra.unwrap()[0].term.as_ref(), "hello_world");
    }

    #[test]
    fn test_sliding_window() {
        let filter = ShingleTokenFilter::new(2, 2);
        let words = ["a", "b", "c", "d", "e"];
        let mut results = Vec::new();
        for (i, w) in words.iter().enumerate() {
            let mut t = make_token(w, i as u32);
            if let (_, Some(extra)) = filter.filter(&mut t) {
                for e in extra {
                    results.push(e.term.as_ref().to_owned());
                }
            }
        }
        assert_eq!(results, vec!["a b", "b c", "c d", "d e"]);
    }

    #[test]
    fn test_reset() {
        let filter = ShingleTokenFilter::new(2, 2);
        let mut t1 = make_token("hello", 0);
        filter.filter(&mut t1);
        filter.reset();
        // After reset, next token should not produce shingle from "hello"
        let mut t2 = make_token("world", 0);
        let (_, extra) = filter.filter(&mut t2);
        assert!(extra.is_none());
    }

    #[test]
    fn test_shingle_spans_first_to_last_offsets() {
        // Lucene semantics: a shingle's offsets span from the first token's
        // start_offset to the last token's end_offset, anchored at the first
        // token's position. The previous implementation incorrectly used the
        // latest token's offsets, which broke highlighting on shingled fields.
        let filter = ShingleTokenFilter::new(2, 2);

        let mut t1 = Token {
            term: Cow::Borrowed("the"),
            start_offset: 0,
            end_offset: 3,
            position: 0,
        };
        let (_, _) = filter.filter(&mut t1);

        let mut t2 = Token {
            term: Cow::Borrowed("quick"),
            start_offset: 4,
            end_offset: 9,
            position: 1,
        };
        let (_, extra) = filter.filter(&mut t2);
        let extra = extra.unwrap();
        assert_eq!(extra.len(), 1);
        assert_eq!(extra[0].term.as_ref(), "the quick");
        assert_eq!(
            extra[0].start_offset, 0,
            "start should be first token's start"
        );
        assert_eq!(extra[0].end_offset, 9, "end should be last token's end");
        assert_eq!(
            extra[0].position, 0,
            "position should be first token's position"
        );
    }
}
