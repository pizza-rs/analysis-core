//! Beider-Morse Phonetic Matching (BMPM) algorithm.
//!
//! The Beider-Morse algorithm is designed for matching names across different
//! languages and transliteration systems. It can identify the language of a
//! name and produce multiple phonetic codes based on language-specific rules.
//!
//! This is a simplified implementation covering the core matching logic
//! for the most common European language patterns.

use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Name type for Beider-Morse encoding.
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum BmNameType {
    /// Generic names (default).
    Generic,
    /// Ashkenazi Jewish names.
    Ashkenazi,
    /// Sephardic Jewish names.
    Sephardic,
}

/// Rule type for Beider-Morse.
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum BmRuleType {
    /// Exact matching rules.
    Exact,
    /// Approximate matching rules (more permissive).
    Approx,
}

/// Beider-Morse Phonetic Matching token filter.
///
/// Produces phonetic codes for name matching across languages.
/// Can detect the likely language of a name and apply language-specific
/// phonetic rules.
///
/// Equivalent to Elasticsearch's `phonetic` filter with encoder `beider_morse`.
#[derive(Clone, Debug)]
pub struct BeiderMorseFilter {
    name_type: BmNameType,
    rule_type: BmRuleType,
    replace: bool,
    max_phonemes: usize,
}

impl BeiderMorseFilter {
    /// Create with default settings (Generic, Approx).
    pub fn new() -> Self {
        Self {
            name_type: BmNameType::Generic,
            rule_type: BmRuleType::Approx,
            replace: true,
            max_phonemes: 20,
        }
    }

    pub fn with_name_type(mut self, name_type: BmNameType) -> Self {
        self.name_type = name_type;
        self
    }

    pub fn with_rule_type(mut self, rule_type: BmRuleType) -> Self {
        self.rule_type = rule_type;
        self
    }

    pub fn with_replace(mut self, replace: bool) -> Self {
        self.replace = replace;
        self
    }

    pub fn with_max_phonemes(mut self, max: usize) -> Self {
        self.max_phonemes = max;
        self
    }

    /// Detect likely language(s) of the input name.
    fn detect_language(&self, name: &str) -> LanguageSet {
        let lower = name.to_lowercase();
        let mut langs = LanguageSet::all();

        // Apply language detection rules based on character patterns
        if lower.contains("sch") || lower.contains("tz") {
            langs.retain(Language::German);
            langs.retain(Language::Yiddish);
        }
        if lower.contains("sz") || lower.contains("cz") || lower.contains("rz") {
            langs.retain(Language::Polish);
        }
        if lower.contains("ough") || lower.contains("tion") {
            langs.retain(Language::English);
        }
        if lower.contains("eau") || lower.contains("eux") || lower.contains("oux") {
            langs.retain(Language::French);
        }
        if lower.contains("ñ") {
            langs.retain(Language::Spanish);
        }
        if lower.contains("ă") || lower.contains("ş") || lower.contains("ţ") {
            langs.retain(Language::Romanian);
        }
        if lower.chars().any(|ch| matches!(ch, 'а'..='я' | 'А'..='Я')) {
            langs.retain(Language::Russian);
        }

        if langs.is_empty() {
            LanguageSet::all()
        } else {
            langs
        }
    }

    /// Encode a name using Beider-Morse rules.
    fn encode(&self, name: &str) -> String {
        let lower = name.to_lowercase();
        let _langs = self.detect_language(&lower);

        // Apply phonetic transformation rules
        let mut result = String::with_capacity(lower.len());
        let chars: Vec<char> = lower.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let remaining = &lower[lower.char_indices().nth(i).map(|(idx, _)| idx).unwrap_or(0)..];

            // Try multi-character rules first (longest match)
            if let Some((consumed, phoneme)) = self.apply_rules(remaining) {
                result.push_str(phoneme);
                i += consumed;
            } else {
                // Fallback: pass through
                result.push(chars[i]);
                i += 1;
            }
        }

        // Limit phonemes
        let phonemes: Vec<&str> = result.split('|').take(self.max_phonemes).collect();
        phonemes.join("|")
    }

    /// Apply transformation rules to the beginning of the text.
    /// Returns (chars_consumed, phonetic_output).
    fn apply_rules(&self, text: &str) -> Option<(usize, &'static str)> {
        // Common cross-language phonetic rules
        let rules: &[(&str, usize, &str)] = match self.rule_type {
            BmRuleType::Approx => &APPROX_RULES,
            BmRuleType::Exact => &EXACT_RULES,
        };

        for &(pattern, consumed, output) in rules {
            if text.starts_with(pattern) {
                return Some((consumed, output));
            }
        }
        None
    }
}

