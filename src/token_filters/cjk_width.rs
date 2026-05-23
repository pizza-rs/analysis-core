use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Normalizes CJK width differences:
/// - Fullwidth ASCII variants (U+FF01–U+FF5E) → Basic Latin (U+0021–U+007E)
/// - Halfwidth Katakana (U+FF65–U+FF9F) → Fullwidth Katakana equivalents
///
/// This is essential for CJK text search where users may mix fullwidth and
/// halfwidth characters.
#[derive(Clone, Debug, Default)]
pub struct CjkWidthTokenFilter;

impl CjkWidthTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for CjkWidthTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();

        // Fast path: check if any fullwidth/halfwidth chars exist
        let needs_folding = text.chars().any(|c| {
            let cp = c as u32;
            (0xFF01..=0xFF5E).contains(&cp) || (0xFF65..=0xFF9F).contains(&cp)
        });

        if !needs_folding {
            return (false, None);
        }

        let mut result = String::with_capacity(text.len());
        let mut chars = text.chars().peekable();

        while let Some(c) = chars.next() {
            let cp = c as u32;
            if (0xFF01..=0xFF5E).contains(&cp) {
                // Fullwidth ASCII → Basic Latin
                let folded = char::from_u32(cp - 0xFEE0).unwrap_or(c);
                result.push(folded);
            } else if (0xFF65..=0xFF9F).contains(&cp) {
                // Halfwidth Katakana → Fullwidth Katakana
                let (base, can_combine) = halfwidth_to_fullwidth_kana(cp);
                if can_combine {
                    // Check for voicing marks
                    if let Some(&next) = chars.peek() {
                        let next_cp = next as u32;
                        if next_cp == 0xFF9E {
                            // Dakuten (voiced)
                            chars.next();
                            if let Some(voiced) = apply_dakuten(base) {
                                result.push(voiced);
                            } else {
                                result.push(base);
                                result.push('\u{3099}');
                            }
                            continue;
                        } else if next_cp == 0xFF9F {
                            // Handakuten (semi-voiced)
                            chars.next();
                            if let Some(semi) = apply_handakuten(base) {
                                result.push(semi);
                            } else {
                                result.push(base);
                                result.push('\u{309A}');
                            }
                            continue;
                        }
                    }
                }
                result.push(base);
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

/// Convert a halfwidth katakana code point to its fullwidth equivalent.
/// Returns (fullwidth_char, can_combine_with_voicing_mark).
fn halfwidth_to_fullwidth_kana(cp: u32) -> (char, bool) {
    match cp {
        0xFF65 => ('\u{30FB}', false), // Halfwidth Katakana Middle Dot
        0xFF66 => ('\u{30F2}', false), // ヲ
        0xFF67 => ('\u{30A1}', false), // ァ
        0xFF68 => ('\u{30A3}', false), // ィ
        0xFF69 => ('\u{30A5}', false), // ゥ
        0xFF6A => ('\u{30A7}', false), // ェ
        0xFF6B => ('\u{30A9}', false), // ォ
        0xFF6C => ('\u{30E3}', false), // ャ
        0xFF6D => ('\u{30E5}', false), // ュ
        0xFF6E => ('\u{30E7}', false), // ョ
        0xFF6F => ('\u{30C3}', false), // ッ
        0xFF70 => ('\u{30FC}', false), // ー (prolonged sound mark)
        0xFF71 => ('\u{30A2}', false), // ア
        0xFF72 => ('\u{30A4}', false), // イ
        0xFF73 => ('\u{30A6}', true),  // ウ (can become ヴ)
        0xFF74 => ('\u{30A8}', false), // エ
        0xFF75 => ('\u{30AA}', false), // オ
        0xFF76 => ('\u{30AB}', true),  // カ
        0xFF77 => ('\u{30AD}', true),  // キ
        0xFF78 => ('\u{30AF}', true),  // ク
        0xFF79 => ('\u{30B1}', true),  // ケ
        0xFF7A => ('\u{30B3}', true),  // コ
        0xFF7B => ('\u{30B5}', true),  // サ
        0xFF7C => ('\u{30B7}', true),  // シ
        0xFF7D => ('\u{30B9}', true),  // ス
        0xFF7E => ('\u{30BB}', true),  // セ
        0xFF7F => ('\u{30BD}', true),  // ソ
        0xFF80 => ('\u{30BF}', true),  // タ
        0xFF81 => ('\u{30C1}', true),  // チ
        0xFF82 => ('\u{30C4}', true),  // ツ
        0xFF83 => ('\u{30C6}', true),  // テ
        0xFF84 => ('\u{30C8}', true),  // ト
        0xFF85 => ('\u{30CA}', false), // ナ
        0xFF86 => ('\u{30CB}', false), // ニ
        0xFF87 => ('\u{30CC}', false), // ヌ
        0xFF88 => ('\u{30CD}', false), // ネ
        0xFF89 => ('\u{30CE}', false), // ノ
        0xFF8A => ('\u{30CF}', true),  // ハ (can become バ or パ)
        0xFF8B => ('\u{30D2}', true),  // ヒ
        0xFF8C => ('\u{30D5}', true),  // フ
        0xFF8D => ('\u{30D8}', true),  // ヘ
        0xFF8E => ('\u{30DB}', true),  // ホ
        0xFF8F => ('\u{30DE}', false), // マ
        0xFF90 => ('\u{30DF}', false), // ミ
        0xFF91 => ('\u{30E0}', false), // ム
        0xFF92 => ('\u{30E1}', false), // メ
        0xFF93 => ('\u{30E2}', false), // モ
        0xFF94 => ('\u{30E4}', false), // ヤ
        0xFF95 => ('\u{30E6}', false), // ユ
        0xFF96 => ('\u{30E8}', false), // ヨ
        0xFF97 => ('\u{30E9}', false), // ラ
        0xFF98 => ('\u{30EA}', false), // リ
        0xFF99 => ('\u{30EB}', false), // ル
        0xFF9A => ('\u{30EC}', false), // レ
        0xFF9B => ('\u{30ED}', false), // ロ
        0xFF9C => ('\u{30EF}', false), // ワ
        0xFF9D => ('\u{30F3}', false), // ン
        0xFF9E => ('\u{3099}', false), // Dakuten (combining)
        0xFF9F => ('\u{309A}', false), // Handakuten (combining)
        _ => (char::from_u32(cp).unwrap_or('?'), false),
    }
}

/// Apply dakuten (voicing mark) to a fullwidth katakana character.
fn apply_dakuten(c: char) -> Option<char> {
    let cp = c as u32;
    match cp {
        // Ka-row: カ→ガ, キ→ギ, etc.
        0x30AB | 0x30AD | 0x30AF | 0x30B1 | 0x30B3 => char::from_u32(cp + 1),
        // Sa-row: サ→ザ, etc.
        0x30B5 | 0x30B7 | 0x30B9 | 0x30BB | 0x30BD => char::from_u32(cp + 1),
        // Ta-row: タ→ダ, チ→ヂ, ツ→ヅ, テ→デ, ト→ド
        0x30BF | 0x30C1 | 0x30C4 | 0x30C6 | 0x30C8 => char::from_u32(cp + 1),
        // Ha-row: ハ→バ, ヒ→ビ, フ→ブ, ヘ→ベ, ホ→ボ
        0x30CF | 0x30D2 | 0x30D5 | 0x30D8 | 0x30DB => char::from_u32(cp + 1),
        // ウ→ヴ
        0x30A6 => Some('\u{30F4}'),
        _ => None,
    }
}

/// Apply handakuten (semi-voicing mark) to a fullwidth katakana character.
fn apply_handakuten(c: char) -> Option<char> {
    let cp = c as u32;
    match cp {
        // Ha-row only: ハ→パ, ヒ→ピ, フ→プ, ヘ→ペ, ホ→ポ
        0x30CF | 0x30D2 | 0x30D5 | 0x30D8 | 0x30DB => char::from_u32(cp + 2),
        _ => None,
    }
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
    fn test_fullwidth_ascii_to_latin() {
        let filter = CjkWidthTokenFilter::new();
        // Ｈｅｌｌｏ (fullwidth) → Hello
        let mut token = make_token("\u{FF28}\u{FF45}\u{FF4C}\u{FF4C}\u{FF4F}");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "Hello");
    }

    #[test]
    fn test_fullwidth_digits() {
        let filter = CjkWidthTokenFilter::new();
        // １２３ → 123
        let mut token = make_token("\u{FF11}\u{FF12}\u{FF13}");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "123");
    }

    #[test]
    fn test_halfwidth_katakana() {
        let filter = CjkWidthTokenFilter::new();
        // ｱｲｳ → アイウ
        let mut token = make_token("\u{FF71}\u{FF72}\u{FF73}");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "アイウ");
    }

    #[test]
    fn test_no_change() {
        let filter = CjkWidthTokenFilter::new();
        let mut token = make_token("hello");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "hello");
    }
}
