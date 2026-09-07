use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

// ═══════════════════════════════════════════════════════════════════════════════
// EMOJI FILTERS — Beyond Lucene: full emoji intelligence
// ═══════════════════════════════════════════════════════════════════════════════

/// Converts emoji characters to their text descriptions.
/// E.g. "😀" → "grinning_face", "❤️" → "red_heart"
#[derive(Clone, Debug)]
pub struct EmojiToTextTokenFilter {
    pub keep_original: bool,
}

impl EmojiToTextTokenFilter {
    pub fn new() -> Self {
        Self {
            keep_original: false,
        }
    }

    pub fn keeping_original() -> Self {
        Self {
            keep_original: true,
        }
    }
}

impl Default for EmojiToTextTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for EmojiToTextTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let mut descriptions = Vec::new();
        let mut has_emoji = false;

        for c in text.chars() {
            if let Some(desc) = emoji_to_description(c) {
                has_emoji = true;
                descriptions.push(desc);
            }
        }

        if !has_emoji {
            return (false, None);
        }

        let desc_text = descriptions.join("_");
        if self.keep_original {
            let synonym = Token {
                term: Cow::Owned(desc_text),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            };
            (false, Some(alloc::vec![synonym]))
        } else {
            token.term = Cow::Owned(desc_text);
            (false, None)
        }
    }
}

/// Removes all emoji characters from tokens.
#[derive(Clone, Debug)]
pub struct EmojiRemoveTokenFilter;

impl EmojiRemoveTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EmojiRemoveTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for EmojiRemoveTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let filtered: String = text.chars().filter(|c| !is_emoji(*c)).collect();
        if filtered.len() != text.len() {
            if filtered.is_empty() {
                return (true, None);
            }
            token.term = Cow::Owned(filtered);
        }
        (false, None)
    }
}

/// Extracts only emoji from tokens, emitting each as a separate token.
#[derive(Clone, Debug)]
pub struct EmojiExtractTokenFilter {
    pub remove_from_original: bool,
}

impl EmojiExtractTokenFilter {
    pub fn new() -> Self {
        Self {
            remove_from_original: false,
        }
    }
}

impl Default for EmojiExtractTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for EmojiExtractTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        // Idempotent: a token made up entirely of emoji characters is already
        // an extraction — emitting copies of it again would duplicate the
        // token (and self-feed if extras ever re-enter this filter).
        if !text.chars().all(is_emoji) {
            let emojis: Vec<char> = text.chars().filter(|c| is_emoji(*c)).collect();
            if !emojis.is_empty() {
                let extras: Vec<Token<'a>> = emojis
                    .iter()
                    .map(|e| Token {
                        term: Cow::Owned(String::from(*e)),
                        start_offset: token.start_offset,
                        end_offset: token.end_offset,
                        position: token.position,
                    })
                    .collect();

                if self.remove_from_original {
                    let filtered: String = text.chars().filter(|c| !is_emoji(*c)).collect();
                    if filtered.is_empty() {
                        return (true, Some(extras));
                    }
                    token.term = Cow::Owned(filtered);
                }

                return (false, Some(extras));
            }
        }
        (false, None)
    }
}

/// Detects emoji sentiment and emits a sentiment tag.
/// Positive emoji → "_sentiment:positive", Negative → "_sentiment:negative"
#[derive(Clone, Debug)]
pub struct EmojiSentimentTokenFilter;

impl EmojiSentimentTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EmojiSentimentTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for EmojiSentimentTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        if token.term.starts_with("_sentiment:") {
            return (false, None); // idempotent: never re-tag an emitted tag
        }
        let text = token.term.as_ref();
        let mut pos_score: i32 = 0;
        let mut neg_score: i32 = 0;

        for c in text.chars() {
            match emoji_sentiment(c) {
                1 => pos_score += 1,
                -1 => neg_score += 1,
                _ => {}
            }
        }

        if pos_score == 0 && neg_score == 0 {
            return (false, None);
        }

        let sentiment = if pos_score > neg_score {
            "positive"
        } else if neg_score > pos_score {
            "negative"
        } else {
            "neutral"
        };

        let tag = Token {
            term: Cow::Owned(format!("_sentiment:{}", sentiment)),
            start_offset: token.start_offset,
            end_offset: token.end_offset,
            position: token.position,
        };
        (false, Some(alloc::vec![tag]))
    }
}

