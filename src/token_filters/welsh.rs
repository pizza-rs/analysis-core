use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

// ═══════════════════════════════════════════════════════════════════════════════
// WELSH ANALYSIS — Soft/nasal/aspirate mutation normalization
// Welsh has a complex initial consonant mutation system that transforms
// word-initial consonants based on grammatical context.
// ═══════════════════════════════════════════════════════════════════════════════

/// Welsh mutation normalization filter.
/// Reverses the three types of Welsh initial consonant mutations to root form:
/// - Soft mutation (treiglad meddal): most common
/// - Nasal mutation (treiglad trwynol): after "yn"
/// - Aspirate mutation (treiglad llaes): after "a", "â", "ei", "eu"
#[derive(Clone, Debug)]
pub struct WelshMutationNormTokenFilter;
impl WelshMutationNormTokenFilter {
    pub fn new() -> Self { Self }
}
impl Default for WelshMutationNormTokenFilter {
    fn default() -> Self { Self }
}

impl TokenFilter for WelshMutationNormTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let lower = text.to_lowercase();
        let chars: Vec<char> = lower.chars().collect();

        if chars.len() < 2 {
            return (false, None);
        }

        // Try to reverse soft mutation (most common)
        if let Some(demuated) = reverse_soft_mutation(&chars) {
            token.term = Cow::Owned(demuated);
            return (false, None);
        }

        // Try to reverse nasal mutation
        if let Some(demuated) = reverse_nasal_mutation(&chars) {
            token.term = Cow::Owned(demuated);
            return (false, None);
        }

        // Try to reverse aspirate mutation
        if let Some(demuated) = reverse_aspirate_mutation(&chars) {
            token.term = Cow::Owned(demuated);
            return (false, None);
        }

        (false, None)
    }
}

// Soft mutation: p→b, t→d, c→g, b→f, d→dd, g→(drops), m→f, ll→l, rh→r
fn reverse_soft_mutation(chars: &[char]) -> Option<String> {
    let rest: String = chars[1..].iter().collect();

    match chars[0] {
        'f' => {
            // Could be b→f or m→f — ambiguous; prefer b→f
            Some(format!("b{}", rest))
        }
        'b' => Some(format!("p{}", rest)),
        'd' => {
            // d could be t→d, or dd→d (check for dd)
            if chars.len() >= 2 && chars[1] == 'd' {
                // "dd" is the mutated form of "d"
                let rest2: String = chars[2..].iter().collect();
                Some(format!("d{}", rest2))
            } else {
                Some(format!("t{}", rest))
            }
        }
        'g' => Some(format!("c{}", rest)),
        'l' => {
            // l could be ll→l (soft mutation of ll)
            if chars.len() >= 2 && chars[1] == 'l' {
                None // "ll" is already unmutated
            } else {
                Some(format!("ll{}", rest))
            }
        }
        'r' => {
            // r could be rh→r
            if chars.len() >= 2 && chars[1] == 'h' {
                None // "rh" is already unmutated
            } else {
                Some(format!("rh{}", rest))
            }
        }
        _ => None,
    }
}

// Nasal mutation: p→mh, t→nh, c→ngh, b→m, d→n, g→ng
fn reverse_nasal_mutation(chars: &[char]) -> Option<String> {
    if chars.len() < 2 {
        return None;
    }
    let two: String = chars[0..2].iter().collect();
    let rest2: String = chars[2..].iter().collect();

    match two.as_str() {
        "mh" => Some(format!("p{}", rest2)),
        "nh" => Some(format!("t{}", rest2)),
        _ => {
            if chars.len() >= 3 {
                let three: String = chars[0..3].iter().collect();
                let rest3: String = chars[3..].iter().collect();
                if three == "ngh" {
                    return Some(format!("c{}", rest3));
                }
            }
            // m→b, n→d (but these are also standalone consonants)
            // ng→g (but ng is also a common digraph)
            if two == "ng" {
                Some(format!("g{}", rest2))
            } else {
                None
            }
        }
    }
}

// Aspirate mutation: p→ph, t→th, c→ch
fn reverse_aspirate_mutation(chars: &[char]) -> Option<String> {
    if chars.len() < 2 {
        return None;
    }
    let two: String = chars[0..2].iter().collect();
    let rest: String = chars[2..].iter().collect();

    match two.as_str() {
        "ph" => Some(format!("p{}", rest)),
        "th" => Some(format!("t{}", rest)),
        "ch" => Some(format!("c{}", rest)),
        _ => None,
    }
}

/// Welsh stop word filter.
#[derive(Clone, Debug)]
pub struct WelshStopTokenFilter;
impl WelshStopTokenFilter {
    pub fn new() -> Self { Self }
}
impl Default for WelshStopTokenFilter {
    fn default() -> Self { Self }
}

impl TokenFilter for WelshStopTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let lower = token.term.as_ref().to_lowercase();
        if is_welsh_stop(&lower) {
            return (true, None);
        }
        (false, None)
    }
}

fn is_welsh_stop(word: &str) -> bool {
    matches!(word,
        "a" | "ac" | "am" | "ar" | "at" | "â" |
        "bod" | "bydd" | "yn" | "y" | "yr" |
        "i" | "ei" | "eu" | "fy" | "dy" | "ein" | "eich" |
        "mae" | "maen" | "yw" | "ydy" | "oedd" | "roedd" |
        "o" | "gan" | "gyda" | "heb" | "rhwng" | "wrth" |
        "dan" | "tan" | "dros" | "tros" | "drwy" | "trwy" |
        "er" | "ers" | "hyd" | "nes" |
        "ni" | "chi" | "nhw" | "fe" | "hi" | "fi" |
        "hon" | "hyn" | "hwn" | "hynny" | "hwnnw" |
        "pan" | "pe" | "os" | "ond" | "neu" | "na" | "nid" |
        "dim" | "un" | "dau" | "dwy" | "tri" | "tair" |
        "wedi" | "cyn" | "ar" | "mewn" | "allan" | "eto" |
        "hefyd" | "iawn" | "felly" | "wedyn" | "nawr" | "yma" | "yna"
    )
}
