use alloc::string::String;
use pizza_engine::analysis::Normalizer;
use regex::Regex;

/// Replaces occurrences of a regex pattern with a replacement string.
///
/// Regex-based pattern replacement normalizer.
#[derive(Clone, Debug)]
pub struct PatternReplaceNormalizer {
    pattern: Regex,
    replacement: String,
}

impl PatternReplaceNormalizer {
    /// Create a new PatternReplaceNormalizer.
    ///
    /// # Panics
    /// Panics if the pattern is not a valid regex.
    pub fn new(pattern: &str, replacement: &str) -> Self {
        Self {
            pattern: Regex::new(pattern).expect("invalid regex pattern"),
            replacement: replacement.to_string(),
        }
    }

    /// Create from a pre-compiled Regex.
    pub fn from_regex(pattern: Regex, replacement: String) -> Self {
        Self {
            pattern,
            replacement,
        }
    }
}

impl Normalizer for PatternReplaceNormalizer {
    fn normalize(&self, text: &mut String) {
        let result = self.pattern.replace_all(text, self.replacement.as_str());
        if let alloc::borrow::Cow::Owned(replaced) = result {
            *text = replaced;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_digits() {
        let n = PatternReplaceNormalizer::new(r"\d+", "NUM");
        let mut text = String::from("order 123 and 456");
        n.normalize(&mut text);
        assert_eq!(text, "order NUM and NUM");
    }

    #[test]
    fn test_strip_non_alpha() {
        let n = PatternReplaceNormalizer::new(r"[^a-zA-Z\s]", "");
        let mut text = String::from("hello, world! 123");
        n.normalize(&mut text);
        assert_eq!(text, "hello world ");
    }

    #[test]
    fn test_no_match() {
        let n = PatternReplaceNormalizer::new(r"xyz", "abc");
        let mut text = String::from("hello world");
        n.normalize(&mut text);
        assert_eq!(text, "hello world");
    }

    #[test]
    fn test_capture_groups() {
        let n = PatternReplaceNormalizer::new(r"(\w+)@(\w+)", "$1 at $2");
        let mut text = String::from("user@host");
        n.normalize(&mut text);
        assert_eq!(text, "user at host");
    }
}
