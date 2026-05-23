use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

// ═══════════════════════════════════════════════════════════════════════════════
// QUECHUA ANALYSIS — Agglutinative suffix stripping
// Quechua is the most widely spoken indigenous language of the Americas (~5M)
// It is highly agglutinative with extensive suffixing.
// ═══════════════════════════════════════════════════════════════════════════════

/// Quechua light stemmer — strips common suffixes.
/// Quechua is an agglutinative, exclusively suffixing language.
/// Common suffixes include plural (-kuna), possessives (-y, -yki, -n),
/// case markers (-ta, -man, -manta, -pi, -wan), and verbal suffixes.
#[derive(Clone, Debug)]
pub struct QuechuaStemTokenFilter;
impl QuechuaStemTokenFilter {
    pub fn new() -> Self { Self }
}
impl Default for QuechuaStemTokenFilter {
    fn default() -> Self { Self }
}

impl TokenFilter for QuechuaStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let lower = text.to_lowercase();

        if lower.len() < 4 {
            return (false, None);
        }

        let stemmed = stem_quechua(&lower);
        if stemmed != lower && stemmed.len() >= 2 {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_quechua(word: &str) -> String {
    let mut s = String::from(word);

    // Strip suffixes iteratively (Quechua stacks multiple suffixes)
    // Order matters — strip outermost first

    // Evidential/validational suffixes
    let outer_suffixes = &["puni", "chá", "cha", "raq", "ña"];
    for suffix in outer_suffixes {
        if s.len() > suffix.len() + 3 && s.ends_with(suffix) {
            s.truncate(s.len() - suffix.len());
        }
    }

    // Case suffixes
    let case_suffixes = &[
        "manta",  // ablative "from"
        "kama",   // limitative "until"
        "pura",   // "among"
        "rayku",  // "because of"
        "ntin",   // inclusive
        "kuna",   // plural
        "wan",    // instrumental/comitative "with"
        "man",    // dative/allative "to"
        "paq",    // benefactive "for"
        "pi",     // locative "in"
        "ta",     // accusative (object)
        "pa",     // genitive "of"
        "qa",     // topic marker
    ];

    for suffix in case_suffixes {
        if s.len() > suffix.len() + 2 && s.ends_with(suffix) {
            s.truncate(s.len() - suffix.len());
            break; // Only one case marker per word
        }
    }

    // Possessive suffixes
    let poss_suffixes = &[
        "nchik", "nchis",  // 1st person inclusive plural
        "ykichik", "ykichis", // 2nd person plural
        "nku",             // 3rd person plural
        "yki",             // 2nd person singular
        "y",               // 1st person singular
        "n",               // 3rd person singular
    ];

    for suffix in poss_suffixes {
        if s.len() > suffix.len() + 2 && s.ends_with(suffix) {
            s.truncate(s.len() - suffix.len());
            break;
        }
    }

    s
}

/// Quechua stop word filter.
#[derive(Clone, Debug)]
pub struct QuechuaStopTokenFilter;
impl QuechuaStopTokenFilter {
    pub fn new() -> Self { Self }
}
impl Default for QuechuaStopTokenFilter {
    fn default() -> Self { Self }
}

impl TokenFilter for QuechuaStopTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let lower = token.term.as_ref().to_lowercase();
        if is_quechua_stop(&lower) {
            return (true, None);
        }
        (false, None)
    }
}

fn is_quechua_stop(word: &str) -> bool {
    matches!(word,
        "kay" | "chay" | "pay" | // pronouns this/that/he
        "ñuqa" | "qam" |         // I, you
        "ima" | "pi" | "may" |   // what, who, where
        "mana" |                  // negation
        "ari" | "arí" |          // affirmative
        "hina" |                  // like/as
        "chaymanta" |             // therefore
        "icha" |                  // or
        "aswanqa" |               // but
        "huq" | "huk" |           // one/a
        "chayqa" |                // then/so
        "hinaspa" |               // and then
        "ichaqa"                  // however
    )
}

// ═══════════════════════════════════════════════════════════════════════════════
// GUARANI ANALYSIS — Nasal harmony normalization
// Guarani is co-official with Spanish in Paraguay (~6-7M speakers).
// It has extensive nasal harmony and glottal stop usage.
// ═══════════════════════════════════════════════════════════════════════════════

