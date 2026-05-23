use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Predicate script detection for token classification.
#[derive(Clone, Debug, PartialEq)]
pub enum ScriptType {
    Latin,
    Cjk,
    Cyrillic,
    Arabic,
    Greek,
    Hebrew,
    Devanagari,
    Thai,
    Numeric,
    Punctuation,
    Unknown,
}

/// Detect the dominant script of a string.
pub fn detect_script(text: &str) -> ScriptType {
    let mut latin = 0u32;
    let mut cjk = 0u32;
    let mut cyrillic = 0u32;
    let mut arabic = 0u32;
    let mut greek = 0u32;
    let mut hebrew = 0u32;
    let mut devanagari = 0u32;
    let mut thai = 0u32;
    let mut numeric = 0u32;
    let mut punct = 0u32;

    for ch in text.chars() {
        if ch.is_ascii_alphabetic() || ('\u{00C0}'..='\u{024F}').contains(&ch) {
            latin += 1;
        } else if ('\u{4E00}'..='\u{9FFF}').contains(&ch)
            || ('\u{3400}'..='\u{4DBF}').contains(&ch)
            || ('\u{3040}'..='\u{309F}').contains(&ch)
            || ('\u{30A0}'..='\u{30FF}').contains(&ch)
            || ('\u{AC00}'..='\u{D7AF}').contains(&ch)
        {
            cjk += 1;
        } else if ('\u{0400}'..='\u{04FF}').contains(&ch) {
            cyrillic += 1;
        } else if ('\u{0600}'..='\u{06FF}').contains(&ch) || ('\u{0750}'..='\u{077F}').contains(&ch)
        {
            arabic += 1;
        } else if ('\u{0370}'..='\u{03FF}').contains(&ch) {
            greek += 1;
        } else if ('\u{0590}'..='\u{05FF}').contains(&ch) {
            hebrew += 1;
        } else if ('\u{0900}'..='\u{097F}').contains(&ch) {
            devanagari += 1;
        } else if ('\u{0E00}'..='\u{0E7F}').contains(&ch) {
            thai += 1;
        } else if ch.is_numeric() {
            numeric += 1;
        } else if ch.is_ascii_punctuation() || !ch.is_alphanumeric() {
            punct += 1;
        }
    }

    let counts = [
        (latin, ScriptType::Latin),
        (cjk, ScriptType::Cjk),
        (cyrillic, ScriptType::Cyrillic),
        (arabic, ScriptType::Arabic),
        (greek, ScriptType::Greek),
        (hebrew, ScriptType::Hebrew),
        (devanagari, ScriptType::Devanagari),
        (thai, ScriptType::Thai),
        (numeric, ScriptType::Numeric),
        (punct, ScriptType::Punctuation),
    ];

    counts
        .iter()
        .max_by_key(|(count, _)| *count)
        .filter(|(count, _)| *count > 0)
        .map(|(_, script)| script.clone())
        .unwrap_or(ScriptType::Unknown)
}

/// Token predicate types for filtering.
#[derive(Clone, Debug)]
pub enum TokenPredicateType {
    /// Keep only tokens of the given script type.
    IsScript(ScriptType),
    /// Remove tokens of the given script type.
    IsNotScript(ScriptType),
    /// Keep only tokens matching minimum length.
    MinLength(usize),
    /// Keep only tokens matching maximum length.
    MaxLength(usize),
    /// Keep only tokens that are purely numeric.
    IsNumeric,
    /// Remove tokens that are purely numeric.
    IsNotNumeric,
    /// Keep only tokens that are purely alphabetic.
    IsAlphabetic,
}

/// Predicate-based token filter that removes tokens based on configurable rules.
///
/// Useful for filtering tokens by script, character type, or length.
#[derive(Clone, Debug)]
pub struct PredicateTokenFilter {
    predicates: Vec<TokenPredicateType>,
}

impl PredicateTokenFilter {
    pub fn new(predicates: Vec<TokenPredicateType>) -> Self {
        Self { predicates }
    }

    /// Keep only tokens of the given script.
    pub fn keep_script(script: ScriptType) -> Self {
        Self::new(vec![TokenPredicateType::IsScript(script)])
    }

    /// Remove tokens of the given script.
    pub fn remove_script(script: ScriptType) -> Self {
        Self::new(vec![TokenPredicateType::IsNotScript(script)])
    }

    fn should_remove(&self, token: &Token<'_>) -> bool {
        let text = token.term.as_ref();

        for pred in &self.predicates {
            let passes = match pred {
                TokenPredicateType::IsScript(script) => &detect_script(text) == script,
                TokenPredicateType::IsNotScript(script) => &detect_script(text) != script,
                TokenPredicateType::MinLength(min) => text.len() >= *min,
                TokenPredicateType::MaxLength(max) => text.len() <= *max,
                TokenPredicateType::IsNumeric => text.chars().all(|c| c.is_numeric()),
                TokenPredicateType::IsNotNumeric => !text.chars().all(|c| c.is_numeric()),
                TokenPredicateType::IsAlphabetic => text.chars().all(|c| c.is_alphabetic()),
            };
            if !passes {
                return true; // Token fails a predicate → remove it
            }
        }

        false
    }
}

impl TokenFilter for PredicateTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        (self.should_remove(token), None)
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
    fn test_detect_latin() {
        assert_eq!(detect_script("hello"), ScriptType::Latin);
    }

    #[test]
    fn test_detect_cjk() {
        assert_eq!(detect_script("你好"), ScriptType::Cjk);
    }

    #[test]
    fn test_detect_cyrillic() {
        assert_eq!(detect_script("привет"), ScriptType::Cyrillic);
    }

    #[test]
    fn test_keep_latin_only() {
        let filter = PredicateTokenFilter::keep_script(ScriptType::Latin);
        let mut latin = make_token("hello");
        let mut cjk = make_token("你好");
        let (r1, _) = filter.filter(&mut latin);
        let (r2, _) = filter.filter(&mut cjk);
        assert!(!r1); // keep latin
        assert!(r2); // remove non-latin
    }

    #[test]
    fn test_remove_numeric() {
        let filter = PredicateTokenFilter::new(vec![TokenPredicateType::IsNotNumeric]);
        let mut word = make_token("hello");
        let mut num = make_token("12345");
        let (r1, _) = filter.filter(&mut word);
        let (r2, _) = filter.filter(&mut num);
        assert!(!r1); // keep non-numeric
        assert!(r2); // remove numeric
    }

    #[test]
    fn test_combined_predicates() {
        let filter = PredicateTokenFilter::new(vec![
            TokenPredicateType::MinLength(3),
            TokenPredicateType::IsAlphabetic,
        ]);
        let mut long_alpha = make_token("hello");
        let mut short = make_token("hi");
        let mut with_num = make_token("abc123");
        let (r1, _) = filter.filter(&mut long_alpha);
        let (r2, _) = filter.filter(&mut short);
        let (r3, _) = filter.filter(&mut with_num);
        assert!(!r1); // passes both
        assert!(r2); // too short
        assert!(r3); // not purely alphabetic
    }
}
