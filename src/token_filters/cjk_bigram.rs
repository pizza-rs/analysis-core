use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Forms bigrams from CJK (Han, Hiragana, Katakana, Hangul) characters.
///
/// Consecutive CJK characters are bigrammed: "中国人" → ["中国", "国人"]
/// Non-CJK tokens pass through unchanged.
/// Lone CJK characters (not adjacent to other CJK) are emitted as unigrams.
#[derive(Clone, Debug)]
pub struct CjkBigramTokenFilter {
    output_unigrams: bool,
    han: bool,
    hiragana: bool,
    katakana: bool,
    hangul: bool,
}

impl CjkBigramTokenFilter {
    pub fn new() -> Self {
        Self {
            output_unigrams: false,
            han: true,
            hiragana: true,
            katakana: true,
            hangul: true,
        }
    }

    pub fn with_output_unigrams(mut self, output: bool) -> Self {
        self.output_unigrams = output;
        self
    }

    pub fn with_han(mut self, enabled: bool) -> Self {
        self.han = enabled;
        self
    }

    pub fn with_hiragana(mut self, enabled: bool) -> Self {
        self.hiragana = enabled;
        self
    }

    pub fn with_katakana(mut self, enabled: bool) -> Self {
        self.katakana = enabled;
        self
    }

    pub fn with_hangul(mut self, enabled: bool) -> Self {
        self.hangul = enabled;
        self
    }

    fn is_cjk_char(&self, c: char) -> bool {
        if self.han && is_han(c) {
            return true;
        }
        if self.hiragana && is_hiragana(c) {
            return true;
        }
        if self.katakana && is_katakana(c) {
            return true;
        }
        if self.hangul && is_hangul(c) {
            return true;
        }
        false
    }
}

impl Default for CjkBigramTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for CjkBigramTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let chars: Vec<char> = text.chars().collect();

        // Check if the token contains any CJK characters
        let has_cjk = chars.iter().any(|c| self.is_cjk_char(*c));
        if !has_cjk {
            return (false, None);
        }

        // Collect runs of CJK characters and bigram them
        let mut bigrams: Vec<String> = Vec::new();
        let mut unigrams: Vec<String> = Vec::new();
        let mut i = 0;

        while i < chars.len() {
            if self.is_cjk_char(chars[i]) {
                // Start of a CJK run
                let run_start = i;
                while i < chars.len() && self.is_cjk_char(chars[i]) {
                    i += 1;
                }
                let run = &chars[run_start..i];

                if run.len() == 1 {
                    // Lone CJK character → unigram
                    unigrams.push(run[0].to_string());
                } else {
                    // Generate bigrams from the run
                    for j in 0..run.len() - 1 {
                        let bigram: String = run[j..=j + 1].iter().collect();
                        bigrams.push(bigram);
                    }
                    if self.output_unigrams {
                        for c in run {
                            unigrams.push(c.to_string());
                        }
                    }
                }
            } else {
                // Non-CJK character — skip or include as-is
                i += 1;
            }
        }

        // Combine results
        let mut all_tokens: Vec<String> = Vec::new();
        if self.output_unigrams {
            // Interleave unigrams and bigrams (unigram first at each position)
            all_tokens.extend(unigrams);
            all_tokens.extend(bigrams);
        } else {
            all_tokens.extend(bigrams);
            all_tokens.extend(unigrams); // lone CJK chars as unigrams
        }

        if all_tokens.is_empty() {
            return (true, None);
        }

        let mut iter = all_tokens.into_iter();
        token.term = Cow::Owned(iter.next().unwrap());

        let extra: Vec<Token<'a>> = iter
            .map(|t| Token {
                term: Cow::Owned(t),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            })
            .collect();

        if extra.is_empty() {
            (false, None)
        } else {
            (false, Some(extra))
        }
    }
}

#[inline]
fn is_han(c: char) -> bool {
    let cp = c as u32;
    // CJK Unified Ideographs
    (0x4E00..=0x9FFF).contains(&cp)
        || (0x3400..=0x4DBF).contains(&cp) // Extension A
        || (0x20000..=0x2A6DF).contains(&cp) // Extension B
        || (0x2A700..=0x2B73F).contains(&cp) // Extension C
        || (0x2B740..=0x2B81F).contains(&cp) // Extension D
        || (0xF900..=0xFAFF).contains(&cp) // Compatibility Ideographs
}

#[inline]
fn is_hiragana(c: char) -> bool {
    let cp = c as u32;
    (0x3040..=0x309F).contains(&cp)
}

#[inline]
fn is_katakana(c: char) -> bool {
    let cp = c as u32;
    (0x30A0..=0x30FF).contains(&cp) || (0x31F0..=0x31FF).contains(&cp)
}

#[inline]
fn is_hangul(c: char) -> bool {
    let cp = c as u32;
    (0xAC00..=0xD7AF).contains(&cp) // Hangul Syllables
        || (0x1100..=0x11FF).contains(&cp) // Jamo
        || (0x3130..=0x318F).contains(&cp) // Compatibility Jamo
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
    fn test_cjk_bigrams() {
        let filter = CjkBigramTokenFilter::new();
        let mut token = make_token("中国人");
        let (remove, extra) = filter.filter(&mut token);
        assert!(!remove);
        assert_eq!(token.term.as_ref(), "中国");
        let extra = extra.unwrap();
        assert_eq!(extra[0].term.as_ref(), "国人");
    }

    #[test]
    fn test_lone_cjk_unigram() {
        let filter = CjkBigramTokenFilter::new();
        let mut token = make_token("中");
        let (remove, _) = filter.filter(&mut token);
        assert!(!remove);
        assert_eq!(token.term.as_ref(), "中");
    }

    #[test]
    fn test_non_cjk_passthrough() {
        let filter = CjkBigramTokenFilter::new();
        let mut token = make_token("hello");
        let (remove, extra) = filter.filter(&mut token);
        assert!(!remove);
        assert!(extra.is_none());
        assert_eq!(token.term.as_ref(), "hello");
    }

    #[test]
    fn test_with_unigrams() {
        let filter = CjkBigramTokenFilter::new().with_output_unigrams(true);
        let mut token = make_token("東京");
        let (remove, extra) = filter.filter(&mut token);
        assert!(!remove);
        // Should output unigrams + bigram
        assert!(extra.is_some());
    }
}