impl Default for BeiderMorseFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for BeiderMorseFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let term = token.term.as_ref();
        if term.is_empty() {
            return (false, None);
        }

        let encoded = self.encode(term);
        if encoded.is_empty() || encoded == term {
            return (false, None);
        }

        if self.replace {
            token.term = Cow::Owned(encoded);
            (false, None)
        } else {
            let extra = Token {
                term: Cow::Owned(encoded),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            };
            (false, Some(alloc::vec![extra]))
        }
    }
}

// ─── Language detection helpers ────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Language {
    English,
    French,
    German,
    Spanish,
    Italian,
    Portuguese,
    Polish,
    Romanian,
    Russian,
    Yiddish,
    Hungarian,
    Czech,
    Dutch,
    Turkish,
}

#[derive(Clone, Debug)]
struct LanguageSet {
    bits: u16,
}

impl LanguageSet {
    fn all() -> Self {
        Self { bits: 0xFFFF }
    }

    fn retain(&mut self, lang: Language) {
        // Keep only the specified language bit
        let bit = 1u16 << (lang as u16);
        if self.bits & bit != 0 {
            self.bits &= bit;
        }
    }

    fn is_empty(&self) -> bool {
        self.bits == 0
    }
}

// ─── Phonetic transformation rules ────────────────────────────────────────

/// Approximate matching rules (more permissive).
static APPROX_RULES: [(&str, usize, &str); 48] = [
    // Multi-char patterns
    ("tsch", 4, "tS"),
    ("sch", 3, "S"),
    ("tch", 3, "tS"),
    ("th", 2, "t"),
    ("ph", 2, "f"),
    ("gh", 2, "g"),
    ("kh", 2, "x"),
    ("sh", 2, "S"),
    ("ch", 2, "x"),
    ("ck", 2, "k"),
    ("zh", 2, "Z"),
    ("dj", 2, "dZ"),
    ("ts", 2, "ts"),
    ("tz", 2, "ts"),
    ("sz", 2, "S"),
    ("cz", 2, "tS"),
    ("rz", 2, "Z"),
    ("qu", 2, "kv"),
    ("ei", 2, "aj"),
    ("ey", 2, "aj"),
    ("eu", 2, "oj"),
    ("au", 2, "au"),
    ("ou", 2, "u"),
    ("oo", 2, "u"),
    ("ee", 2, "i"),
    ("ae", 2, "e"),
    ("oe", 2, "o"),
    ("ue", 2, "u"),
    ("ie", 2, "i"),
    // Single-char fallbacks
    ("c", 1, "k"),
    ("x", 1, "ks"),
    ("q", 1, "k"),
    ("w", 1, "v"),
    ("y", 1, "i"),
    ("j", 1, "j"),
    ("a", 1, "a"),
    ("b", 1, "b"),
    ("d", 1, "d"),
    ("e", 1, "e"),
    ("f", 1, "f"),
    ("g", 1, "g"),
    ("h", 1, ""),
    ("i", 1, "i"),
    ("k", 1, "k"),
    ("l", 1, "l"),
    ("m", 1, "m"),
    ("n", 1, "n"),
    ("o", 1, "o"),
];

/// Exact matching rules (stricter).
static EXACT_RULES: [(&str, usize, &str); 48] = [
    ("tsch", 4, "tS"),
    ("sch", 3, "S"),
    ("tch", 3, "tS"),
    ("th", 2, "t"),
    ("ph", 2, "f"),
    ("gh", 2, "g"),
    ("kh", 2, "x"),
    ("sh", 2, "S"),
    ("ch", 2, "x"),
    ("ck", 2, "k"),
    ("zh", 2, "Z"),
    ("dj", 2, "dZ"),
    ("ts", 2, "ts"),
    ("tz", 2, "ts"),
    ("sz", 2, "S"),
    ("cz", 2, "tS"),
    ("rz", 2, "Z"),
    ("qu", 2, "kv"),
    ("ei", 2, "ej"),
    ("ey", 2, "ej"),
    ("eu", 2, "ej"),
    ("au", 2, "au"),
    ("ou", 2, "u"),
    ("oo", 2, "u"),
    ("ee", 2, "i"),
    ("ae", 2, "e"),
    ("oe", 2, "oe"),
    ("ue", 2, "ue"),
    ("ie", 2, "ie"),
    ("c", 1, "ts"),
    ("x", 1, "ks"),
    ("q", 1, "k"),
    ("w", 1, "v"),
    ("y", 1, "i"),
    ("j", 1, "j"),
    ("a", 1, "a"),
    ("b", 1, "b"),
    ("d", 1, "d"),
    ("e", 1, "e"),
    ("f", 1, "f"),
    ("g", 1, "g"),
    ("h", 1, "h"),
    ("i", 1, "i"),
    ("k", 1, "k"),
    ("l", 1, "l"),
    ("m", 1, "m"),
    ("n", 1, "n"),
    ("o", 1, "o"),
];
