use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

// ═══════════════════════════════════════════════════════════════════════════════
// NAVAJO ANALYSIS — Prefix stripping and normalization
// Navajo (Diné bizaad) is the most spoken indigenous language in the US (~170K).
// It's an Athabaskan language with complex prefix morphology and tonal system.
// ═══════════════════════════════════════════════════════════════════════════════

/// Navajo normalization filter — standardizes diacritics and special characters.
/// Navajo uses: á é í ó (high tone), ą ę į ǫ (nasal vowels),
/// ł (voiceless l), and ʼ (glottal stop).
#[derive(Clone, Debug)]
pub struct NavajoNormalizationTokenFilter;
impl NavajoNormalizationTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for NavajoNormalizationTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for NavajoNormalizationTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let lower = text.to_lowercase();

        // Normalize variant representations
        let normalized: String = lower
            .chars()
            .map(|c| match c {
                // Normalize apostrophe variants to standard glottal stop mark
                '\'' | '\u{2019}' | '\u{02BC}' | '\u{02BB}' => '\u{02BC}',
                // Normalize barred L variants
                'Ł' => 'ł',
                _ => c,
            })
            .collect();

        if normalized != text {
            token.term = Cow::Owned(normalized);
        }
        (false, None)
    }
}

/// Navajo tone-insensitive normalization — strips tone marks for fuzzy matching.
/// Removes acute accent (high tone) from vowels: á→a, é→e, í→i, ó→o
/// Optionally also removes nasal ogonek: ą→a, ę→e, į→i, ǫ→o
#[derive(Clone, Debug)]
pub struct NavajoToneRemoveTokenFilter {
    pub remove_nasal: bool,
}
impl NavajoToneRemoveTokenFilter {
    pub fn new(remove_nasal: bool) -> Self {
        Self { remove_nasal }
    }
}
impl Default for NavajoToneRemoveTokenFilter {
    fn default() -> Self {
        Self {
            remove_nasal: false,
        }
    }
}

impl TokenFilter for NavajoToneRemoveTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let mut changed = false;
        let normalized: String = text
            .chars()
            .map(|c| {
                match c {
                    // Strip high tone (acute)
                    'á' | 'Á' => {
                        changed = true;
                        'a'
                    }
                    'é' | 'É' => {
                        changed = true;
                        'e'
                    }
                    'í' | 'Í' => {
                        changed = true;
                        'i'
                    }
                    'ó' | 'Ó' => {
                        changed = true;
                        'o'
                    }
                    // Combined nasal + tone: ą́ is actually two chars, but precomposed forms:
                    '\u{01CE}' => {
                        changed = true;
                        'a'
                    } // ǎ
                    // Nasal vowels (only if configured)
                    'ą' | 'Ą' if self.remove_nasal => {
                        changed = true;
                        'a'
                    }
                    'ę' | 'Ę' if self.remove_nasal => {
                        changed = true;
                        'e'
                    }
                    'į' | 'Į' if self.remove_nasal => {
                        changed = true;
                        'i'
                    }
                    'ǫ' | 'Ǫ' if self.remove_nasal => {
                        changed = true;
                        'o'
                    }
                    _ => c,
                }
            })
            .collect();

        if changed {
            token.term = Cow::Owned(normalized);
        }
        (false, None)
    }
}

/// Navajo light stemmer — strips common verbal/nominal prefixes.
/// Navajo has extensive prefixing morphology but limited suffixing.
/// This strips the most common outer prefixes.
#[derive(Clone, Debug)]
pub struct NavajoStemTokenFilter;
impl NavajoStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for NavajoStemTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for NavajoStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let lower = text.to_lowercase();

        if lower.len() < 4 {
            return (false, None);
        }

        let stemmed = stem_navajo(&lower);
        if stemmed != lower && stemmed.len() >= 2 {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_navajo(word: &str) -> String {
    let mut s = String::from(word);

    // Common Navajo noun prefixes (postpositional/pronominal)
    let prefixes = &[
        "bich'", // toward him/her
        "bik'",  // on him/her
        "bits'", // from him/her
        "bee",   // with it
        "bi",    // his/her/its
        "shi",   // my
        "ni",    // your
        "ha",    // space/area
    ];

    for prefix in prefixes {
        if s.len() > prefix.len() + 2 && s.starts_with(prefix) {
            s = s[prefix.len()..].to_string();
            break;
        }
    }

    // Common Navajo nominal suffixes (limited in Navajo)
    let suffixes = &[
        "ígíí", // nominalizer "the one that"
        "igii", "é", // enclitic
    ];

    for suffix in suffixes {
        if s.len() > suffix.len() + 2 && s.ends_with(suffix) {
            s.truncate(s.len() - suffix.len());
            break;
        }
    }

    s
}

/// Navajo stop word filter.
#[derive(Clone, Debug)]
pub struct NavajoStopTokenFilter;
impl NavajoStopTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for NavajoStopTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for NavajoStopTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let lower = token.term.as_ref().to_lowercase();
        if is_navajo_stop(&lower) {
            return (true, None);
        }
        (false, None)
    }
}

fn is_navajo_stop(word: &str) -> bool {
    matches!(
        word,
        "éí" | "ei" |            // the/that (topic)
        "dóó" | "doo" |          // and / not
        "áko" | "ako" |          // so/then
        "t'áá" | "taa" |        // just/only
        "shi" | "ni" | "bi" |    // I/you/him-her
        "da" |                    // plural marker
        "lá" | "la" |            // past tense marker
        "go" |                    // subordinator
        "yee" |                   // with/by means of
        "baa" |                   // about it
        "bee" |                   // with it
        "doo" |                   // negative
        "nihí" | "nihi" |        // we/us
        "índa" | "inda" |        // just now
        "háálá" | "haala" // because
    )
}

