//! Additional light stemmers for languages previously lacking stemming.
//!
//! Provides suffix-stripping stemmers for Polish, Ukrainian, Armenian,
//! Croatian, Lithuanian, Swedish, Estonian, Slovak, Slovenian, Basque,
//! and Catalan.

use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

// ─── Polish Stemmer ───────────────────────────────────────────────────────

/// Polish stemmer based on suffix stripping (Stempel-inspired).
///
/// Removes common Polish inflectional suffixes (case, gender, number, tense).
#[derive(Clone, Debug, Default)]
pub struct PolishStemTokenFilter;

impl PolishStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for PolishStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.len() < 4 {
            return (false, None);
        }
        let stemmed = stem_polish(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_polish(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    let len = chars.len();

    if len > 7 {
        let suffix: String = chars[len - 5..].iter().collect();
        match suffix.as_str() {
            "ności" | "ością" | "ienie" | "aniem" | "ający" => {
                return chars[..len - 5].iter().collect();
            }
            _ => {}
        }
    }

    if len > 6 {
        let suffix: String = chars[len - 4..].iter().collect();
        match suffix.as_str() {
            "ości" | "nych" | "iego" | "owej" | "owym" | "anie" | "enie" | "eniu" | "aniu"
            | "iach" | "kami" => {
                return chars[..len - 4].iter().collect();
            }
            _ => {}
        }
    }

    if len > 5 {
        let suffix: String = chars[len - 3..].iter().collect();
        match suffix.as_str() {
            "ach" | "ami" | "emu" | "owi" | "ych" | "ego" | "nej" | "nym" | "cie" | "arz"
            | "nik" | "iem" | "ość" => {
                return chars[..len - 3].iter().collect();
            }
            _ => {}
        }
    }

    if len > 4 {
        let suffix: String = chars[len - 2..].iter().collect();
        match suffix.as_str() {
            "ie" | "ej" | "em" | "ze" | "ię" | "ek" | "ce" | "mi" | "ka" | "ki" | "ko" | "ny"
            | "na" | "ne" | "om" | "ów" => {
                return chars[..len - 2].iter().collect();
            }
            _ => {}
        }
    }

    if len > 3 {
        match chars[len - 1] {
            'a' | 'e' | 'i' | 'o' | 'u' | 'y' | 'ę' | 'ą' => {
                return chars[..len - 1].iter().collect();
            }
            _ => {}
        }
    }

    word.to_string()
}

// ─── Ukrainian Stemmer ────────────────────────────────────────────────────

/// Ukrainian stemmer based on suffix stripping.
///
/// Removes common Ukrainian inflectional suffixes (case, gender, number).
#[derive(Clone, Debug, Default)]
pub struct UkrainianStemTokenFilter;

impl UkrainianStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for UkrainianStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let char_count = text.chars().count();
        if char_count < 4 {
            return (false, None);
        }
        let stemmed = stem_ukrainian(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_ukrainian(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    let len = chars.len();

    if len > 7 {
        let suffix: String = chars[len - 5..].iter().collect();
        match suffix.as_str() {
            "ський" | "ської" | "ським" | "ських" | "ність" | "ності" =>
            {
                return chars[..len - 5].iter().collect();
            }
            _ => {}
        }
    }

    if len > 6 {
        let suffix: String = chars[len - 4..].iter().collect();
        match suffix.as_str() {
            "ного" | "ному" | "ній" | "ним" | "них" | "ною" | "ної" | "ення" | "ання" | "ство"
            | "ості" | "ість" => {
                return chars[..len - 4].iter().collect();
            }
            _ => {}
        }
    }

    if len > 5 {
        let suffix: String = chars[len - 3..].iter().collect();
        match suffix.as_str() {
            "ого" | "ому" | "ій" | "им" | "их" | "ам" | "ах" | "ові" | "ами" | "ять" | "ати"
            | "ити" | "ють" | "ені" | "ані" => {
                return chars[..len - 3].iter().collect();
            }
            _ => {}
        }
    }

    if len > 4 {
        let suffix: String = chars[len - 2..].iter().collect();
        match suffix.as_str() {
            "ів" | "ок" | "ки" | "ко" | "ці" | "ти" | "ні" | "на" | "ну" | "не" | "но" | "ем"
            | "ій" | "ям" | "ях" => {
                return chars[..len - 2].iter().collect();
            }
            _ => {}
        }
    }

    if len > 3 {
        match chars[len - 1] {
            'а' | 'е' | 'и' | 'і' | 'о' | 'у' | 'я' | 'ю' | 'ь' => {
                return chars[..len - 1].iter().collect();
            }
            _ => {}
        }
    }

    word.to_string()
}

// ─── Armenian Stemmer ─────────────────────────────────────────────────────

/// Armenian stemmer based on suffix stripping.
///
/// Removes common Armenian inflectional suffixes (case, number, verb forms).
/// Armenian uses the Unicode range U+0530..U+058F.
#[derive(Clone, Debug, Default)]
pub struct ArmenianStemTokenFilter;

impl ArmenianStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for ArmenianStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let char_count = text.chars().count();
        if char_count < 4 {
            return (false, None);
        }
        let stemmed = stem_armenian(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_armenian(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    let len = chars.len();

    // 5-char: plural genitive/dative/instrumental combinations
    if len > 7 {
        let suffix: String = chars[len - 5..].iter().collect();
        match suffix.as_str() {
            "\u{0576}\u{0565}\u{0580}\u{056B}\u{0576}" |   // ներին (dat. pl.)
            "\u{0576}\u{0565}\u{0580}\u{056B}\u{0581}" |   // delays (instr. pl.)
            "\u{0576}\u{0565}\u{0580}\u{0578}\u{057E}" => { // delays (gen. pl.)
                return chars[..len - 5].iter().collect();
            }
            _ => {}
        }
    }

    // 4-char: plural markers + case
    if len > 6 {
        let suffix: String = chars[len - 4..].iter().collect();
        match suffix.as_str() {
            "\u{0576}\u{0565}\u{0580}\u{056B}" |  // ների (gen. pl.)
            "\u{0576}\u{0565}\u{0580}\u{0568}" |  // delays (dat. pl.)
            "\u{0578}\u{057E}\u{056B}\u{0576}" |  // delays (instr.)
            "\u{0578}\u{0582}\u{0569}\u{0575}" => { // illery (abstract)
                return chars[..len - 4].iter().collect();
            }
            _ => {}
        }
    }

    // 3-char: case/number suffixes
    if len > 5 {
        let suffix: String = chars[len - 3..].iter().collect();
        match suffix.as_str() {
            "\u{0576}\u{0565}\u{0580}" |  // delays (pl.)
            "\u{0578}\u{057E}\u{0568}" |  // delays (gen.)
            "\u{056B}\u{0581}\u{0568}" |  // delays (abl.)
            "\u{0578}\u{0582}\u{0574}" => { // delays (dat.)
                return chars[..len - 3].iter().collect();
            }
            _ => {}
        }
    }

    // 2-char: common endings
    if len > 4 {
        let suffix: String = chars[len - 2..].iter().collect();
        match suffix.as_str() {
            "\u{056B}\u{0576}" |  // ին (dat.)
            "\u{0578}\u{057E}" |  // ов (gen.)
            "\u{056B}\u{0581}" |  // delays (abl.)
            "\u{0578}\u{0574}" |  // delays (instr.)
            "\u{0565}\u{0580}" |  // delays (pl.)
            "\u{0568}\u{0576}" => { // delays
                return chars[..len - 2].iter().collect();
            }
            _ => {}
        }
    }

    // 1-char: vowel endings
    if len > 3 {
        match chars[len - 1] {
            '\u{0561}' | '\u{0565}' | '\u{056B}' | '\u{0578}' | '\u{0582}' => {
                // ա, ե, delays, ո, ու
                return chars[..len - 1].iter().collect();
            }
            _ => {}
        }
    }

    word.to_string()
}

// ─── Croatian Stemmer ─────────────────────────────────────────────────────

/// Croatian stemmer based on suffix stripping.
///
/// Removes common Croatian inflectional suffixes (case, gender, number).
#[derive(Clone, Debug, Default)]
pub struct CroatianStemTokenFilter;

impl CroatianStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for CroatianStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.len() < 4 {
            return (false, None);
        }
        let stemmed = stem_croatian(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_croatian(word: &str) -> String {
    let mut result = String::from(word);

    if result.len() > 7 {
        let len = result.len();
        let suffix = &result[len - 5..];
        match suffix {
            "ovski" | "evski" | "inski" => {
                result.truncate(len - 5);
                return result;
            }
            _ => {}
        }
    }

    if result.len() > 6 {
        let len = result.len();
        let suffix = &result[len - 4..];
        match suffix {
            "enog" | "skog" | "skom" | "skih" | "skim" | "anje" | "enje" | "anja" | "enja"
            | "osti" | "ista" | "iste" | "isti" => {
                result.truncate(len - 4);
                return result;
            }
            _ => {}
        }
    }

    if result.len() > 5 {
        let len = result.len();
        let suffix = &result[len - 3..];
        match suffix {
            "ama" | "ima" | "oga" | "ome" | "omu" | "ost" | "nju" | "nje" | "nog" | "nom"
            | "noj" | "nih" | "nim" | "eni" | "ani" | "ati" | "iti" | "uju" | "aje" | "uje"
            | "ije" | "ica" | "ice" | "ici" => {
                result.truncate(len - 3);
                return result;
            }
            _ => {}
        }
    }

    if result.len() > 4 {
        let len = result.len();
        let suffix = &result[len - 2..];
        match suffix {
            "om" | "og" | "im" | "ih" | "em" | "oj" | "an" | "in" | "en" | "ni" | "na" | "no"
            | "ti" | "ju" | "ci" | "ka" | "ke" | "ki" | "ko" | "ku" => {
                result.truncate(len - 2);
                return result;
            }
            _ => {}
        }
    }

    if result.len() > 3 {
        let last = result.as_bytes()[result.len() - 1];
        match last {
            b'a' | b'e' | b'i' | b'o' | b'u' => {
                result.pop();
            }
            _ => {}
        }
    }

    result
}

// ─── Lithuanian Stemmer ───────────────────────────────────────────────────

/// Lithuanian stemmer based on suffix stripping.
///
/// Removes common Lithuanian inflectional suffixes (case, number, verb forms).
#[derive(Clone, Debug, Default)]
pub struct LithuanianStemTokenFilter;

impl LithuanianStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for LithuanianStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.len() < 4 {
            return (false, None);
        }
        let stemmed = stem_lithuanian(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_lithuanian(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    let len = chars.len();

    if len > 7 {
        let suffix: String = chars[len - 5..].iter().collect();
        match suffix.as_str() {
            "acija" | "imams" | "iamas" => {
                return chars[..len - 5].iter().collect();
            }
            _ => {}
        }
    }

    if len > 6 {
        let suffix: String = chars[len - 4..].iter().collect();
        match suffix.as_str() {
            "amas" | "imas" | "umas" | "iais" | "omis" | "amis" | "ėmis" | "iaus" => {
                return chars[..len - 4].iter().collect();
            }
            _ => {}
        }
    }

    if len > 5 {
        let suffix: String = chars[len - 3..].iter().collect();
        match suffix.as_str() {
            "ams" | "oms" | "ėms" | "ius" | "aus" | "oje" | "ėje" | "yje" | "ais" | "iam"
            | "imi" | "umu" | "umi" | "ame" | "ėse" | "ose" | "yse" | "ies" | "ens" | "ėti"
            | "oti" | "yti" | "ati" | "iti" => {
                return chars[..len - 3].iter().collect();
            }
            _ => {}
        }
    }

    if len > 4 {
        let suffix: String = chars[len - 2..].iter().collect();
        match suffix.as_str() {
            "as" | "is" | "us" | "ys" | "os" | "ės" | "ai" | "ei" | "ui" | "am" | "om" | "im"
            | "um" | "ėm" | "ti" => {
                return chars[..len - 2].iter().collect();
            }
            _ => {}
        }
    }

    if len > 3 {
        match chars[len - 1] {
            'a' | 'ą' | 'e' | 'ę' | 'ė' | 'i' | 'į' | 'o' | 'u' | 'ų' | 'ū' | 'y' => {
                return chars[..len - 1].iter().collect();
            }
            _ => {}
        }
    }

    word.to_string()
}

// ─── Swedish Stemmer ──────────────────────────────────────────────────────

/// Swedish light stemmer based on Snowball algorithm principles.
///
/// Removes common Swedish plural, definite form, and derivational suffixes.
#[derive(Clone, Debug, Default)]
pub struct SwedishStemTokenFilter;

impl SwedishStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for SwedishStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.len() < 4 {
            return (false, None);
        }
        let stemmed = stem_swedish(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_swedish(word: &str) -> String {
    let mut result = String::from(word);

    if result.len() > 7 {
        let len = result.len();
        let suffix = &result[len - 5..];
        match suffix {
            "elser" | "ernas" | "arnas" | "ornas" | "andes" | "arens" | "andet" => {
                result.truncate(len - 5);
                return result;
            }
            _ => {}
        }
    }

    if result.len() > 6 {
        let len = result.len();
        let suffix = &result[len - 4..];
        match suffix {
            "else" | "arne" | "erna" | "orna" | "ande" | "aren" | "arna" | "ades" | "ings" => {
                result.truncate(len - 4);
                return result;
            }
            _ => {}
        }
    }

    if result.len() > 5 {
        let len = result.len();
        let suffix = &result[len - 3..];
        match suffix {
            "are" | "ast" | "ade" | "ing" | "arn" | "ens" | "het" | "igt" | "lig" | "els"
            | "iga" => {
                result.truncate(len - 3);
                return result;
            }
            _ => {}
        }
    }

    if result.len() > 4 {
        let len = result.len();
        let suffix = &result[len - 2..];
        match suffix {
            "ar" | "er" | "or" | "en" | "et" | "an" | "ad" | "as" | "at" | "ig" | "al" => {
                result.truncate(len - 2);
                return result;
            }
            _ => {}
        }
    }

    if result.len() > 3 {
        let last = result.as_bytes()[result.len() - 1];
        match last {
            b'a' | b'e' | b's' | b't' => {
                result.pop();
            }
            _ => {}
        }
    }

    result
}

// ─── Estonian Stemmer ──────────────────────────────────────────────────────

/// Estonian light stemmer based on suffix stripping.
///
/// Removes common Estonian inflectional suffixes (case, number).
#[derive(Clone, Debug, Default)]
pub struct EstonianStemTokenFilter;

impl EstonianStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for EstonianStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.len() < 4 {
            return (false, None);
        }
        let stemmed = stem_estonian(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_estonian(word: &str) -> String {
    let mut result = String::from(word);

    if result.len() > 6 {
        let len = result.len();
        let suffix = &result[len - 4..];
        match suffix {
            "mine" | "mise" | "line" | "lise" | "tele" | "teks" | "tena" | "test" | "tega" => {
                result.truncate(len - 4);
                return result;
            }
            _ => {}
        }
    }

    if result.len() > 5 {
        let len = result.len();
        let suffix = &result[len - 3..];
        match suffix {
            "sse" | "ste" | "sta" | "des" | "tes" | "tel" | "tud" | "nud" | "dud" | "lik"
            | "lik" => {
                result.truncate(len - 3);
                return result;
            }
            _ => {}
        }
    }

    if result.len() > 4 {
        let len = result.len();
        let suffix = &result[len - 2..];
        match suffix {
            "le" | "lt" | "ks" | "st" | "na" | "ga" | "ta" | "te" | "de" | "se" | "id" | "il"
            | "is" => {
                result.truncate(len - 2);
                return result;
            }
            _ => {}
        }
    }

    if result.len() > 3 {
        let last = result.as_bytes()[result.len() - 1];
        match last {
            b'd' | b'e' | b'i' | b's' | b't' => {
                result.pop();
            }
            _ => {}
        }
    }

    result
}

// ─── Slovak Stemmer ───────────────────────────────────────────────────────

/// Slovak light stemmer based on suffix stripping.
///
/// Removes common Slovak inflectional suffixes (similar to Czech patterns).
#[derive(Clone, Debug, Default)]
pub struct SlovakStemTokenFilter;

impl SlovakStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for SlovakStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.len() < 4 {
            return (false, None);
        }
        let stemmed = stem_slovak(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_slovak(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    let len = chars.len();

    if len > 6 {
        let suffix: String = chars[len - 4..].iter().collect();
        match suffix.as_str() {
            "osti" | "enie" | "anie" | "ovia" => {
                return chars[..len - 4].iter().collect();
            }
            _ => {}
        }
    }

    if len > 5 {
        let suffix: String = chars[len - 3..].iter().collect();
        match suffix.as_str() {
            "ách" | "ami" | "iam" | "ích" | "ému" | "ého" | "ých" | "ými" | "kov" => {
                return chars[..len - 3].iter().collect();
            }
            _ => {}
        }
    }

    if len > 4 {
        let suffix: String = chars[len - 2..].iter().collect();
        match suffix.as_str() {
            "om" | "ov" | "ou" | "em" | "mi" | "ám" | "ím" | "ej" | "ým" => {
                return chars[..len - 2].iter().collect();
            }
            _ => {}
        }
    }

    if len > 3 {
        match chars[len - 1] {
            'a' | 'e' | 'i' | 'o' | 'u' | 'y' | 'á' | 'é' | 'í' | 'ó' | 'ú' | 'ý' => {
                return chars[..len - 1].iter().collect();
            }
            _ => {}
        }
    }

    word.to_string()
}

// ─── Slovenian Stemmer ────────────────────────────────────────────────────

/// Slovenian light stemmer based on suffix stripping.
///
/// Removes common Slovenian inflectional suffixes.
#[derive(Clone, Debug, Default)]
pub struct SlovenianStemTokenFilter;

impl SlovenianStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for SlovenianStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.len() < 4 {
            return (false, None);
        }
        let stemmed = stem_slovenian(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_slovenian(word: &str) -> String {
    let mut result = String::from(word);

    if result.len() > 6 {
        let len = result.len();
        let suffix = &result[len - 4..];
        match suffix {
            "nost" | "stvo" | "anje" | "enje" => {
                result.truncate(len - 4);
                return result;
            }
            _ => {}
        }
    }

    if result.len() > 5 {
        let len = result.len();
        let suffix = &result[len - 3..];
        match suffix {
            "ama" | "ami" | "oma" | "ega" | "emu" | "imi" | "ost" | "ati" | "iti" | "ejo" => {
                result.truncate(len - 3);
                return result;
            }
            _ => {}
        }
    }

    if result.len() > 4 {
        let len = result.len();
        let suffix = &result[len - 2..];
        match suffix {
            "om" | "em" | "ih" | "im" | "ov" | "ev" | "mi" | "ah" | "am" | "je" | "ni" | "na"
            | "no" => {
                result.truncate(len - 2);
                return result;
            }
            _ => {}
        }
    }

    if result.len() > 3 {
        let last = result.as_bytes()[result.len() - 1];
        match last {
            b'a' | b'e' | b'i' | b'o' | b'u' => {
                result.pop();
            }
            _ => {}
        }
    }

    result
}

// Basque and Catalan stemming live in their own modules (full Snowball
// ports), exported from `token_filters::mod` directly.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token_filters::BasqueStemTokenFilter;
    use crate::token_filters::CatalanStemTokenFilter;

    fn make_token(term: &str) -> Token<'_> {
        Token {
            term: Cow::Borrowed(term),
            start_offset: 0,
            end_offset: term.len() as u32,
            position: 0,
        }
    }

    #[test]
    fn test_polish_stem() {
        let filter = PolishStemTokenFilter::new();
        let mut token = make_token("domami");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "dom");
    }

    #[test]
    fn test_ukrainian_stem() {
        let filter = UkrainianStemTokenFilter::new();
        let mut token = make_token("будинків");
        filter.filter(&mut token);
        assert!(token.term.len() < "будинків".len());
    }

    #[test]
    fn test_croatian_stem() {
        let filter = CroatianStemTokenFilter::new();
        let mut token = make_token("knjigama");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "knjig");
    }

    #[test]
    fn test_lithuanian_stem() {
        let filter = LithuanianStemTokenFilter::new();
        let mut token = make_token("knygos");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "knyg");
    }

    #[test]
    fn test_swedish_stem() {
        let filter = SwedishStemTokenFilter::new();
        let mut token = make_token("hundar");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "hund");
    }

    #[test]
    fn test_estonian_stem() {
        let filter = EstonianStemTokenFilter::new();
        let mut token = make_token("majadega");
        filter.filter(&mut token);
        assert!(token.term.len() < "majadega".len());
    }

    // Expectations from the JDK reference implementation (full Snowball
    // ports live in basque_stem.rs / catalan_stem.rs).
    #[test]
    fn test_basque_stem() {
        let filter = BasqueStemTokenFilter::new();
        let mut token = make_token("etxearen");
        filter.filter(&mut token);
        // genitive -aren is not stripped by the reference algorithm
        assert_eq!(token.term.as_ref(), "etxearen");
        let mut token = make_token("zaldiak");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "zaldi");
    }

    #[test]
    fn test_catalan_stem() {
        let filter = CatalanStemTokenFilter::new();
        let mut token = make_token("informacions");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "inform");
    }

    #[test]
    fn test_slovenian_stem() {
        let filter = SlovenianStemTokenFilter::new();
        let mut token = make_token("mestom");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "mest");
    }

    #[test]
    fn test_slovak_stem() {
        let filter = SlovakStemTokenFilter::new();
        let mut token = make_token("domami");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "dom");
    }
}