/// Guarani normalization filter — handles nasal vowel variants and tilde usage.
/// Guarani uses ã, ẽ, ĩ, õ, ũ, ỹ for nasal vowels and puso (') for glottal stop.
/// This normalizes spelling variations.
#[derive(Clone, Debug)]
pub struct GuaraniNormalizationTokenFilter;
impl GuaraniNormalizationTokenFilter {
    pub fn new() -> Self { Self }
}
impl Default for GuaraniNormalizationTokenFilter {
    fn default() -> Self { Self }
}

impl TokenFilter for GuaraniNormalizationTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let lower = text.to_lowercase();

        // Normalize Guarani-specific characters
        let normalized: String = lower.chars().map(|c| match c {
            // Normalize glottal stop variants to standard puso
            '\u{02BC}' | '\u{2019}' | '\u{0027}' | '\u{2018}' => '\u{02BC}', // → modifier letter apostrophe
            // Keep nasal vowels as-is (they're meaningful)
            _ => c,
        }).collect();

        if normalized != text {
            token.term = Cow::Owned(normalized);
        }
        (false, None)
    }
}

/// Guarani light stemmer — strips common suffixes.
/// Guarani is agglutinative with prefixes and suffixes.
#[derive(Clone, Debug)]
pub struct GuaraniStemTokenFilter;
impl GuaraniStemTokenFilter {
    pub fn new() -> Self { Self }
}
impl Default for GuaraniStemTokenFilter {
    fn default() -> Self { Self }
}

impl TokenFilter for GuaraniStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let lower = text.to_lowercase();

        if lower.len() < 4 {
            return (false, None);
        }

        let stemmed = stem_guarani(&lower);
        if stemmed != lower && stemmed.len() >= 2 {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_guarani(word: &str) -> String {
    let mut s = String::from(word);

    // Strip common Guarani suffixes
    let suffixes = &[
        "kuéra",  // plural
        "kuera",
        "guasu",  // augmentative
        "mi",     // diminutive
        "ite",    // superlative
        "hára",   // agent nominalizer
        "hara",
        "py",     // collective/locative
        "pe",     // locative "in"
        "gui",    // ablative "from"
        "re",     // locative "on"
        "me",     // locative (nasal)
    ];

    for suffix in suffixes {
        if s.len() > suffix.len() + 2 && s.ends_with(suffix) {
            s.truncate(s.len() - suffix.len());
            break;
        }
    }

    // Strip common prefixes
    let prefixes: &[&str] = &[
        "mba'e",  // thing/what (nominal prefix)
        "mbae",
        "ñe'ẽ",   // speech prefix
    ];

    for prefix in prefixes {
        if s.len() > prefix.len() + 2 && s.starts_with(prefix) {
            s = s[prefix.len()..].to_string();
            break;
        }
    }

    s
}

/// Guarani stop word filter.
#[derive(Clone, Debug)]
pub struct GuaraniStopTokenFilter;
impl GuaraniStopTokenFilter {
    pub fn new() -> Self { Self }
}
impl Default for GuaraniStopTokenFilter {
    fn default() -> Self { Self }
}

impl TokenFilter for GuaraniStopTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let lower = token.term.as_ref().to_lowercase();
        if is_guarani_stop(&lower) {
            return (true, None);
        }
        (false, None)
    }
}

fn is_guarani_stop(word: &str) -> bool {
    matches!(word,
        "ha" | "ha'e" | "hae" | // and, he/she
        "che" | "nde" | "ore" | "ñande" | // I, you, we(excl), we(incl)
        "ko" | "pe" | "amo" |   // this, that, that(far)
        "ndaha'éi" | "nahániri" | // negation
        "avei" |                  // also
        "katu" |                  // indeed/well
        "heta" | "mbovy" |       // many, few
        "upéicha" |               // thus
        "upéi" |                  // then
        "ágã" | "aga" |          // now
        "ky'a" | "porã" |        // some particles
        "rupi" |                  // through/by
        "rehe"                   // about/concerning
    )
}
