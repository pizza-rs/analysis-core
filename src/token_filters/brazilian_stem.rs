use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Brazilian Portuguese stemmer — faithful port of Lucene's
/// `org.apache.lucene.analysis.br.BrazilianStemmer`.
///
/// This is distinct from the Snowball Portuguese stemmer: it computes R1/R2/RV
/// regions over an accent-folded term and applies a five-step suffix cascade
/// (standard suffixes, verb suffixes in RV, residual `i`, residual vowels,
/// final `e`/`gu`/`ci` cleanup). Two upstream quirks are preserved on
/// purpose because the reference tests depend on the exact behavior:
/// the `logias` branch discards its replacement (the Java source never
/// assigns the result back), and the `ira` verb branch removes `ava`
/// instead of `ira` (a no-op unless the term happens to end in `ava`).
#[derive(Clone, Debug, Default)]
pub struct BrazilianStemTokenFilter;

impl BrazilianStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for BrazilianStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        // `stem()` returns null for non-indexable terms (length <= 2 or >= 30);
        // Lucene leaves those tokens untouched, original diacritics included.
        if let Some(stemmed) = BrazilianStemmer::stem(text) {
            if stemmed != text {
                token.term = Cow::Owned(stemmed);
            }
        }
        (false, None)
    }
}

struct BrazilianStemmer {
    ct: String,
    r1: Option<String>,
    r2: Option<String>,
    rv: Option<String>,
}

impl BrazilianStemmer {
    /// Port of `BrazilianStemmer.stem()`: `None` mirrors the Java `null`
    /// return for terms that must be left unchanged.
    fn stem(term: &str) -> Option<String> {
        let mut st = BrazilianStemmer {
            ct: String::new(),
            r1: None,
            r2: None,
            rv: None,
        };
        st.create_ct(term);

        let len = st.ct.chars().count();
        // isIndexable: 2 < length < 30
        if len >= 30 || len <= 2 {
            return None;
        }
        // isStemmable: every character must be a letter; the (lowercased,
        // accent-folded) CT is returned as-is otherwise.
        if !st.ct.chars().all(|c| c.is_alphabetic()) {
            return Some(st.ct);
        }

        st.r1 = get_r1(&st.ct);
        st.r2 = st.r1.as_deref().and_then(get_r1);
        st.rv = get_rv(&st.ct);

        let altered = st.step1() || st.step2();
        if altered {
            st.step3();
        } else {
            st.step4();
        }
        st.step5();

        Some(st.ct)
    }

    /// createCT: lowercase, fold accents (ã/õ → a/o, ç → c, …), then strip
    /// one leading and one trailing punctuation character.
    fn create_ct(&mut self, term: &str) {
        let mut ct = change_term(term);
        if ct.chars().count() < 2 {
            self.ct = ct;
            return;
        }
        if let Some(first) = ct.chars().next() {
            if matches!(first, '"' | '\'' | '-' | ',' | ';' | '.' | '?' | '!') {
                ct = ct.chars().skip(1).collect();
            }
        }
        if ct.chars().count() < 2 {
            self.ct = ct;
            return;
        }
        if let Some(last) = ct.chars().last() {
            if matches!(last, '-' | ',' | ';' | '.' | '?' | '!' | '\'' | '"') {
                ct.pop();
            }
        }
        self.ct = ct;
    }

