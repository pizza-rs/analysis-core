use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Greek stemmer that removes common Greek suffixes.
///
/// Based on the algorithm from Lucene's GreekStemmer, which applies
/// suffix-stripping rules for Greek morphology.
#[derive(Clone, Debug, Default)]
pub struct GreekStemTokenFilter;

impl GreekStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for GreekStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let mut chars: Vec<char> = text.chars().collect();
        let len = chars.len();

        if len < 4 {
            return (false, None);
        }

        if let Some(new_len) = stem_greek(&mut chars, len) {
            if new_len < len && new_len >= 3 {
                let stemmed: String = chars[..new_len].iter().collect();
                token.term = Cow::Owned(stemmed);
            }
        }

        (false, None)
    }
}

/// Lucene GreekStemmer — faithful port of
/// `org.apache.lucene.analysis.el.GreekStemmer`. Input is expected lowercase
/// with accents stripped (the Greek filter chain lowercases and removes
/// tonos/dialytika before stemming).
fn stem_greek(s: &mut [char], len: usize) -> Option<usize> {
    if len < 4 {
        return Some(len); // too short
    }

    let orig_len = len;
    let mut len = len;
    // "short rules": if one hits, the "long list" is skipped.
    len = rule0(s, len);
    len = rule1(s, len);
    len = rule2(s, len);
    len = rule3(s, len);
    len = rule4(s, len);
    len = rule5(s, len);
    len = rule6(s, len);
    len = rule7(s, len);
    len = rule8(s, len);
    len = rule9(s, len);
    len = rule10(s, len);
    len = rule11(s, len);
    len = rule12(s, len);
    len = rule13(s, len);
    len = rule14(s, len);
    len = rule15(s, len);
    len = rule16(s, len);
    len = rule17(s, len);
    len = rule18(s, len);
    len = rule19(s, len);
    len = rule20(s, len);
    // "long list"
    if len == orig_len {
        len = rule21(s, len);
    }

    Some(rule22(s, len))
}

fn ends_with(s: &[char], len: usize, suffix: &str) -> bool {
    let suffix: Vec<char> = suffix.chars().collect();
    len >= suffix.len() && &s[len - suffix.len()..len] == suffix.as_slice()
}

fn in_set(s: &[char], len: usize, set: &[&str]) -> bool {
    let word: String = s[..len].iter().collect();
    set.iter().any(|e| *e == word)
}

fn ends_with_vowel(s: &[char], len: usize) -> bool {
    len > 0 && matches!(s[len - 1], 'α' | 'ε' | 'η' | 'ι' | 'ο' | 'υ' | 'ω')
}

fn ends_with_vowel_no_y(s: &[char], len: usize) -> bool {
    len > 0 && matches!(s[len - 1], 'α' | 'ε' | 'η' | 'ι' | 'ο' | 'ω')
}

const EXC4: &[&str] = &["θ", "δ", "ελ", "γαλ", "ν", "π", "ιδ", "παρ"];

fn rule4(s: &[char], mut len: usize) -> usize {
    if len > 3 && (ends_with(s, len, "εωσ") || ends_with(s, len, "εων")) {
        len -= 3;
        if in_set(s, len, EXC4) {
            len += 1; // add back -ε
        }
    }
    len
}

fn rule5(s: &[char], mut len: usize) -> usize {
    if len > 2 && ends_with(s, len, "ια") {
        len -= 2;
        if ends_with_vowel(s, len) {
            len += 1; // add back -ι
        }
    } else if len > 3 && (ends_with(s, len, "ιου") || ends_with(s, len, "ιων")) {
        len -= 3;
        if ends_with_vowel(s, len) {
            len += 1; // add back -ι
        }
    }
    len
}

