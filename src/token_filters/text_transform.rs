use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

// ═══════════════════════════════════════════════════════════════════════════════
// TEXT TRANSFORM FILTERS — Case, format, style conversions
// ═══════════════════════════════════════════════════════════════════════════════

/// Converts to camelCase: "hello_world" → "helloWorld"
#[derive(Clone, Debug)]
pub struct CamelCaseTokenFilter;
impl CamelCaseTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for CamelCaseTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for CamelCaseTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let mut result = String::with_capacity(text.len());
        let mut capitalize_next = false;
        let mut first = true;
        for c in text.chars() {
            if c == '_' || c == '-' || c == ' ' {
                capitalize_next = true;
            } else if capitalize_next {
                for uc in c.to_uppercase() {
                    result.push(uc);
                }
                capitalize_next = false;
            } else if first {
                result.push(c.to_lowercase().next().unwrap_or(c));
                first = false;
            } else {
                result.push(c);
            }
        }
        if result != text {
            token.term = Cow::Owned(result);
        }
        (false, None)
    }
}

/// Converts to snake_case: "helloWorld" → "hello_world"
#[derive(Clone, Debug)]
pub struct SnakeCaseTokenFilter;
impl SnakeCaseTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for SnakeCaseTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for SnakeCaseTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let mut result = String::with_capacity(text.len() + 4);
        let mut prev_lower = false;
        for c in text.chars() {
            if c.is_uppercase() && prev_lower {
                result.push('_');
                result.push(c.to_lowercase().next().unwrap_or(c));
            } else if c == '-' || c == ' ' {
                result.push('_');
            } else {
                result.push(c.to_lowercase().next().unwrap_or(c));
            }
            prev_lower = c.is_lowercase();
        }
        if result != text {
            token.term = Cow::Owned(result);
        }
        (false, None)
    }
}

/// Converts to kebab-case: "helloWorld" → "hello-world"
#[derive(Clone, Debug)]
pub struct KebabCaseTokenFilter;
impl KebabCaseTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for KebabCaseTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for KebabCaseTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let mut result = String::with_capacity(text.len() + 4);
        let mut prev_lower = false;
        for c in text.chars() {
            if c.is_uppercase() && prev_lower {
                result.push('-');
                result.push(c.to_lowercase().next().unwrap_or(c));
            } else if c == '_' || c == ' ' {
                result.push('-');
            } else {
                result.push(c.to_lowercase().next().unwrap_or(c));
            }
            prev_lower = c.is_lowercase();
        }
        if result != text {
            token.term = Cow::Owned(result);
        }
        (false, None)
    }
}

/// Converts to PascalCase: "hello_world" → "HelloWorld"
#[derive(Clone, Debug)]
pub struct PascalCaseTokenFilter;
impl PascalCaseTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for PascalCaseTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for PascalCaseTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let mut result = String::with_capacity(text.len());
        let mut capitalize_next = true;
        for c in text.chars() {
            if c == '_' || c == '-' || c == ' ' {
                capitalize_next = true;
            } else if capitalize_next {
                for uc in c.to_uppercase() {
                    result.push(uc);
                }
                capitalize_next = false;
            } else {
                result.push(c);
            }
        }
        if result != text {
            token.term = Cow::Owned(result);
        }
        (false, None)
    }
}

/// Converts text to a URL-friendly slug: "Hello World!" → "hello-world"
#[derive(Clone, Debug)]
pub struct SlugifyTokenFilter;
impl SlugifyTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for SlugifyTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for SlugifyTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let mut result = String::with_capacity(text.len());
        let mut prev_dash = false;
        for c in text.chars() {
            if c.is_alphanumeric() {
                result.push(c.to_lowercase().next().unwrap_or(c));
                prev_dash = false;
            } else if !prev_dash && !result.is_empty() {
                result.push('-');
                prev_dash = true;
            }
        }
        let trimmed = result.trim_end_matches('-');
        token.term = Cow::Owned(String::from(trimmed));
        (false, None)
    }
}

/// Splits camelCase/PascalCase into separate tokens.
/// "getElementById" → ["get", "element", "by", "id"] (emits as synonyms).
#[derive(Clone, Debug)]
pub struct CamelCaseSplitTokenFilter {
    pub keep_original: bool,
}
impl CamelCaseSplitTokenFilter {
    pub fn new() -> Self {
        Self {
            keep_original: true,
        }
    }
}
impl Default for CamelCaseSplitTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for CamelCaseSplitTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let parts = split_camel_case(text);
        if parts.len() <= 1 {
            return (false, None);
        }
        let extras: Vec<Token<'a>> = parts
            .iter()
            .map(|part| Token {
                term: Cow::Owned(part.to_lowercase()),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            })
            .collect();
        if self.keep_original {
            (false, Some(extras))
        } else {
            token.term = Cow::Owned(extras[0].term.to_string());
            (false, Some(extras[1..].to_vec()))
        }
    }
}

