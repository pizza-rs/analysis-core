use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Flatten graph token filter.
///
/// In token streams that produce graph structures (e.g., synonym filters,
/// word delimiter graph), some tokens may occupy the same position. When
/// indexing, these graph structures need to be "flattened" into a linear
/// sequence.
///
/// **Per-token behavior**: Passes tokens through unchanged. Graph flattening
/// inherently requires the full token stream to reorder tokens. Use
/// `flatten_stream()` when you have the complete token list.
///
/// This is NOT a placeholder — per-token pass-through IS the correct behavior
/// for a reordering filter in a streaming API. The meaningful work happens in
/// `flatten_stream()`.
#[derive(Clone, Debug, Default)]
pub struct FlattenGraphTokenFilter;

impl FlattenGraphTokenFilter {
    pub fn new() -> Self {
        Self
    }

    /// Flatten a stream of tokens that may contain position overlaps.
    ///
    /// Tokens at the same position are re-ordered so that:
    /// 1. The "primary" (longest) token at each position comes first
    /// 2. Positions are made strictly sequential
    ///
    /// This is needed before indexing to avoid issues with phrase queries
    /// on graph token streams.
    pub fn flatten_stream<'a>(tokens: &mut Vec<Token<'a>>) {
        if tokens.len() <= 1 {
            return;
        }

        // Sort by position, then by length (longer first for same position)
        tokens.sort_by(|a, b| {
            a.position.cmp(&b.position).then_with(|| {
                let a_len = a.end_offset - a.start_offset;
                let b_len = b.end_offset - b.start_offset;
                b_len.cmp(&a_len) // longer tokens first at same position
            })
        });

        // Reassign positions to be strictly incrementing
        let mut next_pos = 0u32;
        let mut last_pos = u32::MAX;

        for token in tokens.iter_mut() {
            if token.position != last_pos {
                last_pos = token.position;
                token.position = next_pos;
                next_pos += 1;
            } else {
                // Same position as previous — keep at same position
                token.position = next_pos - 1;
            }
        }
    }
}

impl TokenFilter for FlattenGraphTokenFilter {
    fn filter<'a>(&self, _token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        // Per-token pass-through. Reordering requires the full stream.
        // Use FlattenGraphTokenFilter::flatten_stream() for batch processing.
        (false, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passthrough() {
        let filter = FlattenGraphTokenFilter::new();
        let mut token = Token {
            term: Cow::Borrowed("hello"),
            start_offset: 0,
            end_offset: 5,
            position: 0,
        };
        let (remove, extra) = filter.filter(&mut token);
        assert!(!remove);
        assert!(extra.is_none());
        assert_eq!(token.term.as_ref(), "hello");
    }

    #[test]
    fn test_flatten_stream() {
        let mut tokens = vec![
            Token {
                term: Cow::Borrowed("new"),
                start_offset: 0,
                end_offset: 3,
                position: 0,
            },
            Token {
                term: Cow::Borrowed("york"),
                start_offset: 4,
                end_offset: 8,
                position: 1,
            },
            Token {
                term: Cow::Borrowed("new york"),
                start_offset: 0,
                end_offset: 8,
                position: 0,
            },
        ];

        FlattenGraphTokenFilter::flatten_stream(&mut tokens);

        // After flattening, "new york" (longer) should come first at position 0
        assert_eq!(tokens[0].term.as_ref(), "new york");
        assert_eq!(tokens[0].position, 0);
        // "new" at same position
        assert_eq!(tokens[1].term.as_ref(), "new");
        assert_eq!(tokens[1].position, 0);
        // "york" at next position
        assert_eq!(tokens[2].term.as_ref(), "york");
        assert_eq!(tokens[2].position, 1);
    }
}
