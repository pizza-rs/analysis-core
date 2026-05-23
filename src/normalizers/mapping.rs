use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Normalizer;

/// Character mapping normalizer — configurable string substitutions.
///
/// Character mapping normalizer, implemented
/// as a Pizza normalizer.
#[derive(Clone, Debug)]
pub struct MappingNormalizer {
    mappings: Vec<(String, String)>,
}

impl MappingNormalizer {
    pub fn new() -> Self {
        Self {
            mappings: Vec::new(),
        }
    }

    /// Create from a list of (from, to) mapping pairs.
    pub fn from_mappings(mappings: &[(&str, &str)]) -> Self {
        Self {
            mappings: mappings
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }

    pub fn add_mapping(&mut self, from: &str, to: &str) {
        self.mappings.push((from.to_string(), to.to_string()));
    }
}

impl Default for MappingNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Normalizer for MappingNormalizer {
    fn normalize(&self, text: &mut String) {
        // `replace` already scans the entire string, so the previous
        // `text.contains(from)` guard was a redundant second scan per
        // mapping. Calling `replace` directly returns the same string
        // unchanged when no occurrence is found.
        for (from, to) in &self.mappings {
            if !from.is_empty() {
                *text = text.replace(from.as_str(), to.as_str());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapping_normalizer() {
        let n = MappingNormalizer::from_mappings(&[(":)", "happy"), (":(", "sad")]);
        let mut text = String::from("I am :) today");
        n.normalize(&mut text);
        assert_eq!(text, "I am happy today");
    }

    #[test]
    fn test_mapping_multiple() {
        let n = MappingNormalizer::from_mappings(&[("α", "a"), ("β", "b")]);
        let mut text = String::from("αβγ");
        n.normalize(&mut text);
        assert_eq!(text, "abγ");
    }
}
