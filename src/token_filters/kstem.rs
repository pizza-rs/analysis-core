use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// KStem token filter - a less aggressive English stemmer than Porter.
///
/// KStem (Krovetz stemmer) tends to produce stems that are actual English words,
/// making it suitable for search applications where readability matters.
///
/// Based on the algorithm described by Robert Krovetz in
/// "Viewing Morphology as an Inference Process" (1993).
#[derive(Clone, Debug, Default)]
pub struct KStemTokenFilter;

impl KStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for KStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let word = token.term.as_ref();
        if word.len() < 3 {
            return (false, None);
        }

        if let Some(stemmed) = kstem(word) {
            token.term = Cow::Owned(stemmed);
        }

        (false, None)
    }
}

fn kstem(word: &str) -> Option<String> {
    let lower: String = word.to_lowercase();
    let bytes = lower.as_bytes();

    // Step 1: Handle plurals
    if let Some(s) = handle_plurals(&lower, bytes) {
        return Some(s);
    }

    // Step 2: Handle past tense (-ed)
    if let Some(s) = handle_past_tense(&lower, bytes) {
        return Some(s);
    }

    // Step 3: Handle aspect (-ing)
    if let Some(s) = handle_aspect(&lower, bytes) {
        return Some(s);
    }

    // Step 4: Handle -ity suffix
    if let Some(s) = handle_ity_suffix(&lower) {
        return Some(s);
    }

    // Step 5: Handle -ness suffix
    if let Some(s) = handle_ness_suffix(&lower) {
        return Some(s);
    }

    // Step 6: Handle -tion suffix
    if let Some(s) = handle_tion_suffix(&lower) {
        return Some(s);
    }

    // Step 7: Handle -ment suffix
    if let Some(s) = handle_ment_suffix(&lower) {
        return Some(s);
    }

    // Step 8: Handle -ble suffix
    if let Some(s) = handle_ble_suffix(&lower) {
        return Some(s);
    }

    // Step 9: Handle -ly suffix
    if let Some(s) = handle_ly_suffix(&lower) {
        return Some(s);
    }

    if lower != word {
        Some(lower)
    } else {
        None
    }
}

fn handle_plurals(word: &str, bytes: &[u8]) -> Option<String> {
    let len = bytes.len();
    if len < 4 {
        return None;
    }

    // -ies → -y (e.g., "ponies" → "pony")
    if word.ends_with("ies") && len > 4 {
        return Some(word[..len - 3].to_owned() + "y");
    }

    // -es → -e or remove (e.g., "watches" → "watch", "phases" → "phase")
    if word.ends_with("es") && len > 3 {
        let stem = &word[..len - 2];
        // If removing -es leaves a word ending in sh, ch, x, z, s → just remove s
        if word.ends_with("sses") {
            return Some(word[..len - 2].to_owned());
        }
        if word.ends_with("shes")
            || word.ends_with("ches")
            || word.ends_with("xes")
            || word.ends_with("zes")
        {
            return Some(word[..len - 2].to_owned());
        }
        // Otherwise remove -s if stem looks ok
        if stem.len() >= 3 {
            return Some(stem.to_owned());
        }
    }

    // -s (but not -ss)
    if bytes[len - 1] == b's' && bytes[len - 2] != b's' && len > 3 {
        return Some(word[..len - 1].to_owned());
    }

    None
}

fn handle_past_tense(word: &str, bytes: &[u8]) -> Option<String> {
    let len = bytes.len();
    if !word.ends_with("ed") || len < 5 {
        return None;
    }

    // -ied → -y
    if word.ends_with("ied") && len > 4 {
        return Some(word[..len - 3].to_owned() + "y");
    }

    // -eed → -ee (e.g., "agreed" → "agree")
    if word.ends_with("eed") {
        return Some(word[..len - 1].to_owned());
    }

    // Double consonant before -ed → remove one (e.g., "stopped" → "stop")
    if len > 4 {
        let c1 = bytes[len - 3];
        let c2 = bytes[len - 4];
        if c1 == c2 && !is_vowel(c1) && c1 != b'l' && c1 != b's' && c1 != b'z' {
            return Some(word[..len - 3].to_owned());
        }
    }

    // Default: remove -ed, optionally add -e
    let stem = &word[..len - 2];
    if stem.len() >= 3 {
        // If stem ends in consonant and has no vowel except initial, add -e
        if needs_e(stem) {
            return Some(stem.to_owned() + "e");
        }
        return Some(stem.to_owned());
    }

    None
}

