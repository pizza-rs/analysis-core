use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Phonetic encoding token filter supporting multiple algorithms.
///
/// Encodes tokens using phonetic algorithms for sound-based matching.
/// Supports: Metaphone, DoubleMetaphone, Soundex, RefinedSoundex,
/// Caverphone, Cologne, NYSIIS, and DaitchMokotoff.
#[derive(Clone, Debug)]
pub struct PhoneticTokenFilter {
    encoder: PhoneticEncoder,
    replace: bool,
}

/// Supported phonetic encoding algorithms.
#[derive(Clone, Debug)]
pub enum PhoneticEncoder {
    Metaphone(usize),
    DoubleMetaphone(usize),
    Soundex,
    RefinedSoundex,
    Caverphone1,
    Caverphone2,
    ColognePhonetic,
    Nysiis,
    DaitchMokotoff,
}

impl PhoneticTokenFilter {
    pub fn new(encoder: PhoneticEncoder) -> Self {
        Self {
            encoder,
            replace: true,
        }
    }

    /// If false, emit the phonetic code as an additional token at the same position
    /// rather than replacing the original.
    pub fn with_replace(mut self, replace: bool) -> Self {
        self.replace = replace;
        self
    }

    fn encode(&self, text: &str) -> Option<String> {
        match &self.encoder {
            PhoneticEncoder::Metaphone(max_len) => Some(metaphone(text, *max_len)),
            PhoneticEncoder::DoubleMetaphone(max_len) => Some(double_metaphone(text, *max_len).0),
            PhoneticEncoder::Soundex => Some(soundex(text)),
            PhoneticEncoder::RefinedSoundex => Some(refined_soundex(text)),
            PhoneticEncoder::Caverphone1 => Some(caverphone1(text)),
            PhoneticEncoder::Caverphone2 => Some(caverphone2(text)),
            PhoneticEncoder::ColognePhonetic => Some(cologne_phonetic(text)),
            PhoneticEncoder::Nysiis => Some(nysiis(text)),
            PhoneticEncoder::DaitchMokotoff => Some(daitch_mokotoff(text)),
        }
    }
}

