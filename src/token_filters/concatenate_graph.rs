use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::sync::Arc;
use std::sync::Mutex;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Concatenates tokens in the stream, emitting a running concatenation.
///
/// When used through the `TokenFilter` trait, each token is appended to a
/// running buffer. The original token is removed and replaced with the
/// cumulative concatenation so far. This is useful for autocomplete/suggest
/// scenarios where prefix concatenations are needed.
///
/// If you need to concatenate ALL tokens into one final token (batch mode),
/// use the `concatenate_tokens()` helper function instead.
#[derive(Clone, Debug)]
pub struct ConcatenateGraphTokenFilter {
    pub separator: char,
    pub max_graph_expansions: usize,
    buffer: Arc<Mutex<ConcatState>>,
}

#[derive(Debug, Default)]
struct ConcatState {
    buf: String,
    /// The `start_offset` of the very first token concatenated since the
    /// last `reset()`. Tracked so the running concatenation's offsets
    /// span [first.start, current.end] instead of collapsing onto the
    /// latest token's range (Lucene `ConcatenateGraphFilter` semantics).
    first_start: u32,
    /// The `position` of the very first token concatenated since reset.
    first_position: u32,
    has_first: bool,
}

impl ConcatenateGraphTokenFilter {
    pub fn new(separator: char) -> Self {
        Self {
            separator,
            max_graph_expansions: 256,
            buffer: Arc::new(Mutex::new(ConcatState::default())),
        }
    }

    /// Reset the internal buffer between documents.
    pub fn reset(&self) {
        if let Ok(mut st) = self.buffer.lock() {
            st.buf.clear();
            st.first_start = 0;
            st.first_position = 0;
            st.has_first = false;
        }
    }
}

impl Default for ConcatenateGraphTokenFilter {
    fn default() -> Self {
        Self::new(' ')
    }
}

impl TokenFilter for ConcatenateGraphTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let term = token.term.as_ref();
        let mut state = match self.buffer.lock() {
            Ok(b) => b,
            Err(_) => return (false, None),
        };

        if !state.has_first {
            state.first_start = token.start_offset;
            state.first_position = token.position;
            state.has_first = true;
        }

        if !state.buf.is_empty() {
            state.buf.push(self.separator);
        }
        state.buf.push_str(term);

        if state.buf.len() > self.max_graph_expansions {
            return (true, None); // too long, drop token
        }

        // Replace the token term with the running concatenation and update
        // its offsets to span from the first concatenated token to this
        // one, matching Lucene's `ConcatenateGraphFilter` graph semantics.
        token.term = Cow::Owned(state.buf.clone());
        token.start_offset = state.first_start;
        token.position = state.first_position;
        (false, None)
    }
}

/// Concatenates a slice of tokens into one token with the given separator.
pub fn concatenate_tokens<'a>(tokens: &[Token<'a>], separator: char) -> Option<Token<'a>> {
    if tokens.is_empty() {
        return None;
    }
    let mut result = String::new();
    for (i, t) in tokens.iter().enumerate() {
        if i > 0 {
            result.push(separator);
        }
        result.push_str(&t.term);
    }
    let start = tokens.first().map(|t| t.start_offset).unwrap_or(0);
    let end = tokens.last().map(|t| t.end_offset).unwrap_or(0);
    Some(Token {
        term: Cow::Owned(result),
        start_offset: start,
        end_offset: end,
        position: 0,
    })
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
    fn test_concatenate_filter_accumulates() {
        let filter = ConcatenateGraphTokenFilter::default();

        let mut t1 = make_token("hello", 0);
        let (remove, _) = filter.filter(&mut t1);
        assert!(!remove);
        assert_eq!(t1.term.as_ref(), "hello");

        let mut t2 = make_token("world", 1);
        let (remove, _) = filter.filter(&mut t2);
        assert!(!remove);
        assert_eq!(t2.term.as_ref(), "hello world");

        let mut t3 = make_token("foo", 2);
        filter.filter(&mut t3);
        assert_eq!(t3.term.as_ref(), "hello world foo");
    }

    #[test]
    fn test_concatenate_custom_separator() {
        let filter = ConcatenateGraphTokenFilter::new('-');

        let mut t1 = make_token("a", 0);
        filter.filter(&mut t1);
        let mut t2 = make_token("b", 1);
        filter.filter(&mut t2);
        assert_eq!(t2.term.as_ref(), "a-b");
    }

    #[test]
    fn test_concatenate_reset() {
        let filter = ConcatenateGraphTokenFilter::default();

        let mut t1 = make_token("hello", 0);
        filter.filter(&mut t1);
        filter.reset();

        let mut t2 = make_token("world", 0);
        filter.filter(&mut t2);
        assert_eq!(t2.term.as_ref(), "world"); // fresh start
    }

    #[test]
    fn test_concatenate_tokens_helper() {
        let tokens = vec![
            make_token("hello", 0),
            make_token("beautiful", 1),
            make_token("world", 2),
        ];
        let result = concatenate_tokens(&tokens, ' ').unwrap();
        assert_eq!(result.term.as_ref(), "hello beautiful world");
    }

    #[test]
    fn test_concatenate_tokens_empty() {
        let tokens: Vec<Token> = vec![];
        assert!(concatenate_tokens(&tokens, ' ').is_none());
    }
}