/// Reverses each word in a multi-word token (preserving word order).
/// "hello world" → "olleh dlrow"
#[derive(Clone, Debug)]
pub struct WordReverseTokenFilter;
impl WordReverseTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for WordReverseTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for WordReverseTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.contains(' ') {
            let reversed: String = text
                .split(' ')
                .map(|w| w.chars().rev().collect::<String>())
                .collect::<Vec<_>>()
                .join(" ");
            token.term = Cow::Owned(reversed);
        } else {
            let reversed: String = text.chars().rev().collect();
            token.term = Cow::Owned(reversed);
        }
        (false, None)
    }
}

/// Converts text to Pig Latin. "hello" → "ellohay", "string" → "ingstray"
#[derive(Clone, Debug)]
pub struct PigLatinTokenFilter;
impl PigLatinTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for PigLatinTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for PigLatinTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.is_empty() {
            return (false, None);
        }
        let lower = text.to_lowercase();
        let chars: Vec<char> = lower.chars().collect();
        let result = if is_vowel_char(chars[0]) {
            format!("{}way", lower)
        } else {
            let consonant_end = chars.iter().position(|c| is_vowel_char(*c)).unwrap_or(1);
            format!("{}{}ay", &lower[consonant_end..], &lower[..consonant_end])
        };
        token.term = Cow::Owned(result);
        (false, None)
    }
}

/// Doubles each character (stuttering effect): "hello" → "hheelllloo"
#[derive(Clone, Debug)]
pub struct RepeatCharTokenFilter {
    pub times: usize,
}
impl RepeatCharTokenFilter {
    pub fn new(times: usize) -> Self {
        Self { times }
    }
}
impl Default for RepeatCharTokenFilter {
    fn default() -> Self {
        Self::new(2)
    }
}

impl TokenFilter for RepeatCharTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let repeated: String = text
            .chars()
            .flat_map(|c| core::iter::repeat(c).take(self.times))
            .collect();
        token.term = Cow::Owned(repeated);
        (false, None)
    }
}

/// Collapses repeated characters: "heeellooo" → "helo"
#[derive(Clone, Debug)]
pub struct CollapseRepeatsTokenFilter {
    pub max_repeats: usize,
}
impl CollapseRepeatsTokenFilter {
    pub fn new(max: usize) -> Self {
        Self { max_repeats: max }
    }
}
impl Default for CollapseRepeatsTokenFilter {
    fn default() -> Self {
        Self::new(1)
    }
}

impl TokenFilter for CollapseRepeatsTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let mut result = String::with_capacity(text.len());
        let mut count = 0usize;
        let mut prev: Option<char> = None;
        for c in text.chars() {
            if Some(c) == prev {
                count += 1;
                if count < self.max_repeats {
                    result.push(c);
                }
            } else {
                result.push(c);
                count = 0;
                prev = Some(c);
            }
        }
        if result != text {
            token.term = Cow::Owned(result);
        }
        (false, None)
    }
}

/// Pads token to a minimum length with a specified char.
#[derive(Clone, Debug)]
pub struct PadTokenFilter {
    pub min_length: usize,
    pub pad_char: char,
    pub pad_left: bool,
}
impl PadTokenFilter {
    pub fn new(min_length: usize, pad_char: char, pad_left: bool) -> Self {
        Self {
            min_length,
            pad_char,
            pad_left,
        }
    }
}
impl Default for PadTokenFilter {
    fn default() -> Self {
        Self::new(8, '0', true)
    }
}

impl TokenFilter for PadTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let char_count = text.chars().count();
        if char_count >= self.min_length {
            return (false, None);
        }
        let padding: String = core::iter::repeat(self.pad_char)
            .take(self.min_length - char_count)
            .collect();
        let padded = if self.pad_left {
            format!("{}{}", padding, text)
        } else {
            format!("{}{}", text, padding)
        };
        token.term = Cow::Owned(padded);
        (false, None)
    }
}

/// Masks all but the first/last N characters: "password123" → "pa*******23"
#[derive(Clone, Debug)]
pub struct PartialMaskTokenFilter {
    pub visible_start: usize,
    pub visible_end: usize,
    pub mask_char: char,
}
impl PartialMaskTokenFilter {
    pub fn new(visible_start: usize, visible_end: usize) -> Self {
        Self {
            visible_start,
            visible_end,
            mask_char: '*',
        }
    }
}
impl Default for PartialMaskTokenFilter {
    fn default() -> Self {
        Self::new(2, 2)
    }
}

impl TokenFilter for PartialMaskTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();
        if len <= self.visible_start + self.visible_end {
            return (false, None);
        }
        let start: String = chars[..self.visible_start].iter().collect();
        let end: String = chars[len - self.visible_end..].iter().collect();
        let middle: String = core::iter::repeat(self.mask_char)
            .take(len - self.visible_start - self.visible_end)
            .collect();
        token.term = Cow::Owned(format!("{}{}{}", start, middle, end));
        (false, None)
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────

fn split_camel_case(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let bytes = s.as_bytes();
    for i in 1..bytes.len() {
        if bytes[i].is_ascii_uppercase() && bytes[i - 1].is_ascii_lowercase() {
            parts.push(&s[start..i]);
            start = i;
        }
    }
    if start < s.len() {
        parts.push(&s[start..]);
    }
    parts
}

fn is_vowel_char(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u')
}