const EXC6: &[&str] = &[
    "αλ",
    "αδ",
    "ενδ",
    "αμαν",
    "αμμοχαλ",
    "ηθ",
    "ανηθ",
    "αντιδ",
    "φυσ",
    "βρωμ",
    "γερ",
    "εξωδ",
    "καλπ",
    "καλλιν",
    "καταδ",
    "μουλ",
    "μπαν",
    "μπαγιατ",
    "μπολ",
    "μποσ",
    "νιτ",
    "ξικ",
    "συνομηλ",
    "πετσ",
    "πιτσ",
    "πικαντ",
    "πλιατσ",
    "ποστελν",
    "πρωτοδ",
    "σερτ",
    "συναδ",
    "τσαμ",
    "υποδ",
    "φιλον",
    "φυλοδ",
    "χασ",
];

fn rule6(s: &[char], mut len: usize) -> usize {
    let mut removed = false;
    if len > 3 && (ends_with(s, len, "ικα") || ends_with(s, len, "ικο")) {
        len -= 3;
        removed = true;
    } else if len > 4 && (ends_with(s, len, "ικου") || ends_with(s, len, "ικων")) {
        len -= 4;
        removed = true;
    }

    if removed && (ends_with_vowel(s, len) || in_set(s, len, EXC6)) {
        len += 2; // add back -ικ
    }
    len
}

const EXC7: &[&str] = &[
    "αναπ",
    "αποθ",
    "αποκ",
    "αποστ",
    "βουβ",
    "ξεθ",
    "ουλ",
    "πεθ",
    "πικρ",
    "ποτ",
    "σιχ",
    "χ",
];

fn rule7(s: &[char], mut len: usize) -> usize {
    if len == 5 && ends_with(s, len, "αγαμε") {
        return len - 1;
    }

    if len > 7 && ends_with(s, len, "ηθηκαμε") {
        len -= 7;
    } else if len > 6 && ends_with(s, len, "ουσαμε") {
        len -= 6;
    } else if len > 5
        && (ends_with(s, len, "αγαμε") || ends_with(s, len, "ησαμε") || ends_with(s, len, "ηκαμε"))
    {
        len -= 5;
    }

    if len > 3 && ends_with(s, len, "αμε") {
        len -= 3;
        if in_set(s, len, EXC7) {
            len += 2; // add back -αμ
        }
    }

    len
}

const EXC8A: &[&str] = &["τρ", "τσ"];

const EXC8B: &[&str] = &[
    "βετερ",
    "βουλκ",
    "βραχμ",
    "γ",
    "δραδουμ",
    "θ",
    "καλπουζ",
    "καστελ",
    "κορμορ",
    "λαοπλ",
    "μωαμεθ",
    "μ",
    "μουσουλμ",
    "ν",
    "ουλ",
    "π",
    "πελεκ",
    "πλ",
    "πολισ",
    "πορτολ",
    "σαρακατσ",
    "σουλτ",
    "τσαρλατ",
    "ορφ",
    "τσιγγ",
    "τσοπ",
    "φωτοστεφ",
    "χ",
    "ψυχοπλ",
    "αγ",
    "γαλ",
    "γερ",
    "δεκ",
    "διπλ",
    "αμερικαν",
    "ουρ",
    "πιθ",
    "πουριτ",
    "σ",
    "ζωντ",
    "ικ",
    "καστ",
    "κοπ",
    "λιχ",
    "λουθηρ",
    "μαιντ",
    "μελ",
    "σιγ",
    "σπ",
    "στεγ",
    "τραγ",
    "τσαγ",
    "φ",
    "ερ",
    "αδαπ",
    "αθιγγ",
    "αμηχ",
    "ανικ",
    "ανοργ",
    "απηγ",
    "απιθ",
    "ατσιγγ",
    "βασ",
    "βασκ",
    "βαθυγαλ",
    "βιομηχ",
    "βραχυκ",
    "διατ",
    "διαφ",
    "ενοργ",
    "θυσ",
    "καπνοβιομηχ",
    "καταγαλ",
    "κλιβ",
    "κοιλαρφ",
    "λιβ",
    "μεγλοβιομηχ",
    "μικροβιομηχ",
    "νταβ",
    "ξηροκλιβ",
    "ολιγοδαμ",
    "ολογαλ",
    "πενταρφ",
    "περηφ",
    "περιτρ",
    "πλατ",
    "πολυδαπ",
    "πολυμηχ",
    "στεφ",
    "ταβ",
    "τετ",
    "υπερηφ",
    "υποκοπ",
    "χαμηλοδαπ",
    "ψηλοταβ",
];

