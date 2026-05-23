use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Spanish plural stemmer (full algorithm from Lucene's SpanishPluralStemmer).
/// Handles complex plural rules including invariants, special cases, and accent removal.
#[derive(Clone, Debug)]
pub struct SpanishPluralStemTokenFilter;

impl SpanishPluralStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SpanishPluralStemTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Minimal plural stemmer for Spanish (deprecated in Lucene, but kept for compat).
#[derive(Clone, Debug)]
pub struct SpanishMinimalStemTokenFilter;

impl SpanishMinimalStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SpanishMinimalStemTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u')
}

fn remove_accents(s: &mut Vec<char>) {
    for c in s.iter_mut() {
        *c = match *c {
            'à' | 'á' | 'â' | 'ä' => 'a',
            'ò' | 'ó' | 'ô' | 'ö' => 'o',
            'è' | 'é' | 'ê' | 'ë' => 'e',
            'ù' | 'ú' | 'û' | 'ü' => 'u',
            'ì' | 'í' | 'î' | 'ï' => 'i',
            other => other,
        };
    }
}

fn is_invariant(s: &[char]) -> bool {
    let word: String = s.iter().collect();
    let word_lower = word.to_lowercase();
    INVARIANTS.contains(&word_lower.as_str())
}

fn is_special(s: &[char]) -> bool {
    let word: String = s.iter().collect();
    let word_lower = word.to_lowercase();
    SPECIAL_CASES.contains(&word_lower.as_str())
}

fn stem_plural(chars: &mut Vec<char>) -> usize {
    let mut len = chars.len();
    if len < 4 {
        return len;
    }
    remove_accents(chars);
    if is_invariant(&chars[..len]) {
        return len;
    }
    if is_special(&chars[..len]) {
        return len - 2;
    }

    if chars[len - 1] != 's' {
        return len;
    }

    if !is_vowel(chars[len - 2]) {
        return len - 1;
    }

    if len > 4
        && (chars[len - 4] == 'q' || chars[len - 4] == 'g')
        && chars[len - 3] == 'u'
        && (chars[len - 2] == 'i' || chars[len - 2] == 'e')
    {
        return len - 1;
    }

    if len > 4 && is_vowel(chars[len - 4]) && chars[len - 3] == 'r' && chars[len - 2] == 'e' {
        return len - 2;
    }

    if len > 4
        && is_vowel(chars[len - 4])
        && matches!(chars[len - 3], 'd' | 'l' | 'n' | 'x')
        && chars[len - 2] == 'e'
    {
        return len - 2;
    }

    if len > 3 && (chars[len - 3] == 'y' || chars[len - 3] == 'u') && chars[len - 2] == 'e' {
        return len - 2;
    }

    if len > 4
        && matches!(chars[len - 4], 'u' | 'l' | 'r' | 't' | 'n')
        && chars[len - 3] == 'i'
        && chars[len - 2] == 'e'
    {
        return len - 2;
    }

    if len > 3 && chars[len - 3] == 's' && chars[len - 2] == 'e' {
        return len - 2;
    }

    if len > 3 && is_vowel(chars[len - 3]) && chars[len - 2] == 'i' {
        chars[len - 2] = 'y';
        return len - 1;
    }

    if len > 3 && chars[len - 3] == 'd' && chars[len - 2] == 'i' {
        chars[len - 2] = 'y';
        return len - 1;
    }

    if len > 3 && chars[len - 2] == 'e' && chars[len - 3] == 'c' {
        chars[len - 3] = 'z';
        return len - 2;
    }

    if is_vowel(chars[len - 2]) {
        return len - 1;
    }

    len
}

impl TokenFilter for SpanishPluralStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let mut chars: Vec<char> = text.chars().collect();
        let original_len = chars.len();
        let new_len = stem_plural(&mut chars);
        if new_len < original_len {
            let result: String = chars[..new_len].iter().collect();
            token.term = Cow::Owned(result);
        } else if chars != text.chars().collect::<Vec<_>>() {
            // Accents were removed or chars changed
            let result: String = chars[..new_len].iter().collect();
            token.term = Cow::Owned(result);
        }
        (false, None)
    }
}

fn stem_minimal(chars: &mut Vec<char>) -> usize {
    let len = chars.len();
    if len < 4 || chars[len - 1] != 's' {
        return len;
    }
    // Remove accents
    for c in chars.iter_mut() {
        *c = match *c {
            'à' | 'á' | 'â' | 'ä' => 'a',
            'ò' | 'ó' | 'ô' | 'ö' => 'o',
            'è' | 'é' | 'ê' | 'ë' => 'e',
            'ù' | 'ú' | 'û' | 'ü' => 'u',
            'ì' | 'í' | 'î' | 'ï' => 'i',
            'ñ' => 'n',
            other => other,
        };
    }

    match chars[len - 2] {
        'a' | 'o' => len - 1,
        'e' => {
            if len > 4 && chars[len - 3] == 's' && chars[len - 4] == 'e' {
                len - 2
            } else if len > 3 && chars[len - 3] == 'c' {
                chars[len - 3] = 'z';
                len - 2
            } else if len > 3 {
                len - 2
            } else {
                len
            }
        }
        _ => len,
    }
}

impl TokenFilter for SpanishMinimalStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let mut chars: Vec<char> = text.chars().collect();
        let original_len = chars.len();
        let new_len = stem_minimal(&mut chars);
        if new_len < original_len || chars != text.chars().collect::<Vec<_>>() {
            let result: String = chars[..new_len].iter().collect();
            token.term = Cow::Owned(result);
        }
        (false, None)
    }
}

static INVARIANTS: &[&str] = &[
    "abrebotellas", "abrecartas", "abrelatas", "afueras", "albatros", "albricias",
    "aledaños", "alexis", "alicates", "analisis", "andurriales", "antitesis",
    "añicos", "apendicitis", "apocalipsis", "arcoiris", "aries", "bilis",
    "boletus", "boris", "brindis", "cactus", "canutas", "caries",
    "cascanueces", "cascarrabias", "ciempies", "cifosis", "cortaplumas",
    "corpus", "cosmos", "cosquillas", "creces", "crisis", "cuatrocientas",
    "cuatrocientos", "cuelgacapas", "cuentacuentos", "cuentapasos",
    "cumpleaños", "doscientas", "doscientos", "dosis", "enseres",
    "entonces", "esponsales", "estatus", "exequias", "fauces", "forceps",
    "fotosintesis", "gafas", "gafotas", "gargaras", "gris", "honorarios",
    "ictus", "jueves", "lapsus", "lavacoches", "lavaplatos", "limpiabotas",
    "lunes", "maitines", "martes", "mondadientes", "novecientas",
    "novecientos", "nupcias", "ochocientas", "ochocientos", "pais",
    "paris", "parabrisas", "paracaidas", "parachoques", "paraguas",
    "pararrayos", "pisapapeles", "piscis", "portaaviones", "portamaletas",
    "portamantas", "quinientas", "quinientos", "quitamanchas",
    "recogepelotas", "rictus", "rompeolas", "sacacorchos", "sacapuntas",
    "saltamontes", "salvavidas", "seis", "seiscientas", "seiscientos",
    "setecientas", "setecientos", "sintesis", "tenis", "tifus",
    "trabalenguas", "vacaciones", "venus", "versus", "viacrucis", "virus",
    "viveres", "volandas",
];

static SPECIAL_CASES: &[&str] = &[
    "yoes", "noes", "sies", "clubes", "faralaes", "albalaes", "itemes",
    "albumes", "sandwiches", "relojes", "bojes", "contrarreloj", "carcajes",
];