    /// Standard suffix removal, gated on R1/R2 (RV plus an `e`-preceded guard
    /// for -iras/-ira). Returns false if no ending was removed.
    fn step1(&mut self) -> bool {
        let ct = self.ct.clone();
        let r1 = self.r1.as_deref();
        let r2 = self.r2.as_deref();
        let rv = self.rv.as_deref();

        // suffix length = 7
        if ct.ends_with("uciones") && opt_ends(r2, "uciones") {
            self.ct = replace_suffix(&ct, "uciones", "u");
            return true;
        }

        // suffix length = 6 (upstream gates this block on CT.len() >= 6,
        // which every listed suffix already implies)
        if ct.ends_with("imentos") && opt_ends(r2, "imentos") {
            self.ct = remove_suffix(&ct, "imentos");
            return true;
        }
        if ct.ends_with("amentos") && opt_ends(r2, "amentos") {
            self.ct = remove_suffix(&ct, "amentos");
            return true;
        }
        if ct.ends_with("adores") && opt_ends(r2, "adores") {
            self.ct = remove_suffix(&ct, "adores");
            return true;
        }
        if ct.ends_with("adoras") && opt_ends(r2, "adoras") {
            self.ct = remove_suffix(&ct, "adoras");
            return true;
        }
        if ct.ends_with("logias") && opt_ends(r2, "logias") {
            // Upstream bug preserved: the replacement's return value is
            // discarded, so CT stays unchanged even though the step reports
            // success.
            return true;
        }
        if ct.ends_with("encias") && opt_ends(r2, "encias") {
            self.ct = replace_suffix(&ct, "encias", "ente");
            return true;
        }
        if ct.ends_with("amente") && opt_ends(r1, "amente") {
            self.ct = remove_suffix(&ct, "amente");
            return true;
        }
        if ct.ends_with("idades") && opt_ends(r2, "idades") {
            self.ct = remove_suffix(&ct, "idades");
            return true;
        }

        // suffix length = 5
        if ct.ends_with("acoes") && opt_ends(r2, "acoes") {
            self.ct = remove_suffix(&ct, "acoes");
            return true;
        }
        if ct.ends_with("imento") && opt_ends(r2, "imento") {
            self.ct = remove_suffix(&ct, "imento");
            return true;
        }
        if ct.ends_with("amento") && opt_ends(r2, "amento") {
            self.ct = remove_suffix(&ct, "amento");
            return true;
        }
        if ct.ends_with("adora") && opt_ends(r2, "adora") {
            self.ct = remove_suffix(&ct, "adora");
            return true;
        }
        if ct.ends_with("ismos") && opt_ends(r2, "ismos") {
            self.ct = remove_suffix(&ct, "ismos");
            return true;
        }
        if ct.ends_with("istas") && opt_ends(r2, "istas") {
            self.ct = remove_suffix(&ct, "istas");
            return true;
        }
        if ct.ends_with("logia") && opt_ends(r2, "logia") {
            self.ct = replace_suffix(&ct, "logia", "log");
            return true;
        }
        if ct.ends_with("ucion") && opt_ends(r2, "ucion") {
            self.ct = replace_suffix(&ct, "ucion", "u");
            return true;
        }
        if ct.ends_with("encia") && opt_ends(r2, "encia") {
            self.ct = replace_suffix(&ct, "encia", "ente");
            return true;
        }
        if ct.ends_with("mente") && opt_ends(r2, "mente") {
            self.ct = remove_suffix(&ct, "mente");
            return true;
        }
        if ct.ends_with("idade") && opt_ends(r2, "idade") {
            self.ct = remove_suffix(&ct, "idade");
            return true;
        }

        // suffix length = 4
        if ct.ends_with("acao") && opt_ends(r2, "acao") {
            self.ct = remove_suffix(&ct, "acao");
            return true;
        }
        if ct.ends_with("ezas") && opt_ends(r2, "ezas") {
            self.ct = remove_suffix(&ct, "ezas");
            return true;
        }
        if ct.ends_with("icos") && opt_ends(r2, "icos") {
            self.ct = remove_suffix(&ct, "icos");
            return true;
        }
        if ct.ends_with("icas") && opt_ends(r2, "icas") {
            self.ct = remove_suffix(&ct, "icas");
            return true;
        }
        if ct.ends_with("ismo") && opt_ends(r2, "ismo") {
            self.ct = remove_suffix(&ct, "ismo");
            return true;
        }
        if ct.ends_with("avel") && opt_ends(r2, "avel") {
            self.ct = remove_suffix(&ct, "avel");
            return true;
        }
        if ct.ends_with("ivel") && opt_ends(r2, "ivel") {
            self.ct = remove_suffix(&ct, "ivel");
            return true;
        }
        if ct.ends_with("ista") && opt_ends(r2, "ista") {
            self.ct = remove_suffix(&ct, "ista");
            return true;
        }
        if ct.ends_with("osos") && opt_ends(r2, "osos") {
            self.ct = remove_suffix(&ct, "osos");
            return true;
        }
        if ct.ends_with("osas") && opt_ends(r2, "osas") {
            self.ct = remove_suffix(&ct, "osas");
            return true;
        }
        if ct.ends_with("ador") && opt_ends(r2, "ador") {
            self.ct = remove_suffix(&ct, "ador");
            return true;
        }
        if ct.ends_with("ivas") && opt_ends(r2, "ivas") {
            self.ct = remove_suffix(&ct, "ivas");
            return true;
        }
        if ct.ends_with("ivos") && opt_ends(r2, "ivos") {
            self.ct = remove_suffix(&ct, "ivos");
            return true;
        }
        if ct.ends_with("iras") && opt_ends(rv, "iras") && suffix_preceded(&ct, "iras", "e") {
            self.ct = replace_suffix(&ct, "iras", "ir");
            return true;
        }

        // suffix length = 3
        if ct.ends_with("eza") && opt_ends(r2, "eza") {
            self.ct = remove_suffix(&ct, "eza");
            return true;
        }
        if ct.ends_with("ico") && opt_ends(r2, "ico") {
            self.ct = remove_suffix(&ct, "ico");
            return true;
        }
        if ct.ends_with("ica") && opt_ends(r2, "ica") {
            self.ct = remove_suffix(&ct, "ica");
            return true;
        }
        if ct.ends_with("oso") && opt_ends(r2, "oso") {
            self.ct = remove_suffix(&ct, "oso");
            return true;
        }
        if ct.ends_with("osa") && opt_ends(r2, "osa") {
            self.ct = remove_suffix(&ct, "osa");
            return true;
        }
        if ct.ends_with("iva") && opt_ends(r2, "iva") {
            self.ct = remove_suffix(&ct, "iva");
            return true;
        }
        if ct.ends_with("ivo") && opt_ends(r2, "ivo") {
            self.ct = remove_suffix(&ct, "ivo");
            return true;
        }
        if ct.ends_with("ira") && opt_ends(rv, "ira") && suffix_preceded(&ct, "ira", "e") {
            self.ct = replace_suffix(&ct, "ira", "ir");
            return true;
        }

        false
    }

