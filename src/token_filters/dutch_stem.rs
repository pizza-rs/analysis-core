use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

use super::snowball::Among;
use super::snowball::Grouping;
use super::snowball::SnowballCore;

/// Dutch stemmer — faithful port of the Snowball 3.0.0 `dutch.sbl` algorithm
/// as generated for Lucene (`org.tartarus.snowball.ext.DutchStemmer`).
///
/// The algorithm computes R1/R2, runs four suffix steps, removes `ge-`
/// prefixes and infixes (re-running a residual-suffix step afterwards),
/// undoubles final consonants and maps final `v`→`f` / `z`→`s`, and
/// lengthens vowels after certain deletions (`lichamen` → `lichaam`,
/// `opgraven` → `opgraaf`). The port keeps the generated code's structure,
/// including the labeled-block control flow, so quirks behave identically.
#[derive(Clone, Debug, Default)]
pub struct DutchStemTokenFilter;

impl DutchStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for DutchStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        // Lucene's DutchAnalyzer lowercases ahead of the SnowballFilter.
        let mut stemmer = DutchSnowball::new(&text.to_lowercase());
        stemmer.stem();
        let stemmed = stemmer.result();
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

// ─── Snowball runtime comes from snowball.rs ───────────────────────────────

const G_E_BITS: [u8; 17] = [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 120];
const G_AIOU_BITS: [u8; 20] = [
    1, 65, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 11, 120, 46, 15,
];
const G_AEIOU_BITS: [u8; 20] = [
    17, 65, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 139, 127, 46, 15,
];
const G_V_BITS: [u8; 20] = [
    17, 65, 16, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 139, 127, 46, 15,
];
const G_V_WX_BITS: [u8; 20] = [
    17, 65, 208, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 139, 127, 46, 15,
];

const G_E: Grouping = Grouping {
    bits: &G_E_BITS,
    min: 101,
    max: 235,
};
const G_AIOU: Grouping = Grouping {
    bits: &G_AIOU_BITS,
    min: 97,
    max: 252,
};
const G_AEIOU: Grouping = Grouping {
    bits: &G_AEIOU_BITS,
    min: 97,
    max: 252,
};
const G_V: Grouping = Grouping {
    bits: &G_V_BITS,
    min: 97,
    max: 252,
};
const G_V_WX: Grouping = Grouping {
    bits: &G_V_WX_BITS,
    min: 97,
    max: 252,
};

const A_0: Among = &[
    ("a", 1),
    ("e", 2),
    ("o", 1),
    ("u", 1),
    ("à", 1),
    ("á", 1),
    ("â", 1),
    ("ä", 1),
    ("è", 2),
    ("é", 2),
    ("ê", 2),
    ("eë", 3),
    ("ië", 4),
    ("ò", 1),
    ("ó", 1),
    ("ô", 1),
    ("ö", 1),
    ("ù", 1),
    ("ú", 1),
    ("û", 1),
    ("ü", 1),
];

const A_1: Among = &[
    ("nde", 8),
    ("en", 7),
    ("s", 2),
    ("'s", 1),
    ("es", 4),
    ("ies", 3),
    ("aus", 6),
    ("és", 5),
];

const A_2: Among = &[
    ("de", 5),
    ("ge", 2),
    ("ische", 4),
    ("je", 1),
    ("lijke", 3),
    ("le", 9),
    ("ene", 10),
    ("re", 8),
    ("se", 7),
    ("te", 6),
    ("ieve", 11),
];

const A_3: Among = &[
    ("heid", 3),
    ("fie", 7),
    ("gie", 8),
    ("atie", 1),
    ("isme", 5),
    ("ing", 5),
    ("arij", 6),
    ("erij", 5),
    ("sel", 3),
    ("rder", 4),
    ("ster", 3),
    ("iteit", 2),
    ("dst", 10),
    ("tst", 9),
];

const A_4: Among = &[
    ("end", 9),
    ("atief", 2),
    ("erig", 9),
    ("achtig", 3),
    ("ioneel", 1),
    ("baar", 3),
    ("laar", 5),
    ("naar", 4),
    ("raar", 6),
    ("eriger", 9),
    ("achtiger", 3),
    ("lijker", 8),
    ("tant", 7),
    ("erigst", 9),
    ("achtigst", 3),
    ("lijkst", 8),
];

const A_5: Among = &[("ig", 1), ("iger", 1), ("igst", 1)];

const A_6: Among = &[("ft", 2), ("kt", 1), ("pt", 3)];

