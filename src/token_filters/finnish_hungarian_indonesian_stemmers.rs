use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

// ─── Finnish Light Stemmer ────────────────────────────────────────────────

/// Finnish light stemmer based on Lucene's FinnishLightStemFilter.
///
/// Removes common Finnish case and number suffixes.
#[derive(Clone, Debug, Default)]
pub struct FinnishLightStemTokenFilter;

impl FinnishLightStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for FinnishLightStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.len() < 4 {
            return (false, None);
        }

        let stemmed = stem_finnish_light(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_finnish_light(word: &str) -> String {
    let mut s: Vec<char> = word.chars().collect();
    let len = s.len();

    // Step 1: Plural & possessives (longest match first)
    if len > 7 {
        let suffix: String = s[len - 4..].iter().collect();
        match suffix.as_str() {
            "tten" | "nnen" => {
                s.truncate(len - 4);
                return s.into_iter().collect();
            }
            _ => {}
        }
    }

    if len > 6 {
        let suffix: String = s[len - 3..].iter().collect();
        match suffix.as_str() {
            "ssa" | "ssä" | "sta" | "stä" | "lla" | "llä" | "lta" | "ltä" | "lle" | "tta"
            | "ttä" | "ksi" | "ine" => {
                s.truncate(len - 3);
                return s.into_iter().collect();
            }
            _ => {}
        }
    }

    if len > 5 {
        let suffix: String = s[len - 2..].iter().collect();
        match suffix.as_str() {
            "na" | "nä" | "en" | "in" | "an" | "ön" | "on" | "ät" | "öt" | "it" | "et" | "ia"
            | "iä" | "ta" | "tä" | "ja" | "jä" => {
                s.truncate(len - 2);
                return s.into_iter().collect();
            }
            _ => {}
        }
    }

    if len > 4 {
        let last = *s.last().unwrap();
        match last {
            'a' | 'ä' | 'n' | 't' | 'i' | 'e' => {
                s.pop();
            }
            _ => {}
        }
    }

    s.into_iter().collect()
}

// ─── Hungarian Light Stemmer ──────────────────────────────────────────────

/// Hungarian light stemmer based on Lucene's HungarianLightStemFilter.
///
/// Removes common Hungarian case and number suffixes.
#[derive(Clone, Debug, Default)]
pub struct HungarianLightStemTokenFilter;

impl HungarianLightStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for HungarianLightStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.len() < 4 {
            return (false, None);
        }

        let stemmed = stem_hungarian_light(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_hungarian_light(word: &str) -> String {
    let mut s: Vec<char> = word.chars().collect();
    let len = s.len();

    // 4-char suffixes
    if len > 7 {
        let suffix: String = s[len - 4..].iter().collect();
        match suffix.as_str() {
            "akat" | "eket" | "oket" | "okat" | "eket" | "ünök" | "anok" | "enök" | "änak"
            | "énak" => {
                s.truncate(len - 4);
                return s.into_iter().collect();
            }
            _ => {}
        }
    }

    // 3-char suffixes (strip when at least 3 chars remain, e.g. házban → ház)
    if len >= 6 {
        let suffix: String = s[len - 3..].iter().collect();
        match suffix.as_str() {
            "ban" | "ben" | "nak" | "nek" | "ból" | "ből" | "hoz" | "hez" | "höz" | "ról"
            | "ről" | "tól" | "től" | "val" | "vel" | "ért" | "nek" | "ban" | "vál" | "vél"
            | "ból" | "ből" | "nak" | "nek" => {
                s.truncate(len - 3);
                return s.into_iter().collect();
            }
            _ => {}
        }
    }

    // 2-char suffixes
    if len > 5 {
        let suffix: String = s[len - 2..].iter().collect();
        match suffix.as_str() {
            "at" | "et" | "ot" | "öt" | "ra" | "re" | "ba" | "be" | "on" | "en" | "ön" | "ul"
            | "ül" | "ig" | "ek" | "ok" | "ök" | "ak" | "ás" | "és" => {
                s.truncate(len - 2);
                return s.into_iter().collect();
            }
            _ => {}
        }
    }

    // 1-char suffixes
    if len > 4 {
        let last = *s.last().unwrap();
        match last {
            'á' | 'é' | 'a' | 'e' | 'k' | 't' => {
                s.pop();
            }
            _ => {}
        }
    }

    s.into_iter().collect()
}

// ─── Indonesian Stemmer ───────────────────────────────────────────────────

/// Indonesian stemmer — faithful port of Lucene's `IndonesianStemmer`
/// ("A Study of Stemming Effects on Information Retrieval in Bahasa
/// Indonesia", Fadillah Z. Tala).
///
/// Order of operations matters: inflectional suffixes (particles and
/// possessive pronouns) first, then derivational morphology — first-order
/// prefixes, then (only if a prefix fired) suffixes and second-order
/// prefixes; on prefix failure, second-order prefixes followed by suffixes.
/// Every rule is gated on the word having more than two syllables, tracked
/// as it shrinks.
#[derive(Clone, Debug, Default)]
pub struct IndonesianStemTokenFilter;

impl IndonesianStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

const REMOVED_KE: u32 = 1;
const REMOVED_PENG: u32 = 2;
const REMOVED_DI: u32 = 4;
const REMOVED_MENG: u32 = 8;
const REMOVED_TER: u32 = 16;
const REMOVED_BER: u32 = 32;
const REMOVED_PE: u32 = 64;

struct IndonesianStemmer {
    num_syllables: usize,
    flags: u32,
}

impl TokenFilter for IndonesianStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let stemmed = stem_indonesian(&text.to_lowercase());
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn is_vowel(ch: char) -> bool {
    matches!(ch, 'a' | 'e' | 'i' | 'o' | 'u')
}

fn stem_indonesian(word: &str) -> String {
    let mut text: Vec<char> = word.chars().collect();
    let mut st = IndonesianStemmer {
        num_syllables: text.iter().filter(|c| is_vowel(**c)).count(),
        flags: 0,
    };

    let mut len = text.len();
    if st.num_syllables > 2 {
        len = remove_particle(&mut text, len, &mut st);
    }
    if st.num_syllables > 2 {
        len = remove_possessive_pronoun(&mut text, len, &mut st);
    }
    len = stem_derivational(&mut text, len, &mut st);

    text.truncate(len);
    text.into_iter().collect()
}

fn stem_derivational(text: &mut [char], mut len: usize, st: &mut IndonesianStemmer) -> usize {
    let old_len = len;
    if st.num_syllables > 2 {
        len = remove_first_order_prefix(text, len, st);
    }
    if old_len != len {
        // a rule fired
        let old_len = len;
        if st.num_syllables > 2 {
            len = remove_suffix(text, len, st);
        }
        if old_len != len && st.num_syllables > 2 {
            len = remove_second_order_prefix(text, len, st);
        }
    } else {
        // fail
        if st.num_syllables > 2 {
            len = remove_second_order_prefix(text, len, st);
        }
        if st.num_syllables > 2 {
            len = remove_suffix(text, len, st);
        }
    }
    len
}

fn remove_particle(text: &[char], len: usize, st: &mut IndonesianStemmer) -> usize {
    let s: String = text[..len].iter().collect();
    if s.ends_with("kah") || s.ends_with("lah") || s.ends_with("pun") {
        st.num_syllables -= 1;
        return len - 3;
    }
    len
}

fn remove_possessive_pronoun(text: &[char], len: usize, st: &mut IndonesianStemmer) -> usize {
    let s: String = text[..len].iter().collect();
    if s.ends_with("ku") || s.ends_with("mu") {
        st.num_syllables -= 1;
        return len - 2;
    }
    if s.ends_with("nya") {
        st.num_syllables -= 1;
        return len - 3;
    }
    len
}

fn remove_first_order_prefix(text: &mut [char], len: usize, st: &mut IndonesianStemmer) -> usize {
    let starts = |p: &str| -> bool {
        let pc: Vec<char> = p.chars().collect();
        pc.len() <= len && text[..pc.len()] == pc[..]
    };
    if starts("meng") {
        st.flags |= REMOVED_MENG;
        st.num_syllables -= 1;
        return delete_n(text, len, 4);
    }
    if starts("meny") && len > 4 && is_vowel(text[4]) {
        st.flags |= REMOVED_MENG;
        text[3] = 's';
        st.num_syllables -= 1;
        return delete_n(text, len, 3);
    }
    if starts("men") {
        st.flags |= REMOVED_MENG;
        st.num_syllables -= 1;
        return delete_n(text, len, 3);
    }
    if starts("mem") {
        st.flags |= REMOVED_MENG;
        st.num_syllables -= 1;
        return delete_n(text, len, 3);
    }
    if starts("me") {
        st.flags |= REMOVED_MENG;
        st.num_syllables -= 1;
        return delete_n(text, len, 2);
    }
    if starts("peng") {
        st.flags |= REMOVED_PENG;
        st.num_syllables -= 1;
        return delete_n(text, len, 4);
    }
    if starts("peny") && len > 4 && is_vowel(text[4]) {
        st.flags |= REMOVED_PENG;
        text[3] = 's';
        st.num_syllables -= 1;
        return delete_n(text, len, 3);
    }
    if starts("peny") {
        st.flags |= REMOVED_PENG;
        st.num_syllables -= 1;
        return delete_n(text, len, 4);
    }
    if starts("pen") && len > 3 && is_vowel(text[3]) {
        st.flags |= REMOVED_PENG;
        text[2] = 't';
        st.num_syllables -= 1;
        return delete_n(text, len, 2);
    }
    if starts("pen") {
        st.flags |= REMOVED_PENG;
        st.num_syllables -= 1;
        return delete_n(text, len, 3);
    }
    if starts("pem") {
        st.flags |= REMOVED_PENG;
        st.num_syllables -= 1;
        return delete_n(text, len, 3);
    }
    if starts("di") {
        st.flags |= REMOVED_DI;
        st.num_syllables -= 1;
        return delete_n(text, len, 2);
    }
    if starts("ter") {
        st.flags |= REMOVED_TER;
        st.num_syllables -= 1;
        return delete_n(text, len, 3);
    }
    if starts("ke") {
        st.flags |= REMOVED_KE;
        st.num_syllables -= 1;
        return delete_n(text, len, 2);
    }
    len
}

fn remove_second_order_prefix(text: &mut [char], len: usize, st: &mut IndonesianStemmer) -> usize {
    let starts = |p: &str| -> bool {
        let pc: Vec<char> = p.chars().collect();
        pc.len() <= len && text[..pc.len()] == pc[..]
    };
    if starts("ber") {
        st.flags |= REMOVED_BER;
        st.num_syllables -= 1;
        return delete_n(text, len, 3);
    }
    if len == 7 && starts("belajar") {
        st.flags |= REMOVED_BER;
        st.num_syllables -= 1;
        return delete_n(text, len, 3);
    }
    if starts("be") && len > 4 && !is_vowel(text[2]) && text[3] == 'e' && text[4] == 'r' {
        st.flags |= REMOVED_BER;
        st.num_syllables -= 1;
        return delete_n(text, len, 2);
    }
    if starts("per") {
        st.num_syllables -= 1;
        return delete_n(text, len, 3);
    }
    if len == 7 && starts("pelajar") {
        st.num_syllables -= 1;
        return delete_n(text, len, 3);
    }
    if starts("pe") {
        st.flags |= REMOVED_PE;
        st.num_syllables -= 1;
        return delete_n(text, len, 2);
    }
    len
}

fn remove_suffix(text: &[char], len: usize, st: &mut IndonesianStemmer) -> usize {
    let s: String = text[..len].iter().collect();
    if s.ends_with("kan")
        && (st.flags & REMOVED_KE) == 0
        && (st.flags & REMOVED_PENG) == 0
        && (st.flags & REMOVED_PE) == 0
    {
        st.num_syllables -= 1;
        return len - 3;
    }
    if s.ends_with("an")
        && (st.flags & REMOVED_DI) == 0
        && (st.flags & REMOVED_MENG) == 0
        && (st.flags & REMOVED_TER) == 0
    {
        st.num_syllables -= 1;
        return len - 2;
    }
    if s.ends_with('i')
        && !s.ends_with("si")
        && (st.flags & REMOVED_BER) == 0
        && (st.flags & REMOVED_KE) == 0
        && (st.flags & REMOVED_PENG) == 0
    {
        st.num_syllables -= 1;
        return len - 1;
    }
    len
}

/// Port of StemmerUtil.deleteN: delete `n` chars starting at `from`.
fn delete_n(text: &mut [char], len: usize, n: usize) -> usize {
    text.copy_within(n..len, 0);
    len - n
}

#[cfg(test)]
mod tests {
    use super::*;
    use pizza_engine::analysis::Token;

    fn make_token(term: &str) -> Token<'_> {
        Token {
            term: Cow::Borrowed(term),
            start_offset: 0,
            end_offset: term.len() as u32,
            position: 0,
        }
    }

    #[test]
    fn test_finnish_light_stem() {
        let filter = FinnishLightStemTokenFilter::new();
        let mut token = make_token("kirjassa");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "kirja");
    }

    #[test]
    fn test_finnish_light_short() {
        let filter = FinnishLightStemTokenFilter::new();
        let mut token = make_token("ala");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "ala"); // too short to stem
    }

    #[test]
    fn test_hungarian_light_stem() {
        let filter = HungarianLightStemTokenFilter::new();
        let mut token = make_token("házban");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "ház");
    }

    #[test]
    fn test_hungarian_light_short() {
        let filter = HungarianLightStemTokenFilter::new();
        let mut token = make_token("ház");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "ház"); // too short
    }

    #[test]
    fn test_indonesian_suffix() {
        let filter = IndonesianStemTokenFilter::new();
        // mem- prefix fires; afterwards only 2 syllables remain, so the
        // -kan suffix rule is gated off (Lucene semantics).
        let mut token = make_token("memakan");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "akan");
        // meng- + -kan: both fire while syllables allow
        let mut token = make_token("mengambilkan");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "ambil");
    }

    #[test]
    fn test_indonesian_prefix() {
        let filter = IndonesianStemTokenFilter::new();
        let mut token = make_token("berlari");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "lari");
    }

    // Vectors from Lucene's TestIndonesianStemmer (full stemming).
    #[test]
    fn test_indonesian_lucene_examples() {
        let filter = IndonesianStemTokenFilter::new();
        for (input, expected) in [
            ("bukukah", "buku"),
            ("adalah", "ada"),
            ("bukupun", "buku"),
            ("bukuku", "buku"),
            ("bukumu", "buku"),
            ("bukunya", "buku"),
            ("mengukur", "ukur"),
            ("menyapu", "sapu"),
            ("menduga", "duga"),
            ("menuduh", "uduh"),
            ("membaca", "baca"),
            ("merusak", "rusak"),
            ("pengukur", "ukur"),
            ("penyapu", "sapu"),
            ("penduga", "duga"),
            ("pembaca", "baca"),
            ("diukur", "ukur"),
            ("tersapu", "sapu"),
            ("kekasih", "kasih"),
            ("berlari", "lari"),
            ("belajar", "ajar"),
            ("bekerja", "kerja"),
            ("perjelas", "jelas"),
            ("pelajar", "ajar"),
            ("pekerja", "kerja"),
            ("tarikkan", "tarik"),
            ("ambilkan", "ambil"),
            ("mengambilkan", "ambil"),
            ("makanan", "makan"),
            ("janjian", "janji"),
            ("perjanjian", "janji"),
            ("tandai", "tanda"),
            ("dapati", "dapat"),
            ("mendapati", "dapat"),
            ("pantai", "panta"),
            ("penyalahgunaan", "salahguna"),
            ("menyalahgunakan", "salahguna"),
            ("disalahgunakan", "salahguna"),
            ("pertanggungjawaban", "tanggungjawab"),
            ("mempertanggungjawabkan", "tanggungjawab"),
            ("dipertanggungjawabkan", "tanggungjawab"),
            ("pelaksanaan", "laksana"),
            ("pelaksana", "laksana"),
            ("melaksanakan", "laksana"),
            ("dilaksanakan", "laksana"),
            ("melibatkan", "libat"),
            ("terlibat", "libat"),
            ("penculikan", "culik"),
            ("menculik", "culik"),
            ("diculik", "culik"),
            ("penculik", "culik"),
            ("perubahan", "ubah"),
            ("peledakan", "ledak"),
            ("penanganan", "tangan"),
            ("kepolisian", "polisi"),
            ("kenaikan", "naik"),
            ("bersenjata", "senjata"),
            ("penyelewengan", "seleweng"),
            ("kecelakaan", "celaka"),
            ("gigi", "gigi"),
        ] {
            let mut token = make_token(input);
            filter.filter(&mut token);
            assert_eq!(token.term.as_ref(), expected, "vector {input}");
        }
    }
}