fn handle_aspect(word: &str, bytes: &[u8]) -> Option<String> {
    let len = bytes.len();
    if !word.ends_with("ing") || len < 5 {
        return None;
    }

    // Double consonant before -ing → remove one
    if len > 5 {
        let c1 = bytes[len - 4];
        let c2 = bytes[len - 5];
        if c1 == c2 && !is_vowel(c1) && c1 != b'l' && c1 != b's' && c1 != b'z' {
            return Some(word[..len - 4].to_owned());
        }
    }

    // Default: remove -ing, optionally add -e
    let stem = &word[..len - 3];
    if stem.len() >= 3 {
        if needs_e(stem) {
            return Some(stem.to_owned() + "e");
        }
        return Some(stem.to_owned());
    }

    None
}

fn handle_ity_suffix(word: &str) -> Option<String> {
    if word.ends_with("ity") && word.len() > 5 {
        let stem = &word[..word.len() - 3];
        if stem.len() >= 3 {
            return Some(stem.to_owned());
        }
    }
    None
}

fn handle_ness_suffix(word: &str) -> Option<String> {
    if word.ends_with("ness") && word.len() > 6 {
        let mut stem = word[..word.len() - 4].to_owned();
        // -iness → -y
        if stem.ends_with('i') {
            stem.pop();
            stem.push('y');
        }
        if stem.len() >= 3 {
            return Some(stem);
        }
    }
    None
}

fn handle_tion_suffix(word: &str) -> Option<String> {
    if word.ends_with("tion") && word.len() > 6 {
        let stem = &word[..word.len() - 4];
        if stem.len() >= 3 {
            // -ation → -ate
            if word.ends_with("ation") && word.len() > 7 {
                return Some(word[..word.len() - 5].to_owned() + "ate");
            }
            return Some(stem.to_owned() + "t");
        }
    }
    None
}

fn handle_ment_suffix(word: &str) -> Option<String> {
    if word.ends_with("ment") && word.len() > 6 {
        let stem = &word[..word.len() - 4];
        if stem.len() >= 3 {
            return Some(stem.to_owned());
        }
    }
    None
}

fn handle_ble_suffix(word: &str) -> Option<String> {
    if word.ends_with("able") && word.len() > 6 {
        let stem = &word[..word.len() - 4];
        if stem.len() >= 3 {
            return Some(stem.to_owned());
        }
    }
    if word.ends_with("ible") && word.len() > 6 {
        let stem = &word[..word.len() - 4];
        if stem.len() >= 3 {
            return Some(stem.to_owned());
        }
    }
    None
}

fn handle_ly_suffix(word: &str) -> Option<String> {
    if word.ends_with("ly") && word.len() > 4 {
        let stem = &word[..word.len() - 2];
        if stem.len() >= 3 {
            return Some(stem.to_owned());
        }
    }
    None
}

fn is_vowel(b: u8) -> bool {
    matches!(b, b'a' | b'e' | b'i' | b'o' | b'u')
}

fn needs_e(stem: &str) -> bool {
    let bytes = stem.as_bytes();
    let len = bytes.len();
    if len < 2 {
        return false;
    }
    // If the stem ends in a consonant preceded by a single vowel, it likely needs -e
    let last = bytes[len - 1];
    let prev = bytes[len - 2];
    !is_vowel(last) && is_vowel(prev) && (len < 3 || !is_vowel(bytes[len - 3]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stem(word: &str) -> String {
        let mut token = Token {
            term: Cow::Borrowed(word),
            start_offset: 0,
            end_offset: word.len() as u32,
            position: 0,
        };
        let filter = KStemTokenFilter::new();
        filter.filter(&mut token);
        token.term.into_owned()
    }

    #[test]
    fn test_plurals() {
        assert_eq!(stem("ponies"), "pony");
        assert_eq!(stem("watches"), "watch");
        assert_eq!(stem("cats"), "cat");
    }

    #[test]
    fn test_past_tense() {
        assert_eq!(stem("agreed"), "agree");
        assert_eq!(stem("studied"), "study");
    }

    #[test]
    fn test_ing() {
        assert_eq!(stem("running"), "run");
        assert_eq!(stem("making"), "make");
    }

    #[test]
    fn test_ness() {
        assert_eq!(stem("happiness"), "happy");
        assert_eq!(stem("darkness"), "dark");
    }

    #[test]
    fn test_short_words_unchanged() {
        assert_eq!(stem("go"), "go");
        assert_eq!(stem("an"), "an");
    }
}