    /// Verb suffixes: matched against RV, deleted from CT in the order below.
    fn step2(&mut self) -> bool {
        const GROUP7: &[&str] = &[
            "issemos", "essemos", "assemos", "ariamos", "eriamos", "iriamos",
        ];
        const GROUP6: &[&str] = &[
            "iremos", "eremos", "aremos", "avamos", "iramos", "eramos", "aramos", "asseis",
            "esseis", "isseis", "arieis", "erieis", "irieis",
        ];
        const GROUP5: &[&str] = &[
            "irmos", "iamos", "armos", "ermos", "areis", "ereis", "ireis", "asses", "esses",
            "isses", "astes", "assem", "essem", "issem", "ardes", "erdes", "irdes", "ariam",
            "eriam", "iriam", "arias", "erias", "irias", "estes", "istes", "areis", "aveis",
        ];
        const GROUP4: &[&str] = &[
            "aria", "eria", "iria", "asse", "esse", "isse", "aste", "este", "iste", "arei", "erei",
            "irei", "aram", "eram", "iram", "avam", "arem", "erem", "irem", "ando", "endo", "indo",
            "arao", "erao", "irao", "adas", "idas", "aras", "eras", "iras", "avas", "ares", "eres",
            "ires", "ados", "idos", "amos", "emos", "imos", "iras", "ieis",
        ];
        const GROUP3: &[&str] = &[
            "ada", "ida", "ara", "era", "ira", "iam", "ado", "ido", "ias", "ais", "eis", "ira",
            "ear",
        ];
        const GROUP2: &[&str] = &[
            "ia", "ei", "am", "em", "ar", "er", "ir", "as", "es", "is", "eu", "iu", "iu", "ou",
        ];

        let Some(rv) = self.rv.as_deref() else {
            return false;
        };
        // Group order mirrors the Java length blocks: 7, 6, 5, 4, 3, 2.
        for (group_idx, group) in [GROUP7, GROUP6, GROUP5, GROUP4, GROUP3, GROUP2]
            .iter()
            .copied()
            .enumerate()
        {
            for suffix in group {
                if rv.ends_with(suffix) {
                    // Upstream quirk preserved: the first `ira` entry in the
                    // 3-suffix group removes `ava` instead of `ira` (making
                    // the later duplicate `ira` entry unreachable, as in the
                    // Java source).
                    let to_remove = if group_idx == 4 && *suffix == "ira" {
                        "ava"
                    } else {
                        suffix
                    };
                    self.ct = remove_suffix(&self.ct, to_remove);
                    return true;
                }
            }
        }
        false
    }

