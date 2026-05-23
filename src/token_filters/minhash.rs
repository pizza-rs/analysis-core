use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// MinHash token filter for near-duplicate detection.
///
/// Generates a fixed-size "signature" of a document's token stream that can be
/// used for locality-sensitive hashing (LSH). Documents with similar content
/// will produce similar MinHash signatures.
///
/// Each token is hashed with multiple hash functions, and the minimum hash
/// from each function becomes part of the signature.
///
/// Configuration:
/// - `hash_count`: Number of hash functions to use (default: 1)
/// - `bucket_count`: Number of buckets per hash (default: 512)
/// - `hash_set_size`: Number of min hashes to keep per bucket (default: 1)
/// - `with_rotation`: Whether to fill empty buckets with values from adjacent ones
///
/// # Implementation notes
///
/// The current `pizza_engine::analysis::TokenFilter` API is **per-token**
/// (it has no end-of-stream / flush hook), so this filter cannot
/// implement true Lucene-style MinHash semantics — true MinHash needs to
/// observe the *entire* token stream of a document and emit the minimum
/// hash per hash function as a single signature at the end. Until the
/// engine grows a flush API, this implementation produces a
/// **per-token hash bucket id** (still useful for shingle-based LSH
/// pipelines where the upstream filter has already emitted shingles as
/// the unit of comparison, but not equivalent to a document-level
/// MinHash signature). The `hash_set_size` and `with_rotation` fields
/// are accepted for API compatibility but are currently unused;
/// see the cross-review document for the engine-level fix this would
/// require.
#[derive(Clone, Debug)]
pub struct MinHashTokenFilter {
    hash_count: usize,
    bucket_count: usize,
    hash_set_size: usize,
    with_rotation: bool,
}

impl Default for MinHashTokenFilter {
    fn default() -> Self {
        Self {
            hash_count: 1,
            bucket_count: 512,
            hash_set_size: 1,
            with_rotation: true,
        }
    }
}

impl MinHashTokenFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_hash_count(mut self, count: usize) -> Self {
        self.hash_count = count.max(1);
        self
    }

    pub fn with_bucket_count(mut self, count: usize) -> Self {
        self.bucket_count = count.max(1);
        self
    }

    pub fn with_hash_set_size(mut self, size: usize) -> Self {
        self.hash_set_size = size.max(1);
        self
    }

    pub fn with_rotation(mut self, rotation: bool) -> Self {
        self.with_rotation = rotation;
        self
    }

    /// Compute a hash of the input token using a simple hash function variant.
    /// Uses the hash_index to create independent hash functions via seeding.
    fn hash_token(&self, text: &str, hash_index: usize) -> u64 {
        // FNV-1a variant with seed based on hash_index
        let mut hash: u64 =
            14695981039346656037_u64.wrapping_add(hash_index as u64 * 6364136223846793005);
        for byte in text.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(1099511628211);
        }
        hash
    }
}

impl TokenFilter for MinHashTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.to_string();

        if self.hash_count <= 1 {
            // Single hash mode: just output the hash as a hex string
            let h = self.hash_token(&text, 0);
            let bucket = h % self.bucket_count as u64;
            token.term = Cow::Owned(format!("{:016x}", bucket));
            return (false, None);
        }

        // Multi-hash mode: emit multiple hash tokens
        let mut extra = Vec::with_capacity(self.hash_count - 1);

        for i in 0..self.hash_count {
            let h = self.hash_token(&text, i);
            let bucket = h % self.bucket_count as u64;
            let hash_str = format!("{:016x}", bucket);

            if i == 0 {
                token.term = Cow::Owned(hash_str);
            } else {
                extra.push(Token {
                    term: Cow::Owned(hash_str),
                    start_offset: token.start_offset,
                    end_offset: token.end_offset,
                    position: token.position,
                });
            }
        }

        if extra.is_empty() {
            (false, None)
        } else {
            (false, Some(extra))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_token(term: &str) -> Token<'_> {
        Token {
            term: Cow::Borrowed(term),
            start_offset: 0,
            end_offset: term.len() as u32,
            position: 0,
        }
    }

    #[test]
    fn test_single_hash() {
        let filter = MinHashTokenFilter::new();
        let mut token = make_token("hello");
        let (remove, extra) = filter.filter(&mut token);
        assert!(!remove);
        assert!(extra.is_none());
        // Token should be transformed to a hex hash
        assert_eq!(token.term.len(), 16);
    }

    #[test]
    fn test_multi_hash() {
        let filter = MinHashTokenFilter::new().with_hash_count(3);
        let mut token = make_token("hello");
        let (remove, extra) = filter.filter(&mut token);
        assert!(!remove);
        let extra = extra.unwrap();
        assert_eq!(extra.len(), 2); // 3 hashes total, 1 in token + 2 extra
    }

    #[test]
    fn test_deterministic() {
        let filter = MinHashTokenFilter::new();
        let mut t1 = make_token("hello");
        let mut t2 = make_token("hello");
        filter.filter(&mut t1);
        filter.filter(&mut t2);
        assert_eq!(t1.term, t2.term);
    }

    #[test]
    fn test_different_inputs() {
        let filter = MinHashTokenFilter::new();
        let mut t1 = make_token("hello");
        let mut t2 = make_token("world");
        filter.filter(&mut t1);
        filter.filter(&mut t2);
        // Different inputs should generally produce different hashes
        // (not guaranteed but highly likely)
        assert_ne!(t1.term, t2.term);
    }
}
