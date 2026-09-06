use alloc::borrow::Cow;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use hashbrown::HashSet;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;
use std::sync::Mutex;

/// Creates bigrams of adjacent tokens when one or both are "common words"
/// (similar to stop words).
///
/// Instead of removing common/stop words entirely, this filter creates bigrams
/// containing them. This preserves phrase-query capability while reducing the
/// index impact of very frequent terms.
///
/// For example, with common words ["the", "of"], the tokens ["city", "of", "new", "york"]
/// produce additional bigram tokens: ["city_of", "of_new"].
///
/// Uses interior mutability to track the previous token across calls to `filter()`.
#[derive(Clone, Debug)]
pub struct CommonGramsTokenFilter {
    common_words: HashSet<String>,
    ignore_case: bool,
    separator: String,
    state: Arc<Mutex<CommonGramsState>>,
}

#[derive(Clone, Debug, Default)]
struct CommonGramsState {
    prev_term: Option<String>,
    prev_was_common: bool,
    prev_start_offset: u32,
    prev_end_offset: u32,
    prev_position: u32,
}

impl CommonGramsTokenFilter {
    pub fn new(words: Vec<String>) -> Self {
        Self {
            common_words: words.into_iter().collect(),
            ignore_case: false,
            separator: String::from("_"),
            state: Arc::new(Mutex::new(CommonGramsState::default())),
        }
    }

    pub fn with_ignore_case(mut self, ignore_case: bool) -> Self {
        if ignore_case {
            self.common_words = self.common_words.iter().map(|w| w.to_lowercase()).collect();
        }
        self.ignore_case = ignore_case;
        self
    }

    pub fn with_separator(mut self, sep: &str) -> Self {
        self.separator = String::from(sep);
        self
    }

    fn is_common(&self, term: &str) -> bool {
        if self.ignore_case {
            self.common_words.contains(&term.to_lowercase())
        } else {
            self.common_words.contains(term)
        }
    }

    /// Reset state between documents.
    pub fn reset(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.prev_term = None;
            state.prev_was_common = false;
            state.prev_start_offset = 0;
            state.prev_end_offset = 0;
            state.prev_position = 0;
        }
    }
}

impl TokenFilter for CommonGramsTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let term = token.term.as_ref();
        let current_is_common = self.is_common(term);

        let mut state = match self.state.lock() {
            Ok(s) => s,
            Err(_) => return (false, None),
        };

        // Lucene's CommonGramsFilter anchors the bigram at the FIRST (previous)
        // token's position with offsets spanning from prev.start to current.end.
        // Emitting at current.position would shift phrase positions by one.
        let bigram = if let Some(ref prev) = state.prev_term {
            if state.prev_was_common || current_is_common {
                let mut gram = prev.clone();
                gram.push_str(&self.separator);
                gram.push_str(term);
                Some(Token {
                    term: Cow::Owned(gram),
                    start_offset: state.prev_start_offset,
                    end_offset: token.end_offset,
                    position: state.prev_position,
                })
            } else {
                None
            }
        } else {
            None
        };

        state.prev_term = Some(term.to_owned());
        state.prev_was_common = current_is_common;
        state.prev_start_offset = token.start_offset;
        state.prev_end_offset = token.end_offset;
        state.prev_position = token.position;

        match bigram {
            Some(bigram_token) => (false, Some(alloc::vec![bigram_token])),
            None => (false, None),
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

    #[test]
    fn test_common_grams_filter() {
        let filter = CommonGramsTokenFilter::new(vec!["the".to_string(), "of".to_string()]);

        let mut t1 = make_token("city", 0);
        let (_, extra) = filter.filter(&mut t1);
        assert!(extra.is_none()); // no previous token

        let mut t2 = make_token("of", 1);
        let (_, extra) = filter.filter(&mut t2);
        let extra = extra.unwrap();
        assert_eq!(extra[0].term.as_ref(), "city_of"); // "of" is common

        let mut t3 = make_token("new", 2);
        let (_, extra) = filter.filter(&mut t3);
        let extra = extra.unwrap();
        assert_eq!(extra[0].term.as_ref(), "of_new"); // prev "of" was common

        let mut t4 = make_token("york", 3);
        let (_, extra) = filter.filter(&mut t4);
        assert!(extra.is_none()); // neither "new" nor "york" is common
    }

    #[test]
    fn test_common_grams_ignore_case() {
        let filter = CommonGramsTokenFilter::new(vec!["the".to_string()]).with_ignore_case(true);

        let mut t1 = make_token("hello", 0);
        filter.filter(&mut t1);

        let mut t2 = make_token("THE", 1);
        let (_, extra) = filter.filter(&mut t2);
        let extra = extra.unwrap();
        assert_eq!(extra[0].term.as_ref(), "hello_THE");
    }

    #[test]
    fn test_common_grams_both_common() {
        let filter = CommonGramsTokenFilter::new(vec!["the".to_string(), "of".to_string()]);

        let mut t1 = make_token("the", 0);
        filter.filter(&mut t1);

        let mut t2 = make_token("of", 1);
        let (_, extra) = filter.filter(&mut t2);
        let extra = extra.unwrap();
        assert_eq!(extra[0].term.as_ref(), "the_of"); // both common
    }

    #[test]
    fn test_common_grams_custom_separator() {
        let filter = CommonGramsTokenFilter::new(vec!["of".to_string()]).with_separator("-");

        let mut t1 = make_token("city", 0);
        filter.filter(&mut t1);

        let mut t2 = make_token("of", 1);
        let (_, extra) = filter.filter(&mut t2);
        assert_eq!(extra.unwrap()[0].term.as_ref(), "city-of");
    }

    #[test]
    fn test_reset() {
        let filter = CommonGramsTokenFilter::new(vec!["the".to_string()]);

        let mut t1 = make_token("the", 0);
        filter.filter(&mut t1);
        filter.reset();

        // After reset, "world" should not produce bigram with "the"
        let mut t2 = make_token("world", 0);
        let (_, extra) = filter.filter(&mut t2);
        assert!(extra.is_none());
    }

    #[test]
    fn test_bigram_anchored_at_first_token() {
        // Lucene's CommonGramsFilter emits the bigram at the FIRST (previous)
        // token's position with offsets spanning prev.start..current.end.
        // The previous implementation used the current token's position/offsets,
        // which broke phrase queries spanning common words.
        let filter = CommonGramsTokenFilter::new(vec!["of".to_string()]);

        let mut t1 = Token {
            term: Cow::Borrowed("city"),
            start_offset: 0,
            end_offset: 4,
            position: 0,
        };
        filter.filter(&mut t1);

        let mut t2 = Token {
            term: Cow::Borrowed("of"),
            start_offset: 5,
            end_offset: 7,
            position: 1,
        };
        let (_, extra) = filter.filter(&mut t2);
        let extra = extra.unwrap();
        assert_eq!(extra[0].term.as_ref(), "city_of");
        assert_eq!(
            extra[0].start_offset, 0,
            "bigram start = first token's start"
        );
        assert_eq!(extra[0].end_offset, 7, "bigram end = second token's end");
        assert_eq!(
            extra[0].position, 0,
            "bigram position = first token's position"
        );
    }
}
