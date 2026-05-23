use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Spanish light stemmer.
///
/// Removes common Spanish plural and gender suffixes.
#[derive(Clone, Debug, Default)]
pub struct SpanishLightStemTokenFilter;

impl SpanishLightStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for SpanishLightStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.len() < 5 {
            return (false, None);
        }

        let stemmed = stem_spanish_light(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_spanish_light(word: &str) -> String {
    let mut result = String::from(word);

    // Remove plural markers
    if result.ends_with("eses") {
        result.truncate(result.len() - 4);
        result.push_str("és");
        return result;
    }

    if result.ends_with("ces") {
        result.truncate(result.len() - 3);
        result.push('z');
        return result;
    }

    if result.ends_with("os") || result.ends_with("as") || result.ends_with("es") {
        result.truncate(result.len() - 2);
        return result;
    }

    if result.ends_with('o') || result.ends_with('a') || result.ends_with('e') {
        result.pop();
        return result;
    }

    if result.ends_with('s') {
        result.pop();
    }

    result
}

/// Italian light stemmer.
///
/// Removes common Italian suffixes.
#[derive(Clone, Debug, Default)]
pub struct ItalianLightStemTokenFilter;

impl ItalianLightStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for ItalianLightStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.len() < 5 {
            return (false, None);
        }

        let stemmed = stem_italian_light(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_italian_light(word: &str) -> String {
    let mut result = String::from(word);

    // Remove plural/gender suffixes
    if result.ends_with("chi") || result.ends_with("ghi") {
        result.truncate(result.len() - 1); // chi → ch, ghi → gh
        result.push('o');
        return result;
    }

    if result.ends_with("ione") || result.ends_with("ioni") {
        result.truncate(result.len() - 4);
        return result;
    }

    if result.ends_with("mente") {
        result.truncate(result.len() - 5);
        return result;
    }

    if result.ends_with('i')
        || result.ends_with('e')
        || result.ends_with('a')
        || result.ends_with('o')
    {
        result.pop();
    }

    result
}

/// Portuguese light stemmer.
///
/// Removes common Portuguese suffixes.
#[derive(Clone, Debug, Default)]
pub struct PortugueseLightStemTokenFilter;

impl PortugueseLightStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for PortugueseLightStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.len() < 5 {
            return (false, None);
        }

        let stemmed = stem_portuguese_light(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_portuguese_light(word: &str) -> String {
    let mut result = String::from(word);

    // Remove plural markers
    if result.ends_with("ões") || result.ends_with("ães") {
        result.truncate(result.len() - "ões".len());
        result.push_str("ão");
        return result;
    }

    if result.ends_with("ais") {
        result.truncate(result.len() - 3);
        result.push_str("al");
        return result;
    }

    if result.ends_with("éis") {
        result.truncate(result.len() - "éis".len());
        result.push_str("el");
        return result;
    }

    if result.ends_with("eis") {
        result.truncate(result.len() - 3);
        result.push_str("el");
        return result;
    }

    if result.ends_with("os") || result.ends_with("as") || result.ends_with("es") {
        result.truncate(result.len() - 2);
        return result;
    }

    if result.ends_with('s') {
        result.pop();
    }

    result
}

/// Russian light stemmer.
///
/// Removes common Russian suffixes.
#[derive(Clone, Debug, Default)]
pub struct RussianLightStemTokenFilter;

impl RussianLightStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for RussianLightStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let chars: Vec<char> = text.chars().collect();
        if chars.len() < 4 {
            return (false, None);
        }

        if let Some(new_len) = stem_russian_light(&chars) {
            if new_len < chars.len() {
                let stemmed: String = chars[..new_len].iter().collect();
                token.term = Cow::Owned(stemmed);
            }
        }
        (false, None)
    }
}

fn stem_russian_light(chars: &[char]) -> Option<usize> {
    let len = chars.len();

    // 5-char suffixes
    if len > 6 {
        let suffix: String = chars[len - 5..].iter().collect();
        match suffix.as_str() {
            "ейший" | "ейшая" | "ейшее" | "ейшие" => return Some(len - 5),
            _ => {}
        }
    }

    // 4-char suffixes
    if len > 5 {
        let suffix: String = chars[len - 4..].iter().collect();
        match suffix.as_str() {
            "ость" | "ений" | "ения" => return Some(len - 4),
            _ => {}
        }
    }

    // 3-char suffixes
    if len > 4 {
        let suffix: String = chars[len - 3..].iter().collect();
        match suffix.as_str() {
            "ами" | "ями" | "ому" | "ому" | "ого" | "ним" | "ных" | "ить" | "ать" | "ять"
            | "ной" | "ное" | "ная" | "ные" => return Some(len - 3),
            _ => {}
        }
    }

    // 2-char suffixes
    if len > 3 {
        let suffix: String = chars[len - 2..].iter().collect();
        match suffix.as_str() {
            "ов" | "ев" | "ей" | "ий" | "ая" | "ое" | "ые" | "ий" | "ам" | "ям" | "ом" | "ем"
            | "ах" | "ях" | "ую" | "юю" | "ть" | "ей" | "ий" | "ых" | "их" => {
                return Some(len - 2)
            }
            _ => {}
        }
    }

    // 1-char suffixes
    if len > 3 {
        let last = chars[len - 1];
        match last {
            'а' | 'е' | 'и' | 'о' | 'у' | 'ы' | 'ь' | 'я' | 'й' => return Some(len - 1),
            _ => {}
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_token(term: &str) -> Token<'_> {
        Token {
            term: Cow::Borrowed(term),
            start_offset: 0,
            end_offset: term.len() as u32,
            position: 0,
        }
    }

    #[test]
    fn test_spanish_light() {
        let filter = SpanishLightStemTokenFilter::new();
        let mut token = make_token("gatos");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "gat");
    }

    #[test]
    fn test_italian_light() {
        let filter = ItalianLightStemTokenFilter::new();
        let mut token = make_token("ragazzi");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "ragazz");
    }

    #[test]
    fn test_portuguese_light() {
        let filter = PortugueseLightStemTokenFilter::new();
        let mut token = make_token("gatos");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "gato");
    }

    #[test]
    fn test_russian_light() {
        let filter = RussianLightStemTokenFilter::new();
        let mut token = make_token("книги");
        filter.filter(&mut token);
        // Should remove suffix
        assert!(token.term.as_ref().len() < "книги".len());
    }
}
