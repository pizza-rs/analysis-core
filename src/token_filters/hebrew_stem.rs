use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

// ═══════════════════════════════════════════════════════════════════════════════
// HEBREW ANALYSIS — Stemmer and normalization for Hebrew text
// ═══════════════════════════════════════════════════════════════════════════════

/// Removes Hebrew niqqud (vowel diacritics) and cantillation marks.
/// Hebrew text is often stored with or without niqqud; this normalizes both forms.
#[derive(Clone, Debug)]
pub struct HebrewNiqqudRemoveTokenFilter;
impl HebrewNiqqudRemoveTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for HebrewNiqqudRemoveTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for HebrewNiqqudRemoveTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let cleaned: String = text
            .chars()
            .filter(|c| {
                let cp = *c as u32;
                // Remove niqqud (U+0591–U+05BD, U+05BF, U+05C1–U+05C2, U+05C4–U+05C5, U+05C7)
                // These are cantillation marks and vowel points
                !((0x0591..=0x05BD).contains(&cp)
                    || cp == 0x05BF
                    || (0x05C1..=0x05C2).contains(&cp)
                    || (0x05C4..=0x05C5).contains(&cp)
                    || cp == 0x05C7)
            })
            .collect();
        if cleaned.len() != text.len() {
            token.term = Cow::Owned(cleaned);
        }
        (false, None)
    }
}

/// Hebrew light stemmer — removes common prefixes and suffixes.
/// Based on Hebrew morphology: strips definite article, prepositions,
/// conjunctions, and common noun/verb suffixes.
#[derive(Clone, Debug)]
pub struct HebrewStemTokenFilter;
impl HebrewStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for HebrewStemTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for HebrewStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let chars: Vec<char> = text.chars().collect();

        // Only process Hebrew text (U+0590–U+05FF)
        if chars.is_empty()
            || !chars
                .iter()
                .any(|c| (*c as u32) >= 0x05D0 && (*c as u32) <= 0x05EA)
        {
            return (false, None);
        }

        let mut start = 0;
        let mut end = chars.len();

        // Strip common prefixes (single-letter): ה (ha, the), ו (ve, and),
        // ב (be, in), כ (ke, like), ל (le, to), מ (mi, from), ש (she, that)
        if end - start > 3 {
            let first = chars[start];
            if matches!(first, 'ה' | 'ו' | 'ב' | 'כ' | 'ל' | 'מ' | 'ש') {
                start += 1;
            }
        }

        // Strip two-letter prefixes: וה (veha), שה (sheha), מה (meha),
        // לה (leha), בה (beha), כש (keshe)
        if start == 0 && end - start > 4 && chars.len() >= 2 {
            let prefix: String = chars[0..2].iter().collect();
            if matches!(prefix.as_str(), "וה" | "שה" | "מה" | "לה" | "בה" | "כש") {
                start += 2;
            }
        }

        // Strip common suffixes
        if end - start > 3 {
            let last = chars[end - 1];
            let second_last = if end >= 2 { Some(chars[end - 2]) } else { None };

            // ים (masculine plural), ות (feminine plural)
            if end - start > 4 {
                if let Some(sl) = second_last {
                    if (sl == 'י' && last == 'ם') || (sl == 'ו' && last == 'ת') {
                        end -= 2;
                    }
                }
            }

            // ה (feminine singular suffix) — only if still long enough
            if end - start > 3 && chars[end - 1] == 'ה' {
                end -= 1;
            }
        }

        if start != 0 || end != chars.len() {
            let stemmed: String = chars[start..end].iter().collect();
            if !stemmed.is_empty() {
                token.term = Cow::Owned(stemmed);
            }
        }
        (false, None)
    }
}

/// Normalizes Hebrew final letters (sofit) to their regular forms.
/// ך→כ, ם→מ, ן→נ, ף→פ, ץ→צ
#[derive(Clone, Debug)]
pub struct HebrewFinalFormNormTokenFilter;
impl HebrewFinalFormNormTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for HebrewFinalFormNormTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for HebrewFinalFormNormTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let normalized: String = text
            .chars()
            .map(|c| match c {
                'ך' => 'כ', // Final Kaf → Kaf
                'ם' => 'מ', // Final Mem → Mem
                'ן' => 'נ', // Final Nun → Nun
                'ף' => 'פ', // Final Pe → Pe
                'ץ' => 'צ', // Final Tsadi → Tsadi
                _ => c,
            })
            .collect();
        if normalized != text {
            token.term = Cow::Owned(normalized);
        }
        (false, None)
    }
}
