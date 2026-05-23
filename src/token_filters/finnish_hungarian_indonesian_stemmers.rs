use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

// ─── Finnish Light Stemmer ────────────────────────────────────────────────

/// Finnish light stemmer based on Lucene's FinnishLightStemFilter.
///
/// Removes common Finnish case and number suffixes.
#[derive(Clone, Debug, Default)]
pub struct FinnishLightStemTokenFilter;

impl FinnishLightStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for FinnishLightStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.len() < 4 {
            return (false, None);
        }

        let stemmed = stem_finnish_light(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_finnish_light(word: &str) -> String {
    let mut s: Vec<char> = word.chars().collect();
    let len = s.len();

    // Step 1: Plural & possessives (longest match first)
    if len > 7 {
        let suffix: String = s[len - 4..].iter().collect();
        match suffix.as_str() {
            "tten" | "nnen" => {
                s.truncate(len - 4);
                return s.into_iter().collect();
            }
            _ => {}
        }
    }

    if len > 6 {
        let suffix: String = s[len - 3..].iter().collect();
        match suffix.as_str() {
            "ssa" | "ssä" | "sta" | "stä" | "lla" | "llä" | "lta" | "ltä" | "lle" | "tta"
            | "ttä" | "ksi" | "ine" => {
                s.truncate(len - 3);
                return s.into_iter().collect();
            }
            _ => {}
        }
    }

    if len > 5 {
        let suffix: String = s[len - 2..].iter().collect();
        match suffix.as_str() {
            "na" | "nä" | "en" | "in" | "an" | "ön" | "on" | "ät" | "öt" | "it" | "et" | "ia"
            | "iä" | "ta" | "tä" | "ja" | "jä" => {
                s.truncate(len - 2);
                return s.into_iter().collect();
            }
            _ => {}
        }
    }

    if len > 4 {
        let last = *s.last().unwrap();
        match last {
            'a' | 'ä' | 'n' | 't' | 'i' | 'e' => {
                s.pop();
            }
            _ => {}
        }
    }

    s.into_iter().collect()
}

// ─── Hungarian Light Stemmer ──────────────────────────────────────────────

/// Hungarian light stemmer based on Lucene's HungarianLightStemFilter.
///
/// Removes common Hungarian case and number suffixes.
#[derive(Clone, Debug, Default)]
pub struct HungarianLightStemTokenFilter;

impl HungarianLightStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for HungarianLightStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.len() < 4 {
            return (false, None);
        }

        let stemmed = stem_hungarian_light(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_hungarian_light(word: &str) -> String {
    let mut s: Vec<char> = word.chars().collect();
    let len = s.len();

    // 4-char suffixes
    if len > 7 {
        let suffix: String = s[len - 4..].iter().collect();
        match suffix.as_str() {
            "akat" | "eket" | "oket" | "okat" | "eket" | "ünök" | "anok" | "enök" | "änak"
            | "énak" => {
                s.truncate(len - 4);
                return s.into_iter().collect();
            }
            _ => {}
        }
    }

    // 3-char suffixes
    if len > 6 {
        let suffix: String = s[len - 3..].iter().collect();
        match suffix.as_str() {
            "ban" | "ben" | "nak" | "nek" | "ból" | "ből" | "hoz" | "hez" | "höz" | "ról"
            | "ről" | "tól" | "től" | "val" | "vel" | "ért" | "nek" | "ban" | "vál" | "vél"
            | "ból" | "ből" | "nak" | "nek" => {
                s.truncate(len - 3);
                return s.into_iter().collect();
            }
            _ => {}
        }
    }

    // 2-char suffixes
    if len > 5 {
        let suffix: String = s[len - 2..].iter().collect();
        match suffix.as_str() {
            "at" | "et" | "ot" | "öt" | "ra" | "re" | "ba" | "be" | "on" | "en" | "ön" | "ul"
            | "ül" | "ig" | "ek" | "ok" | "ök" | "ak" | "ás" | "és" => {
                s.truncate(len - 2);
                return s.into_iter().collect();
            }
            _ => {}
        }
    }

    // 1-char suffixes
    if len > 4 {
        let last = *s.last().unwrap();
        match last {
            'á' | 'é' | 'a' | 'e' | 'k' | 't' => {
                s.pop();
            }
            _ => {}
        }
    }

    s.into_iter().collect()
}

// ─── Indonesian Stemmer ───────────────────────────────────────────────────

/// Indonesian stemmer based on Lucene's IndonesianStemFilter.
///
/// Removes common Indonesian prefixes and suffixes following
/// the Asian Federation for Natural Language Processing approach.
#[derive(Clone, Debug, Default)]
pub struct IndonesianStemTokenFilter;

impl IndonesianStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for IndonesianStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.len() < 4 {
            return (false, None);
        }

        let stemmed = stem_indonesian(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_indonesian(word: &str) -> String {
    let mut result = String::from(word);
    let original_len = result.len();

    // Remove particle suffixes: -lah, -kah, -tah, -pun
    if result.ends_with("lah") || result.ends_with("kah") || result.ends_with("tah") {
        result.truncate(result.len() - 3);
    } else if result.ends_with("pun") {
        result.truncate(result.len() - 3);
    }

    // Remove possessive suffixes: -ku, -mu, -nya
    if result.ends_with("nya") {
        result.truncate(result.len() - 3);
    } else if result.ends_with("ku") || result.ends_with("mu") {
        result.truncate(result.len() - 2);
    }

    // Remove derivational suffix: -kan, -an, -i
    let suffix_removed;
    if result.len() > 4 && result.ends_with("kan") {
        result.truncate(result.len() - 3);
        suffix_removed = true;
    } else if result.len() > 4 && result.ends_with("an") {
        result.truncate(result.len() - 2);
        suffix_removed = true;
    } else if result.len() > 4 && result.ends_with('i') {
        result.truncate(result.len() - 1);
        suffix_removed = true;
    } else {
        suffix_removed = false;
    }

    // Remove prefixes: me-, ber-, di-, ke-, se-, per-, ter-, pe-
    if result.len() > 4 {
        if result.starts_with("mem")
            || result.starts_with("men")
            || result.starts_with("meng")
            || result.starts_with("meny")
            || result.starts_with("menge")
        {
            if result.starts_with("menge") && result.len() > 7 {
                result = result[5..].to_string();
            } else if result.starts_with("meng") && result.len() > 6 {
                result = result[4..].to_string();
            } else if result.starts_with("meny") && result.len() > 6 {
                result = result[4..].to_string();
            } else if result.starts_with("men") && result.len() > 5 {
                result = result[3..].to_string();
            } else if result.starts_with("mem") && result.len() > 5 {
                result = result[3..].to_string();
            } else if result.starts_with("me") && result.len() > 4 {
                result = result[2..].to_string();
            }
        } else if result.starts_with("ber") && result.len() > 5 {
            result = result[3..].to_string();
        } else if result.starts_with("di") && result.len() > 4 {
            result = result[2..].to_string();
        } else if result.starts_with("per") && result.len() > 5 {
            result = result[3..].to_string();
        } else if result.starts_with("ter") && result.len() > 5 {
            result = result[3..].to_string();
        } else if result.starts_with("ke") && result.len() > 4 {
            result = result[2..].to_string();
        } else if result.starts_with("se") && result.len() > 4 {
            result = result[2..].to_string();
        } else if result.starts_with("pe") && result.len() > 4 && !suffix_removed {
            result = result[2..].to_string();
        }
    }

    // Only return stemmed result if we actually shortened it
    if result.len() < original_len && result.len() >= 3 {
        result
    } else {
        word.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pizza_engine::analysis::Token;

    fn make_token(term: &str) -> Token<'_> {
        Token {
            term: Cow::Borrowed(term),
            start_offset: 0,
            end_offset: term.len() as u32,
            position: 0,
        }
    }

    #[test]
    fn test_finnish_light_stem() {
        let filter = FinnishLightStemTokenFilter::new();
        let mut token = make_token("kirjassa");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "kirja");
    }

    #[test]
    fn test_finnish_light_short() {
        let filter = FinnishLightStemTokenFilter::new();
        let mut token = make_token("ala");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "ala"); // too short to stem
    }

    #[test]
    fn test_hungarian_light_stem() {
        let filter = HungarianLightStemTokenFilter::new();
        let mut token = make_token("házban");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "ház");
    }

    #[test]
    fn test_hungarian_light_short() {
        let filter = HungarianLightStemTokenFilter::new();
        let mut token = make_token("ház");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "ház"); // too short
    }

    #[test]
    fn test_indonesian_suffix() {
        let filter = IndonesianStemTokenFilter::new();
        let mut token = make_token("memakan");
        filter.filter(&mut token);
        // "memakan" -> strip prefix "mem" -> "akan" or strip suffix "kan" -> "mema" -> ...
        // Actual result depends on order. Our impl: strip suffixes first -> "mema" -> strip "me" prefix -> "ma"
        // But "ma" is < 3 chars, so we keep "mema" and try prefix: starts_with "mem" -> "a" which is too short
        // So the function should try: suffix "kan" -> "mema", then prefix "mem" on "mema" -> "a" too short
        // Falls back to just suffix: "mema"
        assert!(token.term.as_ref().len() < "memakan".len());
    }

    #[test]
    fn test_indonesian_prefix() {
        let filter = IndonesianStemTokenFilter::new();
        let mut token = make_token("berlari");
        filter.filter(&mut token);
        // strip suffix: no match, strip prefix "ber" -> "lari"
        assert_eq!(token.term.as_ref(), "lari");
    }
}
