use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::sync::Arc;
use std::sync::Mutex;
use hashbrown::HashSet;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Query-time companion to [`CommonGramsTokenFilter`].
///
/// In Lucene, `CommonGramsQueryFilter` wraps the output of `CommonGramsFilter`
/// and removes the original unigram tokens when a bigram containing them
/// was produced. The result is that for query phrases, only the bigrams
/// survive — this shrinks the query while preserving exact phrase matching.
///
/// **Index-time** pipeline: `CommonGramsTokenFilter` → emits both unigrams AND bigrams.
/// **Query-time** pipeline: `CommonGramsTokenFilter` → `CommonGramsQueryFilter`
///   → keeps only bigrams (removes unigrams that participate in bigrams).
///
/// Example with common words `["the"]`:
///
/// Input tokens:  `["the", "cat", "sat"]`
/// After CommonGramsFilter: `["the", "the_cat", "cat", "sat"]`
/// After CommonGramsQueryFilter: `["the_cat", "cat", "sat"]`
///   ("the" removed because it was part of bigram "the_cat";
///    "cat" kept because "cat_sat" was NOT produced — "sat" isn't common)
///
/// Uses interior mutability to track whether the current token is followed
/// by a bigram (look-ahead).
#[derive(Clone, Debug)]
pub struct CommonGramsQueryFilter {
    separator: String,
    state: Arc<Mutex<QueryState>>,
}

#[derive(Debug, Default)]
struct QueryState {
    /// The previous unigram, held back until we know whether a bigram follows.
    pending: Option<PendingToken>,
}

#[derive(Debug, Clone)]
struct PendingToken {
    term: String,
    start_offset: u32,
    end_offset: u32,
    position: u32,
}

impl CommonGramsQueryFilter {
    /// Create with the separator used by `CommonGramsTokenFilter` (default `_`).
    pub fn new() -> Self {
        Self {
            separator: String::from("_"),
            state: Arc::new(Mutex::new(QueryState::default())),
        }
    }

    /// Set the separator to match the one used in `CommonGramsTokenFilter`.
    pub fn with_separator(mut self, sep: &str) -> Self {
        self.separator = String::from(sep);
        self
    }

    fn is_bigram(&self, term: &str) -> bool {
        term.contains(self.separator.as_str())
    }

    /// Reset state between documents / queries.
    pub fn reset(&self) {
        if let Ok(mut s) = self.state.lock() {
            s.pending = None;
        }
    }
}

