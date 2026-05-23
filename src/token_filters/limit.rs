use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::AtomicU32;
use core::sync::atomic::Ordering;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Limits the number of tokens that pass through.
#[derive(Clone, Debug)]
pub struct LimitTokenFilter {
    pub max_token_count: u32,
    counter: Arc<AtomicU32>,
}

impl LimitTokenFilter {
    pub fn new(max_token_count: u32) -> Self {
        Self {
            max_token_count,
            counter: Arc::new(AtomicU32::new(0)),
        }
    }

    /// Reset the counter (call between documents).
    pub fn reset(&self) {
        self.counter.store(0, Ordering::Relaxed);
    }
}

impl TokenFilter for LimitTokenFilter {
    fn filter<'a>(&self, _token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let count = self.counter.fetch_add(1, Ordering::Relaxed);
        if count >= self.max_token_count {
            return (true, None);
        }
        (false, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limit_filter() {
        let f = LimitTokenFilter::new(2);
        let mut t1 = Token::new("a", 0, 1, 0);
        let mut t2 = Token::new("b", 2, 3, 1);
        let mut t3 = Token::new("c", 4, 5, 2);

        assert_eq!(f.filter(&mut t1).0, false);
        assert_eq!(f.filter(&mut t2).0, false);
        assert_eq!(f.filter(&mut t3).0, true); // 3rd token exceeds limit
    }
}