fn rule8(s: &mut [char], mut len: usize) -> usize {
    let mut removed = false;

    // NOTE: Java precedence — && binds tighter than ||, so the length guard
    // only applies to the first suffix of each branch.
    if len > 8 && ends_with(s, len, "ιουντανε") {
        len -= 8;
        removed = true;
    } else if (len > 7 && ends_with(s, len, "ιοντανε"))
        || ends_with(s, len, "ουντανε")
        || ends_with(s, len, "ηθηκανε")
    {
        len -= 7;
        removed = true;
    } else if (len > 6 && ends_with(s, len, "ιοτανε"))
        || ends_with(s, len, "οντανε")
        || ends_with(s, len, "ουσανε")
    {
        len -= 6;
        removed = true;
    } else if (len > 5 && ends_with(s, len, "αγανε"))
        || ends_with(s, len, "ησανε")
        || ends_with(s, len, "οτανε")
        || ends_with(s, len, "ηκανε")
    {
        len -= 5;
        removed = true;
    }

    if removed && in_set(s, len, EXC8A) {
        // add -αγαν (we removed > 4 chars so it's safe)
        len += 4;
        s[len - 4] = 'α';
        s[len - 3] = 'γ';
        s[len - 2] = 'α';
        s[len - 1] = 'ν';
    }

    if len > 3 && ends_with(s, len, "ανε") {
        len -= 3;
        if ends_with_vowel_no_y(s, len) || in_set(s, len, EXC8B) {
            len += 2; // add back -αν
        }
    }

    len
}

const EXC9: &[&str] = &[
    "αβαρ",
    "βεν",
    "εναρ",
    "αβρ",
    "αδ",
    "αθ",
    "αν",
    "απλ",
    "βαρον",
    "ντρ",
    "σκ",
    "κοπ",
    "μπορ",
    "νιφ",
    "παγ",
    "παρακαλ",
    "σερπ",
    "σκελ",
    "συρφ",
    "τοκ",
    "υ",
    "δ",
    "εμ",
    "θαρρ",
    "θ",
];

fn rule9(s: &[char], mut len: usize) -> usize {
    if len > 5 && ends_with(s, len, "ησετε") {
        len -= 5;
    }

    if len > 3 && ends_with(s, len, "ετε") {
        len -= 3;
        if in_set(s, len, EXC9)
            || ends_with_vowel_no_y(s, len)
            || ends_with(s, len, "οδ")
            || ends_with(s, len, "αιρ")
            || ends_with(s, len, "φορ")
            || ends_with(s, len, "ταθ")
            || ends_with(s, len, "διαθ")
            || ends_with(s, len, "σχ")
            || ends_with(s, len, "ενδ")
            || ends_with(s, len, "ευρ")
            || ends_with(s, len, "τιθ")
            || ends_with(s, len, "υπερθ")
            || ends_with(s, len, "ραθ")
            || ends_with(s, len, "ενθ")
            || ends_with(s, len, "ροθ")
            || ends_with(s, len, "σθ")
            || ends_with(s, len, "πυρ")
            || ends_with(s, len, "αιν")
            || ends_with(s, len, "συνδ")
            || ends_with(s, len, "συν")
            || ends_with(s, len, "συνθ")
            || ends_with(s, len, "χωρ")
            || ends_with(s, len, "πον")
            || ends_with(s, len, "βρ")
            || ends_with(s, len, "καθ")
            || ends_with(s, len, "ευθ")
            || ends_with(s, len, "εκθ")
            || ends_with(s, len, "νετ")
            || ends_with(s, len, "ρον")
            || ends_with(s, len, "αρκ")
            || ends_with(s, len, "βαρ")
            || ends_with(s, len, "βολ")
            || ends_with(s, len, "ωφελ")
        {
            len += 2; // add back -ετ
        }
    }

    len
}

