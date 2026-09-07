use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

use super::snowball::Among;
use super::snowball::Grouping;
use super::snowball::SnowballCore;

/// Basque stemmer — faithful port of the Snowball `basque.sbl` (eu) algorithm
/// as generated for Lucene (`org.tartarus.snowball.ext.BasqueStemmer`).
///
/// Pipeline: mark RV/R1/R2, then repeatedly strip verb suffixes (aditzak)
/// and noun/declension suffixes (izenak) until neither fires, then adjective
/// suffixes (adjetiboak). Rule tables are generated from the reference;
/// validated by differential testing against the JDK reference implementation.
#[derive(Clone, Debug, Default)]
pub struct BasqueStemTokenFilter;

impl BasqueStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for BasqueStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        // Lucene's BasqueAnalyzer lowercases ahead of SnowballFilter.
        let stemmed = stem_basque(&text.to_lowercase());
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

pub(crate) fn stem_basque(word: &str) -> String {
    let mut s = BasqueSnowball {
        core: SnowballCore::new(word),
        pv: 0,
        p1: 0,
        p2: 0,
    };
    s.stem();
    s.core.result()
}

type SnowballAmong = Among;

struct BasqueSnowball {
    core: SnowballCore,
    pv: usize,
    p1: usize,
    p2: usize,
}