const A_7: Among = &[
    ("bb", 1),
    ("cc", 2),
    ("dd", 3),
    ("ff", 4),
    ("gg", 5),
    ("hh", 6),
    ("jj", 7),
    ("kk", 8),
    ("ll", 9),
    ("mm", 10),
    ("nn", 11),
    ("pp", 12),
    ("qq", 13),
    ("rr", 14),
    ("ss", 15),
    ("tt", 16),
    ("v", 4),
    ("vv", 17),
    ("ww", 18),
    ("xx", 19),
    ("z", 15),
    ("zz", 20),
];

const A_8: Among = &[("d", 1), ("t", 2)];

const A_9: Among = &[
    ("", -1),
    ("eft", 1),
    ("vaa", 1),
    ("val", 1),
    ("vali", -1),
    ("vare", 1),
];

const A_10: Among = &[("ë", 1), ("ï", 2)];

const A_11: Among = &[("ë", 1), ("ï", 2)];

struct DutchSnowball {
    core: SnowballCore,
    p1: usize,
    p2: usize,
    ge_removed: bool,
}

impl DutchSnowball {
    fn new(word: &str) -> Self {
        let core = SnowballCore::new(word);
        let limit = core.limit;
        DutchSnowball {
            core,
            p1: limit,
            p2: limit,
            ge_removed: false,
        }
    }

    fn result(&self) -> String {
        self.core.result()
    }

    fn save(&self) -> isize {
        self.core.save()
    }

    fn restore(&mut self, v: isize) {
        self.core.restore(v)
    }

    fn in_grouping(&mut self, g: &Grouping) -> bool {
        self.core.in_grouping(g)
    }

    fn in_grouping_b(&mut self, g: &Grouping) -> bool {
        self.core.in_grouping_b(g)
    }

    fn out_grouping(&mut self, g: &Grouping) -> bool {
        self.core.out_grouping(g)
    }

    fn out_grouping_b(&mut self, g: &Grouping) -> bool {
        self.core.out_grouping_b(g)
    }

    fn eq_s(&mut self, s: &str) -> bool {
        self.core.eq_s(s)
    }

    fn eq_s_b(&mut self, s: &str) -> bool {
        self.core.eq_s_b(s)
    }

    fn find_among(&mut self, v: Among) -> i32 {
        self.core.find_among(v)
    }

    fn find_among_b(&mut self, v: Among) -> i32 {
        self.core.find_among_b(v)
    }

    fn slice_from_str(&mut self, s: &str) {
        self.core.slice_from_str(s)
    }

    fn slice_del(&mut self) {
        self.core.slice_del()
    }

    fn insert_at(&mut self, c_bra: usize, c_ket: usize, s: &[char]) {
        self.core.insert_at(c_bra, c_ket, s)
    }

    fn r_r1(&self) -> bool {
        self.p1 <= self.core.cursor
    }

    fn r_r2(&self) -> bool {
        self.p2 <= self.core.cursor
    }