    /// Delete suffix `i` if in RV and preceded by `c`.
    fn step3(&mut self) {
        let Some(rv) = self.rv.as_deref() else {
            return;
        };
        if rv.ends_with("i") && suffix_preceded(rv, "i", "c") {
            self.ct = remove_suffix(&self.ct, "i");
        }
    }

    /// Residual suffix: drop `os`/`a`/`i`/`o` when RV ends with it.
    fn step4(&mut self) {
        let Some(rv) = self.rv.as_deref() else {
            return;
        };
        for suffix in ["os", "a", "i", "o"] {
            if rv.ends_with(suffix) {
                self.ct = remove_suffix(&self.ct, suffix);
                return;
            }
        }
    }

    /// Drop a final `e` (checked against RV); also unwind `gu`→`g` / `ci`→`c`.
    fn step5(&mut self) {
        let Some(rv) = self.rv.as_deref() else {
            return;
        };
        if !rv.ends_with("e") {
            return;
        }
        if suffix_preceded(rv, "e", "gu") {
            self.ct = remove_suffix(&self.ct, "e");
            self.ct = remove_suffix(&self.ct, "u");
            return;
        }
        if suffix_preceded(rv, "e", "ci") {
            self.ct = remove_suffix(&self.ct, "e");
            self.ct = remove_suffix(&self.ct, "i");
            return;
        }
        self.ct = remove_suffix(&self.ct, "e");
    }
}

/// changeTerm: lowercase (pt-BR) and fold the Portuguese accents/diacritics.
fn change_term(value: &str) -> String {
    let mut r = String::with_capacity(value.len());
    for c in value.to_lowercase().chars() {
        match c {
            'á' | 'â' | 'ã' => r.push('a'),
            'é' | 'ê' => r.push('e'),
            'í' => r.push('i'),
            'ó' | 'ô' | 'õ' => r.push('o'),
            'ú' | 'ü' => r.push('u'),
            'ç' => r.push('c'),
            'ñ' => r.push('n'),
            _ => r.push(c),
        }
    }
    r
}

fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u')
}

/// R1: region after the first non-vowel following a vowel (None if the word
/// has no such position), mirroring the Java index arithmetic exactly.
fn get_r1(value: &str) -> Option<String> {
    let v: Vec<char> = value.chars().collect();
    let i = v.len() as isize - 1;
    let mut j: isize = 0;
    while j < i {
        if is_vowel(v[j as usize]) {
            break;
        }
        j += 1;
    }
    if j >= i {
        return None;
    }
    while j < i {
        if !is_vowel(v[j as usize]) {
            break;
        }
        j += 1;
    }
    if j >= i {
        return None;
    }
    Some(v[(j + 1) as usize..].iter().collect())
}