fn rule10(s: &mut [char], mut len: usize) -> usize {
    if len > 5 && (ends_with(s, len, "οντασ") || ends_with(s, len, "ωντασ")) {
        len -= 5;
        if len == 3 && ends_with(s, len, "αρχ") {
            len += 3; // add back *ντ
            s[len - 3] = 'ο';
        }
        if ends_with(s, len, "κρε") {
            len += 3; // add back *ντ
            s[len - 3] = 'ω';
        }
    }
    len
}

fn rule11(s: &mut [char], mut len: usize) -> usize {
    if len > 6 && ends_with(s, len, "ομαστε") {
        len -= 6;
        if len == 2 && ends_with(s, len, "ον") {
            len += 5; // add back -ομαστ
        }
    } else if len > 7 && ends_with(s, len, "ιομαστε") {
        len -= 7;
        if len == 2 && ends_with(s, len, "ον") {
            len += 5;
            s[len - 5] = 'ο';
            s[len - 4] = 'μ';
            s[len - 3] = 'α';
            s[len - 2] = 'σ';
            s[len - 1] = 'τ';
        }
    }
    len
}

const EXC12A: &[&str] = &["π", "απ", "συμπ", "ασυμπ", "ακαταπ", "αμεταμφ"];
const EXC12B: &[&str] = &["αλ", "αρ", "εκτελ", "ζ", "μ", "ξ", "παρακαλ", "προ", "νισ"];

fn rule12(s: &[char], mut len: usize) -> usize {
    if len > 5 && ends_with(s, len, "ιεστε") {
        len -= 5;
        if in_set(s, len, EXC12A) {
            len += 4; // add back -ιεστ
        }
    }

    if len > 4 && ends_with(s, len, "εστε") {
        len -= 4;
        if in_set(s, len, EXC12B) {
            len += 3; // add back -εστ
        }
    }

    len
}

const EXC13: &[&str] = &["διαθ", "θ", "παρακαταθ", "προσθ", "συνθ"];

fn rule13(s: &[char], mut len: usize) -> usize {
    if len > 6 && ends_with(s, len, "ηθηκεσ") {
        len -= 6;
    } else if len > 5 && (ends_with(s, len, "ηθηκα") || ends_with(s, len, "ηθηκε")) {
        len -= 5;
    }

    let mut removed = false;

    if len > 4 && ends_with(s, len, "ηκεσ") {
        len -= 4;
        removed = true;
    } else if len > 3 && (ends_with(s, len, "ηκα") || ends_with(s, len, "ηκε")) {
        len -= 3;
        removed = true;
    }

    if removed
        && (in_set(s, len, EXC13)
            || ends_with(s, len, "σκωλ")
            || ends_with(s, len, "σκουλ")
            || ends_with(s, len, "ναρθ")
            || ends_with(s, len, "σφ")
            || ends_with(s, len, "οθ")
            || ends_with(s, len, "πιθ"))
    {
        len += 2; // add back the -ηκ
    }

    len
}

const EXC14: &[&str] = &[
    "φαρμακ",
    "χαδ",
    "αγκ",
    "αναρρ",
    "βρομ",
    "εκλιπ",
    "λαμπιδ",
    "λεχ",
    "μ",
    "πατ",
    "ρ",
    "λ",
    "μεδ",
    "μεσαζ",
    "υποτειν",
    "αμ",
    "αιθ",
    "ανηκ",
    "δεσποζ",
    "ενδιαφερ",
    "δε",
    "δευτερευ",
    "καθαρευ",
    "πλε",
    "τσα",
];

