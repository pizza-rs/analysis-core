//! Bundled stop word lists for 57 languages.
//!
//! Each language provides a constant `&[&str]` array of commonly used stop words,
//! compatible with the `StopTokenFilter`.
//!
//! Standard stop word lists for various languages.

mod afrikaans;
mod amharic;
mod arabic;
mod armenian;
mod azerbaijani;
mod basque;
mod bengali;
mod brazilian;
mod bulgarian;
mod catalan;
mod chinese;
mod cjk;
mod croatian;
mod czech;
mod danish;
mod dutch;
mod english;
mod estonian;
mod finnish;
mod french;
mod galician;
mod georgian;
mod german;
mod greek;
mod hebrew;
mod hindi;
mod hungarian;
mod indonesian;
mod irish;
mod italian;
mod japanese;
mod korean;
mod latvian;
mod lithuanian;
mod malay;
mod marathi;
mod mongolian;
mod nepali;
mod norwegian;
mod persian;
mod polish;
mod portuguese;
mod romanian;
mod russian;
mod serbian;
mod slovak;
mod slovenian;
mod sorani;
mod spanish;
mod swahili;
mod swedish;
mod tagalog;
mod tamil;
mod thai;
mod turkish;
mod ukrainian;
mod urdu;
mod vietnamese;

pub use afrikaans::AFRIKAANS_STOP_WORDS;
pub use amharic::AMHARIC_STOP_WORDS;
pub use arabic::ARABIC_STOP_WORDS;
pub use armenian::ARMENIAN_STOP_WORDS;
pub use azerbaijani::AZERBAIJANI_STOP_WORDS;
pub use basque::BASQUE_STOP_WORDS;
pub use bengali::BENGALI_STOP_WORDS;
pub use brazilian::BRAZILIAN_STOP_WORDS;
pub use bulgarian::BULGARIAN_STOP_WORDS;
pub use catalan::CATALAN_STOP_WORDS;
pub use chinese::CHINESE_STOP_WORDS;
pub use cjk::CJK_STOP_WORDS;
pub use croatian::CROATIAN_STOP_WORDS;
pub use czech::CZECH_STOP_WORDS;
pub use danish::DANISH_STOP_WORDS;
pub use dutch::DUTCH_STOP_WORDS;
pub use english::ENGLISH_STOP_WORDS;
pub use estonian::ESTONIAN_STOP_WORDS;
pub use finnish::FINNISH_STOP_WORDS;
pub use french::FRENCH_STOP_WORDS;
pub use galician::GALICIAN_STOP_WORDS;
pub use georgian::GEORGIAN_STOP_WORDS;
pub use german::GERMAN_STOP_WORDS;
pub use greek::GREEK_STOP_WORDS;
pub use hebrew::HEBREW_STOP_WORDS;
pub use hindi::HINDI_STOP_WORDS;
pub use hungarian::HUNGARIAN_STOP_WORDS;
pub use indonesian::INDONESIAN_STOP_WORDS;
pub use irish::IRISH_STOP_WORDS;
pub use italian::ITALIAN_STOP_WORDS;
pub use japanese::JAPANESE_STOP_WORDS;
pub use korean::KOREAN_STOP_WORDS;
pub use latvian::LATVIAN_STOP_WORDS;
pub use lithuanian::LITHUANIAN_STOP_WORDS;
pub use malay::MALAY_STOP_WORDS;
pub use marathi::MARATHI_STOP_WORDS;
pub use mongolian::MONGOLIAN_STOP_WORDS;
pub use nepali::NEPALI_STOP_WORDS;
pub use norwegian::NORWEGIAN_STOP_WORDS;
pub use persian::PERSIAN_STOP_WORDS;
pub use polish::POLISH_STOP_WORDS;
pub use portuguese::PORTUGUESE_STOP_WORDS;
pub use romanian::ROMANIAN_STOP_WORDS;
pub use russian::RUSSIAN_STOP_WORDS;
pub use serbian::SERBIAN_STOP_WORDS;
pub use slovak::SLOVAK_STOP_WORDS;
pub use slovenian::SLOVENIAN_STOP_WORDS;
pub use sorani::SORANI_STOP_WORDS;
pub use spanish::SPANISH_STOP_WORDS;
pub use swahili::SWAHILI_STOP_WORDS;
pub use swedish::SWEDISH_STOP_WORDS;
pub use tagalog::TAGALOG_STOP_WORDS;
pub use tamil::TAMIL_STOP_WORDS;
pub use thai::THAI_STOP_WORDS;
pub use turkish::TURKISH_STOP_WORDS;
pub use ukrainian::UKRAINIAN_STOP_WORDS;
pub use urdu::URDU_STOP_WORDS;
pub use vietnamese::VIETNAMESE_STOP_WORDS;