/// Replaces text emoticons with emoji descriptions.
/// E.g. ":)" → "smile", ":(" → "sad", "<3" → "heart"
#[derive(Clone, Debug)]
pub struct EmoticonToTextTokenFilter;

impl EmoticonToTextTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EmoticonToTextTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for EmoticonToTextTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let replacement = match text {
            ":)" | ":-)" | ":D" | ":-D" | "=)" => "smile",
            ":(" | ":-(" | "=(" => "sad",
            ";)" | ";-)" => "wink",
            ":P" | ":-P" | ":p" | ":-p" => "tongue",
            "<3" => "heart",
            "</3" => "broken_heart",
            ":O" | ":-O" | ":o" => "surprised",
            "XD" | "xD" => "laughing",
            ">:(" | ">:-(" => "angry",
            ":'(" | ":'‑(" => "crying",
            "O:)" | "0:)" => "angel",
            ">:)" | ">:-)" => "devil",
            "B)" | "B-)" => "cool",
            "://" => return (false, None), // URL part, not emoticon
            ":/" | ":-/" => "skeptical",
            ":|" | ":-|" => "neutral_face",
            _ => return (false, None),
        };
        token.term = Cow::Owned(String::from(replacement));
        (false, None)
    }
}

// ─── Emoji helper functions ────────────────────────────────────────────────

fn is_emoji(c: char) -> bool {
    let cp = c as u32;
    matches!(cp,
        0x1F600..=0x1F64F | // Emoticons
        0x1F300..=0x1F5FF | // Misc Symbols and Pictographs
        0x1F680..=0x1F6FF | // Transport and Map
        0x1F1E0..=0x1F1FF | // Flags
        0x2600..=0x26FF   | // Misc symbols
        0x2700..=0x27BF   | // Dingbats
        0xFE00..=0xFE0F   | // Variation Selectors
        0x1F900..=0x1F9FF | // Supplemental Symbols
        0x1FA00..=0x1FA6F | // Chess Symbols
        0x1FA70..=0x1FAFF | // Symbols Extended-A
        0x200D            | // ZWJ
        0x2764            | // Heart
        0x2B50              // Star
    )
}

