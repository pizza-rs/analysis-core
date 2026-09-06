use alloc::borrow::Cow;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use hashbrown::HashSet;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;
use std::sync::Mutex;

/// Outputs a single "fingerprint" token from the entire token stream.
///
/// The fingerprint is created by:
/// 1. Collecting all unique tokens
/// 2. Sorting them lexicographically
/// 3. Concatenating with a separator (default: space)
///
/// This is useful for document similarity detection and deduplication.
///
/// When used through the `TokenFilter` trait, it accumulates all tokens and
/// removes them from the stream (returns true). Call `emit_fingerprint()` to
/// get the final fingerprint token, or use `FingerprintAccumulator` for
/// more control.
///
/// In per-token mode: each token is removed and accumulated. The accumulated
/// result is available via `take_fingerprint()`.
#[derive(Clone, Debug)]
pub struct FingerprintTokenFilter {
    max_output_size: usize,
    separator: String,
    state: Arc<Mutex<FingerprintState>>,
}

#[derive(Clone, Debug, Default)]
struct FingerprintState {
    /// Ordered list of unique terms seen so far (preserves insertion
    /// order so that the sorted+joined fingerprint is reproducible).
    terms: Vec<String>,
    /// Membership index. Kept in sync with `terms` so that the per-token
    /// `filter` call is O(1) amortized instead of O(N) (Vec::contains
    /// scan), avoiding O(N²) behaviour on long token streams.
    seen: HashSet<String>,
}

impl FingerprintTokenFilter {
    pub fn new() -> Self {
        Self {
            max_output_size: 1024,
            separator: String::from(" "),
            state: Arc::new(Mutex::new(FingerprintState::default())),
        }
    }

    pub fn with_max_output_size(mut self, max: usize) -> Self {
        self.max_output_size = max;
        self
    }

    pub fn with_separator(mut self, sep: &str) -> Self {
        self.separator = String::from(sep);
        self
    }

    /// Take the fingerprint from accumulated tokens, resetting internal state.
    /// Returns None if no tokens were accumulated or result exceeds max size.
    pub fn take_fingerprint(&self) -> Option<String> {
        let mut state = self.state.lock().ok()?;
        if state.terms.is_empty() {
            return None;
        }
        state.terms.sort();
        state.terms.dedup();
        let result = state.terms.join(&self.separator);
        state.terms.clear();
        state.seen.clear();
        if result.len() > self.max_output_size {
            return None;
        }
        Some(result)
    }

    /// Reset internal state.
    pub fn reset(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.terms.clear();
            state.seen.clear();
        }
    }
}

impl Default for FingerprintTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for FingerprintTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let term = token.term.as_ref().to_owned();
        let mut state = match self.state.lock() {
            Ok(s) => s,
            Err(_) => return (false, None),
        };

        // Add to accumulator if not already present (O(1) lookup via
        // the parallel HashSet; previously this was an O(N) Vec scan
        // making the whole filter O(N²) per document).
        if !state.seen.contains(&term) {
            state.seen.insert(term.clone());
            state.terms.push(term);
        }

        // Remove original token — fingerprint replaces the entire stream
        (true, None)
    }
}

/// Accumulates tokens to produce a fingerprint.
/// Use this for stream-level fingerprint generation.
#[derive(Clone, Debug)]
pub struct FingerprintAccumulator {
    max_output_size: usize,
    separator: String,
    terms: Vec<String>,
}

impl FingerprintAccumulator {
    pub fn new(max_output_size: usize, separator: &str) -> Self {
        Self {
            max_output_size,
            separator: String::from(separator),
            terms: Vec::new(),
        }
    }

    /// Add a token term to the accumulator.
    pub fn add(&mut self, term: &str) {
        let s = String::from(term);
        if !self.terms.contains(&s) {
            self.terms.push(s);
        }
    }

    /// Generate the fingerprint. Returns None if output exceeds max size.
    pub fn fingerprint(&mut self) -> Option<String> {
        self.terms.sort();
        let result = self.terms.join(&self.separator);
        if result.len() > self.max_output_size {
            return None;
        }
        Some(result)
    }

    /// Reset for reuse.
    pub fn reset(&mut self) {
        self.terms.clear();
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
    fn test_fingerprint_filter_removes_tokens() {
        let filter = FingerprintTokenFilter::new();

        let mut t1 = make_token("the", 0);
        let (remove, _) = filter.filter(&mut t1);
        assert!(remove); // token removed, accumulated

        let mut t2 = make_token("quick", 1);
        let (remove, _) = filter.filter(&mut t2);
        assert!(remove);

        let mut t3 = make_token("brown", 2);
        let (remove, _) = filter.filter(&mut t3);
        assert!(remove);

        let mut t4 = make_token("the", 3); // duplicate
        let (remove, _) = filter.filter(&mut t4);
        assert!(remove);

        // Get the fingerprint
        let fp = filter.take_fingerprint().unwrap();
        assert_eq!(fp, "brown quick the"); // sorted, deduplicated
    }

    #[test]
    fn test_fingerprint_max_size() {
        let filter = FingerprintTokenFilter::new().with_max_output_size(5);

        let mut t1 = make_token("hello", 0);
        filter.filter(&mut t1);
        let mut t2 = make_token("world", 1);
        filter.filter(&mut t2);

        assert!(filter.take_fingerprint().is_none()); // "hello world" > 5
    }

    #[test]
    fn test_fingerprint_custom_separator() {
        let filter = FingerprintTokenFilter::new().with_separator("_");

        let mut t1 = make_token("b", 0);
        filter.filter(&mut t1);
        let mut t2 = make_token("a", 1);
        filter.filter(&mut t2);
        let mut t3 = make_token("c", 2);
        filter.filter(&mut t3);

        assert_eq!(filter.take_fingerprint().unwrap(), "a_b_c");
    }

    #[test]
    fn test_fingerprint_reset() {
        let filter = FingerprintTokenFilter::new();
        let mut t1 = make_token("hello", 0);
        filter.filter(&mut t1);
        filter.reset();
        assert!(filter.take_fingerprint().is_none()); // empty after reset
    }

    #[test]
    fn test_fingerprint_accumulator() {
        let mut acc = FingerprintAccumulator::new(1024, " ");
        acc.add("the");
        acc.add("quick");
        acc.add("brown");
        acc.add("fox");
        acc.add("the"); // duplicate, ignored

        let fp = acc.fingerprint().unwrap();
        assert_eq!(fp, "brown fox quick the");
    }

    #[test]
    fn test_fingerprint_accumulator_max_size() {
        let mut acc = FingerprintAccumulator::new(5, " ");
        acc.add("hello");
        acc.add("world");
        assert!(acc.fingerprint().is_none()); // "hello world" > 5
    }
}
