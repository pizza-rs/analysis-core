use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

// ═══════════════════════════════════════════════════════════════════════════════
// TIBETAN ANALYSIS — Syllable segmentation and normalization
// Tibetan script (U+0F00–U+0FFF) uses tsek (་ U+0F0B) as syllable delimiter.
// No word boundaries exist — segmentation is critical.
// ═══════════════════════════════════════════════════════════════════════════════

/// Tibetan tsek segmentation filter — splits Tibetan text on the tsek mark (་).
/// Each syllable becomes a separate token. This is essential for Tibetan
/// search since Tibetan has no word spaces.
#[derive(Clone, Debug)]
pub struct TibetanTsekSegmentTokenFilter;
impl TibetanTsekSegmentTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for TibetanTsekSegmentTokenFilter {
    fn default() -> Self {
        Self
    }
}

const TIBETAN_TSEK: char = '་'; // U+0F0B

impl TokenFilter for TibetanTsekSegmentTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.to_string();

        // Only process text containing Tibetan characters
        if !text
            .chars()
            .any(|c| (c as u32) >= 0x0F00 && (c as u32) <= 0x0FFF)
        {
            return (false, None);
        }

        // Split on tsek
        let syllables: Vec<&str> = text
            .split(TIBETAN_TSEK)
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if syllables.len() <= 1 {
            // Single syllable or no tsek — just clean trailing tsek
            let cleaned: String = text.chars().filter(|c| *c != TIBETAN_TSEK).collect();
            if cleaned != text && !cleaned.is_empty() {
                token.term = Cow::Owned(cleaned);
            }
            return (false, None);
        }

        // First syllable becomes the current token
        token.term = Cow::Owned(syllables[0].to_string());

        // Remaining syllables become additional tokens
        let mut extra: Vec<Token<'a>> = Vec::with_capacity(syllables.len() - 1);
        for (i, syl) in syllables[1..].iter().enumerate() {
            extra.push(Token {
                term: Cow::Owned(syl.to_string()),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position + (i as u32) + 1,
            });
        }
        (false, Some(extra))
    }
}

/// Removes Tibetan punctuation marks (shad ། , nyis-shad ༎, etc.)
/// while preserving the actual syllable content.
#[derive(Clone, Debug)]
pub struct TibetanPunctuationRemoveTokenFilter;
impl TibetanPunctuationRemoveTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for TibetanPunctuationRemoveTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for TibetanPunctuationRemoveTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let cleaned: String = text
            .chars()
            .filter(|c| {
                let cp = *c as u32;
                // Remove Tibetan punctuation: shad, nyis-shad, special marks
                !matches!(cp,
                    0x0F04..=0x0F12 | // Head marks, marks
                    0x0F14 |          // Comma
                    0x0F85 |          // Paluta
                    0x0F3A..=0x0F3D | // Brackets
                    0x0FBE..=0x0FC5 | // Astrological signs
                    0x0FC7..=0x0FCC | // More signs
                    0x0FCE..=0x0FCF   // More signs
                )
            })
            .collect();
        if cleaned != text {
            if cleaned.is_empty() {
                return (true, None);
            }
            token.term = Cow::Owned(cleaned);
        }
        (false, None)
    }
}

/// Tibetan stop syllable filter — removes common Tibetan grammatical particles.
/// These include case markers, conjunctions, and auxiliary particles.
#[derive(Clone, Debug)]
pub struct TibetanStopSyllableTokenFilter;
impl TibetanStopSyllableTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for TibetanStopSyllableTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for TibetanStopSyllableTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        // Common Tibetan grammatical particles (without tsek)
        if is_tibetan_stop(text) {
            return (true, None);
        }
        (false, None)
    }
}

fn is_tibetan_stop(syllable: &str) -> bool {
    matches!(
        syllable,
        "གི" | "གིས" | "ཀྱི" | "ཀྱིས" |  // Genitive/instrumental
        "ཡི" | "ཡིས" |                     // Genitive/instrumental
        "ནི" |                              // Topic marker
        "དང" |                              // Conjunction "and"
        "ལ" | "ན" | "ར" | "སུ" |           // Dative/locative
        "ནས" |                              // Ablative
        "ཀྱང" | "ཡང" |                     // "also/even"
        "པ" | "བ" |                          // Nominalizer
        "ཅིག" | "ཞིག" | "ཤིག" |             // Indefinite article
        "ཡིན" | "རེད" | "ཡོད" | "འདུག" |   // Copulas
        "དེ" | "འདི" | "དེར" |              // Demonstratives
        "ལས" | "བས" // Comparative
    )
}
