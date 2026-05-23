use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use hashbrown::HashMap;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Synonym expansion mode.
#[derive(Clone, Debug, PartialEq)]
pub enum SynonymMode {
    /// Expand: all synonyms are produced at the same position (expand mode).
    Expand,
    /// Contract: map all synonyms to a single canonical form.
    Contract,
}

/// A synonym rule mapping a source term to one or more replacements.
#[derive(Clone, Debug)]
struct SynonymRule {
    replacements: Vec<String>,
    mode: SynonymMode,
}

/// Applies synonym expansion or contraction to tokens.
///
/// Supports both explicit mapping (a => b, c) and equivalence (a, b, c) formats.
#[derive(Clone, Debug)]
pub struct SynonymTokenFilter {
    rules: HashMap<String, SynonymRule>,
    ignore_case: bool,
}

impl SynonymTokenFilter {
    /// Create a new empty SynonymTokenFilter.
    pub fn new(ignore_case: bool) -> Self {
        Self {
            rules: HashMap::new(),
            ignore_case,
        }
    }

    /// Add an explicit mapping: `source => replacement1, replacement2, ...`
    /// In expand mode, source emits all replacements at the same position.
    /// In contract mode, source is replaced by the first replacement.
    pub fn add_mapping(&mut self, source: &str, replacements: &[&str], mode: SynonymMode) {
        let key = if self.ignore_case {
            source.to_lowercase()
        } else {
            source.to_string()
        };
        self.rules.insert(
            key,
            SynonymRule {
                replacements: replacements.iter().map(|s| s.to_string()).collect(),
                mode,
            },
        );
    }

    /// Add an equivalence set: all terms map to each other (expand mode).
    /// e.g. `["fast", "quick", "speedy"]` means each term produces all others.
    pub fn add_equivalence(&mut self, terms: &[&str]) {
        for &source in terms {
            let others: Vec<String> = terms
                .iter()
                .filter(|&&t| t != source)
                .map(|&t| t.to_string())
                .collect();
            let key = if self.ignore_case {
                source.to_lowercase()
            } else {
                source.to_string()
            };
            self.rules.insert(
                key,
                SynonymRule {
                    replacements: others,
                    mode: SynonymMode::Expand,
                },
            );
        }
    }

    /// Parse a synonym definition line.
    /// Supports:
    /// - `a, b, c` (equivalence)
    /// - `a => b, c` (explicit mapping, expand)
    pub fn parse_line(&mut self, line: &str) {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            return;
        }

        if let Some((left, right)) = line.split_once("=>") {
            // Explicit mapping
            let source = left.trim();
            let replacements: Vec<&str> = right
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            if !source.is_empty() && !replacements.is_empty() {
                self.add_mapping(source, &replacements, SynonymMode::Expand);
            }
        } else {
            // Equivalence
            let terms: Vec<&str> = line
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            if terms.len() >= 2 {
                self.add_equivalence(&terms);
            }
        }
    }

    /// Parse multiple synonym lines (e.g. from a file).
    pub fn parse_rules(&mut self, text: &str) {
        for line in text.lines() {
            self.parse_line(line);
        }
    }
}

impl TokenFilter for SynonymTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let lookup_key = if self.ignore_case {
            token.term.to_lowercase()
        } else {
            token.term.to_string()
        };

        if let Some(rule) = self.rules.get(&lookup_key) {
            match rule.mode {
                SynonymMode::Expand => {
                    // Keep original token, emit synonyms at same position
                    let extras: Vec<Token<'a>> = rule
                        .replacements
                        .iter()
                        .map(|syn| Token {
                            term: Cow::Owned(syn.clone()),
                            start_offset: token.start_offset,
                            end_offset: token.end_offset,
                            position: token.position,
                        })
                        .collect();
                    (
                        false,
                        if extras.is_empty() {
                            None
                        } else {
                            Some(extras)
                        },
                    )
                }
                SynonymMode::Contract => {
                    // Replace token with first replacement
                    if let Some(canonical) = rule.replacements.first() {
                        token.term = Cow::Owned(canonical.clone());
                    }
                    (false, None)
                }
            }
        } else {
            (false, None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_synonyms() {
        let mut f = SynonymTokenFilter::new(true);
        f.add_equivalence(&["fast", "quick", "speedy"]);

        let mut token = Token::new("fast", 0, 4, 0);
        let (remove, extra) = f.filter(&mut token);
        assert!(!remove);
        assert_eq!(token.term, "fast"); // original kept
        let extra = extra.unwrap();
        assert_eq!(extra.len(), 2);
        let terms: Vec<&str> = extra.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"quick"));
        assert!(terms.contains(&"speedy"));
    }

    #[test]
    fn test_contract_synonyms() {
        let mut f = SynonymTokenFilter::new(true);
        f.add_mapping("quick", &["fast"], SynonymMode::Contract);
        f.add_mapping("speedy", &["fast"], SynonymMode::Contract);

        let mut token = Token::new("quick", 0, 5, 0);
        let (remove, extra) = f.filter(&mut token);
        assert!(!remove);
        assert!(extra.is_none());
        assert_eq!(token.term, "fast");
    }

    #[test]
    fn test_parse_equivalence_line() {
        let mut f = SynonymTokenFilter::new(false);
        f.parse_line("big, large, huge");

        let mut token = Token::new("big", 0, 3, 0);
        let (_, extra) = f.filter(&mut token);
        let extra = extra.unwrap();
        assert_eq!(extra.len(), 2);
    }

    #[test]
    fn test_parse_explicit_mapping() {
        let mut f = SynonymTokenFilter::new(false);
        f.parse_line("ipod => i-pod, i pod");

        let mut token = Token::new("ipod", 0, 4, 0);
        let (_, extra) = f.filter(&mut token);
        let extra = extra.unwrap();
        assert_eq!(extra.len(), 2);
    }

    #[test]
    fn test_no_match() {
        let f = SynonymTokenFilter::new(false);
        let mut token = Token::new("hello", 0, 5, 0);
        let (remove, extra) = f.filter(&mut token);
        assert!(!remove);
        assert!(extra.is_none());
    }

    #[test]
    fn test_case_insensitive() {
        let mut f = SynonymTokenFilter::new(true);
        f.add_equivalence(&["fast", "quick"]);

        let mut token = Token::new("FAST", 0, 4, 0);
        let (_, extra) = f.filter(&mut token);
        let extra = extra.unwrap();
        assert_eq!(extra[0].term, "quick");
    }
}