/// Get stop words for a language by name (lowercase).
///
/// Returns `None` if the language is not supported.
pub fn get_stop_words(language: &str) -> Option<&'static [&'static str]> {
    match language {
        "afrikaans" | "_afrikaans_" => Some(&AFRIKAANS_STOP_WORDS),
        "amharic" | "_amharic_" => Some(&AMHARIC_STOP_WORDS),
        "arabic" | "_arabic_" => Some(&ARABIC_STOP_WORDS),
        "armenian" | "_armenian_" => Some(&ARMENIAN_STOP_WORDS),
        "azerbaijani" | "_azerbaijani_" => Some(&AZERBAIJANI_STOP_WORDS),
        "basque" | "_basque_" => Some(&BASQUE_STOP_WORDS),
        "bengali" | "_bengali_" => Some(&BENGALI_STOP_WORDS),
        "brazilian" | "_brazilian_" => Some(&BRAZILIAN_STOP_WORDS),
        "bulgarian" | "_bulgarian_" => Some(&BULGARIAN_STOP_WORDS),
        "catalan" | "_catalan_" => Some(&CATALAN_STOP_WORDS),
        "chinese" | "_chinese_" => Some(&CHINESE_STOP_WORDS),
        "cjk" | "_cjk_" => Some(&CJK_STOP_WORDS),
        "croatian" | "_croatian_" => Some(&CROATIAN_STOP_WORDS),
        "czech" | "_czech_" => Some(&CZECH_STOP_WORDS),
        "danish" | "_danish_" => Some(&DANISH_STOP_WORDS),
        "dutch" | "_dutch_" => Some(&DUTCH_STOP_WORDS),
        "english" | "_english_" => Some(&ENGLISH_STOP_WORDS),
        "estonian" | "_estonian_" => Some(&ESTONIAN_STOP_WORDS),
        "finnish" | "_finnish_" => Some(&FINNISH_STOP_WORDS),
        "french" | "_french_" => Some(&FRENCH_STOP_WORDS),
        "galician" | "_galician_" => Some(&GALICIAN_STOP_WORDS),
        "georgian" | "_georgian_" => Some(&GEORGIAN_STOP_WORDS),
        "german" | "_german_" => Some(&GERMAN_STOP_WORDS),
        "greek" | "_greek_" => Some(&GREEK_STOP_WORDS),
        "hebrew" | "_hebrew_" => Some(&HEBREW_STOP_WORDS),
        "hindi" | "_hindi_" => Some(&HINDI_STOP_WORDS),
        "hungarian" | "_hungarian_" => Some(&HUNGARIAN_STOP_WORDS),
        "indonesian" | "_indonesian_" => Some(&INDONESIAN_STOP_WORDS),
        "irish" | "_irish_" => Some(&IRISH_STOP_WORDS),
        "italian" | "_italian_" => Some(&ITALIAN_STOP_WORDS),
        "japanese" | "_japanese_" => Some(&JAPANESE_STOP_WORDS),
        "korean" | "_korean_" => Some(&KOREAN_STOP_WORDS),
        "latvian" | "_latvian_" => Some(&LATVIAN_STOP_WORDS),
        "lithuanian" | "_lithuanian_" => Some(&LITHUANIAN_STOP_WORDS),
        "malay" | "_malay_" => Some(&MALAY_STOP_WORDS),
        "marathi" | "_marathi_" => Some(&MARATHI_STOP_WORDS),
        "mongolian" | "_mongolian_" => Some(&MONGOLIAN_STOP_WORDS),
        "nepali" | "_nepali_" => Some(&NEPALI_STOP_WORDS),
        "norwegian" | "_norwegian_" => Some(&NORWEGIAN_STOP_WORDS),
        "persian" | "_persian_" => Some(&PERSIAN_STOP_WORDS),
        "polish" | "_polish_" => Some(&POLISH_STOP_WORDS),
        "portuguese" | "_portuguese_" => Some(&PORTUGUESE_STOP_WORDS),
        "romanian" | "_romanian_" => Some(&ROMANIAN_STOP_WORDS),
        "russian" | "_russian_" => Some(&RUSSIAN_STOP_WORDS),
        "serbian" | "_serbian_" => Some(&SERBIAN_STOP_WORDS),
        "slovak" | "_slovak_" => Some(&SLOVAK_STOP_WORDS),
        "slovenian" | "_slovenian_" => Some(&SLOVENIAN_STOP_WORDS),
        "sorani" | "_sorani_" => Some(&SORANI_STOP_WORDS),
        "spanish" | "_spanish_" => Some(&SPANISH_STOP_WORDS),
        "swahili" | "_swahili_" => Some(&SWAHILI_STOP_WORDS),
        "swedish" | "_swedish_" => Some(&SWEDISH_STOP_WORDS),
        "tagalog" | "filipino" | "_tagalog_" => Some(&TAGALOG_STOP_WORDS),
        "tamil" | "_tamil_" => Some(&TAMIL_STOP_WORDS),
        "thai" | "_thai_" => Some(&THAI_STOP_WORDS),
        "turkish" | "_turkish_" => Some(&TURKISH_STOP_WORDS),
        "ukrainian" | "_ukrainian_" => Some(&UKRAINIAN_STOP_WORDS),
        "urdu" | "_urdu_" => Some(&URDU_STOP_WORDS),
        "vietnamese" | "_vietnamese_" => Some(&VIETNAMESE_STOP_WORDS),
        _ => None,
    }
}
