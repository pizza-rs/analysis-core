use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Brazilian Portuguese stemmer based on the Lucene BrazilianStemmer.
///
/// This is distinct from the Snowball Portuguese stemmer. It applies
/// specific rules for Brazilian Portuguese morphology.
#[derive(Clone, Debug, Default)]
pub struct BrazilianStemTokenFilter;

impl BrazilianStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for BrazilianStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.len() < 4 {
            return (false, None);
        }

        let stemmed = stem_brazilian(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_brazilian(word: &str) -> String {
    let lower = word.to_lowercase();
    let len = lower.len();

    if len < 4 {
        return lower;
    }

    // Try plural reduction
    if let Some(result) = reduce_plural(&lower) {
        return result;
    }

    // Try feminine to masculine
    if let Some(result) = reduce_feminine(&lower) {
        return result;
    }

    // Try adverb stripping
    if lower.ends_with("mente") && len > 7 {
        return lower[..len - 5].to_string();
    }

    // Try augmentative/diminutive removal
    if let Some(result) = reduce_augmentative(&lower) {
        return result;
    }

    // Try verb suffix removal
    if let Some(result) = remove_verb_suffix(&lower) {
        return result;
    }

    // Try noun suffix removal
    if let Some(result) = remove_noun_suffix(&lower) {
        return result;
    }

    lower
}

fn reduce_plural(word: &str) -> Option<String> {
    let len = word.len();

    if word.ends_with("ns") && len > 3 {
        let mut result = word[..len - 2].to_string();
        result.push('m');
        return Some(result);
    }

    if word.ends_with("ões") && len > 4 {
        let mut result = word[..len - "ões".len()].to_string();
        result.push_str("ão");
        return Some(result);
    }

    if word.ends_with("ães") && len > 3 {
        let result = word[..len - 1].to_string();
        return Some(result);
    }

    if word.ends_with("ais") && len > 4 {
        let mut result = word[..len - 2].to_string();
        result.push('l');
        return Some(result);
    }

    if word.ends_with("éis") && len > 4 {
        let mut result = word[..len - "éis".len()].to_string();
        result.push_str("el");
        return Some(result);
    }

    if word.ends_with("eis") && len > 4 {
        let mut result = word[..len - 2].to_string();
        result.push_str("el");
        return Some(result);
    }

    if word.ends_with("óis") && len > 4 {
        let mut result = word[..len - "óis".len()].to_string();
        result.push_str("ol");
        return Some(result);
    }

    if word.ends_with("is") && len > 3 {
        let mut result = word[..len - 2].to_string();
        result.push('l');
        return Some(result);
    }

    if word.ends_with("les") && len > 4 {
        return Some(word[..len - 2].to_string());
    }

    if word.ends_with("res") && len > 4 {
        return Some(word[..len - 2].to_string());
    }

    if word.ends_with("s") && len > 3 {
        return Some(word[..len - 1].to_string());
    }

    None
}

fn reduce_feminine(word: &str) -> Option<String> {
    let len = word.len();

    if word.ends_with("ona") && len > 4 {
        let mut result = word[..len - 3].to_string();
        result.push_str("ão");
        return Some(result);
    }

    if word.ends_with("ora") && len > 4 {
        return Some(word[..len - 1].to_string());
    }

    if word.ends_with("na") && len > 4 {
        let mut result = word[..len - 1].to_string();
        result.push('o');
        return Some(result);
    }

    if word.ends_with("inha") && len > 5 {
        let mut result = word[..len - 4].to_string();
        result.push_str("inho");
        return Some(result);
    }

    if word.ends_with("esa") && len > 4 {
        let mut result = word[..len - 3].to_string();
        result.push_str("ês");
        return Some(result);
    }

    if word.ends_with("osa") && len > 4 {
        let mut result = word[..len - 1].to_string();
        result.push('o');
        return Some(result);
    }

    if word.ends_with("íaca") && len > 5 {
        let mut result = word[..len - "íaca".len()].to_string();
        result.push_str("íaco");
        return Some(result);
    }

    if word.ends_with("ica") && len > 4 {
        let mut result = word[..len - 1].to_string();
        result.push('o');
        return Some(result);
    }

    if word.ends_with("ida") && len > 4 {
        let mut result = word[..len - 1].to_string();
        result.push('o');
        return Some(result);
    }

    if word.ends_with("ada") && len > 4 {
        let mut result = word[..len - 1].to_string();
        result.push('o');
        return Some(result);
    }

    if word.ends_with("a") && len > 3 {
        let mut result = word[..len - 1].to_string();
        result.push('o');
        return Some(result);
    }

    None
}

fn reduce_augmentative(word: &str) -> Option<String> {
    let len = word.len();

    let suffixes = [
        ("díssimo", 7usize),
        ("abilíssimo", "abilíssimo".len()),
        ("íssimo", "íssimo".len()),
        ("ésimo", "ésimo".len()),
        ("érrimo", "érrimo".len()),
        ("zinho", 5),
        ("quinho", 6),
        ("uinho", 5),
        ("adinho", 6),
        ("inho", 4),
        ("alhão", "alhão".len()),
        ("uça", "uça".len()),
        ("aça", "aça".len()),
        ("adão", "adão".len()),
        ("ázio", "ázio".len()),
        ("arraz", 5),
        ("arra", 4),
        ("zão", "zão".len()),
        ("ão", "ão".len()),
    ];

    for &(suffix, suffix_len) in &suffixes {
        if word.ends_with(suffix) && len > suffix_len + 2 {
            return Some(word[..len - suffix_len].to_string());
        }
    }

    None
}

fn remove_verb_suffix(word: &str) -> Option<String> {
    let len = word.len();

    let suffixes = [
        "aríamos", "eríamos", "iríamos", "ássemos", "êssemos", "íssemos", "aríeis", "eríeis",
        "iríeis", "ásseis", "ésseis", "ísseis", "áramos", "éramos", "íramos", "ávamos", "aremos",
        "eremos", "iremos", "ariam", "eriam", "iriam", "assem", "essem", "issem", "aram", "eram",
        "iram", "avam", "arem", "erem", "irem", "ando", "endo", "indo", "adas", "idas", "arás",
        "aras", "eras", "iras", "arei", "erei", "irei", "aria", "eria", "iria", "asse", "esse",
        "isse", "aste", "este", "iste", "aram", "ara", "era", "ira", "ava", "iam", "ado", "ido",
        "ará", "erá", "irá", "ar", "er", "ir", "ou", "eu", "iu",
    ];

    for suffix in &suffixes {
        if word.ends_with(suffix) && len > suffix.len() + 2 {
            return Some(word[..len - suffix.len()].to_string());
        }
    }

    None
}

fn remove_noun_suffix(word: &str) -> Option<String> {
    let len = word.len();

    let suffixes = [
        "encialista",
        "abilidade",
        "icionista",
        "entemente",
        "ividade",
        "imento",
        "amento",
        "ização",
        "mente",
        "idade",
        "ência",
        "ância",
        "ismo",
        "ista",
        "ável",
        "ível",
        "eira",
        "eiro",
        "ente",
        "ante",
        "ação",
        "ição",
        "ador",
        "edor",
        "ível",
        "oso",
        "osa",
        "eza",
        "ura",
    ];

    for suffix in &suffixes {
        if word.ends_with(suffix) && len > suffix.len() + 2 {
            return Some(word[..len - suffix.len()].to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use pizza_engine::analysis::TokenFilter;

    #[test]
    fn test_brazilian_plural() {
        let filter = BrazilianStemTokenFilter::new();

        let mut token = Token::new("coações", 0, 10, 0);
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "coação");

        let mut token = Token::new("livros", 0, 6, 0);
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "livro");
    }

    #[test]
    fn test_brazilian_short_words() {
        let filter = BrazilianStemTokenFilter::new();
        let mut token = Token::new("sol", 0, 3, 0);
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "sol");
    }
}