impl BasqueSnowball {
    fn r_mark_regions(&mut self) -> bool {
        self.pv = self.core.limit;
        self.p1 = self.core.limit;
        self.p2 = self.core.limit;
        let v_1 = self.core.cursor;
        'lab0: {
            'lab1: {
                let v_2 = self.core.cursor;
                'lab2: {
                    if !self.core.in_grouping(&G_V) {
                        break 'lab2;
                    }
                    'lab3: {
                        let v_3 = self.core.cursor;
                        'lab4: {
                            if !self.core.out_grouping(&G_V) {
                                break 'lab4;
                            }
                            if !self.core.go_out_grouping(&G_V) {
                                break 'lab4;
                            }
                            self.core.cursor += 1;
                            break 'lab3;
                        }
                        self.core.cursor = v_3;
                        if !self.core.in_grouping(&G_V) {
                            break 'lab2;
                        }
                        if !self.core.go_in_grouping(&G_V) {
                            break 'lab2;
                        }
                        self.core.cursor += 1;
                    }
                    break 'lab1;
                }
                self.core.cursor = v_2;
                if !self.core.out_grouping(&G_V) {
                    break 'lab0;
                }
                'lab5: {
                    let v_4 = self.core.cursor;
                    'lab6: {
                        if !self.core.out_grouping(&G_V) {
                            break 'lab6;
                        }
                        if !self.core.go_out_grouping(&G_V) {
                            break 'lab6;
                        }
                        self.core.cursor += 1;
                        break 'lab5;
                    }
                    self.core.cursor = v_4;
                    if !self.core.in_grouping(&G_V) {
                        break 'lab0;
                    }
                    if self.core.cursor >= self.core.limit {
                        break 'lab0;
                    }
                    self.core.cursor += 1;
                }
            }
            self.pv = self.core.cursor;
        }
        self.core.cursor = v_1;
        let v_5 = self.core.cursor;
        'lab7: {
            if !self.core.go_out_grouping(&G_V) {
                break 'lab7;
            }
            self.core.cursor += 1;
            if !self.core.go_in_grouping(&G_V) {
                break 'lab7;
            }
            self.core.cursor += 1;
            self.p1 = self.core.cursor;
            if !self.core.go_out_grouping(&G_V) {
                break 'lab7;
            }
            self.core.cursor += 1;
            if !self.core.go_in_grouping(&G_V) {
                break 'lab7;
            }
            self.core.cursor += 1;
            self.p2 = self.core.cursor;
        }
        self.core.cursor = v_5;
        true
    }

    fn r_rv(&self) -> bool {
        self.pv <= self.core.cursor
    }

    fn r_r2(&self) -> bool {
        self.p2 <= self.core.cursor
    }

    fn r_r1(&self) -> bool {
        self.p1 <= self.core.cursor
    }

    fn r_aditzak(&mut self) -> bool {
        self.core.ket = self.core.cursor;
        let among_var = self.core.find_among_b(A_0);
        if among_var == 0 {
            return false;
        }
        self.core.bra = self.core.cursor;
        match among_var {
            1 => {
                if !self.r_rv() {
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

    fn r_izenak(&mut self) -> bool {
        self.core.ket = self.core.cursor;
        let among_var = self.core.find_among_b(A_1);
        if among_var == 0 {
            return false;
        }
        self.core.bra = self.core.cursor;
        match among_var {
            1 => {
                if !self.r_rv() {
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
            3 => self.core.slice_from_str("jok"),
            4 => {
                if !self.r_r1() {
                    return false;
                }
                self.core.slice_del();
            }
            5 => self.core.slice_from_str("tra"),
            6 => self.core.slice_from_str("minutu"),
            _ => {}
        }
        true
    }

    fn r_adjetiboak(&mut self) -> bool {
        self.core.ket = self.core.cursor;
        let among_var = self.core.find_among_b(A_2);
        if among_var == 0 {
            return false;
        }
        self.core.bra = self.core.cursor;
        match among_var {
            1 => {
                if !self.r_rv() {
                    return false;
                }
                self.core.slice_del();
            }
            2 => self.core.slice_from_str("z"),
            _ => {}
        }
        true
    }

    fn stem(&mut self) -> bool {
        self.r_mark_regions();
        self.core.limit_backward = self.core.cursor;
        self.core.cursor = self.core.limit;

        'loop0: loop {
            let v_1 = self.core.save();
            'lab0: {
                if !self.r_aditzak() {
                    break 'lab0;
                }
                continue 'loop0;
            }
            self.core.restore(v_1);
            break 'loop0;
        }

        'loop1: loop {
            let v_2 = self.core.save();
            'lab1: {
                if !self.r_izenak() {
                    break 'lab1;
                }
                continue 'loop1;
            }
            self.core.restore(v_2);
            break 'loop1;
        }

        let v_3 = self.core.save();
        self.r_adjetiboak();
        self.core.restore(v_3);

        self.core.cursor = self.core.limit_backward;
        true
    }
}

const A_0: SnowballAmong = &[
    ("idea", 1),
    ("bidea", 1),
    ("kidea", 1),
    ("pidea", 1),
    ("kundea", 1),
    ("galea", 1),
    ("tailea", 1),
    ("tzailea", 1),
    ("gunea", 1),
    ("kunea", 1),
    ("tzaga", 1),
    ("gaia", 1),
    ("aldia", 1),
    ("taldia", 1),
    ("karia", 1),
    ("garria", 2),
    ("karria", 1),
    ("ka", 1),
    ("tzaka", 1),
    ("la", 1),
    ("mena", 1),
    ("pena", 1),
    ("kina", 1),
    ("ezina", 1),
    ("tezina", 1),
    ("kuna", 1),
    ("tuna", 1),
    ("kizuna", 1),
    ("era", 1),
    ("bera", 1),
    ("arabera", -1),
    ("kera", 1),
    ("pera", 1),
    ("orra", 1),
    ("korra", 1),
    ("dura", 1),
    ("gura", 1),
    ("kura", 1),
    ("tura", 1),
    ("eta", 1),
    ("keta", 1),
    ("gailua", 1),
    ("eza", 1),
    ("erreza", 1),
    ("tza", 2),
    ("gaitza", 1),
    ("kaitza", 1),
    ("kuntza", 1),
    ("ide", 1),
    ("bide", 1),
    ("kide", 1),
    ("pide", 1),
    ("kunde", 1),
    ("tzake", 1),
    ("tzeke", 1),
    ("le", 1),
    ("gale", 1),
    ("taile", 1),
    ("tzaile", 1),
    ("gune", 1),
    ("kune", 1),
    ("tze", 1),
    ("atze", 1),
    ("gai", 1),
    ("aldi", 1),
    ("taldi", 1),
    ("ki", 1),
    ("ari", 1),
    ("kari", 1),
    ("lari", 1),
    ("tari", 1),
    ("etari", 1),
    ("garri", 2),
    ("karri", 1),
    ("arazi", 1),
    ("tarazi", 1),
    ("an", 1),
    ("ean", 1),
    ("rean", 1),
    ("kan", 1),
    ("etan", 1),
    ("atseden", -1),
    ("men", 1),
    ("pen", 1),
    ("kin", 1),
    ("rekin", 1),
    ("ezin", 1),
    ("tezin", 1),
    ("tun", 1),
    ("kizun", 1),
    ("go", 1),
    ("ago", 1),
    ("tio", 1),
    ("dako", 1),
    ("or", 1),
    ("kor", 1),
    ("tzat", 1),
    ("du", 1),
    ("gailu", 1),
    ("tu", 1),
    ("atu", 1),
    ("aldatu", 1),
    ("tatu", 1),
    ("baditu", -1),
    ("ez", 1),
    ("errez", 1),
    ("tzez", 1),
    ("gaitz", 1),
    ("kaitz", 1),
];
const A_1: SnowballAmong = &[
    ("ada", 1),
    ("kada", 1),
    ("anda", 1),
    ("denda", 1),
    ("gabea", 1),
    ("kabea", 1),
    ("aldea", 1),
    ("kaldea", 1),
    ("taldea", 1),
    ("ordea", 1),
    ("zalea", 1),
    ("tzalea", 1),
    ("gilea", 1),
    ("emea", 1),
    ("kumea", 1),
    ("nea", 1),
    ("enea", 1),
    ("zionea", 1),
    ("unea", 1),
    ("gunea", 1),
    ("pea", 1),
    ("aurrea", 1),
    ("tea", 1),
    ("kotea", 1),
    ("artea", 1),
    ("ostea", 1),
    ("etxea", 1),
    ("ga", 1),
    ("anga", 1),
    ("gaia", 1),
    ("aldia", 1),
    ("taldia", 1),
    ("handia", 1),
    ("mendia", 1),
    ("geia", 1),
    ("egia", 1),
    ("degia", 1),
    ("tegia", 1),
    ("nahia", 1),
    ("ohia", 1),
    ("kia", 1),
    ("tokia", 1),
    ("oia", 1),
    ("koia", 1),
    ("aria", 1),
    ("karia", 1),
    ("laria", 1),
    ("taria", 1),
    ("eria", 1),
    ("keria", 1),
    ("teria", 1),
    ("garria", 2),
    ("larria", 1),
    ("kirria", 1),
    ("duria", 1),
    ("asia", 1),
    ("tia", 1),
    ("ezia", 1),
    ("bizia", 1),
    ("ontzia", 1),
    ("ka", 1),
    ("joka", 3),
    ("aurka", -1),
    ("ska", 1),
    ("xka", 1),
    ("zka", 1),
    ("gibela", 1),
    ("gela", 1),
    ("kaila", 1),
    ("skila", 1),
    ("tila", 1),
    ("ola", 1),
    ("na", 1),
    ("kana", 1),
    ("ena", 1),
    ("garrena", 1),
    ("gerrena", 1),
    ("urrena", 1),
    ("zaina", 1),
    ("tzaina", 1),
    ("kina", 1),
    ("mina", 1),
    ("garna", 1),
    ("una", 1),
    ("duna", 1),
    ("asuna", 1),
    ("tasuna", 1),
    ("ondoa", 1),
    ("kondoa", 1),
    ("ngoa", 1),
    ("zioa", 1),
    ("koa", 1),
    ("takoa", 1),
    ("zkoa", 1),
    ("noa", 1),
    ("zinoa", 1),
    ("aroa", 1),
    ("taroa", 1),
    ("zaroa", 1),
    ("eroa", 1),
    ("oroa", 1),
    ("osoa", 1),
    ("toa", 1),
    ("ttoa", 1),
    ("ztoa", 1),
    ("txoa", 1),
    ("tzoa", 1),
    ("\u{00F1}oa", 1),
    ("ra", 1),
    ("ara", 1),
    ("dara", 1),
    ("liara", 1),
    ("tiara", 1),
    ("tara", 1),
    ("etara", 1),
    ("tzara", 1),
    ("bera", 1),
    ("kera", 1),
    ("pera", 1),
    ("ora", 2),
    ("tzarra", 1),
    ("korra", 1),
    ("tra", 1),
    ("sa", 1),
    ("osa", 1),
    ("ta", 1),
    ("eta", 1),
    ("keta", 1),
    ("sta", 1),
    ("dua", 1),
    ("mendua", 1),
    ("ordua", 1),
    ("lekua", 1),
    ("burua", 1),
    ("durua", 1),
    ("tsua", 1),
    ("tua", 1),
    ("mentua", 1),
    ("estua", 1),
    ("txua", 1),
    ("zua", 1),
    ("tzua", 1),
    ("za", 1),
    ("eza", 1),
    ("eroza", 1),
    ("tza", 2),
    ("koitza", 1),
    ("antza", 1),
    ("gintza", 1),
    ("kintza", 1),
    ("kuntza", 1),
    ("gabe", 1),
    ("kabe", 1),
    ("kide", 1),
    ("alde", 1),
    ("kalde", 1),
    ("talde", 1),
    ("orde", 1),
    ("ge", 1),
    ("zale", 1),
    ("tzale", 1),
    ("gile", 1),
    ("eme", 1),
    ("kume", 1),
    ("ne", 1),
    ("zione", 1),
    ("une", 1),
    ("gune", 1),
    ("pe", 1),
    ("aurre", 1),
    ("te", 1),
    ("kote", 1),
    ("arte", 1),
    ("oste", 1),
    ("etxe", 1),
    ("gai", 1),
    ("di", 1),
    ("aldi", 1),
    ("taldi", 1),
    ("geldi", -1),
    ("handi", 1),
    ("mendi", 1),
    ("gei", 1),
    ("egi", 1),
    ("degi", 1),
    ("tegi", 1),
    ("nahi", 1),
    ("ohi", 1),
    ("ki", 1),
    ("toki", 1),
    ("oi", 1),
    ("goi", 1),
    ("koi", 1),
    ("ari", 1),
    ("kari", 1),
    ("lari", 1),
    ("tari", 1),
    ("garri", 2),
    ("larri", 1),
    ("kirri", 1),
    ("duri", 1),
    ("asi", 1),
    ("ti", 1),
    ("ontzi", 1),
    ("\u{00F1}i", 1),
    ("ak", 1),
    ("ek", 1),
    ("tarik", 1),
    ("gibel", 1),
    ("ail", 1),
    ("kail", 1),
    ("kan", 1),
    ("tan", 1),
    ("etan", 1),
    ("en", 4),
    ("ren", 2),
    ("garren", 1),
    ("gerren", 1),
    ("urren", 1),
    ("ten", 4),
    ("tzen", 4),
    ("zain", 1),
    ("tzain", 1),
    ("kin", 1),
    ("min", 1),
    ("dun", 1),
    ("asun", 1),
    ("tasun", 1),
    ("aizun", 1),
    ("ondo", 1),
    ("kondo", 1),
    ("go", 1),
    ("ngo", 1),
    ("zio", 1),
    ("ko", 1),
    ("trako", 5),
    ("tako", 1),
    ("etako", 1),
    ("eko", 1),
    ("tariko", 1),
    ("sko", 1),
    ("tuko", 1),
    ("minutuko", 6),
    ("zko", 1),
    ("no", 1),
    ("zino", 1),
    ("ro", 1),
    ("aro", 1),
    ("igaro", -1),
    ("taro", 1),
    ("zaro", 1),
    ("ero", 1),
    ("giro", 1),
    ("oro", 1),
    ("oso", 1),
    ("to", 1),
    ("tto", 1),
    ("zto", 1),
    ("txo", 1),
    ("tzo", 1),
    ("gintzo", 1),
    ("\u{00F1}o", 1),
    ("zp", 1),
    ("ar", 1),
    ("dar", 1),
    ("behar", 1),
    ("zehar", -1),
    ("liar", 1),
    ("tiar", 1),
    ("tar", 1),
    ("tzar", 1),
    ("or", 2),
    ("kor", 1),
    ("os", 1),
    ("ket", 1),
    ("du", 1),
    ("mendu", 1),
    ("ordu", 1),
    ("leku", 1),
    ("buru", 2),
    ("duru", 1),
    ("tsu", 1),
    ("tu", 1),
    ("tatu", 4),
    ("mentu", 1),
    ("estu", 1),
    ("txu", 1),
    ("zu", 1),
    ("tzu", 1),
    ("gintzu", 1),
    ("z", 1),
    ("ez", 1),
    ("eroz", 1),
    ("tz", 1),
    ("koitz", 1),
];
const A_2: SnowballAmong = &[
    ("zlea", 2),
    ("keria", 1),
    ("la", 1),
    ("era", 1),
    ("dade", 1),
    ("tade", 1),
    ("date", 1),
    ("tate", 1),
    ("gi", 1),
    ("ki", 1),
    ("ik", 1),
    ("lanik", 1),
    ("rik", 1),
    ("larik", 1),
    ("ztik", 1),
    ("go", 1),
    ("ro", 1),
    ("ero", 1),
    ("to", 1),
];
const G_V_BITS: [u8; 3] = [17, 65, 16];
const G_V: Grouping = Grouping {
    bits: &G_V_BITS,
    min: 97,
    max: 117,
};

#[cfg(test)]
mod tests {
    use super::*;
    use pizza_engine::analysis::TokenFilter;

    // Vectors from Lucene's TestBasqueAnalyzer.
    #[test]
    fn test_lucene_vectors() {
        let f = BasqueStemTokenFilter::new();
        let mut token = Token::new("zaldi", 0, 5, 0);
        f.filter(&mut token);
        assert_eq!(token.term.as_ref(), "zaldi");
        let mut token = Token::new("zaldiak", 0, 7, 0);
        f.filter(&mut token);
        assert_eq!(token.term.as_ref(), "zaldi");
        let mut token = Token::new("mendiari", 0, 8, 0);
        f.filter(&mut token);
        assert_eq!(token.term.as_ref(), "mendi");
        // the JDK reference leaves genitive -aren unstemmed
        let mut token = Token::new("etxearen", 0, 8, 0);
        f.filter(&mut token);
        assert_eq!(token.term.as_ref(), "etxearen");
    }

    // Differential vectors generated by running the JDK reference
    // implementation (Lucene BasqueStemmer) over the Lucene Basque stopword
    // list plus analyzer test words.
    #[test]
    fn test_differential_vocabulary() {
        let data = include_str!("data/basque_stem.txt");
        let mut checked = 0;
        for line in data.lines() {
            let line = line.trim_end_matches('\r');
            if line.is_empty() {
                continue;
            }
            let Some((input, expected)) = line.split_once('\t') else {
                continue;
            };
            assert_eq!(stem_basque(input), expected, "vector {input}");
            checked += 1;
        }
        assert!(checked > 100, "expected a real vocabulary, got {checked}");
    }
}