    /// V: the text before the cursor ends in a vowel or "ij".
    fn r_v(&mut self) -> bool {
        let v_1 = self.save();
        'lab0: {
            let v_2 = self.save();
            'lab1: {
                if !self.in_grouping_b(&G_V) {
                    break 'lab1;
                }
                break 'lab0;
            }
            self.restore(v_2);
            if !self.eq_s_b("ij") {
                return false;
            }
        }
        self.restore(v_1);
        true
    }

    /// VX: the text one position before the cursor ends in a vowel or "ij".
    fn r_vx(&mut self) -> bool {
        let v_1 = self.save();
        if self.core.cursor <= self.core.limit_backward {
            return false;
        }
        self.core.cursor -= 1;
        'lab0: {
            let v_2 = self.save();
            'lab1: {
                if !self.in_grouping_b(&G_V) {
                    break 'lab1;
                }
                break 'lab0;
            }
            self.restore(v_2);
            if !self.eq_s_b("ij") {
                return false;
            }
        }
        self.restore(v_1);
        true
    }

    /// C: the text before the cursor ends in a consonant other than "ij".
    fn r_c(&mut self) -> bool {
        let v_1 = self.save();
        {
            let v_2 = self.save();
            if self.eq_s_b("ij") {
                return false;
            }
            self.restore(v_2);
        }
        if !self.out_grouping_b(&G_V) {
            return false;
        }
        self.restore(v_1);
        true
    }

    /// Euphony: after deleting a suffix, restore vowel length when the
    /// remaining stem ends in a single vowel preceded by a consonant that is
    /// not w/x (duplicate it), or repair `eë`/`ië` clusters.
    fn r_lengthen_v(&mut self) -> bool {
        let v_1 = self.save();
        'lab0: {
            if !self.out_grouping_b(&G_V_WX) {
                break 'lab0;
            }
            self.core.ket = self.core.cursor;
            let among_var = self.find_among_b(A_0);
            if among_var == 0 {
                break 'lab0;
            }
            self.core.bra = self.core.cursor;
            match among_var {
                1 => {
                    let v_2 = self.save();
                    'lab1: {
                        let v_3 = self.save();
                        'lab2: {
                            if !self.out_grouping_b(&G_AEIOU) {
                                break 'lab2;
                            }
                            break 'lab1;
                        }
                        self.restore(v_3);
                        if self.core.cursor > self.core.limit_backward {
                            break 'lab0;
                        }
                    }
                    self.restore(v_2);
                    let slice: Vec<char> = self.core.text[self.core.bra..self.core.ket].to_vec();
                    let c = self.core.cursor;
                    self.insert_at(self.core.cursor, self.core.cursor, &slice);
                    self.core.cursor = c;
                }
                2 => {
                    let v_4 = self.save();
                    'lab3: {
                        let v_5 = self.save();
                        'lab4: {
                            if !self.out_grouping_b(&G_AEIOU) {
                                break 'lab4;
                            }
                            break 'lab3;
                        }
                        self.restore(v_5);
                        if self.core.cursor > self.core.limit_backward {
                            break 'lab0;
                        }
                    }
                    let v_6 = self.save();
                    'lab5: {
                        'lab6: {
                            let v_7 = self.save();
                            'lab7: {
                                if !self.in_grouping_b(&G_AIOU) {
                                    break 'lab7;
                                }
                                break 'lab6;
                            }
                            self.restore(v_7);
                            if !self.in_grouping_b(&G_E) {
                                break 'lab5;
                            }
                            if self.core.cursor > self.core.limit_backward {
                                break 'lab5;
                            }
                        }
                        break 'lab0;
                    }
                    self.restore(v_6);
                    let v_8 = self.save();
                    'lab8: {
                        if self.core.cursor <= self.core.limit_backward {
                            break 'lab8;
                        }
                        self.core.cursor -= 1;
                        if !self.in_grouping_b(&G_AIOU) {
                            break 'lab8;
                        }
                        if !self.out_grouping_b(&G_AEIOU) {
                            break 'lab8;
                        }
                        break 'lab0;
                    }
                    self.restore(v_8);
                    self.restore(v_4);
                    let slice: Vec<char> = self.core.text[self.core.bra..self.core.ket].to_vec();
                    let c = self.core.cursor;
                    self.insert_at(self.core.cursor, self.core.cursor, &slice);
                    self.core.cursor = c;
                }
                3 => self.slice_from_str("eëe"),
                4 => self.slice_from_str("iee"),
                _ => {}
            }
        }
        self.restore(v_1);
        true
    }

    // ─── suffix steps ───────────────────────────────────────────────────────

    fn r_step_1(&mut self) -> bool {
        self.core.ket = self.core.cursor;
        let among_var = self.find_among_b(A_1);
        if among_var == 0 {
            return false;
        }
        self.core.bra = self.core.cursor;
        match among_var {
            1 => self.slice_del(),
            2 => {
                if !self.r_r1() {
                    return false;
                }
                let v_1 = self.save();
                'lab0: {
                    if !self.eq_s_b("t") {
                        break 'lab0;
                    }
                    if !self.r_r1() {
                        break 'lab0;
                    }
                    return false;
                }
                self.restore(v_1);
                if !self.r_c() {
                    return false;
                }
                self.slice_del();
            }
            3 => {
                if !self.r_r1() {
                    return false;
                }
                self.slice_from_str("ie");
            }
            4 => 'lab1: {
                let v_2 = self.save();
                'lab2: {
                    let v_3 = self.save();
                    if !self.eq_s_b("ar") {
                        break 'lab2;
                    }
                    if !self.r_r1() {
                        break 'lab2;
                    }
                    if !self.r_c() {
                        break 'lab2;
                    }
                    self.restore(v_3);
                    self.slice_del();
                    self.r_lengthen_v();
                    break 'lab1;
                }
                self.restore(v_2);
                'lab3: {
                    let v_4 = self.save();
                    if !self.eq_s_b("er") {
                        break 'lab3;
                    }
                    if !self.r_r1() {
                        break 'lab3;
                    }
                    if !self.r_c() {
                        break 'lab3;
                    }
                    self.restore(v_4);
                    self.slice_del();
                    break 'lab1;
                }
                self.restore(v_2);
                if !self.r_r1() {
                    return false;
                }
                if !self.r_c() {
                    return false;
                }
                self.slice_from_str("e");
            }
            5 => {
                if !self.r_r1() {
                    return false;
                }
                self.slice_from_str("é");
            }
            6 => {
                if !self.r_r1() {
                    return false;
                }
                if !self.r_v() {
                    return false;
                }
                self.slice_from_str("au");
            }
            7 => 'lab4: {
                let v_5 = self.save();
                'lab5: {
                    if !self.eq_s_b("hed") {
                        break 'lab5;
                    }
                    if !self.r_r1() {
                        break 'lab5;
                    }
                    self.core.bra = self.core.cursor;
                    self.slice_from_str("heid");
                    break 'lab4;
                }
                self.restore(v_5);
                'lab6: {
                    if !self.eq_s_b("nd") {
                        break 'lab6;
                    }
                    self.slice_del();
                    break 'lab4;
                }
                self.restore(v_5);
                'lab7: {
                    if !self.eq_s_b("d") {
                        break 'lab7;
                    }
                    if !self.r_r1() {
                        break 'lab7;
                    }
                    if !self.r_c() {
                        break 'lab7;
                    }
                    self.core.bra = self.core.cursor;
                    self.slice_del();
                    break 'lab4;
                }
                self.restore(v_5);
                'lab8: {
                    'lab9: {
                        let v_6 = self.save();
                        'lab10: {
                            if !self.eq_s_b("i") {
                                break 'lab10;
                            }
                            break 'lab9;
                        }
                        self.restore(v_6);
                        if !self.eq_s_b("j") {
                            break 'lab8;
                        }
                    }
                    if !self.r_v() {
                        break 'lab8;
                    }
                    self.slice_del();
                    break 'lab4;
                }
                self.restore(v_5);
                if !self.r_r1() {
                    return false;
                }
                if !self.r_c() {
                    return false;
                }
                self.slice_del();
                self.r_lengthen_v();
            }
            8 => self.slice_from_str("nd"),
            _ => {}
        }
        true
    }

    fn r_step_2(&mut self) -> bool {
        self.core.ket = self.core.cursor;
        let among_var = self.find_among_b(A_2);
        if among_var == 0 {
            return false;
        }
        self.core.bra = self.core.cursor;
        match among_var {
            1 => 'lab0: {
                let v_1 = self.save();
                'lab1: {
                    if !self.eq_s_b("'t") {
                        break 'lab1;
                    }
                    self.core.bra = self.core.cursor;
                    self.slice_del();
                    break 'lab0;
                }
                self.restore(v_1);
                'lab2: {
                    if !self.eq_s_b("et") {
                        break 'lab2;
                    }
                    self.core.bra = self.core.cursor;
                    if !self.r_r1() {
                        break 'lab2;
                    }
                    if !self.r_c() {
                        break 'lab2;
                    }
                    self.slice_del();
                    break 'lab0;
                }
                self.restore(v_1);
                'lab3: {
                    if !self.eq_s_b("rnt") {
                        break 'lab3;
                    }
                    self.core.bra = self.core.cursor;
                    self.slice_from_str("rn");
                    break 'lab0;
                }
                self.restore(v_1);
                'lab4: {
                    if !self.eq_s_b("t") {
                        break 'lab4;
                    }
                    self.core.bra = self.core.cursor;
                    if !self.r_r1() {
                        break 'lab4;
                    }
                    if !self.r_vx() {
                        break 'lab4;
                    }
                    self.slice_del();
                    break 'lab0;
                }
                self.restore(v_1);
                'lab5: {
                    if !self.eq_s_b("ink") {
                        break 'lab5;
                    }
                    self.core.bra = self.core.cursor;
                    self.slice_from_str("ing");
                    break 'lab0;
                }
                self.restore(v_1);
                'lab6: {
                    if !self.eq_s_b("mp") {
                        break 'lab6;
                    }
                    self.core.bra = self.core.cursor;
                    self.slice_from_str("m");
                    break 'lab0;
                }
                self.restore(v_1);
                'lab7: {
                    if !self.eq_s_b("'") {
                        break 'lab7;
                    }
                    self.core.bra = self.core.cursor;
                    if !self.r_r1() {
                        break 'lab7;
                    }
                    self.slice_del();
                    break 'lab0;
                }
                self.restore(v_1);
                self.core.bra = self.core.cursor;
                if !self.r_r1() {
                    return false;
                }
                if !self.r_c() {
                    return false;
                }
                self.slice_del();
            }
            2 => {
                if !self.r_r1() {
                    return false;
                }
                self.slice_from_str("g");
            }
            3 => {
                if !self.r_r1() {
                    return false;
                }
                self.slice_from_str("lijk");
            }
            4 => {
                if !self.r_r1() {
                    return false;
                }
                self.slice_from_str("isch");
            }
            5 => {
                if !self.r_r1() {
                    return false;
                }
                if !self.r_c() {
                    return false;
                }
                self.slice_del();
            }
            6 => {
                if !self.r_r1() {
                    return false;
                }
                self.slice_from_str("t");
            }
            7 => {
                if !self.r_r1() {
                    return false;
                }
                self.slice_from_str("s");
            }
            8 => {
                if !self.r_r1() {
                    return false;
                }
                self.slice_from_str("r");
            }
            9 => {
                if !self.r_r1() {
                    return false;
                }
                self.slice_del();
                let ins = ['l'];
                self.insert_at(self.core.cursor, self.core.cursor, &ins);
                self.r_lengthen_v();
            }
            10 => {
                if !self.r_r1() {
                    return false;
                }
                if !self.r_c() {
                    return false;
                }
                self.slice_del();
                let ins = ['e', 'n'];
                self.insert_at(self.core.cursor, self.core.cursor, &ins);
                self.r_lengthen_v();
            }
            11 => {
                if !self.r_r1() {
                    return false;
                }
                if !self.r_c() {
                    return false;
                }
                self.slice_from_str("ief");
            }
            _ => {}
        }
        true
    }

    fn r_step_3(&mut self) -> bool {
        self.core.ket = self.core.cursor;
        let among_var = self.find_among_b(A_3);
        if among_var == 0 {
            return false;
        }
        self.core.bra = self.core.cursor;
        match among_var {
            1 => {
                if !self.r_r1() {
                    return false;
                }
                self.slice_from_str("eer");
            }
            2 => {
                if !self.r_r1() {
                    return false;
                }
                self.slice_del();
                self.r_lengthen_v();
            }
            3 => {
                if !self.r_r1() {
                    return false;
                }
                self.slice_del();
            }
            4 => self.slice_from_str("r"),
            5 => 'lab0: {
                let v_1 = self.save();
                'lab1: {
                    if !self.eq_s_b("ild") {
                        break 'lab1;
                    }
                    self.slice_from_str("er");
                    break 'lab0;
                }
                self.restore(v_1);
                if !self.r_r1() {
                    return false;
                }
                self.slice_del();
                self.r_lengthen_v();
            }
            6 => {
                if !self.r_r1() {
                    return false;
                }
                if !self.r_c() {
                    return false;
                }
                self.slice_from_str("aar");
            }
            7 => {
                if !self.r_r2() {
                    return false;
                }
                self.slice_del();
                let ins = ['f'];
                self.insert_at(self.core.cursor, self.core.cursor, &ins);
                self.r_lengthen_v();
            }
            8 => {
                if !self.r_r2() {
                    return false;
                }
                self.slice_del();
                let ins = ['g'];
                self.insert_at(self.core.cursor, self.core.cursor, &ins);
                self.r_lengthen_v();
            }
            9 => {
                if !self.r_r1() {
                    return false;
                }
                if !self.r_c() {
                    return false;
                }
                self.slice_from_str("t");
            }
            10 => {
                if !self.r_r1() {
                    return false;
                }
                if !self.r_c() {
                    return false;
                }
                self.slice_from_str("d");
            }
            _ => {}
        }
        true
    }

    fn r_step_4(&mut self) -> bool {
        'lab0: {
            let v_1 = self.save();
            'lab1: {
                self.core.ket = self.core.cursor;
                let among_var = self.find_among_b(A_4);
                if among_var == 0 {
                    break 'lab1;
                }
                self.core.bra = self.core.cursor;
                match among_var {
                    1 => {
                        if !self.r_r1() {
                            break 'lab1;
                        }
                        self.slice_from_str("ie");
                    }
                    2 => {
                        if !self.r_r1() {
                            break 'lab1;
                        }
                        self.slice_from_str("eer");
                    }
                    3 => {
                        if !self.r_r1() {
                            break 'lab1;
                        }
                        self.slice_del();
                    }
                    4 => {
                        if !self.r_r1() {
                            break 'lab1;
                        }
                        if !self.r_v() {
                            break 'lab1;
                        }
                        self.slice_from_str("n");
                    }
                    5 => {
                        if !self.r_r1() {
                            break 'lab1;
                        }
                        if !self.r_v() {
                            break 'lab1;
                        }
                        self.slice_from_str("l");
                    }
                    6 => {
                        if !self.r_r1() {
                            break 'lab1;
                        }
                        if !self.r_v() {
                            break 'lab1;
                        }
                        self.slice_from_str("r");
                    }
                    7 => {
                        if !self.r_r1() {
                            break 'lab1;
                        }
                        self.slice_from_str("teer");
                    }
                    8 => {
                        if !self.r_r1() {
                            break 'lab1;
                        }
                        self.slice_from_str("lijk");
                    }
                    9 => {
                        if !self.r_r1() {
                            break 'lab1;
                        }
                        if !self.r_c() {
                            break 'lab1;
                        }
                        self.slice_del();
                        self.r_lengthen_v();
                    }
                    _ => {}
                }
                break 'lab0;
            }
            self.restore(v_1);
            self.core.ket = self.core.cursor;
            if self.find_among_b(A_5) == 0 {
                return false;
            }
            self.core.bra = self.core.cursor;
            if !self.r_r1() {
                return false;
            }
            let v_2 = self.save();
            'lab2: {
                if !self.eq_s_b("inn") {
                    break 'lab2;
                }
                if self.core.cursor > self.core.limit_backward {
                    break 'lab2;
                }
                return false;
            }
            self.restore(v_2);
            if !self.r_c() {
                return false;
            }
            self.slice_del();
            self.r_lengthen_v();
        }
        true
    }

    fn r_step_7(&mut self) -> bool {
        self.core.ket = self.core.cursor;
        let among_var = self.find_among_b(A_6);
        if among_var == 0 {
            return false;
        }
        self.core.bra = self.core.cursor;
        match among_var {
            1 => self.slice_from_str("k"),
            2 => self.slice_from_str("f"),
            3 => self.slice_from_str("p"),
            _ => {}
        }
        true
    }

    fn r_step_6(&mut self) -> bool {
        self.core.ket = self.core.cursor;
        let among_var = self.find_among_b(A_7);
        if among_var == 0 {
            return false;
        }
        self.core.bra = self.core.cursor;
        match among_var {
            1 => self.slice_from_str("b"),
            2 => self.slice_from_str("c"),
            3 => self.slice_from_str("d"),
            4 => self.slice_from_str("f"),
            5 => self.slice_from_str("g"),
            6 => self.slice_from_str("h"),
            7 => self.slice_from_str("j"),
            8 => self.slice_from_str("k"),
            9 => self.slice_from_str("l"),
            10 => self.slice_from_str("m"),
            11 => {
                let v_1 = self.save();
                'lab0: {
                    if !self.eq_s_b("i") {
                        break 'lab0;
                    }
                    if self.core.cursor > self.core.limit_backward {
                        break 'lab0;
                    }
                    return false;
                }
                self.restore(v_1);
                self.slice_from_str("n");
            }
            12 => self.slice_from_str("p"),
            13 => self.slice_from_str("q"),
            14 => self.slice_from_str("r"),
            15 => self.slice_from_str("s"),
            16 => self.slice_from_str("t"),
            17 => self.slice_from_str("v"),
            18 => self.slice_from_str("w"),
            19 => self.slice_from_str("x"),
            20 => self.slice_from_str("z"),
            _ => {}
        }
        true
    }

    /// Residual suffix step run after `ge-` removal: strip `d`/`t` after a
    /// consonant, with special handling for -nd/-cht clusters.
    fn r_step_1c(&mut self) -> bool {
        self.core.ket = self.core.cursor;
        let among_var = self.find_among_b(A_8);
        if among_var == 0 {
            return false;
        }
        self.core.bra = self.core.cursor;
        if !self.r_r1() {
            return false;
        }
        if !self.r_c() {
            return false;
        }
        match among_var {
            1 => {
                let v_1 = self.save();
                'lab0: {
                    if !self.eq_s_b("n") {
                        break 'lab0;
                    }
                    if !self.r_r1() {
                        break 'lab0;
                    }
                    return false;
                }
                self.restore(v_1);
                'lab1: {
                    let v_2 = self.save();
                    'lab2: {
                        if !self.eq_s_b("in") {
                            break 'lab2;
                        }
                        if self.core.cursor > self.core.limit_backward {
                            break 'lab2;
                        }
                        self.slice_from_str("n");
                        break 'lab1;
                    }
                    self.restore(v_2);
                    self.slice_del();
                }
            }
            2 => {
                let v_3 = self.save();
                'lab3: {
                    if !self.eq_s_b("h") {
                        break 'lab3;
                    }
                    if !self.r_r1() {
                        break 'lab3;
                    }
                    return false;
                }
                self.restore(v_3);
                let v_4 = self.save();
                'lab4: {
                    if !self.eq_s_b("en") {
                        break 'lab4;
                    }
                    if self.core.cursor > self.core.limit_backward {
                        break 'lab4;
                    }
                    return false;
                }
                self.restore(v_4);
                self.slice_del();
            }
            _ => {}
        }
        true
    }

    // ─── ge- prefix / infix removal ─────────────────────────────────────────

    fn r_lose_prefix(&mut self) -> bool {
        self.core.bra = self.core.cursor;
        if !self.eq_s("ge") {
            return false;
        }
        self.core.ket = self.core.cursor;
        let v_1 = self.core.cursor;
        if self.core.cursor + 3 > self.core.limit {
            return false;
        }
        self.core.cursor = v_1;
        let v_2 = self.core.cursor;
        // advance to the first vowel or "ij" after the prefix
        'golab0: loop {
            let v_3 = self.core.cursor;
            'lab1: {
                'lab2: {
                    let v_4 = self.core.cursor;
                    'lab3: {
                        if !self.eq_s("ij") {
                            break 'lab3;
                        }
                        break 'lab2;
                    }
                    self.core.cursor = v_4;
                    if !self.in_grouping(&G_V) {
                        break 'lab1;
                    }
                }
                break 'golab0;
            }
            self.core.cursor = v_3;
            if self.core.cursor >= self.core.limit {
                return false;
            }
            self.core.cursor += 1;
        }
        // consume the rest of the vowel run
        'run0: loop {
            let v_5 = self.core.cursor;
            'lab4: {
                'lab5: {
                    let v_6 = self.core.cursor;
                    'lab6: {
                        if !self.eq_s("ij") {
                            break 'lab6;
                        }
                        break 'lab5;
                    }
                    self.core.cursor = v_6;
                    if !self.in_grouping(&G_V) {
                        break 'lab4;
                    }
                }
                continue 'run0;
            }
            self.core.cursor = v_5;
            break 'run0;
        }
        if self.core.cursor >= self.core.limit {
            return false;
        }
        self.core.cursor = v_2;
        if self.find_among(A_9) == 1 {
            return false;
        }
        self.ge_removed = true;
        self.slice_del();
        let v_7 = self.core.cursor;
        'lab8: {
            self.core.bra = self.core.cursor;
            let among_var = self.find_among(A_10);
            if among_var == 0 {
                break 'lab8;
            }
            self.core.ket = self.core.cursor;
            match among_var {
                1 => self.slice_from_str("e"),
                2 => self.slice_from_str("i"),
                _ => {}
            }
        }
        self.core.cursor = v_7;
        true
    }

    fn r_lose_infix(&mut self) -> bool {
        if self.core.cursor >= self.core.limit {
            return false;
        }
        self.core.cursor += 1;
        // find the first "ge" after the opening character
        'golab0: loop {
            self.core.bra = self.core.cursor;
            if self.eq_s("ge") {
                self.core.ket = self.core.cursor;
                break 'golab0;
            }
            if self.core.cursor >= self.core.limit {
                return false;
            }
            self.core.cursor += 1;
        }
        let v_1 = self.core.cursor;
        if self.core.cursor + 3 > self.core.limit {
            return false;
        }
        self.core.cursor = v_1;
        let v_2 = self.core.cursor;
        'golab2: loop {
            let v_3 = self.core.cursor;
            'lab3: {
                'lab4: {
                    let v_4 = self.core.cursor;
                    'lab5: {
                        if !self.eq_s("ij") {
                            break 'lab5;
                        }
                        break 'lab4;
                    }
                    self.core.cursor = v_4;
                    if !self.in_grouping(&G_V) {
                        break 'lab3;
                    }
                }
                break 'golab2;
            }
            self.core.cursor = v_3;
            if self.core.cursor >= self.core.limit {
                return false;
            }
            self.core.cursor += 1;
        }
        'run1: loop {
            let v_5 = self.core.cursor;
            'lab6: {
                'lab7: {
                    let v_6 = self.core.cursor;
                    'lab8: {
                        if !self.eq_s("ij") {
                            break 'lab8;
                        }
                        break 'lab7;
                    }
                    self.core.cursor = v_6;
                    if !self.in_grouping(&G_V) {
                        break 'lab6;
                    }
                }
                continue 'run1;
            }
            self.core.cursor = v_5;
            break 'run1;
        }
        if self.core.cursor >= self.core.limit {
            return false;
        }
        self.core.cursor = v_2;
        self.ge_removed = true;
        self.slice_del();
        let v_7 = self.core.cursor;
        'lab10: {
            self.core.bra = self.core.cursor;
            let among_var = self.find_among(A_11);
            if among_var == 0 {
                break 'lab10;
            }
            self.core.ket = self.core.cursor;
            match among_var {
                1 => self.slice_from_str("e"),
                2 => self.slice_from_str("i"),
                _ => {}
            }
        }
        self.core.cursor = v_7;
        true
    }

    // ─── R1/R2 measure ──────────────────────────────────────────────────────

    fn r_measure(&mut self) -> bool {
        self.p1 = self.core.limit;
        self.p2 = self.core.limit;
        let v_1 = self.core.cursor;
        'lab0: {
            // go past any non-vowels
            'skip1: loop {
                'lab1: {
                    if !self.out_grouping(&G_V) {
                        break 'lab1;
                    }
                    continue 'skip1;
                }
                break 'skip1;
            }
            // one vowel run (vowels or "ij")
            let mut v_2: i32 = 1;
            'run1: loop {
                let v_3 = self.core.cursor;
                'lab2: {
                    'lab3: {
                        let v_4 = self.core.cursor;
                        'lab4: {
                            if !self.eq_s("ij") {
                                break 'lab4;
                            }
                            break 'lab3;
                        }
                        self.core.cursor = v_4;
                        if !self.in_grouping(&G_V) {
                            break 'lab2;
                        }
                    }
                    v_2 = v_2.wrapping_sub(1);
                    continue 'run1;
                }
                self.core.cursor = v_3;
                break 'run1;
            }
            if v_2 > 0 {
                break 'lab0;
            }
            if !self.out_grouping(&G_V) {
                break 'lab0;
            }
            self.p1 = self.core.cursor;
            // and again for p2
            'skip2: loop {
                'lab5: {
                    if !self.out_grouping(&G_V) {
                        break 'lab5;
                    }
                    continue 'skip2;
                }
                break 'skip2;
            }
            let mut v_5: i32 = 1;
            'run2: loop {
                let v_6 = self.core.cursor;
                'lab6: {
                    'lab7: {
                        let v_7 = self.core.cursor;
                        'lab8: {
                            if !self.eq_s("ij") {
                                break 'lab8;
                            }
                            break 'lab7;
                        }
                        self.core.cursor = v_7;
                        if !self.in_grouping(&G_V) {
                            break 'lab6;
                        }
                    }
                    v_5 = v_5.wrapping_sub(1);
                    continue 'run2;
                }
                self.core.cursor = v_6;
                break 'run2;
            }
            if v_5 > 0 {
                break 'lab0;
            }
            if !self.out_grouping(&G_V) {
                break 'lab0;
            }
            self.p2 = self.core.cursor;
        }
        self.core.cursor = v_1;
        true
    }

    // ─── driver ─────────────────────────────────────────────────────────────

    fn stem(&mut self) -> bool {
        let mut stemmed = false;
        self.r_measure();
        self.core.limit_backward = self.core.cursor;
        self.core.cursor = self.core.limit;

        let v_1 = self.save();
        if self.r_step_1() {
            stemmed = true;
        }
        self.restore(v_1);
        let v_2 = self.save();
        if self.r_step_2() {
            stemmed = true;
        }
        self.restore(v_2);
        let v_3 = self.save();
        if self.r_step_3() {
            stemmed = true;
        }
        self.restore(v_3);
        let v_4 = self.save();
        if self.r_step_4() {
            stemmed = true;
        }
        self.restore(v_4);

        self.core.cursor = self.core.limit_backward;
        self.ge_removed = false;
        let v_5 = self.core.cursor;
        {
            let v_6 = self.core.cursor;
            if self.r_lose_prefix() {
                self.core.cursor = v_6;
                self.r_measure();
            }
        }
        self.core.cursor = v_5;
        self.core.limit_backward = self.core.cursor;
        self.core.cursor = self.core.limit;
        let v_7 = self.save();
        if self.ge_removed {
            stemmed = true;
            self.r_step_1c();
        }
        self.restore(v_7);

        self.core.cursor = self.core.limit_backward;
        self.ge_removed = false;
        let v_8 = self.core.cursor;
        {
            let v_9 = self.core.cursor;
            if self.r_lose_infix() {
                self.core.cursor = v_9;
                self.r_measure();
            }
        }
        self.core.cursor = v_8;
        self.core.limit_backward = self.core.cursor;
        self.core.cursor = self.core.limit;
        let v_10 = self.save();
        if self.ge_removed {
            stemmed = true;
            self.r_step_1c();
        }
        self.restore(v_10);
        self.core.cursor = self.core.limit_backward;
        self.core.cursor = self.core.limit;

        let v_11 = self.save();
        if self.r_step_7() {
            stemmed = true;
        }
        self.restore(v_11);
        let v_12 = self.save();
        if stemmed {
            self.r_step_6();
        }
        self.restore(v_12);
        self.core.cursor = self.core.limit_backward;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pizza_engine::analysis::TokenFilter;

    fn stem(word: &str) -> String {
        let mut s = DutchSnowball::new(&word.to_lowercase());
        s.stem();
        s.result()
    }

    #[test]
    fn test_lengthening_and_undoubling() {
        assert_eq!(stem("lichamen"), "lichaam");
        assert_eq!(stem("opgraven"), "opgraaf");
        assert_eq!(stem("ophalen"), "ophaal");
        assert_eq!(stem("huizen"), "huis");
        assert_eq!(stem("opheffen"), "ophef");
    }

    #[test]
    fn test_licht_family() {
        assert_eq!(stem("lichte"), "licht");
        assert_eq!(stem("lichtten"), "licht");
        assert_eq!(stem("lichtste"), "licht");
        assert_eq!(stem("lichtje"), "licht");
    }

    #[test]
    fn test_filter_wrapper() {
        let filter = DutchStemTokenFilter::new();
        let mut token = Token::new("huizen", 0, 6, 0);
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "huis");

        let mut token = Token::new("", 0, 0, 0);
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "");
    }
}
