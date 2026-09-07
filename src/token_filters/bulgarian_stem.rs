use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Bulgarian light stemmer based on the Lucene BulgarianStemmer.
///
/// Removes common Bulgarian suffixes to normalize inflected forms.
/// Handles noun, adjective, and verb endings.
#[derive(Clone, Debug, Default)]
pub struct BulgarianStemTokenFilter;

impl BulgarianStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for BulgarianStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let stemmed = stem_bulgarian(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

/// Lucene BulgarianStemmer — faithful port of
/// `org.apache.lucene.analysis.bg.BulgarianStemmer` (light stemming per
/// "A light stemmer for Bulgarian", Saralegi & Lopez de Lacalle). Operates
/// on a char vector with Java's mutate-and-return-length semantics.
fn stem_bulgarian(word: &str) -> String {
    let mut s: Vec<char> = word.chars().collect();
    let n = s.len();
    let len = stem_bulgarian_chars(&mut s, n);
    s[..len].iter().collect()
}

fn stem_bulgarian_chars(s: &mut [char], mut len: usize) -> usize {
    if len < 4 {
        return len; // do not stem
    }

    if len > 5 && ends_with(s, len, "ища") {
        return len - 3;
    }

    len = remove_article(s, len);
    len = remove_plural(s, len);

    if len > 3 {
        if ends_with(s, len, "я") {
            len -= 1;
        }
        if ends_with(s, len, "а") || ends_with(s, len, "о") || ends_with(s, len, "е") {
            len -= 1;
        }
    }

    // The rule rewriting ен -> н is duplicated in the paper; the perl
    // implementation fixes it (as does this port).
    if len > 4 && ends_with(s, len, "ен") {
        s[len - 2] = 'н';
        len -= 1;
    }

    if len > 5 && s[len - 2] == 'ъ' {
        s[len - 2] = s[len - 1]; // ъN -> N
        len -= 1;
    }

    len
}

fn remove_article(s: &[char], len: usize) -> usize {
    if len > 6 && ends_with(s, len, "ият") {
        return len - 3;
    }

    if len > 5 {
        if ends_with(s, len, "ът")
            || ends_with(s, len, "то")
            || ends_with(s, len, "те")
            || ends_with(s, len, "та")
            || ends_with(s, len, "ия")
        {
            return len - 2;
        }
    }

    if len > 4 && ends_with(s, len, "ят") {
        return len - 2;
    }

    len
}

fn remove_plural(s: &mut [char], len: usize) -> usize {
    if len > 6 {
        if ends_with(s, len, "овци") {
            return len - 3; // replace with о
        }
        if ends_with(s, len, "ове") {
            return len - 3;
        }
        if ends_with(s, len, "еве") {
            s[len - 3] = 'й';
            return len - 2;
        }
    }

    if len > 5 {
        if ends_with(s, len, "ища") {
            return len - 3;
        }
        if ends_with(s, len, "та") {
            return len - 2;
        }
        if ends_with(s, len, "ци") {
            s[len - 2] = 'к';
            return len - 1;
        }
        if ends_with(s, len, "зи") {
            s[len - 2] = 'г';
            return len - 1;
        }

        if s[len - 3] == 'е' && s[len - 1] == 'и' {
            s[len - 3] = 'я'; // е → я, drop и
            return len - 1;
        }
    }

    if len > 4 {
        if ends_with(s, len, "си") {
            s[len - 2] = 'х';
            return len - 1;
        }
        if ends_with(s, len, "и") {
            return len - 1;
        }
    }

    len
}

fn ends_with(s: &[char], len: usize, suffix: &str) -> bool {
    let suffix: Vec<char> = suffix.chars().collect();
    len >= suffix.len() && &s[len - suffix.len()..len] == suffix.as_slice()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pizza_engine::analysis::TokenFilter;

    #[test]
    fn test_bulgarian_stem() {
        let filter = BulgarianStemTokenFilter::new();

        // Test definite article removal
        let mut token = Token::new("книгата", 0, 14, 0);
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "книг");

        // Short words unchanged
        let mut token = Token::new("аз", 0, 4, 0);
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "аз");
    }
}
