//! Hunspell morphological stemming filter.
//!
//! Provides stemming based on Hunspell dictionary rules (.dic + .aff files).
//! Hunspell uses affix rules to strip/replace prefixes and suffixes,
//! yielding the base form of words.
//!
//! This implementation provides a simplified rule-based approach that
//! processes Hunspell-style affix rules for stemming.

use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// A parsed Hunspell affix rule.
#[derive(Clone, Debug)]
pub struct AffixRule {
    /// Whether this is a suffix (true) or prefix (false).
    pub is_suffix: bool,
    /// Characters to strip from the word before applying.
    pub strip: String,
    /// Characters to add after stripping.
    pub affix: String,
    /// Condition pattern (simplified regex) the word must match.
    pub condition: String,
}

/// Hunspell-based stemming filter.
///
/// Strips affixes from tokens using Hunspell-style rules to produce stems.
/// Requires affix rules to be loaded (either from .aff files or pre-configured).
///
/// Equivalent to Elasticsearch's `hunspell` token filter.
#[derive(Clone, Debug)]
pub struct HunspellStemFilter {
    suffix_rules: Vec<AffixRule>,
    prefix_rules: Vec<AffixRule>,
    /// If true, remove duplicates from the output stems.
    dedup: bool,
    /// If true, only emit the longest stem.
    longest_only: bool,
}

impl HunspellStemFilter {
    /// Create an empty Hunspell filter (no rules loaded).
    pub fn new() -> Self {
        Self {
            suffix_rules: Vec::new(),
            prefix_rules: Vec::new(),
            dedup: true,
            longest_only: false,
        }
    }

    /// Create with pre-loaded affix rules.
    pub fn with_rules(rules: Vec<AffixRule>) -> Self {
        let mut suffix_rules = Vec::new();
        let mut prefix_rules = Vec::new();

        for rule in rules {
            if rule.is_suffix {
                suffix_rules.push(rule);
            } else {
                prefix_rules.push(rule);
            }
        }

        Self {
            suffix_rules,
            prefix_rules,
            dedup: true,
            longest_only: false,
        }
    }

    /// Set whether to deduplicate stems.
    pub fn with_dedup(mut self, dedup: bool) -> Self {
        self.dedup = dedup;
        self
    }

    /// Set whether to only emit the longest stem.
    pub fn with_longest_only(mut self, longest_only: bool) -> Self {
        self.longest_only = longest_only;
        self
    }

    /// Add a suffix rule.
    pub fn add_suffix_rule(&mut self, strip: &str, affix: &str, condition: &str) {
        self.suffix_rules.push(AffixRule {
            is_suffix: true,
            strip: strip.to_string(),
            affix: affix.to_string(),
            condition: condition.to_string(),
        });
    }

    /// Add a prefix rule.
    pub fn add_prefix_rule(&mut self, strip: &str, affix: &str, condition: &str) {
        self.prefix_rules.push(AffixRule {
            is_suffix: false,
            strip: strip.to_string(),
            affix: affix.to_string(),
            condition: condition.to_string(),
        });
    }

    /// Apply suffix rules to produce stems.
    fn apply_suffix_rules(&self, word: &str) -> Vec<String> {
        let mut stems = Vec::new();

        for rule in &self.suffix_rules {
            if rule.affix.is_empty() {
                continue;
            }
            // Check if word ends with the affix
            if word.ends_with(&rule.affix) {
                let base_len = word.len() - rule.affix.len();
                let base = &word[..base_len];

                // Check condition (simplified: just check minimum length)
                if base.len() < 2 {
                    continue;
                }

                // Build stem: base + strip chars
                let mut stem = base.to_string();
                if !rule.strip.is_empty() && rule.strip != "0" {
                    stem.push_str(&rule.strip);
                }
                stems.push(stem);
            }
        }

        stems
    }

    /// Apply prefix rules to produce stems.
    fn apply_prefix_rules(&self, word: &str) -> Vec<String> {
        let mut stems = Vec::new();

        for rule in &self.prefix_rules {
            if rule.affix.is_empty() {
                continue;
            }
            if word.starts_with(&rule.affix) {
                let rest = &word[rule.affix.len()..];
                if rest.len() < 2 {
                    continue;
                }
                let mut stem = String::new();
                if !rule.strip.is_empty() && rule.strip != "0" {
                    stem.push_str(&rule.strip);
                }
                stem.push_str(rest);
                stems.push(stem);
            }
        }

        stems
    }

    /// Stem a word, returning all possible stems.
    fn stem_word(&self, word: &str) -> Vec<String> {
        let lower = word.to_lowercase();
        let mut all_stems = Vec::new();

        // Apply suffix rules
        all_stems.extend(self.apply_suffix_rules(&lower));

        // Apply prefix rules
        all_stems.extend(self.apply_prefix_rules(&lower));

        if self.dedup {
            all_stems.sort();
            all_stems.dedup();
        }

        if self.longest_only && !all_stems.is_empty() {
            all_stems.sort_by(|a, b| b.len().cmp(&a.len()));
            all_stems.truncate(1);
        }

        all_stems
    }
}

impl Default for HunspellStemFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for HunspellStemFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let term = token.term.as_ref();
        let stems = self.stem_word(term);

        if stems.is_empty() {
            return (false, None);
        }

        // First stem replaces the token
        token.term = Cow::Owned(stems[0].clone());

        // Additional stems emitted as extra tokens at same position
        if stems.len() > 1 {
            let extras: Vec<Token<'a>> = stems[1..]
                .iter()
                .map(|s| Token {
                    term: Cow::Owned(s.clone()),
                    start_offset: token.start_offset,
                    end_offset: token.end_offset,
                    position: token.position,
                })
                .collect();
            (false, Some(extras))
        } else {
            (false, None)
        }
    }
}