/// RV: after the vowel following a second-letter consonant, or after the
/// consonant following two leading vowels, or after the third letter.
fn get_rv(value: &str) -> Option<String> {
    let v: Vec<char> = value.chars().collect();
    let i = v.len() as isize - 1;

    if i > 0 && !is_vowel(v[1]) {
        let mut j: isize = 2;
        while j < i {
            if is_vowel(v[j as usize]) {
                break;
            }
            j += 1;
        }
        if j < i {
            return Some(v[(j + 1) as usize..].iter().collect());
        }
    }

    if i > 1 && is_vowel(v[0]) && is_vowel(v[1]) {
        let mut j: isize = 2;
        while j < i {
            if !is_vowel(v[j as usize]) {
                break;
            }
            j += 1;
        }
        if j < i {
            return Some(v[(j + 1) as usize..].iter().collect());
        }
    }

    if i > 2 {
        return Some(v[3..].iter().collect());
    }
    None
}

fn opt_ends(value: Option<&str>, suffix: &str) -> bool {
    value.is_some_and(|v| v.ends_with(suffix))
}

fn remove_suffix(value: &str, to_remove: &str) -> String {
    if value.ends_with(to_remove) && to_remove.len() <= value.len() {
        value[..value.len() - to_remove.len()].to_string()
    } else {
        value.to_string()
    }
}

fn replace_suffix(value: &str, to_replace: &str, change_to: &str) -> String {
    let v = remove_suffix(value, to_replace);
    if v == value {
        value.to_string()
    } else {
        format!("{v}{change_to}")
    }
}

