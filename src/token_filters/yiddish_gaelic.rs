use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

// ═══════════════════════════════════════════════════════════════════════════════
// YIDDISH ANALYSIS — Normalization for Yiddish (Hebrew-script variant)
// ═══════════════════════════════════════════════════════════════════════════════

/// Normalizes Yiddish orthographic variants.
/// Yiddish uses Hebrew alphabet differently — certain letter combinations
/// represent vowels. This normalizes common spelling variations.
#[derive(Clone, Debug)]
pub struct YiddishNormalizationTokenFilter;
impl YiddishNormalizationTokenFilter {
    pub fn new() -> Self { Self }
}
impl Default for YiddishNormalizationTokenFilter {
    fn default() -> Self { Self }
}

impl TokenFilter for YiddishNormalizationTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let mut result = String::with_capacity(text.len());
        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];
            let cp = c as u32;

            // Normalize Yiddish-specific Unicode points
            match cp {
                // ײ (U+05F2, Double Yod) → יי
                0x05F2 => {
                    result.push('י');
                    result.push('י');
                }
                // װ (U+05F0, Double Vav) → וו
                0x05F0 => {
                    result.push('ו');
                    result.push('ו');
                }
                // ױ (U+05F1, Vav-Yod) → וי
                0x05F1 => {
                    result.push('ו');
                    result.push('י');
                }
                // Remove niqqud marks from Yiddish text too
                0x0591..=0x05BD | 0x05BF | 0x05C1..=0x05C2 | 0x05C4..=0x05C5 | 0x05C7 => {}
                // Geresh and gershayim (used in abbreviations) — keep
                _ => result.push(c),
            }
            i += 1;
        }

        if result != text {
            token.term = Cow::Owned(result);
        }
        (false, None)
    }
}

/// Light Yiddish stemmer — strips common Germanic and Slavic suffixes
/// found in Yiddish words written in Hebrew script or transliterated.
#[derive(Clone, Debug)]
pub struct YiddishStemTokenFilter;
impl YiddishStemTokenFilter {
    pub fn new() -> Self { Self }
}
impl Default for YiddishStemTokenFilter {
    fn default() -> Self { Self }
}

impl TokenFilter for YiddishStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        // Work on transliterated Yiddish (Latin script)
        let lower = text.to_lowercase();

        // Check if it's Latin-script Yiddish (transliterated)
        if lower.chars().all(|c| c.is_ascii_alphabetic() || c == '-' || c == '\'') && lower.len() > 4 {
            let stemmed = stem_yiddish_latin(&lower);
            if stemmed != lower {
                token.term = Cow::Owned(stemmed);
                return (false, None);
            }
        }
        (false, None)
    }
}

fn stem_yiddish_latin(word: &str) -> String {
    let mut s = String::from(word);

    // Yiddish plural suffixes (Germanic origin)
    let suffixes = &[
        "lekh",  // diminutive plural
        "ung",   // nominalization (-ung)
        "keit",  // -keit/-heit
        "heyt",
        "shaft", // -schaft
        "ish",   // adjective
        "ikh",   // -lich/-ig
        "er",    // comparative / agent
        "im",    // Hebrew plural
        "es",    // genitive/plural
        "en",    // infinitive / plural
        "s",     // plural/genitive
        "n",     // plural
    ];

    for suffix in suffixes {
        if s.len() > suffix.len() + 3 && s.ends_with(suffix) {
            s.truncate(s.len() - suffix.len());
            break;
        }
    }
    s
}

// ═══════════════════════════════════════════════════════════════════════════════
// SCOTTISH GAELIC ANALYSIS — Lenition normalization
// ═══════════════════════════════════════════════════════════════════════════════

/// Scottish Gaelic lenition normalization.
/// Lenition in Gaelic adds 'h' after the initial consonant (bh, ch, dh, fh, gh, mh, ph, sh, th).
/// This filter strips lenition 'h' to normalize to root form.
#[derive(Clone, Debug)]
pub struct ScottishGaelicLenitionTokenFilter;
impl ScottishGaelicLenitionTokenFilter {
    pub fn new() -> Self { Self }
}
impl Default for ScottishGaelicLenitionTokenFilter {
    fn default() -> Self { Self }
}

impl TokenFilter for ScottishGaelicLenitionTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let lower = text.to_lowercase();
        let chars: Vec<char> = lower.chars().collect();

        if chars.len() < 3 {
            return (false, None);
        }

        // Check for lenition pattern: consonant + 'h' at start
        if chars.len() >= 2 && chars[1] == 'h' {
            let initial = chars[0];
            if matches!(initial, 'b' | 'c' | 'd' | 'f' | 'g' | 'm' | 'p' | 's' | 't') {
                // Special case: 'sh' and 'th' effectively silence the consonant
                // For search purposes, remove the 'h' to get the root
                let delenited: String = core::iter::once(initial).chain(chars[2..].iter().copied()).collect();
                token.term = Cow::Owned(delenited);
                return (false, None);
            }
        }
        (false, None)
    }
}

/// Scottish Gaelic stop word filter for common Gaelic particles.
/// Gaelic has many short particles (an, am, na, nan, ag, air, etc.)
#[derive(Clone, Debug)]
pub struct ScottishGaelicStopTokenFilter;
impl ScottishGaelicStopTokenFilter {
    pub fn new() -> Self { Self }
}
impl Default for ScottishGaelicStopTokenFilter {
    fn default() -> Self { Self }
}

impl TokenFilter for ScottishGaelicStopTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let lower = text.to_lowercase();
        if is_gaelic_stop_word(&lower) {
            return (true, None);
        }
        (false, None)
    }
}

fn is_gaelic_stop_word(word: &str) -> bool {
    matches!(word,
        "an" | "am" | "a" | "na" | "nan" | "nam" | "ag" | "air" | "aig" |
        "ann" | "anns" | "às" | "bho" | "bhon" | "de" | "den" | "do" |
        "don" | "e" | "eadar" | "fo" | "gu" | "gun" | "gus" | "le" |
        "leis" | "mar" | "mu" | "mun" | "o" | "on" | "ri" | "ris" |
        "ro" | "tro" | "troimh" | "tha" | "is" | "bi" | "bha" |
        "chan" | "cha" | "nach" | "ach" | "agus" | "no" | "neo" |
        "ma" | "mur" | "ged" | "nuair" | "far" | "oir" | "gur" |
        "seo" | "sin" | "siud" | "mo" | "do" | "a" | "ar" | "ur" |
        "mi" | "thu" | "e" | "i" | "sinn" | "sibh" | "iad"
    )
}
