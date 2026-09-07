use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

use super::snowball::Among;
use super::snowball::Grouping;
use super::snowball::SnowballCore;

/// Catalan stemmer — faithful port of the Snowball `catalan.sbl` algorithm
/// as generated for Lucene (`org.tartarus.snowball.ext.CatalanStemmer`).
///
/// Pipeline: mark R1/R2, strip attached pronouns, standard or verb suffixes
/// (whichever fires first), residual suffixes, then a cleaning pass that
/// folds the accented vowels and the middle dot. Rule tables are generated
/// from the reference; validated by differential testing against the JDK
/// reference implementation.
#[derive(Clone, Debug, Default)]
pub struct CatalanStemTokenFilter;

impl CatalanStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for CatalanStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        // Lucene's CatalanAnalyzer lowercases ahead of SnowballFilter.
        let stemmed = stem_catalan(&text.to_lowercase());
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

pub(crate) fn stem_catalan(word: &str) -> String {
    let mut s = CatalanSnowball {
        core: SnowballCore::new(word),
        p1: 0,
        p2: 0,
    };
    s.stem();
    s.core.result()
}

type SnowballAmong = Among;

struct CatalanSnowball {
    core: SnowballCore,
    p1: usize,
    p2: usize,
}

impl CatalanSnowball {
    fn r_mark_regions(&mut self) -> bool {
        self.p1 = self.core.limit;
        self.p2 = self.core.limit;
        let v_1 = self.core.cursor;
        'lab0: {
            if !self.core.go_out_grouping(&G_V) {
                break 'lab0;
            }
            self.core.cursor += 1;
            if !self.core.go_in_grouping(&G_V) {
                break 'lab0;
            }
            self.core.cursor += 1;
            self.p1 = self.core.cursor;
            if !self.core.go_out_grouping(&G_V) {
                break 'lab0;
            }
            self.core.cursor += 1;
            if !self.core.go_in_grouping(&G_V) {
                break 'lab0;
            }
            self.core.cursor += 1;
            self.p2 = self.core.cursor;
        }
        self.core.cursor = v_1;
        true
    }

    fn r_cleaning(&mut self) -> bool {
        'outer: loop {
            let v_1 = self.core.cursor;
            'lab0: {
                self.core.bra = self.core.cursor;
                let among_var = self.core.find_among(A_0);
                self.core.ket = self.core.cursor;
                match among_var {
                    1 => self.core.slice_from_str("a"),
                    2 => self.core.slice_from_str("e"),
                    3 => self.core.slice_from_str("i"),
                    4 => self.core.slice_from_str("o"),
                    5 => self.core.slice_from_str("u"),
                    6 => self.core.slice_from_str("."),
                    7 => {
                        if self.core.cursor >= self.core.limit {
                            break 'lab0;
                        }
                        self.core.cursor += 1;
                    }
                    _ => {}
                }
                continue 'outer;
            }
            self.core.cursor = v_1;
            break 'outer;
        }
        true
    }

    fn r_r1(&self) -> bool {
        self.p1 <= self.core.cursor
    }

    fn r_r2(&self) -> bool {
        self.p2 <= self.core.cursor
    }

    fn r_attached_pronoun(&mut self) -> bool {
        self.core.ket = self.core.cursor;
        if self.core.find_among_b(A_1) == 0 {
            return false;
        }
        self.core.bra = self.core.cursor;
        if !self.r_r1() {
            return false;
        }
        self.core.slice_del();
        true
    }

    fn r_standard_suffix(&mut self) -> bool {
        self.core.ket = self.core.cursor;
        let among_var = self.core.find_among_b(A_2);
        if among_var == 0 {
            return false;
        }
        self.core.bra = self.core.cursor;
        match among_var {
            1 => {
                if !self.r_r1() {
                    return false;
                }
                self.core.slice_del();
            }
            2 => {
                if !self.r_r2() {
                    return false;
                }
                self.core.slice_del();
            }
            3 => {
                if !self.r_r2() {
                    return false;
                }
                self.core.slice_from_str("log");
            }
            4 => {
                if !self.r_r2() {
                    return false;
                }
                self.core.slice_from_str("ic");
            }
            5 => {
                if !self.r_r1() {
                    return false;
                }
                self.core.slice_from_str("c");
            }
            _ => {}
        }
        true
    }

    fn r_verb_suffix(&mut self) -> bool {
        self.core.ket = self.core.cursor;
        let among_var = self.core.find_among_b(A_3);
        if among_var == 0 {
            return false;
        }
        self.core.bra = self.core.cursor;
        match among_var {
            1 => {
                if !self.r_r1() {
                    return false;
                }
                self.core.slice_del();
            }
            2 => {
                if !self.r_r2() {
                    return false;
                }
                self.core.slice_del();
            }
            _ => {}
        }
        true
    }

    fn r_residual_suffix(&mut self) -> bool {
        self.core.ket = self.core.cursor;
        let among_var = self.core.find_among_b(A_4);
        if among_var == 0 {
            return false;
        }
        self.core.bra = self.core.cursor;
        match among_var {
            1 => {
                if !self.r_r1() {
                    return false;
                }
                self.core.slice_del();
            }
            2 => {
                if !self.r_r1() {
                    return false;
                }
                self.core.slice_from_str("ic");
            }
            _ => {}
        }
        true
    }

    fn stem(&mut self) -> bool {
        self.r_mark_regions();
        self.core.limit_backward = self.core.cursor;
        self.core.cursor = self.core.limit;

        let v_1 = self.core.save();
        self.r_attached_pronoun();
        self.core.restore(v_1);

        let v_2 = self.core.save();
        'lab0: {
            'lab1: {
                let v_3 = self.core.save();
                'lab2: {
                    if !self.r_standard_suffix() {
                        break 'lab2;
                    }
                    break 'lab1;
                }
                self.core.restore(v_3);
                if !self.r_verb_suffix() {
                    break 'lab0;
                }
            }
        }
        self.core.restore(v_2);

        let v_4 = self.core.save();
        self.r_residual_suffix();
        self.core.restore(v_4);

        self.core.cursor = self.core.limit_backward;
        let v_5 = self.core.cursor;
        self.r_cleaning();
        self.core.cursor = v_5;
        true
    }
}