fn rule14(s: &[char], mut len: usize) -> usize {
    let mut removed = false;

    if len > 5 && ends_with(s, len, "ουσεσ") {
        len -= 5;
        removed = true;
    } else if len > 4 && (ends_with(s, len, "ουσα") || ends_with(s, len, "ουσε")) {
        len -= 4;
        removed = true;
    }

    if removed
        && (in_set(s, len, EXC14)
            || ends_with_vowel(s, len)
            || ends_with(s, len, "ποδαρ")
            || ends_with(s, len, "βλεπ")
            || ends_with(s, len, "πανταχ")
            || ends_with(s, len, "φρυδ")
            || ends_with(s, len, "μαντιλ")
            || ends_with(s, len, "μαλλ")
            || ends_with(s, len, "κυματ")
            || ends_with(s, len, "λαχ")
            || ends_with(s, len, "ληγ")
            || ends_with(s, len, "φαγ")
            || ends_with(s, len, "ομ")
            || ends_with(s, len, "πρωτ"))
    {
        len += 3; // add back -ουσ
    }

    len
}

const EXC15A: &[&str] = &[
    "αβαστ",
    "πολυφ",
    "αδηφ",
    "παμφ",
    "ρ",
    "ασπ",
    "αφ",
    "αμαλ",
    "αμαλλι",
    "ανυστ",
    "απερ",
    "ασπαρ",
    "αχαρ",
    "δερβεν",
    "δροσοπ",
    "ξεφ",
    "νεοπ",
    "νομοτ",
    "ολοπ",
    "ομοτ",
    "προστ",
    "προσωποπ",
    "συμπ",
    "συντ",
    "τ",
    "υποτ",
    "χαρ",
    "αειπ",
    "αιμοστ",
    "ανυπ",
    "αποτ",
    "αρτιπ",
    "διατ",
    "εν",
    "επιτ",
    "κροκαλοπ",
    "σιδηροπ",
    "λ",
    "ναυ",
    "ουλαμ",
    "ουρ",
    "π",
    "τρ",
    "μ",
];
const EXC15B: &[&str] = &["ψοφ", "ναυλοχ"];

fn rule15(s: &[char], mut len: usize) -> usize {
    let mut removed = false;
    if len > 4 && ends_with(s, len, "αγεσ") {
        len -= 4;
        removed = true;
    } else if len > 3 && (ends_with(s, len, "αγα") || ends_with(s, len, "αγε")) {
        len -= 3;
        removed = true;
    }

    if removed {
        let cond1 = in_set(s, len, EXC15A)
            || ends_with(s, len, "οφ")
            || ends_with(s, len, "πελ")
            || ends_with(s, len, "χορτ")
            || ends_with(s, len, "λλ")
            || ends_with(s, len, "σφ")
            || ends_with(s, len, "ρπ")
            || ends_with(s, len, "φρ")
            || ends_with(s, len, "πρ")
            || ends_with(s, len, "λοχ")
            || ends_with(s, len, "σμην");

        let cond2 = in_set(s, len, EXC15B) || ends_with(s, len, "κολλ");

        if cond1 && !cond2 {
            len += 2; // add back -αγ
        }
    }

    len
}

const EXC16: &[&str] = &["ν", "χερσον", "δωδεκαν", "ερημον", "μεγαλον", "επταν"];

fn rule16(s: &[char], mut len: usize) -> usize {
    let mut removed = false;
    if len > 4 && ends_with(s, len, "ησου") {
        len -= 4;
        removed = true;
    } else if len > 3 && (ends_with(s, len, "ησε") || ends_with(s, len, "ησα")) {
        len -= 3;
        removed = true;
    }

    if removed && in_set(s, len, EXC16) {
        len += 2; // add back -ησ
    }

    len
}

const EXC17: &[&str] = &[
    "ασβ",
    "σβ",
    "αχρ",
    "χρ",
    "απλ",
    "αειμν",
    "δυσχρ",
    "ευχρ",
    "κοινοχρ",
    "παλιμψ",
];

fn rule17(s: &mut [char], mut len: usize) -> usize {
    if len > 4 && ends_with(s, len, "ηστε") {
        len -= 4;
        if in_set(s, len, EXC17) {
            len += 3; // add back the -ηστ
        }
    }
    len
}