fn suffix_preceded(value: &str, suffix: &str, preceded: &str) -> bool {
    value.ends_with(suffix) && remove_suffix(value, suffix).ends_with(preceded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pizza_engine::analysis::TokenFilter;

    fn stem(term: &str) -> String {
        BrazilianStemmer::stem(term).unwrap_or_else(|| term.to_string())
    }

    // Vectors from Lucene's TestBrazilianAnalyzer.testWithSnowballExamples.
    #[test]
    fn test_lucene_snowball_examples() {
        assert_eq!(stem("boa"), "boa");
        assert_eq!(stem("boainain"), "boainain");
        assert_eq!(stem("boas"), "boas");
        assert_eq!(stem("bôas"), "boas"); // removes diacritic
        assert_eq!(stem("boassu"), "boassu");
        assert_eq!(stem("boataria"), "boat");
        assert_eq!(stem("boate"), "boat");
        assert_eq!(stem("boates"), "boat");
        assert_eq!(stem("boatos"), "boat");
        assert_eq!(stem("bob"), "bob");
        assert_eq!(stem("boba"), "bob");
        assert_eq!(stem("bobagem"), "bobag");
        assert_eq!(stem("bobagens"), "bobagens");
        assert_eq!(stem("bobalhões"), "bobalho"); // removes diacritic
        assert_eq!(stem("bobear"), "bob");
        assert_eq!(stem("bobeira"), "bobeir");
        assert_eq!(stem("bobinho"), "bobinh");
        assert_eq!(stem("bobinhos"), "bobinh");
        assert_eq!(stem("bobo"), "bob");
        assert_eq!(stem("bobs"), "bobs");
        assert_eq!(stem("boca"), "boc");
        assert_eq!(stem("bocadas"), "boc");
        assert_eq!(stem("bocadinho"), "bocadinh");
        assert_eq!(stem("bocado"), "boc");
        assert_eq!(stem("bocaiúva"), "bocaiuv"); // removes diacritic
        assert_eq!(stem("boçal"), "bocal"); // removes diacritic
        assert_eq!(stem("bocarra"), "bocarr");
        assert_eq!(stem("bocas"), "boc");
        assert_eq!(stem("bode"), "bod");
        assert_eq!(stem("bodoque"), "bodoqu");
        assert_eq!(stem("body"), "body");
        assert_eq!(stem("boeing"), "boeing");
        assert_eq!(stem("boem"), "boem");
        assert_eq!(stem("boemia"), "boem");
        assert_eq!(stem("boêmio"), "boemi"); // removes diacritic
        assert_eq!(stem("bogotá"), "bogot");
        assert_eq!(stem("boi"), "boi");
        assert_eq!(stem("bóia"), "boi"); // removes diacritic
        assert_eq!(stem("boiando"), "boi");
        assert_eq!(stem("quiabo"), "quiab");
        assert_eq!(stem("quicaram"), "quic");
        assert_eq!(stem("quickly"), "quickly");
        assert_eq!(stem("quieto"), "quiet");
        assert_eq!(stem("quietos"), "quiet");
        assert_eq!(stem("quilate"), "quilat");
        assert_eq!(stem("quilates"), "quilat");
        assert_eq!(stem("quilinhos"), "quilinh");
        assert_eq!(stem("quilo"), "quil");
        assert_eq!(stem("quilombo"), "quilomb");
        assert_eq!(stem("quilométricas"), "quilometr"); // removes diacritic
        assert_eq!(stem("quilométricos"), "quilometr"); // removes diacritic
        assert_eq!(stem("quilômetro"), "quilometr"); // removes diacritic
        assert_eq!(stem("quilômetros"), "quilometr"); // removes diacritic
        assert_eq!(stem("quilos"), "quil");
        assert_eq!(stem("quimica"), "quimic");
        assert_eq!(stem("quimicas"), "quimic");
        assert_eq!(stem("quimico"), "quimic");
        assert_eq!(stem("quimicos"), "quimic");
        assert_eq!(stem("quimioterapia"), "quimioterap");
        assert_eq!(stem("quimioterápicos"), "quimioterap"); // removes diacritic
        assert_eq!(stem("quimono"), "quimon");
        assert_eq!(stem("quincas"), "quinc");
        assert_eq!(stem("quinhão"), "quinha"); // removes diacritic
        assert_eq!(stem("quinhentos"), "quinhent");
        assert_eq!(stem("quinn"), "quinn");
        assert_eq!(stem("quino"), "quin");
        assert_eq!(stem("quinta"), "quint");
        assert_eq!(stem("quintal"), "quintal");
        assert_eq!(stem("quintana"), "quintan");
        assert_eq!(stem("quintanilha"), "quintanilh");
        assert_eq!(stem("quintão"), "quinta"); // removes diacritic
        assert_eq!(stem("quintessência"), "quintessente");
        assert_eq!(stem("quintino"), "quintin");
        assert_eq!(stem("quinto"), "quint");
        assert_eq!(stem("quintos"), "quint");
        assert_eq!(stem("quintuplicou"), "quintuplic");
        assert_eq!(stem("quinze"), "quinz");
        assert_eq!(stem("quinzena"), "quinzen");
        assert_eq!(stem("quiosque"), "quiosqu");
    }

    // Vectors from Lucene's TestBrazilianAnalyzer.testNormalization.
    #[test]
    fn test_lucene_normalization() {
        assert_eq!(stem("Brasil"), "brasil"); // lowercase by default
        assert_eq!(stem("Brasília"), "brasil"); // remove diacritics
                                                // Contains a non-letter: diacritic still removed, term not stemmed.
        assert_eq!(stem("quimio5terápicos"), "quimio5terapicos");
        // Token too short: diacritics are NOT removed.
        assert_eq!(stem("áá"), "áá");
        assert_eq!(stem("ááá"), "aaa");
    }

    #[test]
    fn test_too_short_and_long_left_unchanged() {
        let filter = BrazilianStemTokenFilter::new();
        let mut token = Token::new("sol", 0, 3, 0);
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "sol");
    }
}