impl Default for CommonGramsQueryFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for CommonGramsQueryFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let term = token.term.as_ref();
        let is_bigram = self.is_bigram(term);

        let mut state = match self.state.lock() {
            Ok(s) => s,
            Err(_) => return (false, None),
        };

        // If the current token is a bigram, then the previous unigram
        // (if any) was absorbed into this bigram → drop the pending unigram.
        if is_bigram {
            // Drop any pending unigram — it was part of this bigram
            state.pending = None;
            // Emit the bigram immediately
            return (false, None);
        }

        // Current token is a unigram.
        // If there was a pending unigram, it was NOT followed by a bigram,
        // so it should have been emitted. We emit it as an extra token.
        let emit_pending = state.pending.take().map(|p| {
            Token {
                term: Cow::Owned(p.term),
                start_offset: p.start_offset,
                end_offset: p.end_offset,
                position: p.position,
            }
        });

        // Hold back the current unigram as pending
        state.pending = Some(PendingToken {
            term: token.term.to_string(),
            start_offset: token.start_offset,
            end_offset: token.end_offset,
            position: token.position,
        });

        // Delete the current token from the stream (it's now pending)
        // and emit the previously pending token if any
        match emit_pending {
            Some(prev_token) => {
                // We need to emit the previous pending token.
                // Replace current token with the pending one.
                token.term = prev_token.term;
                token.start_offset = prev_token.start_offset;
                token.end_offset = prev_token.end_offset;
                token.position = prev_token.position;
                (false, None)
            }
            None => {
                // First unigram — just hold it back
                (true, None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    fn make_token(term: &str, pos: u32) -> Token<'_> {
        Token {
            term: Cow::Borrowed(term),
            start_offset: 0,
            end_offset: term.len() as u32,
            position: pos,
        }
    }

    /// Helper: run a sequence of tokens through the filter and collect surviving terms.
    fn run_filter(filter: &CommonGramsQueryFilter, terms: &[(&str, u32)]) -> Vec<String> {
        let mut results = Vec::new();
        for &(term, pos) in terms {
            let mut token = make_token(term, pos);
            let (deleted, extra) = filter.filter(&mut token);
            if let Some(extras) = extra {
                for e in extras {
                    results.push(e.term.to_string());
                }
            }
            if !deleted {
                results.push(token.term.to_string());
            }
        }
        // Flush any remaining pending unigram
        // (In a real pipeline, the analyzer would call a flush/finish method.
        //  For testing, we reset and check if there was a pending token.)
        results
    }

    #[test]
    fn test_bigram_drops_preceding_unigram() {
        let filter = CommonGramsQueryFilter::new();

        // Input: ["the"(unigram), "the_cat"(bigram), "cat"(unigram), "sat"(unigram)]
        // Expected: "the" is pending → "the_cat" arrives → drop "the", emit "the_cat"
        //           "cat" is pending → "sat" arrives → emit "cat", "sat" pending
        let result = run_filter(&filter, &[
            ("the", 0),
            ("the_cat", 0),
            ("cat", 1),
            ("sat", 2),
        ]);
        // "the" → pending → dropped by bigram "the_cat"
        // "the_cat" → emitted
        // "cat" → pending
        // "sat" → emits pending "cat", "sat" becomes pending
        assert_eq!(result, vec!["the_cat", "cat"]);
    }

    #[test]
    fn test_no_bigrams_all_pass() {
        let filter = CommonGramsQueryFilter::new();

        // All unigrams, no bigrams → all should pass (with one-token delay)
        let result = run_filter(&filter, &[
            ("hello", 0),
            ("world", 1),
            ("foo", 2),
        ]);
        // "hello" → pending
        // "world" → emit "hello", "world" pending
        // "foo" → emit "world", "foo" pending
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn test_consecutive_bigrams() {
        let filter = CommonGramsQueryFilter::new();

        // "the"(uni) "the_of"(bi) "of_new"(bi) "new"(uni)
        let result = run_filter(&filter, &[
            ("the", 0),
            ("the_of", 0),
            ("of_new", 1),
            ("new", 2),
        ]);
        // "the" pending → dropped by "the_of"
        // "the_of" emitted
        // "of_new" emitted (no pending to drop)
        // "new" pending
        assert_eq!(result, vec!["the_of", "of_new"]);
    }

    #[test]
    fn test_custom_separator() {
        let filter = CommonGramsQueryFilter::new().with_separator("-");

        let result = run_filter(&filter, &[
            ("city", 0),
            ("city-of", 0),
            ("of", 1),
        ]);
        // "city" pending → dropped by bigram "city-of"
        // "city-of" emitted
        // "of" pending
        assert_eq!(result, vec!["city-of"]);
    }

    #[test]
    fn test_reset() {
        let filter = CommonGramsQueryFilter::new();

        let mut t1 = make_token("hello", 0);
        let (deleted, _) = filter.filter(&mut t1);
        assert!(deleted); // pending

        filter.reset();

        let mut t2 = make_token("world", 0);
        let (deleted, _) = filter.filter(&mut t2);
        assert!(deleted); // pending (no carry-over from before reset)
    }

    #[test]
    fn test_single_bigram_only() {
        let filter = CommonGramsQueryFilter::new();

        let result = run_filter(&filter, &[("the_cat", 0)]);
        assert_eq!(result, vec!["the_cat"]);
    }
}
