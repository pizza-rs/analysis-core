use pizza_engine::analysis::Normalizer;

/// Trim normalizer - strips leading and trailing whitespace.
#[derive(Clone, Debug, Default)]
pub struct TrimNormalizer;

impl TrimNormalizer {
    pub fn new() -> Self {
        Self
    }
}

impl Normalizer for TrimNormalizer {
    fn normalize(&self, text: &mut String) {
        let trimmed = text.trim();
        if trimmed.len() != text.len() {
            *text = trimmed.to_owned();
        }
    }
}

/// Whitespace collapse normalizer - replaces sequences of whitespace with a single space.
#[derive(Clone, Debug, Default)]
pub struct CollapseWhitespaceNormalizer;

impl CollapseWhitespaceNormalizer {
    pub fn new() -> Self {
        Self
    }
}

impl Normalizer for CollapseWhitespaceNormalizer {
    fn normalize(&self, text: &mut String) {
        let mut result = String::with_capacity(text.len());
        let mut prev_was_space = false;

        for ch in text.chars() {
            if ch.is_whitespace() {
                if !prev_was_space {
                    result.push(' ');
                    prev_was_space = true;
                }
            } else {
                result.push(ch);
                prev_was_space = false;
            }
        }

        if result != *text {
            *text = result;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // LowercaseNormalizer / UppercaseNormalizer are re-exported from
    // pizza_engine in `super::super::mod`; bring them in explicitly so the
    // test module compiles standalone (the parent module's `pub use`
    // does not flow into a child module's namespace).
    use pizza_engine::analysis::LowercaseNormalizer;
    use pizza_engine::analysis::UppercaseNormalizer;

    #[test]
    fn test_lowercase() {
        let n = LowercaseNormalizer;
        let mut s = "Hello WORLD".to_string();
        n.normalize(&mut s);
        assert_eq!(s, "hello world");
    }

    #[test]
    fn test_uppercase() {
        let n = UppercaseNormalizer;
        let mut s = "Hello World".to_string();
        n.normalize(&mut s);
        assert_eq!(s, "HELLO WORLD");
    }

    #[test]
    fn test_trim() {
        let n = TrimNormalizer;
        let mut s = "  hello  ".to_string();
        n.normalize(&mut s);
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_collapse_whitespace() {
        let n = CollapseWhitespaceNormalizer;
        let mut s = "hello   world\t\nfoo".to_string();
        n.normalize(&mut s);
        assert_eq!(s, "hello world foo");
    }
}
