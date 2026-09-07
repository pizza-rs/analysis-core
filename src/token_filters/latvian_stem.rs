use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Latvian stemmer based on the Lucene LatvianStemmer.
///
/// Removes Latvian noun/adjective/verb endings to produce stems.
#[derive(Clone, Debug, Default)]
pub struct LatvianStemTokenFilter;

impl LatvianStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for LatvianStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.len() < 4 {
            return (false, None);
        }

        let stemmed = stem_latvian(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

/// Lucene LatvianStemmer — faithful port of
/// `org.apache.lucene.analysis.lv.LatvianStemmer` (Karl Kreslins' algorithm).
fn stem_latvian(word: &str) -> String {
    // (suffix, suffix vowel count, palatalizes)
    const AFFIXES: &[(&str, usize, bool)] = &[
        ("ajiem", 3, false),
        ("ajai", 3, false),
        ("ajam", 2, false),
        ("ajām", 2, false),
        ("ajos", 2, false),
        ("ajās", 2, false),
        ("iem", 2, true),
        ("ajā", 2, false),
        ("ais", 2, false),
        ("ai", 2, false),
        ("ei", 2, false),
        ("ām", 1, false),
        ("am", 1, false),
        ("ēm", 1, false),
        ("īm", 1, false),
        ("im", 1, false),
        ("um", 1, false),
        ("us", 1, true),
        ("as", 1, false),
        ("ās", 1, false),
        ("es", 1, false),
        ("os", 1, true),
        ("ij", 1, false),
        ("īs", 1, false),
        ("ēs", 1, false),
        ("is", 1, false),
        ("ie", 1, false),
        ("u", 1, true),
        ("a", 1, true),
        ("i", 1, true),
        ("e", 1, false),
        ("ā", 1, false),
        ("ē", 1, false),
        ("ī", 1, false),
        ("ū", 1, false),
        ("o", 1, false),
        ("s", 0, false),
        ("š", 0, false),
    ];

    let mut s: Vec<char> = word.chars().collect();
    let mut len = s.len();
    let num_vowels = s[..len].iter().filter(|c| is_vowel(**c)).count();

    for (affix, vc, palatalizes) in AFFIXES {
        let alen = affix.chars().count();
        if num_vowels > *vc && len >= alen + 3 && ends_with_lat(&s, len, affix) {
            len -= alen;
            return if *palatalizes {
                unpalatalize(&mut s, len)
            } else {
                s[..len].iter().collect()
            };
        }
    }

    s[..len].iter().collect()
}

fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'ā' | 'ī' | 'ē' | 'ū')
}

/// Most cases are handled except for the ambiguous ones:
/// s→š, t→š, d→ž, z→ž.
fn unpalatalize(s: &mut [char], len: usize) -> String {
    let mut len = len;
    // -u removal: 2, 5, or 6 gen. pl. — only these two can apply then.
    if s[len] == 'u' {
        // kš -> kst
        if ends_with_lat(s, len, "kš") {
            len += 1;
            s[len - 2] = 's';
            s[len - 1] = 't';
            return s[..len].iter().collect();
        }
        // ņņ -> nn
        if ends_with_lat(s, len, "ņņ") {
            s[len - 2] = 'n';
            s[len - 1] = 'n';
            return s[..len].iter().collect();
        }
    }

    // otherwise all other rules
    if ends_with_lat(s, len, "pj")
        || ends_with_lat(s, len, "bj")
        || ends_with_lat(s, len, "mj")
        || ends_with_lat(s, len, "vj")
    {
        // labial consonant
        len -= 1;
    } else if ends_with_lat(s, len, "šņ") {
        s[len - 2] = 's';
        s[len - 1] = 'n';
    } else if ends_with_lat(s, len, "žņ") {
        s[len - 2] = 'z';
        s[len - 1] = 'n';
    } else if ends_with_lat(s, len, "šļ") {
        s[len - 2] = 's';
        s[len - 1] = 'l';
    } else if ends_with_lat(s, len, "žļ") {
        s[len - 2] = 'z';
        s[len - 1] = 'l';
    } else if ends_with_lat(s, len, "ļņ") {
        s[len - 2] = 'l';
        s[len - 1] = 'n';
    } else if ends_with_lat(s, len, "ļļ") {
        s[len - 2] = 'l';
        s[len - 1] = 'l';
    } else if s[len - 1] == 'č' {
        s[len - 1] = 'c';
    } else if s[len - 1] == 'ļ' {
        s[len - 1] = 'l';
    } else if s[len - 1] == 'ņ' {
        s[len - 1] = 'n';
    }

    s[..len].iter().collect()
}

fn ends_with_lat(s: &[char], len: usize, suffix: &str) -> bool {
    let suffix: Vec<char> = suffix.chars().collect();
    len >= suffix.len() && &s[len - suffix.len()..len] == suffix.as_slice()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pizza_engine::analysis::TokenFilter;

    #[test]
    fn test_latvian_stem() {
        let filter = LatvianStemTokenFilter::new();

        // Test noun ending removal
        let mut token = Token::new("grāmatas", 0, 12, 0);
        filter.filter(&mut token);
        assert!(token.term.len() < "grāmatas".len());

        // Short word unchanged
        let mut token = Token::new("un", 0, 2, 0);
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "un");
    }
}