fn emoji_to_description(c: char) -> Option<&'static str> {
    Some(match c as u32 {
        0x1F600 => "grinning_face",
        0x1F601 => "beaming_face",
        0x1F602 => "joy",
        0x1F603 => "smiley",
        0x1F604 => "smile",
        0x1F605 => "sweat_smile",
        0x1F606 => "laughing",
        0x1F607 => "innocent",
        0x1F608 => "smiling_imp",
        0x1F609 => "wink",
        0x1F60A => "blush",
        0x1F60B => "yum",
        0x1F60C => "relieved",
        0x1F60D => "heart_eyes",
        0x1F60E => "sunglasses",
        0x1F60F => "smirk",
        0x1F610 => "neutral_face",
        0x1F611 => "expressionless",
        0x1F612 => "unamused",
        0x1F613 => "sweat",
        0x1F614 => "pensive",
        0x1F615 => "confused",
        0x1F616 => "confounded",
        0x1F617 => "kissing",
        0x1F618 => "kissing_heart",
        0x1F619 => "kissing_smiling_eyes",
        0x1F61A => "kissing_closed_eyes",
        0x1F61B => "stuck_out_tongue",
        0x1F61C => "stuck_out_tongue_winking",
        0x1F61D => "stuck_out_tongue_closed_eyes",
        0x1F61E => "disappointed",
        0x1F61F => "worried",
        0x1F620 => "angry",
        0x1F621 => "rage",
        0x1F622 => "cry",
        0x1F623 => "persevere",
        0x1F624 => "triumph",
        0x1F625 => "disappointed_relieved",
        0x1F626 => "frowning",
        0x1F627 => "anguished",
        0x1F628 => "fearful",
        0x1F629 => "weary",
        0x1F62A => "sleepy",
        0x1F62B => "tired_face",
        0x1F62C => "grimacing",
        0x1F62D => "sob",
        0x1F62E => "open_mouth",
        0x1F62F => "hushed",
        0x1F630 => "cold_sweat",
        0x1F631 => "scream",
        0x1F632 => "astonished",
        0x1F633 => "flushed",
        0x1F634 => "sleeping",
        0x1F635 => "dizzy_face",
        0x1F636 => "no_mouth",
        0x1F637 => "mask",
        0x1F44D => "thumbsup",
        0x1F44E => "thumbsdown",
        0x1F44F => "clap",
        0x1F44B => "wave",
        0x1F44C => "ok_hand",
        0x1F4AF => "hundred_points",
        0x1F525 => "fire",
        0x1F4A9 => "poop",
        0x1F4A5 => "boom",
        0x1F4AB => "dizzy",
        0x1F4AC => "speech_balloon",
        0x2764 => "red_heart",
        0x1F494 => "broken_heart",
        0x1F495 => "two_hearts",
        0x1F496 => "sparkling_heart",
        0x1F497 => "heartpulse",
        0x1F498 => "cupid",
        0x1F499 => "blue_heart",
        0x1F49A => "green_heart",
        0x1F49B => "yellow_heart",
        0x1F49C => "purple_heart",
        0x2B50 => "star",
        0x1F31F => "star2",
        0x1F308 => "rainbow",
        0x2600 => "sunny",
        0x1F327 => "rain",
        0x26A1 => "zap",
        0x1F4A1 => "bulb",
        0x1F389 => "tada",
        0x1F38A => "confetti_ball",
        0x1F3C6 => "trophy",
        0x1F680 => "rocket",
        0x2708 => "airplane",
        0x1F697 => "car",
        // Food & drink
        0x1F355 => "pizza",
        0x1F354 => "hamburger",
        0x1F35F => "fries",
        0x1F356 => "meat_on_bone",
        0x1F357 => "poultry_leg",
        0x1F349 => "watermelon",
        0x1F34E => "apple",
        0x1F34C => "banana",
        0x1F353 => "strawberry",
        0x1F36C => "candy",
        0x1F36B => "chocolate",
        0x1F366 => "ice_cream",
        0x1F370 => "cake",
        0x1F382 => "birthday_cake",
        0x1F36A => "cookie",
        0x2615 => "coffee",
        0x1F375 => "tea",
        0x1F37A => "beer",
        0x1F37B => "beers",
        0x1F377 => "wine_glass",
        0x1F374 => "cocktail",
        0x1F36E => "custard",
        0x1F36D => "lollipop",
        0x1F344 => "mushroom",
        // Common activity & object emoji
        0x26BD => "soccer",
        0x1F3C0 => "basketball",
        0x1F3C8 => "american_football",
        0x26F3 => "golf",
        0x1F3AF => "dart",
        0x1F3AE => "video_game",
        0x1F3B5 => "musical_note",
        0x1F4BB => "computer",
        0x1F4F1 => "phone",
        0x1F4BE => "floppy_disk",
        0x1F4DA => "books",
        0x1F4D6 => "open_book",
        0x270D => "writing_hand",
        0x1F58D => "crayon",
        0x1F4DD => "memo",
        0x1F4C8 => "chart_up",
        0x1F4C9 => "chart_down",
        0x1F9E0 => "brain",
        0x1F52D => "telescope",
        0x1F52E => "crystal_ball",
        _ => return None,
    })
}