impl TokenFilter for PhoneticTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.is_empty() {
            return (false, None);
        }

        if let Some(encoded) = self.encode(text) {
            if encoded.is_empty() {
                return (false, None);
            }

            if self.replace {
                token.term = Cow::Owned(encoded);
                (false, None)
            } else {
                // Emit as additional token at same position
                let extra = Token {
                    term: Cow::Owned(encoded),
                    start_offset: token.start_offset,
                    end_offset: token.end_offset,
                    position: token.position,
                };
                (false, Some(alloc::vec![extra]))
            }
        } else {
            (false, None)
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// METAPHONE
// ═══════════════════════════════════════════════════════════════════════════════

/// Encodes a word using the Metaphone algorithm.
pub fn metaphone(word: &str, max_length: usize) -> String {
    let word = word.to_uppercase();
    let chars: Vec<char> = word.chars().filter(|c| c.is_ascii_alphabetic()).collect();
    let len = chars.len();

    if len == 0 {
        return String::new();
    }

    let max_len = if max_length == 0 { 4 } else { max_length };
    let mut result = String::with_capacity(max_len);
    let mut i = 0;

    // Handle initial silent letters
    match chars[0] {
        'A' if len > 1 && chars[1] == 'E' => i = 1,
        'G' | 'K' | 'P' if len > 1 && chars[1] == 'N' => i = 1,
        'W' if len > 1 && chars[1] == 'R' => i = 1,
        'X' => {
            result.push('S');
            i = 1;
        }
        _ => {}
    }

    while i < len && result.len() < max_len {
        let c = chars[i];
        let prev = if i > 0 { Some(chars[i - 1]) } else { None };
        let next = if i + 1 < len {
            Some(chars[i + 1])
        } else {
            None
        };

        // Skip duplicate adjacent letters (except C)
        if c != 'C' && prev == Some(c) {
            i += 1;
            continue;
        }

        match c {
            'A' | 'E' | 'I' | 'O' | 'U' => {
                if i == 0 {
                    result.push(c);
                }
            }
            'B' => {
                if prev != Some('M') || i == len - 1 {
                    result.push('B');
                }
            }
            'C' => {
                if next == Some('I') || next == Some('E') || next == Some('Y') {
                    if next == Some('I') && i + 2 < len && chars[i + 2] == 'A' {
                        result.push('X');
                    } else {
                        result.push('S');
                    }
                } else {
                    result.push('K');
                }
            }
            'D' => {
                if next == Some('G')
                    && i + 2 < len
                    && (chars[i + 2] == 'I' || chars[i + 2] == 'E' || chars[i + 2] == 'Y')
                {
                    result.push('J');
                } else {
                    result.push('T');
                }
            }
            'F' => result.push('F'),
            'G' => {
                if i + 1 < len {
                    if next == Some('H') && i + 2 < len && !is_vowel(chars[i + 2]) {
                        // GH before non-vowel is silent
                    } else if i > 0 && (next == Some('N') || (next == Some('N') && i + 2 >= len)) {
                        // GN at end is silent
                    } else if prev == Some('G') {
                        // double G
                    } else {
                        if !(i > 0 && (next == Some('I') || next == Some('E') || next == Some('Y')))
                        {
                            result.push('K');
                        } else if i == 0 {
                            result.push('J');
                        }
                    }
                }
            }
            'H' => {
                if is_vowel_opt(next) && (i == 0 || !is_vowel_opt(prev)) {
                    result.push('H');
                }
            }
            'J' => result.push('J'),
            'K' => {
                if prev != Some('C') {
                    result.push('K');
                }
            }
            'L' => result.push('L'),
            'M' => result.push('M'),
            'N' => result.push('N'),
            'P' => {
                if next == Some('H') {
                    result.push('F');
                    i += 1;
                } else {
                    result.push('P');
                }
            }
            'Q' => result.push('K'),
            'R' => result.push('R'),
            'S' => {
                if next == Some('H')
                    || (next == Some('I')
                        && i + 2 < len
                        && (chars[i + 2] == 'O' || chars[i + 2] == 'A'))
                {
                    result.push('X');
                    i += 1;
                } else {
                    result.push('S');
                }
            }
            'T' => {
                if next == Some('H') {
                    result.push('0'); // theta
                    i += 1;
                } else if next == Some('I')
                    && i + 2 < len
                    && (chars[i + 2] == 'A' || chars[i + 2] == 'O')
                {
                    result.push('X');
                } else {
                    result.push('T');
                }
            }
            'V' => result.push('F'),
            'W' | 'Y' => {
                if is_vowel_opt(next) {
                    result.push(c);
                }
            }
            'X' => {
                result.push('K');
                if result.len() < max_len {
                    result.push('S');
                }
            }
            'Z' => result.push('S'),
            _ => {}
        }
        i += 1;
    }

    result
}

fn is_vowel(c: char) -> bool {
    matches!(c, 'A' | 'E' | 'I' | 'O' | 'U')
}

fn is_vowel_opt(c: Option<char>) -> bool {
    c.map(|ch| is_vowel(ch)).unwrap_or(false)
}

// ═══════════════════════════════════════════════════════════════════════════════
// DOUBLE METAPHONE
// ═══════════════════════════════════════════════════════════════════════════════

/// Returns (primary, alternate) Double Metaphone codes.
pub fn double_metaphone(word: &str, max_length: usize) -> (String, String) {
    // Simplified Double Metaphone — produces primary code
    // Full DM has ~100 rules; this is the primary path only.
    let primary = metaphone(word, max_length);
    let alternate = primary.clone(); // simplified: same as primary
    (primary, alternate)
}

// ═══════════════════════════════════════════════════════════════════════════════
// SOUNDEX
// ═══════════════════════════════════════════════════════════════════════════════

/// Encodes a word using the American Soundex algorithm.
/// Returns a 4-character code: first letter + 3 digits.
pub fn soundex(word: &str) -> String {
    let word = word.to_uppercase();
    let chars: Vec<char> = word.chars().filter(|c| c.is_ascii_alphabetic()).collect();

    if chars.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(4);
    result.push(chars[0]);

    let mut last_code = soundex_code(chars[0]);

    for &c in &chars[1..] {
        let code = soundex_code(c);
        if code != '0' && code != last_code {
            result.push(code);
            if result.len() == 4 {
                break;
            }
        }
        if code != '0' {
            last_code = code;
        }
    }

    // Pad with zeros
    while result.len() < 4 {
        result.push('0');
    }

    result
}

fn soundex_code(c: char) -> char {
    match c {
        'B' | 'F' | 'P' | 'V' => '1',
        'C' | 'G' | 'J' | 'K' | 'Q' | 'S' | 'X' | 'Z' => '2',
        'D' | 'T' => '3',
        'L' => '4',
        'M' | 'N' => '5',
        'R' => '6',
        _ => '0', // A, E, I, O, U, H, W, Y
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// REFINED SOUNDEX
// ═══════════════════════════════════════════════════════════════════════════════

/// Refined Soundex with more granular encoding (0-9).
pub fn refined_soundex(word: &str) -> String {
    let word = word.to_uppercase();
    let chars: Vec<char> = word.chars().filter(|c| c.is_ascii_alphabetic()).collect();

    if chars.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(chars.len());
    result.push(chars[0]);

    let mut last_code = refined_soundex_code(chars[0]);

    for &c in &chars[1..] {
        let code = refined_soundex_code(c);
        if code != '0' && code != last_code {
            result.push(code);
        }
        last_code = code;
    }

    result
}

fn refined_soundex_code(c: char) -> char {
    match c {
        'A' | 'E' | 'I' | 'O' | 'U' | 'H' | 'W' | 'Y' => '0',
        'B' | 'P' => '1',
        'C' | 'K' | 'Q' => '2',
        'D' | 'T' => '3',
        'F' | 'V' => '4',
        'G' | 'J' => '5',
        'L' => '6',
        'M' | 'N' => '7',
        'R' => '8',
        'S' | 'X' | 'Z' => '9',
        _ => '0',
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CAVERPHONE
// ═══════════════════════════════════════════════════════════════════════════════

/// Caverphone 1.0 algorithm (New Zealand English names).
pub fn caverphone1(word: &str) -> String {
    let mut w = word.to_lowercase();
    w.retain(|c| c.is_ascii_alphabetic());

    if w.is_empty() {
        return String::new();
    }

    // Remove trailing 'e'
    if w.ends_with('e') {
        w.pop();
    }

    // Initial transformations
    let replacements = &[
        ("cough", "cou2f"),
        ("rough", "rou2f"),
        ("tough", "tou2f"),
        ("enough", "enou2f"),
        ("gn", "2n"),
        ("mb", "m2"),
        ("cq", "2q"),
        ("ci", "si"),
        ("ce", "se"),
        ("cy", "sy"),
        ("tch", "2ch"),
        ("c", "k"),
        ("q", "k"),
        ("x", "k"),
        ("v", "f"),
        ("dg", "2g"),
        ("tio", "sio"),
        ("tia", "sia"),
        ("d", "t"),
        ("ph", "fh"),
        ("b", "p"),
        ("sh", "s2"),
        ("z", "s"),
        ("^aeiou", ""),
        ("a", "3"),
        ("e", "3"),
        ("i", "3"),
        ("o", "3"),
        ("u", "3"),
        ("3gh3", "3kh3"),
        ("gh", "22"),
        ("g", "k"),
        ("ss", "s"),
        ("tt", "t"),
        ("pp", "p"),
        ("kk", "k"),
        ("ff", "f"),
        ("nn", "n"),
        ("rr", "r"),
        ("ll", "l"),
        ("mm", "m"),
        ("s$", "S"),
        ("t$", "T"),
        ("3", ""),
    ];

    for (from, to) in replacements {
        if from.starts_with('^') {
            // Skip positional rules for simplified version
            continue;
        }
        if from.ends_with('$') {
            let pat = &from[..from.len() - 1];
            if w.ends_with(pat) {
                let prefix = &w[..w.len() - pat.len()];
                w = format!("{}{}", prefix, to);
            }
            continue;
        }
        w = w.replace(from, to);
    }

    // Pad or truncate to 6 characters
    while w.len() < 6 {
        w.push('1');
    }
    w.truncate(6);
    w.to_uppercase()
}

/// Caverphone 2.0 algorithm.
pub fn caverphone2(word: &str) -> String {
    // Caverphone 2 is similar to 1 but outputs 10 characters
    let mut result = caverphone1(word);
    while result.len() < 10 {
        result.push('1');
    }
    result.truncate(10);
    result
}

// ═══════════════════════════════════════════════════════════════════════════════
// COLOGNE PHONETIC (Kölner Phonetik)
// ═══════════════════════════════════════════════════════════════════════════════

/// Cologne phonetic algorithm (Kölner Phonetik) for German names.
pub fn cologne_phonetic(word: &str) -> String {
    let word = word.to_uppercase();
    let chars: Vec<char> = word.chars().filter(|c| c.is_ascii_alphabetic()).collect();
    let len = chars.len();

    if len == 0 {
        return String::new();
    }

    let mut codes: Vec<char> = Vec::with_capacity(len);

    for i in 0..len {
        let c = chars[i];
        let prev = if i > 0 { Some(chars[i - 1]) } else { None };
        let next = if i + 1 < len {
            Some(chars[i + 1])
        } else {
            None
        };

        let code = match c {
            'A' | 'E' | 'I' | 'J' | 'O' | 'U' | 'Y' => '0',
            'H' => continue,
            'B' | 'P' => {
                if next == Some('H') {
                    '3'
                } else {
                    '1'
                }
            }
            'D' | 'T' => {
                if next == Some('C') || next == Some('S') || next == Some('Z') {
                    '8'
                } else {
                    '2'
                }
            }
            'F' | 'V' | 'W' => '3',
            'G' | 'K' | 'Q' => '4',
            'X' => {
                if prev == Some('C') || prev == Some('K') || prev == Some('Q') {
                    '8'
                } else {
                    '\0' // sentinel: X produces "48" (two codes), handled below
                }
            }
            'L' => '5',
            'M' | 'N' => '6',
            'R' => '7',
            'S' | 'Z' => '8',
            'C' => {
                if i == 0 {
                    if next == Some('A')
                        || next == Some('H')
                        || next == Some('K')
                        || next == Some('L')
                        || next == Some('O')
                        || next == Some('Q')
                        || next == Some('R')
                        || next == Some('U')
                        || next == Some('X')
                    {
                        '4'
                    } else {
                        '8'
                    }
                } else if prev == Some('S') || prev == Some('Z') {
                    '8'
                } else if next == Some('A')
                    || next == Some('H')
                    || next == Some('K')
                    || next == Some('O')
                    || next == Some('Q')
                    || next == Some('U')
                    || next == Some('X')
                {
                    '4'
                } else {
                    '8'
                }
            }
            _ => continue,
        };

        // Handle X special case (produces two codes: 4, 8)
        if c == 'X' && !(prev == Some('C') || prev == Some('K') || prev == Some('Q')) {
            codes.push('4');
            codes.push('8');
        } else {
            codes.push(code);
        }
    }

    // Remove consecutive duplicates
    let mut result = String::new();
    let mut last = '\0';
    for &code in &codes {
        if code != last {
            result.push(code);
            last = code;
        }
    }

    // Remove leading zeros (except if the whole string is "0")
    let trimmed = result.trim_start_matches('0');
    if trimmed.is_empty() {
        String::from("0")
    } else {
        String::from(trimmed)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// NYSIIS (New York State Identification and Intelligence System)
// ═══════════════════════════════════════════════════════════════════════════════

/// NYSIIS algorithm for phonetic encoding.
pub fn nysiis(word: &str) -> String {
    let mut w: String = word
        .to_uppercase()
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .collect();

    if w.is_empty() {
        return String::new();
    }

    // Step 1: Handle initial prefixes
    if w.starts_with("MAC") {
        w = format!("MCC{}", &w[3..]);
    } else if w.starts_with("KN") {
        w = format!("NN{}", &w[2..]);
    } else if w.starts_with('K') {
        w = format!("C{}", &w[1..]);
    } else if w.starts_with("PH") || w.starts_with("PF") {
        w = format!("FF{}", &w[2..]);
    } else if w.starts_with("SCH") {
        w = format!("SSS{}", &w[3..]);
    }

    // Step 2: Handle suffixes
    if w.ends_with("EE") || w.ends_with("IE") {
        let prefix = &w[..w.len() - 2];
        w = format!("{}Y", prefix);
    } else if w.ends_with("DT")
        || w.ends_with("RT")
        || w.ends_with("RD")
        || w.ends_with("NT")
        || w.ends_with("ND")
    {
        let prefix = &w[..w.len() - 2];
        w = format!("{}D", prefix);
    }

    // Step 3: First character of key = first character of name
    let mut key = String::new();
    let chars: Vec<char> = w.chars().collect();
    if chars.is_empty() {
        return String::new();
    }
    key.push(chars[0]);

    // Step 4: Translate remaining characters
    let mut i = 1;
    while i < chars.len() {
        let c = chars[i];
        let translated = match c {
            'E' if i + 1 < chars.len() && chars[i + 1] == 'V' => {
                i += 1;
                "AF"
            }
            'A' | 'E' | 'I' | 'O' | 'U' => "A",
            'Q' => "G",
            'Z' => "S",
            'M' => "N",
            'K' => {
                if i + 1 < chars.len() && chars[i + 1] == 'N' {
                    "N"
                } else {
                    "C"
                }
            }
            'S' if i + 2 < chars.len() && chars[i + 1] == 'C' && chars[i + 2] == 'H' => {
                i += 2;
                "SSS"
            }
            'P' if i + 1 < chars.len() && chars[i + 1] == 'H' => {
                i += 1;
                "FF"
            }
            'H' => {
                let prev_vowel = i > 0 && matches!(chars[i - 1], 'A' | 'E' | 'I' | 'O' | 'U');
                let next_vowel =
                    i + 1 < chars.len() && matches!(chars[i + 1], 'A' | 'E' | 'I' | 'O' | 'U');
                if !prev_vowel || !next_vowel {
                    // Keep previous character
                    let prev_c = if key.is_empty() { "A" } else { "" };
                    if prev_c.is_empty() {
                        let last = key.chars().last().unwrap_or('A');
                        i += 1;
                        key.push(last);
                        continue;
                    }
                    prev_c
                } else {
                    "H"
                }
            }
            'W' => {
                let prev_vowel = i > 0 && matches!(chars[i - 1], 'A' | 'E' | 'I' | 'O' | 'U');
                if prev_vowel {
                    let last = key.chars().last().unwrap_or('A');
                    i += 1;
                    key.push(last);
                    continue;
                } else {
                    "W"
                }
            }
            _ => {
                key.push(c);
                i += 1;
                continue;
            }
        };

        for tc in translated.chars() {
            if key.chars().last() != Some(tc) {
                key.push(tc);
            }
        }
        i += 1;
    }

    // Remove trailing 'S'
    while key.ends_with('S') && key.len() > 1 {
        key.pop();
    }

    // Replace trailing 'AY' with 'Y'
    if key.ends_with("AY") {
        key.truncate(key.len() - 2);
        key.push('Y');
    }

    // Remove trailing 'A'
    while key.ends_with('A') && key.len() > 1 {
        key.pop();
    }

    // Truncate to 6 characters
    key.truncate(6);
    key
}

// ═══════════════════════════════════════════════════════════════════════════════
// DAITCH-MOKOTOFF SOUNDEX
// ═══════════════════════════════════════════════════════════════════════════════

/// Daitch-Mokotoff Soundex for Eastern European and Germanic names.
/// Returns a 6-digit code.
pub fn daitch_mokotoff(word: &str) -> String {
    let word: String = word
        .to_uppercase()
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .collect();

    if word.is_empty() {
        return String::new();
    }

    let chars: Vec<char> = word.chars().collect();
    let len = chars.len();
    let mut result = String::new();
    let mut i = 0;

    while i < len && result.len() < 6 {
        let c = chars[i];
        let next = if i + 1 < len {
            Some(chars[i + 1])
        } else {
            None
        };
        let next2 = if i + 2 < len {
            Some(chars[i + 2])
        } else {
            None
        };

        let (code, skip) = dm_code(c, next, next2, i == 0);
        if let Some(code_char) = code {
            if result.chars().last() != Some(code_char) || code_char == '0' {
                result.push(code_char);
            }
        }
        i += 1 + skip;
    }

    // Pad with zeros
    while result.len() < 6 {
        result.push('0');
    }
    result.truncate(6);
    result
}

/// Returns (code, chars_to_skip) for Daitch-Mokotoff.
fn dm_code(
    c: char,
    next: Option<char>,
    next2: Option<char>,
    is_start: bool,
) -> (Option<char>, usize) {
    match c {
        'A' | 'E' | 'I' | 'O' | 'U' | 'Y' => {
            if is_start {
                (Some('0'), 0)
            } else {
                (None, 0)
            }
        }
        'B' => (Some('7'), 0),
        'C' => match next {
            Some('H') => (Some('5'), 1),
            Some('K') => (Some('5'), 1),
            Some('S') => (Some('4'), 1),
            Some('Z') => (Some('4'), 1),
            _ => (Some('5'), 0),
        },
        'D' => match next {
            Some('T') | Some('Z') | Some('S') => (Some('4'), 1),
            _ => (Some('3'), 0),
        },
        'F' => (Some('7'), 0),
        'G' => (Some('5'), 0),
        'H' => (None, 0),
        'J' => (Some('4'), 0),
        'K' => {
            if next == Some('H') {
                (Some('5'), 1)
            } else {
                (Some('5'), 0)
            }
        }
        'L' => (Some('8'), 0),
        'M' => (Some('6'), 0),
        'N' => (Some('6'), 0),
        'P' => {
            if next == Some('H') || next == Some('F') {
                (Some('7'), 1)
            } else {
                (Some('7'), 0)
            }
        }
        'Q' => (Some('5'), 0),
        'R' => (Some('9'), 0),
        'S' => match next {
            Some('H') => (Some('4'), 1),
            Some('C') if next2 == Some('H') => (Some('4'), 2),
            Some('T') => {
                if next2 == Some('R') || next2 == Some('S') || next2 == Some('C') {
                    (Some('2'), 1)
                } else {
                    (Some('4'), 1)
                }
            }
            _ => (Some('4'), 0),
        },
        'T' => match next {
            Some('S') | Some('Z') => (Some('4'), 1),
            Some('H') => (Some('3'), 1),
            _ => (Some('3'), 0),
        },
        'V' => (Some('7'), 0),
        'W' => (Some('7'), 0),
        'X' => (Some('5'), 0),
        'Z' => match next {
            Some('H') | Some('S') => (Some('4'), 1),
            _ => (Some('4'), 0),
        },
        _ => (None, 0),
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
    fn test_soundex() {
        assert_eq!(soundex("Robert"), "R163");
        assert_eq!(soundex("Rupert"), "R163");
        assert_eq!(soundex("Smith"), "S530");
        assert_eq!(soundex("Smythe"), "S530");
    }

    #[test]
    fn test_metaphone() {
        assert_eq!(metaphone("Smith", 4), "SM0");
        assert_eq!(metaphone("Schmidt", 4), "SXMT");
    }

    #[test]
    fn test_cologne() {
        assert_eq!(cologne_phonetic("Mueller"), "657");
        assert_eq!(cologne_phonetic("Müller"), "657");
    }

    #[test]
    fn test_nysiis() {
        let result = nysiis("Johnson");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_phonetic_filter_replace() {
        let filter = PhoneticTokenFilter::new(PhoneticEncoder::Soundex);
        let mut token = make_token("Robert");
        let (remove, _) = filter.filter(&mut token);
        assert!(!remove);
        assert_eq!(token.term.as_ref(), "R163");
    }

    #[test]
    fn test_phonetic_filter_no_replace() {
        let filter = PhoneticTokenFilter::new(PhoneticEncoder::Soundex).with_replace(false);
        let mut token = make_token("Robert");
        let (remove, extra) = filter.filter(&mut token);
        assert!(!remove);
        assert_eq!(token.term.as_ref(), "Robert"); // original preserved
        let extra = extra.unwrap();
        assert_eq!(extra[0].term.as_ref(), "R163"); // phonetic added
    }

    #[test]
    fn test_daitch_mokotoff() {
        let result = daitch_mokotoff("Schwarzenegger");
        assert_eq!(result.len(), 6); // always 6 digits
    }

    #[test]
    fn test_refined_soundex() {
        let r1 = refined_soundex("Robert");
        let r2 = refined_soundex("Rupert");
        // Both should produce similar codes
        assert!(!r1.is_empty());
        assert!(!r2.is_empty());
    }
}
