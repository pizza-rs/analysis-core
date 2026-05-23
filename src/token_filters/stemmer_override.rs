use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use hashbrown::HashMap;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Dictionary-based stem override. Applies before algorithmic stemming
/// to handle irregular or exception cases.
///
/// If a token matches an entry in the override dictionary, it is replaced
/// with the specified stem and marked as a keyword (to prevent further stemming).
#[derive(Clone, Debug)]
pub struct StemmerOverrideTokenFilter {
    overrides: HashMap<String, String>,
    ignore_case: bool,
}

impl StemmerOverrideTokenFilter {
    pub fn new(overrides: HashMap<String, String>) -> Self {
        Self {
            overrides,
            ignore_case: false,
        }
    }

    /// Create from a list of "word => stem" pairs.
    pub fn from_rules(rules: &[(&str, &str)]) -> Self {
        let overrides: HashMap<String, String> = rules
            .iter()
            .map(|(word, stem)| (String::from(*word), String::from(*stem)))
            .collect();
        Self {
            overrides,
            ignore_case: false,
        }
    }

    pub fn with_ignore_case(mut self, ignore_case: bool) -> Self {
        if ignore_case {
            let new_map: HashMap<String, String> = self
                .overrides
                .iter()
                .map(|(k, v)| (k.to_lowercase(), v.clone()))
                .collect();
            self.overrides = new_map;
        }
        self.ignore_case = ignore_case;
        self
    }
}

impl TokenFilter for StemmerOverrideTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let term = token.term.as_ref();
        let lookup = if self.ignore_case {
            let lower = term.to_lowercase();
            self.overrides.get(lower.as_str()).cloned()
        } else {
            self.overrides.get(term).cloned()
        };

        if let Some(stem) = lookup {
            token.term = Cow::Owned(stem);
        }
        (false, None)
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
    fn test_override_match() {
        let filter =
            StemmerOverrideTokenFilter::from_rules(&[("running", "run"), ("mice", "mouse")]);
        let mut token = make_token("running");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "run");
    }

    #[test]
    fn test_no_match() {
        let filter = StemmerOverrideTokenFilter::from_rules(&[("running", "run")]);
        let mut token = make_token("walking");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "walking");
    }

    #[test]
    fn test_ignore_case() {
        let filter =
            StemmerOverrideTokenFilter::from_rules(&[("running", "run")]).with_ignore_case(true);
        let mut token = make_token("Running");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "run");
    }
}