const EXC18: &[&str] = &["ν", "ρ", "σπι", "στραβομουτσ", "κακομουτσ", "εξων"];

fn rule18(s: &mut [char], mut len: usize) -> usize {
    let mut removed = false;

    if len > 6 && (ends_with(s, len, "ησουνε") || ends_with(s, len, "ηθουνε")) {
        len -= 6;
        removed = true;
    } else if len > 4 && ends_with(s, len, "ουνε") {
        len -= 4;
        removed = true;
    }

    if removed && in_set(s, len, EXC18) {
        len += 3;
        s[len - 3] = 'ο';
        s[len - 2] = 'υ';
        s[len - 1] = 'ν';
    }
    len
}

const EXC19: &[&str] = &["παρασουσ", "φ", "χ", "ωριοπλ", "αζ", "αλλοσουσ", "ασουσ"];

fn rule19(s: &mut [char], mut len: usize) -> usize {
    let mut removed = false;

    if len > 6 && (ends_with(s, len, "ησουμε") || ends_with(s, len, "ηθουμε")) {
        len -= 6;
        removed = true;
    } else if len > 4 && ends_with(s, len, "ουμε") {
        len -= 4;
        removed = true;
    }

    if removed && in_set(s, len, EXC19) {
        len += 3;
        s[len - 3] = 'ο';
        s[len - 2] = 'υ';
        s[len - 1] = 'μ';
    }
    len
}

fn rule20(s: &[char], mut len: usize) -> usize {
    if len > 5 && (ends_with(s, len, "ματων") || ends_with(s, len, "ματοσ")) {
        len -= 3;
    } else if len > 4 && ends_with(s, len, "ματα") {
        len -= 2;
    }
    len
}

fn rule21(s: &[char], mut len: usize) -> usize {
    if len > 9 && ends_with(s, len, "ιοντουσαν") {
        return len - 9;
    }

    if len > 8
        && (ends_with(s, len, "ιομασταν")
            || ends_with(s, len, "ιοσασταν")
            || ends_with(s, len, "ιουμαστε")
            || ends_with(s, len, "οντουσαν"))
    {
        return len - 8;
    }

    if len > 7
        && (ends_with(s, len, "ιεμαστε")
            || ends_with(s, len, "ιεσαστε")
            || ends_with(s, len, "ιομουνα")
            || ends_with(s, len, "ιοσαστε")
            || ends_with(s, len, "ιοσουνα")
            || ends_with(s, len, "ιουνται")
            || ends_with(s, len, "ιουνταν")
            || ends_with(s, len, "ηθηκατε")
            || ends_with(s, len, "ομασταν")
            || ends_with(s, len, "οσασταν")
            || ends_with(s, len, "ουμαστε"))
    {
        return len - 7;
    }

    if len > 6
        && (ends_with(s, len, "ιομουν")
            || ends_with(s, len, "ιονταν")
            || ends_with(s, len, "ιοσουν")
            || ends_with(s, len, "ηθειτε")
            || ends_with(s, len, "ηθηκαν")
            || ends_with(s, len, "ομουνα")
            || ends_with(s, len, "οσαστε")
            || ends_with(s, len, "οσουνα")
            || ends_with(s, len, "ουνται")
            || ends_with(s, len, "ουνταν")
            || ends_with(s, len, "ουσατε"))
    {
        return len - 6;
    }

    if len > 5
        && (ends_with(s, len, "αγατε")
            || ends_with(s, len, "ιεμαι")
            || ends_with(s, len, "ιεται")
            || ends_with(s, len, "ιεσαι")
            || ends_with(s, len, "ιοταν")
            || ends_with(s, len, "ιουμα")
            || ends_with(s, len, "ηθεισ")
            || ends_with(s, len, "ηθουν")
            || ends_with(s, len, "ηκατε")
            || ends_with(s, len, "ησατε")
            || ends_with(s, len, "ησουν")
            || ends_with(s, len, "ομουν")
            || ends_with(s, len, "ονται")
            || ends_with(s, len, "ονταν")
            || ends_with(s, len, "οσουν")
            || ends_with(s, len, "ουμαι")
            || ends_with(s, len, "ουσαν"))
    {
        return len - 5;
    }

    if len > 4
        && (ends_with(s, len, "αγαν")
            || ends_with(s, len, "αμαι")
            || ends_with(s, len, "ασαι")
            || ends_with(s, len, "αται")
            || ends_with(s, len, "ειτε")
            || ends_with(s, len, "εσαι")
            || ends_with(s, len, "εται")
            || ends_with(s, len, "ηδεσ")
            || ends_with(s, len, "ηδων")
            || ends_with(s, len, "ηθει")
            || ends_with(s, len, "ηκαν")
            || ends_with(s, len, "ησαν")
            || ends_with(s, len, "ησει")
            || ends_with(s, len, "ησεσ")
            || ends_with(s, len, "ομαι")
            || ends_with(s, len, "οταν"))
    {
        return len - 4;
    }

    if len > 3
        && (ends_with(s, len, "αει")
            || ends_with(s, len, "εισ")
            || ends_with(s, len, "ηθω")
            || ends_with(s, len, "ησω")
            || ends_with(s, len, "ουν")
            || ends_with(s, len, "ουσ"))
    {
        return len - 3;
    }

    if len > 2
        && (ends_with(s, len, "αν")
            || ends_with(s, len, "ασ")
            || ends_with(s, len, "αω")
            || ends_with(s, len, "ει")
            || ends_with(s, len, "εσ")
            || ends_with(s, len, "ησ")
            || ends_with(s, len, "οι")
            || ends_with(s, len, "οσ")
            || ends_with(s, len, "ου")
            || ends_with(s, len, "υσ")
            || ends_with(s, len, "ων"))
    {
        return len - 2;
    }

    if len > 1 && ends_with_vowel(s, len) {
        return len - 1;
    }

    len
}