// ═══════════════════════════════════════════════════════════════════════════════
// NAHUATL ANALYSIS — Classical/Modern Nahuatl stemmer
// Nahuatl is spoken by ~1.7M people in Mexico.
// It's an agglutinative, polysynthetic language with prefixes and suffixes.
// ═══════════════════════════════════════════════════════════════════════════════

/// Nahuatl light stemmer — strips common morphological affixes.
/// Handles both Classical and Modern Nahuatl forms.
#[derive(Clone, Debug)]
pub struct NahuatlStemTokenFilter;
impl NahuatlStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for NahuatlStemTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for NahuatlStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let lower = text.to_lowercase();

        if lower.len() < 4 {
            return (false, None);
        }

        let stemmed = stem_nahuatl(&lower);
        if stemmed != lower && stemmed.len() >= 3 {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_nahuatl(word: &str) -> String {
    let mut s = String::from(word);

    // Strip common Nahuatl suffixes (noun/verb)
    let suffixes = &[
        "tzintli", // reverential
        "tzintle", "tzin", // reverential/diminutive
        "tli",  // absolutive singular
        "tl",   // absolutive singular (after vowel)
        "li",   // absolutive variant
        "tin",  // plural
        "meh",  // plural (modern)
        "me",   // plural
        "h",    // plural (Classical)
        "yotl", // abstract noun
        "yot", "lli", // locative
        "can", // place
        "yan", // place
        "pan", // on/upon
    ];

    for suffix in suffixes {
        if s.len() > suffix.len() + 3 && s.ends_with(suffix) {
            s.truncate(s.len() - suffix.len());
            break;
        }
    }

    // Strip common noun prefixes
    let prefixes = &[
        "no", // my
        "mo", // your/reflexive
        "to", // our
        "in", // their/the
        "i",  // his/her (before consonant)
    ];

    for prefix in prefixes {
        if s.len() > prefix.len() + 3 && s.starts_with(prefix) {
            s = s[prefix.len()..].to_string();
            break;
        }
    }

    s
}

/// Nahuatl stop word filter.
#[derive(Clone, Debug)]
pub struct NahuatlStopTokenFilter;
impl NahuatlStopTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for NahuatlStopTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for NahuatlStopTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let lower = token.term.as_ref().to_lowercase();
        if is_nahuatl_stop(&lower) {
            return (true, None);
        }
        (false, None)
    }
}

fn is_nahuatl_stop(word: &str) -> bool {
    matches!(
        word,
        "in" | "ín" |           // the/definite article
        "ihuan" | "huan" |      // and
        "ica" |                  // with
        "ipan" |                 // in/on
        "itech" |                // at/to
        "amo" |                  // not
        "neh" | "teh" | "yeh" | // I, you, he/she
        "tech" |                 // us/our
        "nochi" |                // all
        "ce" | "ome" |          // one, two
        "zan" | "san" |         // only
        "axan" | "axcan" |      // now
        "niman" |                // then/immediately
        "tlein" | "tlen" |      // what/that (relative)
        "kenin" | "quenin" |    // how
        "kanin" | "canin" // where
    )
}

// ═══════════════════════════════════════════════════════════════════════════════
// AYMARA ANALYSIS — Suffixing agglutinative language
// Aymara is spoken by ~1.7M people in Bolivia, Peru, Chile.
// ═══════════════════════════════════════════════════════════════════════════════

/// Aymara light stemmer — strips common agglutinative suffixes.
#[derive(Clone, Debug)]
pub struct AymaraStemTokenFilter;
impl AymaraStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for AymaraStemTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for AymaraStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let lower = text.to_lowercase();

        if lower.len() < 4 {
            return (false, None);
        }

        let stemmed = stem_aymara(&lower);
        if stemmed != lower && stemmed.len() >= 2 {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_aymara(word: &str) -> String {
    let mut s = String::from(word);

    // Aymara case and derivational suffixes
    let suffixes = &[
        "nakana", // plural + genitive
        "nakar",  // plural + dative
        "naka",   // plural
        "mpi",    // instrumental "with"
        "taki",   // benefactive "for"
        "nka",    // locative
        "tha",    // ablative "from"
        "na",     // genitive/locative
        "ru",     // dative/allative "to"
        "xa",     // topic marker
        "wa",     // evidential
        "ti",     // question
        "sa",     // conditional
    ];

    for suffix in suffixes {
        if s.len() > suffix.len() + 2 && s.ends_with(suffix) {
            s.truncate(s.len() - suffix.len());
            break;
        }
    }

    s
}

/// Aymara stop word filter.
#[derive(Clone, Debug)]
pub struct AymaraStopTokenFilter;
impl AymaraStopTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for AymaraStopTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for AymaraStopTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let lower = token.term.as_ref().to_lowercase();
        if is_aymara_stop(&lower) {
            return (true, None);
        }
        (false, None)
    }
}

fn is_aymara_stop(word: &str) -> bool {
    matches!(
        word,
        "naya" | "juma" | "jupa" |  // I, you, he/she
        "nanaka" | "jumanaka" |      // we, you(pl)
        "jupanaka" |                  // they
        "aka" | "uka" | "khaya" |    // this, that, that(far)
        "jani" |                      // no/not
        "ukhamaraki" |                // also
        "ukata" |                     // then
        "jan" |                       // without
        "kunasa" |                    // what?
        "khitisa" |                   // who?
        "kawkisa" // where?
    )
}
