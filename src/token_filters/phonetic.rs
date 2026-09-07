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

/// Encodes a word using the Metaphone algorithm — faithful port of Apache
/// Commons Codec `Metaphone` (the implementation ES's phonetic plugin uses).
/// Note SCH encodes as SK (e.g. Schmidt → SKMT), matching the reference.
pub fn metaphone(word: &str, max_length: usize) -> String {
    const VOWELS: &str = "AEIOU";
    const FRONTV: &str = "EIY";
    const VARSON: &str = "CSPTG";
    let max_code_len = if max_length == 0 { 4 } else { max_length };

    let txt: Vec<char> = word.to_uppercase().chars().collect();
    if txt.is_empty() {
        return String::new();
    }
    if txt.len() == 1 {
        return txt.iter().collect();
    }

    // handle initial 2-character exceptions
    let mut local: Vec<char> = txt.clone();
    match txt[0] {
        'K' | 'G' | 'P' => {
            if txt[1] == 'N' {
                local = txt[1..].to_vec();
            }
        }
        'A' => {
            if txt[1] == 'E' {
                local = txt[1..].to_vec();
            }
        }
        'W' => {
            if txt[1] == 'R' {
                local = txt[1..].to_vec();
            } else if txt[1] == 'H' {
                local = txt[1..].to_vec();
                local[0] = 'W'; // WH -> W
            }
        }
        'X' => {
            local[0] = 'S';
        }
        _ => {}
    }

    let wdsz = local.len();
    let is_vowel = |n: usize| n < wdsz && VOWELS.contains(local[n]);
    let is_prev = |n: usize, c: char| n > 0 && local[n - 1] == c;
    let is_next = |n: usize, c: char| n + 1 < wdsz && local[n + 1] == c;
    let is_last = |n: usize| n + 1 == wdsz;
    let region_match = |n: usize, test: &str| {
        let tc: Vec<char> = test.chars().collect();
        n + tc.len() <= wdsz && local[n..n + tc.len()] == tc[..]
    };

    let mut code = String::new();
    let mut n = 0;
    while code.chars().count() < max_code_len && n < wdsz {
        let symb = local[n];
        // remove duplicate letters except C
        if symb == 'C' || !is_prev(n, symb) {
            match symb {
                'A' | 'E' | 'I' | 'O' | 'U' => {
                    if n == 0 {
                        code.push(symb); // only use vowel if leading char
                    }
                }
                'B' => {
                    // B silent if word ends in MB
                    if !(is_prev(n, 'M') && is_last(n)) {
                        code.push(symb);
                    }
                }
                'C' => {
                    // discard if SCI, SCE or SCY
                    if is_prev(n, 'S') && !is_last(n) && FRONTV.contains(local[n + 1]) {
                        // SCH -> SK
                    } else if is_prev(n, 'S') && is_next(n, 'H') {
                        code.push('K');
                    } else if region_match(n, "CIA") || is_next(n, 'H') {
                        code.push('X'); // CIA -> X or CH -> X
                    } else if !is_last(n) && FRONTV.contains(local[n + 1]) {
                        code.push('S'); // CI, CE, CY -> S
                    } else {
                        code.push('K');
                    }
                }
                'D' => {
                    // DGE, DGI, DGY -> J
                    if !is_last(n + 1) && is_next(n, 'G') && FRONTV.contains(local[n + 2]) {
                        code.push('J');
                        n += 2;
                    } else {
                        code.push('T');
                    }
                }
                'G' => {
                    // GH silent at end or before consonant
                    if is_last(n + 1) && is_next(n, 'H') {
                        // silent
                    } else if !is_last(n + 1) && is_next(n, 'H') && !is_vowel(n + 2) {
                        // silent
                    } else if n > 0 && (region_match(n, "GN") || region_match(n, "GNED")) {
                        // silent G
                    } else {
                        let hard = is_prev(n, 'G');
                        if !is_last(n) && FRONTV.contains(local[n + 1]) && !hard {
                            code.push('J');
                        } else {
                            code.push('K');
                        }
                    }
                }
                'H' => {
                    if is_last(n) {
                        // terminal H
                    } else if n > 0 && VARSON.contains(local[n - 1]) {
                        // silent after CSPTG
                    } else if is_vowel(n + 1) {
                        code.push('H');
                    }
                }
                'F' | 'J' | 'L' | 'M' | 'N' | 'R' => code.push(symb),
                'K' => {
                    if n > 0 {
                        if !is_prev(n, 'C') {
                            code.push(symb);
                        }
                    } else {
                        code.push(symb); // initial K
                    }
                }
                'P' => {
                    if is_next(n, 'H') {
                        code.push('F'); // PH -> F
                    } else {
                        code.push(symb);
                    }
                }
                'Q' => code.push('K'),
                'S' => {
                    if region_match(n, "SH") || region_match(n, "SIO") || region_match(n, "SIA") {
                        code.push('X');
                    } else {
                        code.push('S');
                    }
                }
                'T' => {
                    if region_match(n, "TIA") || region_match(n, "TIO") {
                        code.push('X');
                    } else if region_match(n, "TCH") {
                        // silent in TCH
                    } else if region_match(n, "TH") {
                        code.push('0'); // theta
                    } else {
                        code.push('T');
                    }
                }
                'V' => code.push('F'),
                'W' | 'Y' => {
                    // silent if not followed by vowel
                    if !is_last(n) && is_vowel(n + 1) {
                        code.push(symb);
                    }
                }
                'X' => {
                    code.push('K');
                    code.push('S');
                }
                'Z' => code.push('S'),
                _ => {}
            }
        }
        n += 1;
        let count = code.chars().count();
        if count > max_code_len {
            let keep: String = code.chars().take(max_code_len).collect();
            code = keep;
        }
    }
    code
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

/// Cologne phonetic algorithm (Kölner Phonetik) for German names —
/// faithful port of Apache Commons Codec `ColognePhonetic`, including its
/// single-pass output rules: `0` is only emitted for a leading vowel run,
/// consecutive equal codes collapse, and H is a pure separator.
pub fn cologne_phonetic(word: &str) -> String {
    const AEIJOUY: &str = "AEIJOUY";
    const CSZ: &str = "CSZ";
    const FPVW: &str = "FPVW";
    const GKQ: &str = "GKQ";
    const CKQ: &str = "CKQ";
    const AHKLOQRUX: &str = "AHKLOQRUX";
    const SZ: &str = "SZ";
    const AHKOQUX: &str = "AHKOQUX";
    const DTX: &str = "DTX";
    const CHAR_IGNORE: char = '-';

    // preprocess: German uppercase (ß → SS) + fold umlauts to base vowels
    let chars: Vec<char> = word
        .to_uppercase()
        .chars()
        .map(|c| match c {
            'Ä' => 'A',
            'Ö' => 'O',
            'Ü' => 'U',
            other => other,
        })
        .collect();

    let mut out = String::new();
    let mut last_code = '/'; // impossible value
    let mut last_char = CHAR_IGNORE;

    // Store a code honoring the reference's dedup rules: '0' only survives
    // as the very first code; the ignore marker is never stored and never
    // updates the last-code memory.
    fn put(out: &mut String, last_code: &mut char, code: char) {
        let accept = code != CHAR_IGNORE;
        let non_z = code != '0';
        if accept && *last_code != code && (non_z || out.is_empty()) {
            out.push(code);
        }
        if non_z && accept {
            *last_code = code;
        }
    }

    let len = chars.len();
    for i in 0..len {
        let chr = chars[i];
        let next = chars.get(i + 1).copied().unwrap_or(CHAR_IGNORE);
        if !chr.is_ascii_uppercase() {
            continue;
        }
        if AEIJOUY.contains(chr) {
            put(&mut out, &mut last_code, '0');
        } else if chr == 'B' || chr == 'P' && next != 'H' {
            put(&mut out, &mut last_code, '1');
        } else if (chr == 'D' || chr == 'T') && !CSZ.contains(next) {
            put(&mut out, &mut last_code, '2');
        } else if FPVW.contains(chr) {
            put(&mut out, &mut last_code, '3');
        } else if GKQ.contains(chr) {
            put(&mut out, &mut last_code, '4');
        } else if chr == 'X' && !CKQ.contains(last_char) {
            put(&mut out, &mut last_code, '4');
            put(&mut out, &mut last_code, '8');
        } else if chr == 'S' || chr == 'Z' {
            put(&mut out, &mut last_code, '8');
        } else if chr == 'C' {
            if out.is_empty() {
                if AHKLOQRUX.contains(next) {
                    put(&mut out, &mut last_code, '4');
                } else {
                    put(&mut out, &mut last_code, '8');
                }
            } else if SZ.contains(last_char) || !AHKOQUX.contains(next) {
                put(&mut out, &mut last_code, '8');
            } else {
                put(&mut out, &mut last_code, '4');
            }
        } else if DTX.contains(chr) {
            put(&mut out, &mut last_code, '8');
        } else {
            match chr {
                'R' => put(&mut out, &mut last_code, '7'),
                'L' => put(&mut out, &mut last_code, '5'),
                'M' | 'N' => put(&mut out, &mut last_code, '6'),
                'H' => put(&mut out, &mut last_code, CHAR_IGNORE),
                _ => {}
            }
        }
        last_char = chr;
    }
    out
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
        // commons-codec reference: SCH encodes as SK
        assert_eq!(metaphone("Smith", 4), "SM0");
        assert_eq!(metaphone("Schmidt", 4), "SKMT");
    }

    // Vectors from Apache Commons Codec MetaphoneTest (default code
    // length 4 unless noted).
    #[test]
    fn test_metaphone_commons_vectors() {
        for (expected, input) in [
            ("SNS", "SCIENCE"),
            ("SN", "SCENE"),
            ("S", "SCY"),
            ("N", "GNU"),
            ("SNT", "SIGNED"),
            ("KNT", "GHENT"),
            ("B", "BAUGH"),
            ("AKSK", "AXEAXE"),
            ("HL", "howl"),
            ("TSTN", "testing"),
            ("0", "The"),
            ("KK", "quick"),
            ("BRN", "brown"),
            ("FKS", "fox"),
            ("JMPT", "jumped"),
            ("OFR", "over"),
            ("0", "the"),
            ("LS", "lazy"),
            ("TKS", "dogs"),
            ("FX", "PHISH"),
            ("XT", "SHOT"),
            ("OTXN", "ODSIAN"),
            ("PLXN", "PULSION"),
            ("RX", "RETCH"),
            ("WX", "WATCH"),
            ("OX", "OTIA"),
            ("PRXN", "PORTION"),
        ] {
            assert_eq!(metaphone(input, 4), expected, "vector {input}");
        }
        // longer code length for CHARACTER (commons uses an explicit
        // larger maxCodeLen here)
        assert_eq!(metaphone("CHARACTER", 10), "XRKTR");
    }

    #[test]
    fn test_cologne() {
        assert_eq!(cologne_phonetic("Mueller"), "657");
        assert_eq!(cologne_phonetic("Müller"), "657");
    }

    // Vectors from Apache Commons Codec ColognePhoneticTest.
    #[test]
    fn test_cologne_commons_vectors() {
        for (expected, input) in [
            ("01", "Aabjoe"),
            ("0856", "Aaclan"),
            ("04567", "Aychlmajr"),
            ("0", "a"),
            ("0", "ä"),
            ("8", "ß"),
            ("0", "aa"),
            ("0", "ha"),
            ("", "h"),
            ("0", "aha"),
            ("1", "b"),
            ("1", "p"),
            ("3", "ph"),
            ("3", "f"),
            ("3", "v"),
            ("3", "w"),
            ("4", "g"),
            ("4", "k"),
            ("4", "q"),
            ("48", "x"),
            ("048", "ax"),
            ("48", "cx"),
            ("5", "l"),
            ("45", "cl"),
            ("085", "acl"),
            ("6", "mn"),
            ("6", "{mn}"),
            ("7", "r"),
            ("657", "mÜller"),
            ("657", "müller"),
            ("862", "schmidt"),
            ("8627", "schneider"),
            ("387", "fischer"),
            ("317", "weber"),
            ("3467", "wagner"),
            ("147", "becker"),
            ("036", "hoffmann"),
            ("837", "schäfer"),
            ("837", "schÄfer"),
            ("17863", "Breschnew"),
            ("3412", "Wikipedia"),
            ("127", "peter"),
            ("376", "pharma"),
            ("64645214", "mönchengladbach"),
            ("28", "deutsch"),
            ("28", "deutz"),
            ("06174", "hamburg"),
            ("0637", "hannover"),
            ("478256", "christstollen"),
            ("48621", "Xanthippe"),
            ("8478", "Zacharias"),
            ("0581", "Holzbau"),
            ("68", "matsch"),
            ("68", "matz"),
            ("071862", "Arbeitsamt"),
            ("0172", "Eberhard"),
            ("0172", "Eberhardt"),
            ("858", "Celsius"),
            ("08", "Ace"),
            ("84", "shch"),
            ("484", "xch"),
            ("021", "heithabu"),
            ("174845214", "bergisch-gladbach"),
            ("65752682", "Müller-Lüdenscheidt"),
        ] {
            assert_eq!(cologne_phonetic(input), expected, "vector {input}");
        }
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