fn rule22(s: &[char], len: usize) -> usize {
    if ends_with(s, len, "εστερ") || ends_with(s, len, "εστατ") {
        return len - 5;
    }

    if ends_with(s, len, "οτερ")
        || ends_with(s, len, "οτατ")
        || ends_with(s, len, "υτερ")
        || ends_with(s, len, "υτατ")
        || ends_with(s, len, "ωτερ")
        || ends_with(s, len, "ωτατ")
    {
        return len - 4;
    }

    len
}

fn rule0(s: &[char], mut len: usize) -> usize {
    if len > 9 && (ends_with(s, len, "καθεστωτοσ") || ends_with(s, len, "καθεστωτων"))
    {
        return len - 4;
    }

    if len > 8 && (ends_with(s, len, "γεγονοτοσ") || ends_with(s, len, "γεγονοτων"))
    {
        return len - 4;
    }

    if len > 8 && ends_with(s, len, "καθεστωτα") {
        return len - 3;
    }

    if len > 7 && (ends_with(s, len, "τατογιου") || ends_with(s, len, "τατογιων")) {
        return len - 4;
    }

    if len > 7 && ends_with(s, len, "γεγονοτα") {
        return len - 3;
    }

    if len > 7 && ends_with(s, len, "καθεστωσ") {
        return len - 2;
    }

    if len > 6
        && (ends_with(s, len, "σκαγιου")
            || ends_with(s, len, "σκαγιων")
            || ends_with(s, len, "ολογιου")
            || ends_with(s, len, "ολογιων")
            || ends_with(s, len, "κρεατοσ")
            || ends_with(s, len, "κρεατων")
            || ends_with(s, len, "περατοσ")
            || ends_with(s, len, "περατων")
            || ends_with(s, len, "τερατοσ")
            || ends_with(s, len, "τερατων"))
    {
        return len - 4;
    }

    if len > 6 && ends_with(s, len, "τατογια") {
        return len - 3;
    }

    if len > 6 && ends_with(s, len, "γεγονοσ") {
        return len - 2;
    }

    if len > 5
        && (ends_with(s, len, "φαγιου")
            || ends_with(s, len, "φαγιων")
            || ends_with(s, len, "σογιου")
            || ends_with(s, len, "σογιων"))
    {
        return len - 4;
    }

    if len > 5
        && (ends_with(s, len, "σκαγια")
            || ends_with(s, len, "ολογια")
            || ends_with(s, len, "κρεατα")
            || ends_with(s, len, "περατα")
            || ends_with(s, len, "τερατα"))
    {
        return len - 3;
    }

    if len > 4
        && (ends_with(s, len, "φαγια")
            || ends_with(s, len, "σογια")
            || ends_with(s, len, "φωτοσ")
            || ends_with(s, len, "φωτων"))
    {
        return len - 3;
    }

    if len > 4
        && (ends_with(s, len, "κρεασ") || ends_with(s, len, "περασ") || ends_with(s, len, "τερασ"))
    {
        return len - 2;
    }

    if len > 3 && ends_with(s, len, "φωτα") {
        return len - 2;
    }

    if len > 2 && ends_with(s, len, "φωσ") {
        return len - 1;
    }

    len
}

