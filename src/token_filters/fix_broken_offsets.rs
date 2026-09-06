use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Ensures that token offsets are monotonically non-decreasing.
///
/// Lucene's `FixBrokenOffsetsFilter` is a repair filter that detects and
/// corrects tokens whose `start_offset` or `end_offset` violate the
/// monotonicity invariant. This can happen when poorly-behaved tokenizers
/// or filters produce out-of-order offsets.
///
/// The filter enforces:
/// - `start_offset >= previous end_offset` (monotonic)
/// - `end_offset >= start_offset` (well-formed)
///
/// When a violation is detected, offsets are clamped to preserve monotonicity.
#[derive(Debug)]
pub struct FixBrokenOffsetsFilter {
    last_end: std::sync::atomic::AtomicU32,
}

impl Clone for FixBrokenOffsetsFilter {
    fn clone(&self) -> Self {
        Self {
            last_end: std::sync::atomic::AtomicU32::new(
                self.last_end.load(std::sync::atomic::Ordering::Relaxed),
            ),
        }
    }
}

impl FixBrokenOffsetsFilter {
    pub fn new() -> Self {
        Self {
            last_end: std::sync::atomic::AtomicU32::new(0),
        }
    }

    /// Reset between documents.
    pub fn reset(&self) {
        self.last_end.store(0, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Default for FixBrokenOffsetsFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for FixBrokenOffsetsFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let prev_end = self.last_end.load(std::sync::atomic::Ordering::Relaxed);

        // Fix start_offset if it's before the previous end
        if token.start_offset < prev_end {
            token.start_offset = prev_end;
        }

        // Fix end_offset if it's before start_offset
        if token.end_offset < token.start_offset {
            token.end_offset = token.start_offset;
        }

        self.last_end
            .store(token.end_offset, std::sync::atomic::Ordering::Relaxed);

        (false, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_well_formed_offsets_unchanged() {
        let filter = FixBrokenOffsetsFilter::new();

        let mut t1 = Token::new("hello", 0, 5, 0);
        filter.filter(&mut t1);
        assert_eq!(t1.start_offset, 0);
        assert_eq!(t1.end_offset, 5);

        let mut t2 = Token::new("world", 6, 11, 1);
        filter.filter(&mut t2);
        assert_eq!(t2.start_offset, 6);
        assert_eq!(t2.end_offset, 11);
    }

    #[test]
    fn test_start_before_prev_end_clamped() {
        let filter = FixBrokenOffsetsFilter::new();

        let mut t1 = Token::new("hello", 0, 10, 0);
        filter.filter(&mut t1);

        // start_offset=5 < prev_end=10 → clamped to 10
        let mut t2 = Token::new("world", 5, 15, 1);
        filter.filter(&mut t2);
        assert_eq!(t2.start_offset, 10);
        assert_eq!(t2.end_offset, 15);
    }

    #[test]
    fn test_end_before_start_clamped() {
        let filter = FixBrokenOffsetsFilter::new();

        // Pathological: end < start
        let mut t1 = Token::new("oops", 10, 5, 0);
        filter.filter(&mut t1);
        assert_eq!(t1.start_offset, 10);
        assert_eq!(t1.end_offset, 10); // clamped: end = start
    }

    #[test]
    fn test_both_violations() {
        let filter = FixBrokenOffsetsFilter::new();

        let mut t1 = Token::new("first", 0, 20, 0);
        filter.filter(&mut t1);

        // start=5 < prev_end=20 → clamped to 20
        // end=3 < new start=20 → clamped to 20
        let mut t2 = Token::new("second", 5, 3, 1);
        filter.filter(&mut t2);
        assert_eq!(t2.start_offset, 20);
        assert_eq!(t2.end_offset, 20);
    }

    #[test]
    fn test_reset() {
        let filter = FixBrokenOffsetsFilter::new();

        let mut t1 = Token::new("first", 0, 100, 0);
        filter.filter(&mut t1);

        filter.reset();

        // After reset, start=0 should be fine
        let mut t2 = Token::new("second", 0, 5, 0);
        filter.filter(&mut t2);
        assert_eq!(t2.start_offset, 0);
        assert_eq!(t2.end_offset, 5);
    }

    #[test]
    fn test_adjacent_tokens() {
        let filter = FixBrokenOffsetsFilter::new();

        let mut t1 = Token::new("a", 0, 1, 0);
        filter.filter(&mut t1);
        let mut t2 = Token::new("b", 1, 2, 1);
        filter.filter(&mut t2);
        let mut t3 = Token::new("c", 2, 3, 2);
        filter.filter(&mut t3);

        assert_eq!(t3.start_offset, 2);
        assert_eq!(t3.end_offset, 3);
    }
}
