use alloc::string::String;
use alloc::vec::Vec;
use hashbrown::HashSet;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Removes duplicate tokens from the stream.
///
///
/// NOTE: State is tracked per-instance using interior mutability. Create a fresh
/// instance (or call `reset()`) for each document analysis run.
#[derive(Clone, Debug)]
pub struct UniqueTokenFilter {
    only_on_same_position: bool,
    seen: alloc::sync::Arc<std::sync::Mutex<HashSet<String>>>,
}

impl UniqueTokenFilter {
    pub fn new() -> Self {
        Self {
            only_on_same_position: false,
            seen: alloc::sync::Arc::new(std::sync::Mutex::new(HashSet::new())),
        }
    }

    /// Only remove duplicates at the same position (remove_duplicates behavior).
    pub fn only_on_same_position() -> Self {
        Self {
            only_on_same_position: true,
            seen: alloc::sync::Arc::new(std::sync::Mutex::new(HashSet::new())),
        }
    }

    /// Reset state between documents.
    pub fn reset(&self) {
        self.seen.lock().unwrap().clear();
    }
}

impl Default for UniqueTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for UniqueTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let term = token.term.to_string();
        let mut seen = self.seen.lock().unwrap();
        if seen.contains(&term) {
            return (true, None);
        }
        seen.insert(term);
        (false, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unique() {
        let f = UniqueTokenFilter::new();
        let mut t1 = Token::new("hello", 0, 5, 0);
        let mut t2 = Token::new("world", 6, 11, 1);
        let mut t3 = Token::new("hello", 12, 17, 2);

        assert_eq!(f.filter(&mut t1).0, false);
        assert_eq!(f.filter(&mut t2).0, false);
        assert_eq!(f.filter(&mut t3).0, true); // duplicate
    }
}