fn emoji_sentiment(c: char) -> i32 {
    match c as u32 {
        // Positive
        0x1F600..=0x1F60A
        | 0x1F60D..=0x1F60E
        | 0x1F618..=0x1F61A
        | 0x1F44D
        | 0x1F44F
        | 0x1F4AF
        | 0x2764
        | 0x1F495..=0x1F49C
        | 0x1F525
        | 0x1F389..=0x1F38A
        | 0x1F3C6
        | 0x1F680
        | 0x2B50
        | 0x1F31F
        | 0x1F308 => 1,
        // Negative
        0x1F61E..=0x1F625 | 0x1F627..=0x1F62D | 0x1F630..=0x1F631 | 0x1F44E | 0x1F494 | 0x1F4A9 => {
            -1
        }
        _ => 0,
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
    fn extract_pure_emoji_token_is_not_duplicated() {
        let filter = EmojiExtractTokenFilter::new();
        let mut token = make_token("\u{1F355}");
        let (deleted, extras) = filter.filter(&mut token);
        assert!(!deleted);
        assert!(extras.is_none());
        assert_eq!(token.term.as_ref(), "\u{1F355}");
    }

    #[test]
    fn extract_mixed_token_emits_each_emoji_once() {
        let filter = EmojiExtractTokenFilter::new();
        let mut token = make_token("pizza\u{1F355}");
        let (deleted, extras) = filter.filter(&mut token);
        assert!(!deleted);
        assert_eq!(token.term.as_ref(), "pizza\u{1F355}");
        let extras = extras.expect("one extracted emoji");
        assert_eq!(extras.len(), 1);
        assert_eq!(extras[0].term.as_ref(), "\u{1F355}");
    }

    #[test]
    fn extract_remove_from_original_strips_emoji() {
        let filter = EmojiExtractTokenFilter {
            remove_from_original: true,
        };
        let mut token = make_token("pizza\u{1F355}");
        let (deleted, extras) = filter.filter(&mut token);
        assert!(!deleted);
        assert_eq!(token.term.as_ref(), "pizza");
        assert_eq!(
            extras.expect("extracted emoji").len(),
            1,
            "pure-emoji extraction emitted once"
        );
    }

    #[test]
    fn extract_is_idempotent_when_extras_reenter() {
        // Simulate the extras re-entering the filter (pipeline re-entry):
        // filtering an emitted pure-emoji token again must be a no-op, so
        // repeated application cannot grow the token stream.
        let filter = EmojiExtractTokenFilter::new();
        let mut token = make_token("\u{1F355}");
        for _ in 0..100 {
            let (deleted, extras) = filter.filter(&mut token);
            assert!(!deleted);
            assert!(extras.is_none());
        }
        assert_eq!(token.term.as_ref(), "\u{1F355}");
    }

    #[test]
    fn to_text_converts_pizza_emoji() {
        let filter = EmojiToTextTokenFilter {
            keep_original: false,
        };
        let mut token = make_token("\u{1F355}");
        let (deleted, extras) = filter.filter(&mut token);
        assert!(!deleted);
        assert!(extras.is_none());
        assert_eq!(token.term.as_ref(), "pizza");
    }

    #[test]
    fn to_text_keep_original_emits_synonym() {
        let filter = EmojiToTextTokenFilter { keep_original: true };
        let mut token = make_token("\u{1F355}");
        let (deleted, extras) = filter.filter(&mut token);
        assert!(!deleted);
        assert_eq!(token.term.as_ref(), "\u{1F355}");
        let extras = extras.expect("description synonym");
        assert_eq!(extras.len(), 1);
        assert_eq!(extras[0].term.as_ref(), "pizza");
    }

    #[test]
    fn to_text_leaves_plain_text_untouched() {
        let filter = EmojiToTextTokenFilter {
            keep_original: false,
        };
        let mut token = make_token("hello");
        let (deleted, extras) = filter.filter(&mut token);
        assert!(!deleted);
        assert!(extras.is_none());
        assert_eq!(token.term.as_ref(), "hello");
    }

    #[test]
    fn sentiment_tag_passes_through_unchanged() {
        let filter = EmojiSentimentTokenFilter::new();
        let mut token = make_token("_sentiment:positive");
        let (deleted, extras) = filter.filter(&mut token);
        assert!(!deleted);
        assert!(extras.is_none());
    }

    #[test]
    fn sentiment_tags_positive_and_negative() {
        let filter = EmojiSentimentTokenFilter::new();
        let mut token = make_token("\u{1F602}");
        let (_, extras) = filter.filter(&mut token);
        assert_eq!(
            extras.expect("sentiment tag")[0].term.as_ref(),
            "_sentiment:positive"
        );

        let mut token = make_token("\u{1F621}");
        let (_, extras) = filter.filter(&mut token);
        assert_eq!(
            extras.expect("sentiment tag")[0].term.as_ref(),
            "_sentiment:negative"
        );
    }
}