fn rule1(s: &mut [char], mut len: usize) -> usize {
    if len > 4 && (ends_with(s, len, "αδεσ") || ends_with(s, len, "αδων")) {
        len -= 4;
        if !(ends_with(s, len, "οκ")
            || ends_with(s, len, "μαμ")
            || ends_with(s, len, "μαν")
            || ends_with(s, len, "μπαμπ")
            || ends_with(s, len, "πατερ")
            || ends_with(s, len, "γιαγι")
            || ends_with(s, len, "νταντ")
            || ends_with(s, len, "κυρ")
            || ends_with(s, len, "θει")
            || ends_with(s, len, "πεθερ"))
        {
            len += 2; // add back -αδ
        }
    }
    len
}

fn rule2(s: &[char], mut len: usize) -> usize {
    if len > 4 && (ends_with(s, len, "εδεσ") || ends_with(s, len, "εδων")) {
        len -= 4;
        if ends_with(s, len, "οπ")
            || ends_with(s, len, "ιπ")
            || ends_with(s, len, "εμπ")
            || ends_with(s, len, "υπ")
            || ends_with(s, len, "γηπ")
            || ends_with(s, len, "δαπ")
            || ends_with(s, len, "κρασπ")
            || ends_with(s, len, "μιλ")
        {
            len += 2; // add back -εδ
        }
    }
    len
}

fn rule3(s: &[char], mut len: usize) -> usize {
    if len > 5 && (ends_with(s, len, "ουδεσ") || ends_with(s, len, "ουδων")) {
        len -= 5;
        if ends_with(s, len, "αρκ")
            || ends_with(s, len, "καλιακ")
            || ends_with(s, len, "πεταλ")
            || ends_with(s, len, "λιχ")
            || ends_with(s, len, "πλεξ")
            || ends_with(s, len, "σκ")
            || ends_with(s, len, "σ")
            || ends_with(s, len, "φλ")
            || ends_with(s, len, "φρ")
            || ends_with(s, len, "βελ")
            || ends_with(s, len, "λουλ")
            || ends_with(s, len, "χν")
            || ends_with(s, len, "σπ")
            || ends_with(s, len, "τραγ")
            || ends_with(s, len, "φε")
        {
            len += 3; // add back -ουδ
        }
    }
    len
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
    fn test_greek_stem() {
        let filter = GreekStemTokenFilter::new();
        let mut token = make_token("ελληνικα");
        filter.filter(&mut token);
        // Should remove suffix
        assert!(token.term.as_ref().len() < "ελληνικα".len());
    }

    #[test]
    fn test_short_word() {
        let filter = GreekStemTokenFilter::new();
        let mut token = make_token("και");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "και");
    }
}