const A_0: SnowballAmong = &[
    ("", 7),
    ("\u{00B7}", 6),
    ("\u{00E0}", 1),
    ("\u{00E1}", 1),
    ("\u{00E8}", 2),
    ("\u{00E9}", 2),
    ("\u{00EC}", 3),
    ("\u{00ED}", 3),
    ("\u{00EF}", 3),
    ("\u{00F2}", 4),
    ("\u{00F3}", 4),
    ("\u{00FA}", 5),
    ("\u{00FC}", 5),
];
const A_1: SnowballAmong = &[
    ("la", 1),
    ("-la", 1),
    ("sela", 1),
    ("le", 1),
    ("me", 1),
    ("-me", 1),
    ("se", 1),
    ("-te", 1),
    ("hi", 1),
    ("'hi", 1),
    ("li", 1),
    ("-li", 1),
    ("'l", 1),
    ("'m", 1),
    ("-m", 1),
    ("'n", 1),
    ("-n", 1),
    ("ho", 1),
    ("'ho", 1),
    ("lo", 1),
    ("selo", 1),
    ("'s", 1),
    ("las", 1),
    ("selas", 1),
    ("les", 1),
    ("-les", 1),
    ("'ls", 1),
    ("-ls", 1),
    ("'ns", 1),
    ("-ns", 1),
    ("ens", 1),
    ("los", 1),
    ("selos", 1),
    ("nos", 1),
    ("-nos", 1),
    ("vos", 1),
    ("us", 1),
    ("-us", 1),
    ("'t", 1),
];
const A_2: SnowballAmong = &[
    ("ica", 4),
    ("l\u{00F3}gica", 3),
    ("enca", 1),
    ("ada", 2),
    ("ancia", 1),
    ("encia", 1),
    ("\u{00E8}ncia", 1),
    ("\u{00ED}cia", 1),
    ("logia", 3),
    ("inia", 1),
    ("\u{00ED}inia", 1),
    ("eria", 1),
    ("\u{00E0}ria", 1),
    ("at\u{00F2}ria", 1),
    ("alla", 1),
    ("ella", 1),
    ("\u{00ED}vola", 1),
    ("ima", 1),
    ("\u{00ED}ssima", 1),
    ("qu\u{00ED}ssima", 5),
    ("ana", 1),
    ("ina", 1),
    ("era", 1),
    ("sfera", 1),
    ("ora", 1),
    ("dora", 1),
    ("adora", 1),
    ("adura", 1),
    ("esa", 1),
    ("osa", 1),
    ("assa", 1),
    ("essa", 1),
    ("issa", 1),
    ("eta", 1),
    ("ita", 1),
    ("ota", 1),
    ("ista", 1),
    ("ialista", 1),
    ("ionista", 1),
    ("iva", 1),
    ("ativa", 1),
    ("n\u{00E7}a", 1),
    ("log\u{00ED}a", 3),
    ("ic", 4),
    ("\u{00ED}stic", 1),
    ("enc", 1),
    ("esc", 1),
    ("ud", 1),
    ("atge", 1),
    ("ble", 1),
    ("able", 1),
    ("ible", 1),
    ("isme", 1),
    ("ialisme", 1),
    ("ionisme", 1),
    ("ivisme", 1),
    ("aire", 1),
    ("icte", 1),
    ("iste", 1),
    ("ici", 1),
    ("\u{00ED}ci", 1),
    ("logi", 3),
    ("ari", 1),
    ("tori", 1),
    ("al", 1),
    ("il", 1),
    ("all", 1),
    ("ell", 1),
    ("\u{00ED}vol", 1),
    ("isam", 1),
    ("issem", 1),
    ("\u{00EC}ssem", 1),
    ("\u{00ED}ssem", 1),
    ("\u{00ED}ssim", 1),
    ("qu\u{00ED}ssim", 5),
    ("amen", 1),
    ("\u{00EC}ssin", 1),
    ("ar", 1),
    ("ificar", 1),
    ("egar", 1),
    ("ejar", 1),
    ("itar", 1),
    ("itzar", 1),
    ("fer", 1),
    ("or", 1),
    ("dor", 1),
    ("dur", 1),
    ("doras", 1),
    ("ics", 4),
    ("l\u{00F3}gics", 3),
    ("uds", 1),
    ("nces", 1),
    ("ades", 2),
    ("ancies", 1),
    ("encies", 1),
    ("\u{00E8}ncies", 1),
    ("\u{00ED}cies", 1),
    ("logies", 3),
    ("inies", 1),
    ("\u{00ED}nies", 1),
    ("eries", 1),
    ("\u{00E0}ries", 1),
    ("at\u{00F2}ries", 1),
    ("bles", 1),
    ("ables", 1),
    ("ibles", 1),
    ("imes", 1),
    ("\u{00ED}ssimes", 1),
    ("qu\u{00ED}ssimes", 5),
    ("formes", 1),
    ("ismes", 1),
    ("ialismes", 1),
    ("ines", 1),
    ("eres", 1),
    ("ores", 1),
    ("dores", 1),
    ("idores", 1),
    ("dures", 1),
    ("eses", 1),
    ("oses", 1),
    ("asses", 1),
    ("ictes", 1),
    ("ites", 1),
    ("otes", 1),
    ("istes", 1),
    ("ialistes", 1),
    ("ionistes", 1),
    ("iques", 4),
    ("l\u{00F3}giques", 3),
    ("ives", 1),
    ("atives", 1),
    ("log\u{00ED}es", 3),
    ("alleng\u{00FC}es", 1),
    ("icis", 1),
    ("\u{00ED}cis", 1),
    ("logis", 3),
    ("aris", 1),
    ("toris", 1),
    ("ls", 1),
    ("als", 1),
    ("ells", 1),
    ("ims", 1),
    ("\u{00ED}ssims", 1),
    ("qu\u{00ED}ssims", 5),
    ("ions", 1),
    ("cions", 1),
    ("acions", 2),
    ("esos", 1),
    ("osos", 1),
    ("assos", 1),
    ("issos", 1),
    ("ers", 1),
    ("ors", 1),
    ("dors", 1),
    ("adors", 1),
    ("idors", 1),
    ("ats", 1),
    ("itats", 1),
    ("bilitats", 1),
    ("ivitats", 1),
    ("ativitats", 1),
    ("\u{00EF}tats", 1),
    ("ets", 1),
    ("ants", 1),
    ("ents", 1),
    ("ments", 1),
    ("aments", 1),
    ("ots", 1),
    ("uts", 1),
    ("ius", 1),
    ("trius", 1),
    ("atius", 1),
    ("\u{00E8}s", 1),
    ("\u{00E9}s", 1),
    ("\u{00ED}s", 1),
    ("d\u{00ED}s", 1),
    ("\u{00F3}s", 1),
    ("itat", 1),
    ("bilitat", 1),
    ("ivitat", 1),
    ("ativitat", 1),
    ("\u{00EF}tat", 1),
    ("et", 1),
    ("ant", 1),
    ("ent", 1),
    ("ient", 1),
    ("ment", 1),
    ("ament", 1),
    ("isament", 1),
    ("ot", 1),
    ("isseu", 1),
    ("\u{00EC}sseu", 1),
    ("\u{00ED}sseu", 1),
    ("triu", 1),
    ("\u{00ED}ssiu", 1),
    ("atiu", 1),
    ("\u{00F3}", 1),
    ("i\u{00F3}", 1),
    ("ci\u{00F3}", 1),
    ("aci\u{00F3}", 1),
];
const A_3: SnowballAmong = &[
    ("aba", 1),
    ("esca", 1),
    ("isca", 1),
    ("\u{00EF}sca", 1),
    ("ada", 1),
    ("ida", 1),
    ("uda", 1),
    ("\u{00EF}da", 1),
    ("ia", 1),
    ("aria", 1),
    ("iria", 1),
    ("ara", 1),
    ("iera", 1),
    ("ira", 1),
    ("adora", 1),
    ("\u{00EF}ra", 1),
    ("ava", 1),
    ("ixa", 1),
    ("itza", 1),
    ("\u{00ED}a", 1),
    ("ar\u{00ED}a", 1),
    ("er\u{00ED}a", 1),
    ("ir\u{00ED}a", 1),
    ("\u{00EF}a", 1),
    ("isc", 1),
    ("\u{00EF}sc", 1),
    ("ad", 1),
    ("ed", 1),
    ("id", 1),
    ("ie", 1),
    ("re", 1),
    ("dre", 1),
    ("ase", 1),
    ("iese", 1),
    ("aste", 1),
    ("iste", 1),
    ("ii", 1),
    ("ini", 1),
    ("esqui", 1),
    ("eixi", 1),
    ("itzi", 1),
    ("am", 1),
    ("em", 1),
    ("arem", 1),
    ("irem", 1),
    ("\u{00E0}rem", 1),
    ("\u{00ED}rem", 1),
    ("\u{00E0}ssem", 1),
    ("\u{00E9}ssem", 1),
    ("iguem", 1),
    ("\u{00EF}guem", 1),
    ("avem", 1),
    ("\u{00E0}vem", 1),
    ("\u{00E1}vem", 1),
    ("ir\u{00EC}em", 1),
    ("\u{00ED}em", 1),
    ("ar\u{00ED}em", 1),
    ("ir\u{00ED}em", 1),
    ("assim", 1),
    ("essim", 1),
    ("issim", 1),
    ("\u{00E0}ssim", 1),
    ("\u{00E8}ssim", 1),
    ("\u{00E9}ssim", 1),
    ("\u{00ED}ssim", 1),
    ("\u{00EF}m", 1),
    ("an", 1),
    ("aban", 1),
    ("arian", 1),
    ("aran", 1),
    ("ieran", 1),
    ("iran", 1),
    ("\u{00ED}an", 1),
    ("ar\u{00ED}an", 1),
    ("er\u{00ED}an", 1),
    ("ir\u{00ED}an", 1),
    ("en", 1),
    ("ien", 1),
    ("arien", 1),
    ("irien", 1),
    ("aren", 1),
    ("eren", 1),
    ("iren", 1),
    ("\u{00E0}ren", 1),
    ("\u{00EF}ren", 1),
    ("asen", 1),
    ("iesen", 1),
    ("assen", 1),
    ("essen", 1),
    ("issen", 1),
    ("\u{00E9}ssen", 1),
    ("\u{00EF}ssen", 1),
    ("esquen", 1),
    ("isquen", 1),
    ("\u{00EF}squen", 1),
    ("aven", 1),
    ("ixen", 1),
    ("eixen", 1),
    ("\u{00EF}xen", 1),
    ("\u{00EF}en", 1),
    ("in", 1),
    ("inin", 1),
    ("sin", 1),
    ("isin", 1),
    ("assin", 1),
    ("essin", 1),
    ("issin", 1),
    ("\u{00EF}ssin", 1),
    ("esquin", 1),
    ("eixin", 1),
    ("aron", 1),
    ("ieron", 1),
    ("ar\u{00E1}n", 1),
    ("er\u{00E1}n", 1),
    ("ir\u{00E1}n", 1),
    ("i\u{00EF}n", 1),
    ("ado", 1),
    ("ido", 1),
    ("ando", 2),
    ("iendo", 1),
    ("io", 1),
    ("ixo", 1),
    ("eixo", 1),
    ("\u{00EF}xo", 1),
    ("itzo", 1),
    ("ar", 1),
    ("tzar", 1),
    ("er", 1),
    ("eixer", 1),
    ("ir", 1),
    ("ador", 1),
    ("as", 1),
    ("abas", 1),
    ("adas", 1),
    ("idas", 1),
    ("aras", 1),
    ("ieras", 1),
    ("\u{00ED}as", 1),
    ("ar\u{00ED}as", 1),
    ("er\u{00ED}as", 1),
    ("ir\u{00ED}as", 1),
    ("ids", 1),
    ("es", 1),
    ("ades", 1),
    ("ides", 1),
    ("udes", 1),
    ("\u{00EF}des", 1),
    ("atges", 1),
    ("ies", 1),
    ("aries", 1),
    ("iries", 1),
    ("ares", 1),
    ("ires", 1),
    ("adores", 1),
    ("\u{00EF}res", 1),
    ("ases", 1),
    ("ieses", 1),
    ("asses", 1),
    ("esses", 1),
    ("isses", 1),
    ("\u{00EF}sses", 1),
    ("ques", 1),
    ("esques", 1),
    ("\u{00EF}sques", 1),
    ("aves", 1),
    ("ixes", 1),
    ("eixes", 1),
    ("\u{00EF}xes", 1),
    ("\u{00EF}es", 1),
    ("abais", 1),
    ("arais", 1),
    ("ierais", 1),
    ("\u{00ED}ais", 1),
    ("ar\u{00ED}ais", 1),
    ("er\u{00ED}ais", 1),
    ("ir\u{00ED}ais", 1),
    ("aseis", 1),
    ("ieseis", 1),
    ("asteis", 1),
    ("isteis", 1),
    ("inis", 1),
    ("sis", 1),
    ("isis", 1),
    ("assis", 1),
    ("essis", 1),
    ("issis", 1),
    ("\u{00EF}ssis", 1),
    ("esquis", 1),
    ("eixis", 1),
    ("itzis", 1),
    ("\u{00E1}is", 1),
    ("ar\u{00E9}is", 1),
    ("er\u{00E9}is", 1),
    ("ir\u{00E9}is", 1),
    ("ams", 1),
    ("ados", 1),
    ("idos", 1),
    ("amos", 1),
    ("\u{00E1}bamos", 1),
    ("\u{00E1}ramos", 1),
    ("i\u{00E9}ramos", 1),
    ("\u{00ED}amos", 1),
    ("ar\u{00ED}amos", 1),
    ("er\u{00ED}amos", 1),
    ("ir\u{00ED}amos", 1),
    ("aremos", 1),
    ("eremos", 1),
    ("iremos", 1),
    ("\u{00E1}semos", 1),
    ("i\u{00E9}semos", 1),
    ("imos", 1),
    ("adors", 1),
    ("ass", 1),
    ("erass", 1),
    ("ess", 1),
    ("ats", 1),
    ("its", 1),
    ("ents", 1),
    ("\u{00E0}s", 1),
    ("ar\u{00E0}s", 1),
    ("ir\u{00E0}s", 1),
    ("ar\u{00E1}s", 1),
    ("er\u{00E1}s", 1),
    ("ir\u{00E1}s", 1),
    ("\u{00E9}s", 1),
    ("ar\u{00E9}s", 1),
    ("\u{00ED}s", 1),
    ("i\u{00EF}s", 1),
    ("at", 1),
    ("it", 1),
    ("ant", 1),
    ("ent", 1),
    ("int", 1),
    ("ut", 1),
    ("\u{00EF}t", 1),
    ("au", 1),
    ("erau", 1),
    ("ieu", 1),
    ("ineu", 1),
    ("areu", 1),
    ("ireu", 1),
    ("\u{00E0}reu", 1),
    ("\u{00ED}reu", 1),
    ("asseu", 1),
    ("esseu", 1),
    ("eresseu", 1),
    ("\u{00E0}sseu", 1),
    ("\u{00E9}sseu", 1),
    ("igueu", 1),
    ("\u{00EF}gueu", 1),
    ("\u{00E0}veu", 1),
    ("\u{00E1}veu", 1),
    ("itzeu", 1),
    ("\u{00EC}eu", 1),
    ("ir\u{00EC}eu", 1),
    ("\u{00ED}eu", 1),
    ("ar\u{00ED}eu", 1),
    ("ir\u{00ED}eu", 1),
    ("assiu", 1),
    ("issiu", 1),
    ("\u{00E0}ssiu", 1),
    ("\u{00E8}ssiu", 1),
    ("\u{00E9}ssiu", 1),
    ("\u{00ED}ssiu", 1),
    ("\u{00EF}u", 1),
    ("ix", 1),
    ("eix", 1),
    ("\u{00EF}x", 1),
    ("itz", 1),
    ("i\u{00E0}", 1),
    ("ar\u{00E0}", 1),
    ("ir\u{00E0}", 1),
    ("itz\u{00E0}", 1),
    ("ar\u{00E1}", 1),
    ("er\u{00E1}", 1),
    ("ir\u{00E1}", 1),
    ("ir\u{00E8}", 1),
    ("ar\u{00E9}", 1),
    ("er\u{00E9}", 1),
    ("ir\u{00E9}", 1),
    ("\u{00ED}", 1),
    ("i\u{00EF}", 1),
    ("i\u{00F3}", 1),
];
const A_4: SnowballAmong = &[
    ("a", 1),
    ("e", 1),
    ("i", 1),
    ("\u{00EF}n", 1),
    ("o", 1),
    ("ir", 1),
    ("s", 1),
    ("is", 1),
    ("os", 1),
    ("\u{00EF}s", 1),
    ("it", 1),
    ("eu", 1),
    ("iu", 1),
    ("iqu", 2),
    ("itz", 1),
    ("\u{00E0}", 1),
    ("\u{00E1}", 1),
    ("\u{00E9}", 1),
    ("\u{00EC}", 1),
    ("\u{00ED}", 1),
    ("\u{00EF}", 1),
    ("\u{00F3}", 1),
];
const G_V_BITS: [u8; 20] = [
    17, 65, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 129, 81, 6, 10,
];
const G_V: Grouping = Grouping {
    bits: &G_V_BITS,
    min: 97,
    max: 252,
};

#[cfg(test)]
mod tests {
    use super::*;
    use pizza_engine::analysis::TokenFilter;

    // Vectors from Lucene's TestCatalanAnalyzer.
    #[test]
    fn test_lucene_vectors() {
        let f = CatalanStemTokenFilter::new();
        let mut token = Token::new("llengües", 0, 8, 0);
        f.filter(&mut token);
        assert_eq!(token.term.as_ref(), "llengu");
        let mut token = Token::new("llengua", 0, 7, 0);
        f.filter(&mut token);
        assert_eq!(token.term.as_ref(), "llengu");
    }

    // Differential vectors generated by running the JDK reference
    // implementation (Lucene CatalanStemmer) over the Lucene Catalan
    // stopword list plus analyzer test words.
    #[test]
    fn test_differential_vocabulary() {
        let data = include_str!("data/catalan_stem.txt");
        let mut checked = 0;
        for line in data.lines() {
            let line = line.trim_end_matches('\r');
            if line.is_empty() {
                continue;
            }
            let Some((input, expected)) = line.split_once('\t') else {
                continue;
            };
            assert_eq!(stem_catalan(input), expected, "vector {input}");
            checked += 1;
        }
        assert!(checked > 100, "expected a real vocabulary, got {checked}");
    }
}
