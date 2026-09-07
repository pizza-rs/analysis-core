//! Lucene/ES compatibility tests for analysis-core token filters.
//!
//! Test vectors are extracted from the Apache Lucene test suite:
//! https://github.com/apache/lucene/tree/main/lucene/analysis/common/src/test
//!
//! Each test verifies that our implementation produces identical output
//! to Lucene's for the same input.

use pizza_analysis_core::token_filters::stopwords;
use pizza_analysis_core::*;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;
use pizza_engine::analysis::Tokenizer;

// ─── Helper ────────────────────────────────────────────────────────────────

fn check_filter(filter: &dyn TokenFilter, input: &str, expected: &str) {
    let mut token = Token::new(input, 0, input.len() as u32, 0);
    let (deleted, _) = filter.filter(&mut token);
    if !deleted {
        assert_eq!(
            token.term.as_ref(),
            expected,
            "filter({:?}) = {:?}, expected {:?}",
            input,
            token.term.as_ref(),
            expected
        );
    } else {
        panic!(
            "filter({:?}) deleted the token, expected {:?}",
            input, expected
        );
    }
}

fn check_filter_delete(filter: &dyn TokenFilter, input: &str) {
    let mut token = Token::new(input, 0, input.len() as u32, 0);
    let (deleted, _) = filter.filter(&mut token);
    assert!(deleted, "filter({:?}) should delete the token", input);
}

fn check_tokenizer(tokenizer: &dyn Tokenizer, input: &str, expected: &[&str]) {
    let tokens = tokenizer.tokenize(input);
    let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
    assert_eq!(
        terms, expected,
        "tokenize({:?}) = {:?}, expected {:?}",
        input, terms, expected
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Arabic
// ═══════════════════════════════════════════════════════════════════════════

mod arabic_normalization {
    use super::*;

    #[test]
    fn test_alif_madda() {
        let f = ArabicNormalizationTokenFilter::new();
        check_filter(&f, "آجن", "اجن");
    }

    #[test]
    fn test_alif_hamza_above() {
        let f = ArabicNormalizationTokenFilter::new();
        check_filter(&f, "أحمد", "احمد");
    }

    #[test]
    fn test_alif_hamza_below() {
        let f = ArabicNormalizationTokenFilter::new();
        check_filter(&f, "إعاذ", "اعاذ");
    }

    #[test]
    fn test_alif_maksura() {
        let f = ArabicNormalizationTokenFilter::new();
        check_filter(&f, "بنى", "بني");
    }

    #[test]
    fn test_teh_marbuta() {
        let f = ArabicNormalizationTokenFilter::new();
        check_filter(&f, "فاطمة", "فاطمه");
    }

    #[test]
    fn test_tatweel() {
        let f = ArabicNormalizationTokenFilter::new();
        check_filter(&f, "روبرـــــت", "روبرت");
    }

    #[test]
    fn test_fatha() {
        let f = ArabicNormalizationTokenFilter::new();
        check_filter(&f, "مَبنا", "مبنا");
    }

    #[test]
    fn test_kasra() {
        let f = ArabicNormalizationTokenFilter::new();
        check_filter(&f, "علِي", "علي");
    }

    #[test]
    fn test_damma() {
        let f = ArabicNormalizationTokenFilter::new();
        check_filter(&f, "بُوات", "بوات");
    }

    #[test]
    fn test_fathatan() {
        let f = ArabicNormalizationTokenFilter::new();
        check_filter(&f, "ولداً", "ولدا");
    }

    #[test]
    fn test_kasratan() {
        let f = ArabicNormalizationTokenFilter::new();
        check_filter(&f, "ولدٍ", "ولد");
    }

    #[test]
    fn test_dammatan() {
        let f = ArabicNormalizationTokenFilter::new();
        check_filter(&f, "ولدٌ", "ولد");
    }

    #[test]
    fn test_sukun() {
        let f = ArabicNormalizationTokenFilter::new();
        check_filter(&f, "نلْسون", "نلسون");
    }

    #[test]
    fn test_shaddah() {
        let f = ArabicNormalizationTokenFilter::new();
        check_filter(&f, "هتميّ", "هتمي");
    }

    #[test]
    fn test_empty_term() {
        let f = ArabicNormalizationTokenFilter::new();
        check_filter(&f, "", "");
    }
}

mod arabic_stem {
    use super::*;

    #[test]
    fn test_al_prefix() {
        let f = ArabicStemTokenFilter::new();
        check_filter(&f, "الحسن", "حسن");
    }

    #[test]
    fn test_wal_prefix() {
        let f = ArabicStemTokenFilter::new();
        check_filter(&f, "والحسن", "حسن");
    }

    #[test]
    fn test_bal_prefix() {
        let f = ArabicStemTokenFilter::new();
        check_filter(&f, "بالحسن", "حسن");
    }

    #[test]
    fn test_kal_prefix() {
        let f = ArabicStemTokenFilter::new();
        check_filter(&f, "كالحسن", "حسن");
    }

    #[test]
    fn test_fal_prefix() {
        let f = ArabicStemTokenFilter::new();
        check_filter(&f, "فالحسن", "حسن");
    }

    #[test]
    fn test_ll_prefix() {
        let f = ArabicStemTokenFilter::new();
        check_filter(&f, "للاخر", "اخر");
    }

    #[test]
    fn test_wa_prefix() {
        let f = ArabicStemTokenFilter::new();
        check_filter(&f, "وحسن", "حسن");
    }

    #[test]
    fn test_ah_suffix() {
        let f = ArabicStemTokenFilter::new();
        check_filter(&f, "زوجها", "زوج");
    }

    #[test]
    fn test_an_suffix() {
        let f = ArabicStemTokenFilter::new();
        check_filter(&f, "ساهدان", "ساهد");
    }

    #[test]
    fn test_at_suffix() {
        let f = ArabicStemTokenFilter::new();
        check_filter(&f, "ساهدات", "ساهد");
    }

    #[test]
    fn test_wn_suffix() {
        let f = ArabicStemTokenFilter::new();
        check_filter(&f, "ساهدون", "ساهد");
    }

    #[test]
    fn test_yn_suffix() {
        let f = ArabicStemTokenFilter::new();
        check_filter(&f, "ساهدين", "ساهد");
    }

    #[test]
    fn test_yh_suffix() {
        let f = ArabicStemTokenFilter::new();
        check_filter(&f, "ساهديه", "ساهد");
    }

    #[test]
    fn test_yp_suffix() {
        let f = ArabicStemTokenFilter::new();
        check_filter(&f, "ساهدية", "ساهد");
    }

    #[test]
    fn test_h_suffix() {
        let f = ArabicStemTokenFilter::new();
        check_filter(&f, "ساهده", "ساهد");
    }

    #[test]
    fn test_p_suffix() {
        let f = ArabicStemTokenFilter::new();
        check_filter(&f, "ساهدة", "ساهد");
    }

    #[test]
    fn test_y_suffix() {
        let f = ArabicStemTokenFilter::new();
        check_filter(&f, "ساهدي", "ساهد");
    }

    #[test]
    fn test_combo_prefix_suffix() {
        let f = ArabicStemTokenFilter::new();
        check_filter(&f, "وساهدون", "ساهد");
    }

    #[test]
    fn test_combo_suffix() {
        let f = ArabicStemTokenFilter::new();
        check_filter(&f, "ساهدهات", "ساهد");
    }

    #[test]
    fn test_shouldnt_stem() {
        let f = ArabicStemTokenFilter::new();
        check_filter(&f, "الو", "الو");
    }

    #[test]
    fn test_non_arabic() {
        let f = ArabicStemTokenFilter::new();
        check_filter(&f, "English", "English");
    }

    #[test]
    fn test_empty_term() {
        let f = ArabicStemTokenFilter::new();
        check_filter(&f, "", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// German Normalization
// ═══════════════════════════════════════════════════════════════════════════

mod german_normalization {
    use super::*;

    #[test]
    fn test_umlaut_folding() {
        let f = GermanNormalizationTokenFilter::new();
        check_filter(&f, "Schaltflächen", "Schaltflachen");
    }

    #[test]
    fn test_ae_replacement() {
        let f = GermanNormalizationTokenFilter::new();
        check_filter(&f, "Schaltflaechen", "Schaltflachen");
    }

    #[test]
    fn test_u_heuristic() {
        let f = GermanNormalizationTokenFilter::new();
        // 'ue' should not be normalized when preceded by 'a' (dauer stays as-is)
        check_filter(&f, "dauer", "dauer");
    }

    #[test]
    fn test_special_folding() {
        let f = GermanNormalizationTokenFilter::new();
        check_filter(&f, "weißbier", "weissbier");
    }

    #[test]
    fn test_empty_term() {
        let f = GermanNormalizationTokenFilter::new();
        check_filter(&f, "", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// French Light Stem
// ═══════════════════════════════════════════════════════════════════════════

mod french_light_stem {
    use super::*;

    fn check(input: &str, expected: &str) {
        let f = FrenchLightStemTokenFilter::new();
        check_filter(&f, input, expected);
    }

    #[test]
    fn test_plural_x_to_l() {
        check("chevaux", "cheval");
        check("cheval", "cheval");
    }

    #[test]
    fn test_plural_x_to_nothing() {
        check("hiboux", "hibou");
        check("hibou", "hibou");
    }

    #[test]
    fn test_verb_forms() {
        check("chantés", "chant");
        check("chanter", "chant");
        check("chante", "chant");
        check("chant", "chant");
    }

    #[test]
    fn test_feminine_plural() {
        check("baronnes", "baron");
        check("barons", "baron");
        check("baron", "baron");
    }

    #[test]
    fn test_eau_plural() {
        check("peaux", "peau");
        check("peau", "peau");
        check("anneaux", "aneau");
        check("anneau", "aneau");
    }

    #[test]
    fn test_eux_suffix() {
        check("neveux", "neveu");
        check("neveu", "neveu");
        check("affreux", "afreu");
        check("affreuse", "afreu");
    }

    #[test]
    fn test_issement_suffix() {
        check("investissement", "investi");
        check("investir", "investi");
    }

    #[test]
    fn test_ment_suffix() {
        check("pratiquement", "pratiqu");
        check("pratique", "pratiqu");
    }

    #[test]
    fn test_ivement_suffix() {
        check("administrativement", "administratif");
        check("administratif", "administratif");
    }

    #[test]
    fn test_trice_teur() {
        check("justificatrice", "justifi");
        check("justificateur", "justifi");
        check("justifier", "justifi");
    }

    #[test]
    fn test_catrice_cateur() {
        check("educatrice", "eduqu");
        check("eduquer", "eduqu");
        check("communicateur", "comuniqu");
        check("communiquer", "comuniqu");
    }

    #[test]
    fn test_euse_eur() {
        check("acheteuse", "achet");
        check("acheteur", "achet");
        check("planteur", "plant");
        check("plante", "plant");
    }

    #[test]
    fn test_iere_ier() {
        check("bijoutière", "bijouti");
        check("bijoutier", "bijouti");
        check("caissière", "caisi");
        check("caissier", "caisi");
    }

    #[test]
    fn test_ive_if() {
        check("abrasive", "abrasif");
        check("abrasif", "abrasif");
    }

    #[test]
    fn test_folle_fou() {
        check("folle", "fou");
        check("fou", "fou");
    }

    #[test]
    fn test_elle_e() {
        check("personnelle", "person");
        check("personne", "person");
    }

    #[test]
    fn test_ete_et() {
        check("complète", "complet");
        check("complet", "complet");
    }

    #[test]
    fn test_ique() {
        check("aromatique", "aromat");
    }

    #[test]
    fn test_esse() {
        check("faiblesse", "faibl");
        check("faible", "faibl");
    }

    #[test]
    fn test_age() {
        check("patinage", "patin");
        check("patin", "patin");
    }

    #[test]
    fn test_isation() {
        check("sonorisation", "sono");
        check("ritualisation", "rituel");
        check("rituel", "rituel");
    }

    #[test]
    fn test_numbers() {
        // SOLR-3463
        check("1234555", "1234555");
    }

    #[test]
    fn test_empty_term() {
        check("", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Hindi Stem
// ═══════════════════════════════════════════════════════════════════════════

mod hindi_stem {
    use super::*;

    fn check(input: &str, expected: &str) {
        let f = HindiStemTokenFilter::new();
        check_filter(&f, input, expected);
    }

    #[test]
    fn test_masculine_nouns() {
        check("लडका", "लडक");
        check("लडके", "लडक");
        check("लडकों", "लडक");
        check("गुरु", "गुर");
        check("गुरुओं", "गुर");
        check("दोस्त", "दोस्त");
        check("दोस्तों", "दोस्त");
    }

    #[test]
    fn test_feminine_nouns() {
        check("लडकी", "लडक");
        check("लडकियों", "लडक");
        check("किताब", "किताब");
        check("किताबें", "किताब");
        check("किताबों", "किताब");
        check("आध्यापीका", "आध्यापीक");
        check("आध्यापीकाएं", "आध्यापीक");
        check("आध्यापीकाओं", "आध्यापीक");
    }

    #[test]
    fn test_verbs() {
        check("खाना", "खा");
        check("खाता", "खा");
        check("खाती", "खा");
        check("खा", "खा");
    }

    #[test]
    fn test_exceptions() {
        check("कठिनाइयां", "कठिन");
        check("कठिन", "कठिन");
    }

    #[test]
    fn test_empty_term() {
        check("", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Czech Stem
// ═══════════════════════════════════════════════════════════════════════════

mod czech_stem {
    use super::*;

    fn check(input: &str, expected: &str) {
        let f = CzechStemTokenFilter::new();
        check_filter(&f, input, expected);
    }

    #[test]
    fn test_masculine_nouns() {
        check("pán", "pán");
        check("páni", "pán");
        check("pánové", "pán");
        check("pána", "pán");
        check("pánů", "pán");
        check("pánovi", "pán");
        check("pánům", "pán");
        check("pány", "pán");
        check("páne", "pán");
        check("pánech", "pán");
        check("pánem", "pán");
    }

    #[test]
    fn test_masculine_nouns_hrad() {
        check("hrad", "hrad");
        check("hradu", "hrad");
        check("hrade", "hrad");
        check("hradem", "hrad");
        check("hrady", "hrad");
        check("hradech", "hrad");
        check("hradům", "hrad");
        check("hradů", "hrad");
    }

    #[test]
    fn test_masculine_nouns_muz() {
        check("muž", "muh");
        check("muži", "muh");
        check("muže", "muh");
        check("mužů", "muh");
        check("mužům", "muh");
        check("mužích", "muh");
        check("mužem", "muh");
    }

    #[test]
    fn test_feminine_nouns() {
        check("kost", "kost");
        check("kosti", "kost");
        check("kostí", "kost");
        check("kostem", "kost");
        check("kostech", "kost");
        check("kostmi", "kost");
    }

    #[test]
    fn test_neuter_nouns() {
        check("město", "měst");
        check("města", "měst");
        check("měst", "měst");
        check("městu", "měst");
        check("městům", "měst");
        check("městě", "měst");
        check("městech", "měst");
        check("městem", "měst");
        check("městy", "měst");
    }

    #[test]
    fn test_adjectives() {
        check("mladý", "mlad");
        check("mladí", "mlad");
        check("mladého", "mlad");
        check("mladých", "mlad");
        check("mladému", "mlad");
        check("mladým", "mlad");
        check("mladé", "mlad");
        check("mladém", "mlad");
        check("mladými", "mlad");
        check("mladá", "mlad");
        check("mladou", "mlad");
    }

    #[test]
    fn test_possessive() {
        check("Karlův", "karl");
        check("jazykový", "jazyk");
    }

    #[test]
    fn test_exceptions() {
        // št → sk
        check("český", "česk");
        check("čeští", "česk");
        // čt → ck
        check("anglický", "anglick");
        check("angličtí", "anglick");
        // z → h
        check("kniha", "knih");
        check("knize", "knih");
    }

    #[test]
    fn test_dont_stem() {
        check("e", "e");
        check("zi", "zi");
    }

    #[test]
    fn test_empty_term() {
        check("", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Bulgarian Stem
// ═══════════════════════════════════════════════════════════════════════════

mod bulgarian_stem {
    use super::*;

    fn check(input: &str, expected: &str) {
        let f = BulgarianStemTokenFilter::new();
        check_filter(&f, input, expected);
    }

    #[test]
    fn test_masculine_nouns_grad() {
        check("град", "град");
        check("града", "град");
        check("градът", "град");
        check("градове", "град");
        check("градовете", "град");
    }

    #[test]
    fn test_masculine_nouns_narod() {
        check("народ", "народ");
        check("народа", "народ");
        check("народът", "народ");
        check("народи", "народ");
        check("народите", "народ");
        check("народе", "народ");
    }

    #[test]
    fn test_masculine_nouns_pat() {
        check("път", "път");
        check("пътя", "път");
        check("пътят", "път");
        check("пътища", "път");
        check("пътищата", "път");
    }

    #[test]
    fn test_masculine_nouns_maz() {
        check("мъж", "мъж");
        check("мъжа", "мъж");
        check("мъже", "мъж");
        check("мъжете", "мъж");
        check("мъжо", "мъж");
    }

    #[test]
    fn test_masculine_nouns_krak() {
        check("крак", "крак");
        check("крака", "крак");
        check("кракът", "крак");
        check("краката", "крак");
    }

    #[test]
    fn test_masculine_nouns_brat() {
        check("брат", "брат");
        check("брата", "брат");
        check("братът", "брат");
        check("братя", "брат");
        check("братята", "брат");
        check("брате", "брат");
    }

    #[test]
    fn test_feminine_nouns() {
        check("вест", "вест");
        check("вестта", "вест");
        check("вести", "вест");
        check("вестите", "вест");
    }

    #[test]
    fn test_neuter_nouns() {
        check("дърво", "дърв");
        check("дървото", "дърв");
        check("дърва", "дърв");
        check("дървета", "дърв");
        check("дървата", "дърв");
        check("дърветата", "дърв");
    }

    #[test]
    fn test_adjectives() {
        check("красив", "красив");
        check("красивия", "красив");
        check("красивият", "красив");
        check("красива", "красив");
        check("красивата", "красив");
        check("красиво", "красив");
        check("красивото", "красив");
        check("красиви", "красив");
        check("красивите", "красив");
    }

    #[test]
    fn test_exceptions_ci_to_k() {
        // ци → к
        check("собственик", "собственик");
        check("собственика", "собственик");
        check("собственикът", "собственик");
        check("собственици", "собственик");
        check("собствениците", "собственик");
    }

    #[test]
    fn test_exceptions_zi_to_g() {
        // зи → г
        check("подлог", "подлог");
        check("подлога", "подлог");
        check("подлогът", "подлог");
        check("подлози", "подлог");
        check("подлозите", "подлог");
    }

    #[test]
    fn test_exceptions_centre() {
        // ъ deletion
        check("център", "центр");
        check("центъра", "центр");
        check("центърът", "центр");
        check("центрове", "центр");
        check("центровете", "центр");
    }

    #[test]
    fn test_empty_term() {
        check("", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Latvian Stem
// ═══════════════════════════════════════════════════════════════════════════

mod latvian_stem {
    use super::*;

    fn check(input: &str, expected: &str) {
        let f = LatvianStemTokenFilter::new();
        check_filter(&f, input, expected);
    }

    #[test]
    fn test_nouns_decl1() {
        check("tēvs", "tēv");
        check("tēvi", "tēv");
        check("tēva", "tēv");
        check("tēvu", "tēv");
        check("tēvam", "tēv");
        check("tēviem", "tēv");
        check("tēvus", "tēv");
        check("tēvā", "tēv");
        check("tēvos", "tēv");
    }

    #[test]
    fn test_nouns_decl2_palatalization() {
        // c → č palatalization
        check("lācis", "lāc");
        check("lāči", "lāc");
        check("lāča", "lāc");
        check("lāču", "lāc");
        check("lācim", "lāc");
        check("lāčiem", "lāc");
        check("lāčus", "lāc");
        check("lācī", "lāc");
        check("lāčos", "lāc");
    }

    #[test]
    fn test_nouns_decl2_n_palatalization() {
        // n → ņ
        check("akmens", "akmen");
        check("akmeņi", "akmen");
        check("akmeņu", "akmen");
        check("akmenim", "akmen");
        check("akmeņiem", "akmen");
        check("akmeni", "akmen");
        check("akmeņus", "akmen");
        check("akmenī", "akmen");
        check("akmeņos", "akmen");
    }

    #[test]
    fn test_nouns_decl4() {
        check("lapa", "lap");
        check("lapas", "lap");
        check("lapu", "lap");
        check("lapai", "lap");
        check("lapām", "lap");
        check("lapā", "lap");
        check("lapās", "lap");
    }

    #[test]
    fn test_nouns_decl5_palatalization() {
        // l → ļ
        check("egle", "egl");
        check("egles", "egl");
        check("egļu", "egl");
        check("eglei", "egl");
        check("eglēm", "egl");
        check("egli", "egl");
        check("eglē", "egl");
        check("eglēs", "egl");
    }

    #[test]
    fn test_adjectives() {
        check("zils", "zil");
        check("zilais", "zil");
        check("zili", "zil");
        check("zilie", "zil");
        check("zila", "zil");
        check("zilā", "zil");
        check("zilas", "zil");
        check("zilās", "zil");
        check("zilu", "zil");
        check("zilo", "zil");
        check("zilam", "zil");
        check("zilajam", "zil");
        check("ziliem", "zil");
        check("zilajiem", "zil");
        check("zilai", "zil");
        check("zilajai", "zil");
        check("zilām", "zil");
        check("zilajām", "zil");
        check("zilus", "zil");
        check("zilos", "zil");
    }

    #[test]
    fn test_palatalization() {
        check("krāsns", "krāsn");
        check("krāšņu", "krāsn");
        check("zvaigzne", "zvaigzn");
        check("zvaigžņu", "zvaigzn");
        check("vilnis", "viln");
        check("viļņu", "viln");
        check("lelle", "lell");
        check("leļļu", "lell");
        check("pinne", "pinn");
        check("piņņu", "pinn");
    }

    #[test]
    fn test_length() {
        // too short
        check("usa", "usa");
    }

    #[test]
    fn test_empty_term() {
        check("", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Portuguese Light Stem
// ═══════════════════════════════════════════════════════════════════════════

mod portuguese_light_stem {
    use super::*;

    fn check(input: &str, expected: &str) {
        let f = PortugueseLightStemTokenFilter::new();
        check_filter(&f, input, expected);
    }

    #[test]
    fn test_plural_es() {
        check("doutores", "doutor");
        check("doutor", "doutor");
    }

    #[test]
    fn test_plural_ns() {
        check("homens", "homem");
        check("homem", "homem");
    }

    #[test]
    fn test_plural_is() {
        check("papéis", "papel");
        check("papel", "papel");
        check("normais", "normal");
        check("normal", "normal");
    }

    #[test]
    fn test_plural_ois() {
        check("lencóis", "lencol");
        check("lencol", "lencol");
    }

    #[test]
    fn test_plural_barris() {
        check("barris", "barril");
        check("barril", "barril");
    }

    #[test]
    fn test_plural_oes() {
        check("botões", "bota");
        check("botão", "bota");
    }

    #[test]
    fn test_empty_term() {
        check("", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CJK Bigram
// ═══════════════════════════════════════════════════════════════════════════

mod cjk_bigram {
    use super::*;

    #[test]
    fn test_japanese_bigrams() {
        let t = pizza_engine::analysis::StandardTokenizer::new();
        let tokens = t.tokenize("多くの学生が試験に落ちた");
        let f = CjkBigramTokenFilter::new();

        let mut results = Vec::new();
        for mut token in tokens {
            let (deleted, extra) = f.filter(&mut token);
            if !deleted {
                results.push(token.term.to_string());
            }
            if let Some(extras) = extra {
                for et in extras {
                    results.push(et.term.to_string());
                }
            }
        }
        // CJK bigram should produce adjacent character pairs
        assert!(
            results.len() > 1,
            "Should produce bigrams, got {:?}",
            results
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Galician Minimal Stem
// ═══════════════════════════════════════════════════════════════════════════

mod galician_stem {
    use super::*;

    fn check(input: &str, expected: &str) {
        let f = GalicianMinimalStemTokenFilter::new();
        check_filter(&f, input, expected);
    }

    #[test]
    fn test_plural() {
        check("elefantes", "elefante");
        check("elefante", "elefante");
    }

    #[test]
    fn test_plural_es() {
        check("kalóres", "kalór");
        check("kalór", "kalór");
    }

    // Vectors from Lucene's TestGalicianMinimalStemFilter: the minimal
    // stemmer is the RSLP Plural step alone (whole-word exceptions, no
    // accent folding).
    #[test]
    fn test_exceptions() {
        check("mas", "mas");
        check("barcelonês", "barcelonês");
    }

    #[test]
    fn test_empty_term() {
        check("", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ASCII Folding
// ═══════════════════════════════════════════════════════════════════════════

mod ascii_folding {
    use super::*;

    fn check(input: &str, expected: &str) {
        let f = AsciiFoldingTokenFilter::new();
        check_filter(&f, input, expected);
    }

    #[test]
    fn test_latin1_accents_uppercase() {
        check("À", "A");
        check("Á", "A");
        check("Â", "A");
        check("Ã", "A");
        check("Ä", "A");
        check("Å", "A");
        check("Æ", "AE");
        check("Ç", "C");
        check("È", "E");
        check("É", "E");
        check("Ê", "E");
        check("Ë", "E");
        check("Ì", "I");
        check("Í", "I");
        check("Î", "I");
        check("Ï", "I");
        check("Ð", "D");
        check("Ñ", "N");
        check("Ò", "O");
        check("Ó", "O");
        check("Ô", "O");
        check("Õ", "O");
        check("Ö", "O");
        check("Ø", "O");
        check("Ù", "U");
        check("Ú", "U");
        check("Û", "U");
        check("Ü", "U");
        check("Ý", "Y");
        check("Þ", "TH");
    }

    #[test]
    fn test_latin1_accents_lowercase() {
        check("à", "a");
        check("á", "a");
        check("â", "a");
        check("ã", "a");
        check("ä", "a");
        check("å", "a");
        check("æ", "ae");
        check("ç", "c");
        check("è", "e");
        check("é", "e");
        check("ê", "e");
        check("ë", "e");
        check("ì", "i");
        check("í", "i");
        check("î", "i");
        check("ï", "i");
        check("ð", "d");
        check("ñ", "n");
        check("ò", "o");
        check("ó", "o");
        check("ô", "o");
        check("õ", "o");
        check("ö", "o");
        check("ø", "o");
        check("ù", "u");
        check("ú", "u");
        check("û", "u");
        check("ü", "u");
        check("ý", "y");
        check("ÿ", "y");
        check("ß", "ss");
        check("þ", "th");
    }

    #[test]
    fn test_ligatures() {
        check("Œ", "OE");
        check("œ", "oe");
        check("Ĳ", "IJ");
        check("ĳ", "ij");
        check("ﬁ", "fi");
        check("ﬂ", "fl");
        // the rest of Lucene's Latin ligature block
        check("ﬀ", "ff");
        check("ﬃ", "ffi");
        check("ﬄ", "ffl");
        check("ﬆ", "st");
    }

    #[test]
    fn test_unmodified() {
        check("Des", "Des");
        check("mot", "mot");
        check("END", "END");
    }

    #[test]
    fn test_word_with_accents() {
        check("clés", "cles");
        check("CHAÎNE", "CHAINE");
    }

    #[test]
    fn test_empty_term() {
        check("", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Greek Stem
// ═══════════════════════════════════════════════════════════════════════════

mod greek_stem {
    use super::*;

    fn check(input: &str, expected: &str) {
        let f = GreekStemTokenFilter::new();
        // Greek stemmer expects lowercased input (Greek lowercase removes tonos)
        let lc = GreekLowercaseTokenFilter::new();
        let mut token = Token::new(input, 0, input.len() as u32, 0);
        lc.filter(&mut token);
        f.filter(&mut token);
        assert_eq!(
            token.term.as_ref(),
            expected,
            "greek_stem({:?}) = {:?}, expected {:?}",
            input,
            token.term.as_ref(),
            expected
        );
    }

    #[test]
    fn test_masculine_nouns() {
        check("άνθρωπος", "ανθρωπ");
        check("ανθρώπου", "ανθρωπ");
        check("άνθρωπο", "ανθρωπ");
        check("άνθρωπε", "ανθρωπ");
        check("άνθρωποι", "ανθρωπ");
        check("ανθρώπων", "ανθρωπ");
        check("ανθρώπους", "ανθρωπ");
    }

    #[test]
    fn test_masculine_nouns_pelatis() {
        check("πελάτης", "πελατ");
        check("πελάτη", "πελατ");
        check("πελάτες", "πελατ");
        check("πελατών", "πελατ");
    }

    #[test]
    fn test_masculine_nouns_elefantas() {
        check("ελέφαντας", "ελεφαντ");
        check("ελέφαντα", "ελεφαντ");
        check("ελέφαντες", "ελεφαντ");
        check("ελεφάντων", "ελεφαντ");
    }

    #[test]
    fn test_feminine_nouns() {
        check("φορά", "φορ");
        check("φοράς", "φορ");
        check("φορές", "φορ");
        check("φορών", "φορ");
    }

    #[test]
    fn test_feminine_nouns_agelada() {
        check("αγελάδα", "αγελαδ");
        check("αγελάδας", "αγελαδ");
        check("αγελάδες", "αγελαδ");
        check("αγελάδων", "αγελαδ");
    }

    #[test]
    fn test_neuter_nouns() {
        check("βιβλίο", "βιβλι");
        check("βιβλίου", "βιβλ");
        check("βιβλία", "βιβλ");
        check("βιβλίων", "βιβλ");
    }

    #[test]
    fn test_adjectives() {
        check("συνεχής", "συνεχ");
        check("συνεχούς", "συνεχ");
        check("συνεχή", "συνεχ");
        check("συνεχών", "συνεχ");
        check("συνεχείς", "συνεχ");
        check("συνεχές", "συνεχ");
    }

    #[test]
    fn test_verbs() {
        check("ορίζω", "οριζ");
        check("όριζα", "οριζ");
        check("όριζε", "οριζ");
        check("ορίζοντας", "οριζ");
        check("ορίζομαι", "οριζ");
        check("οριζόμουν", "οριζ");
    }

    #[test]
    fn test_empty_term() {
        check("", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Elision
// ═══════════════════════════════════════════════════════════════════════════

mod elision {
    use super::*;

    #[test]
    fn test_french_elision() {
        let f = ElisionTokenFilter::french();
        check_filter(&f, "l'avion", "avion");
        check_filter(&f, "d'avion", "avion");
    }

    #[test]
    fn test_custom_articles() {
        let f = ElisionTokenFilter::new(&["l", "d", "qu"]);
        check_filter(&f, "l'avion", "avion");
        check_filter(&f, "qu'est", "est");
    }

    #[test]
    fn test_no_elision() {
        let f = ElisionTokenFilter::french();
        check_filter(&f, "avion", "avion");
        check_filter(&f, "hello", "hello");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Stop Filter
// ═══════════════════════════════════════════════════════════════════════════

mod stop_filter {
    use super::*;

    #[test]
    fn test_english_stop_words() {
        let words = stopwords::get_stop_words("english").unwrap();
        let f = StopTokenFilter::new(words);
        check_filter_delete(&f, "the");
        check_filter_delete(&f, "is");
        check_filter_delete(&f, "a");
        check_filter_delete(&f, "an");
    }

    #[test]
    fn test_not_stopped() {
        let words = stopwords::get_stop_words("english").unwrap();
        let f = StopTokenFilter::new(words);
        check_filter(&f, "hello", "hello");
        check_filter(&f, "world", "world");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Turkish Lowercase
// ═══════════════════════════════════════════════════════════════════════════

mod turkish_lowercase {
    use super::*;

    #[test]
    fn test_dotted_i() {
        let f = TurkishLowercaseTokenFilter::new();
        check_filter(&f, "İSTANBUL", "istanbul");
    }

    #[test]
    fn test_dotless_i() {
        let f = TurkishLowercaseTokenFilter::new();
        check_filter(&f, "ISTIKLAL", "ıstıklal");
    }

    #[test]
    fn test_mixed() {
        let f = TurkishLowercaseTokenFilter::new();
        check_filter(&f, "İstanbul", "istanbul");
    }

    #[test]
    fn test_empty_term() {
        let f = TurkishLowercaseTokenFilter::new();
        check_filter(&f, "", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Greek Lowercase
// ═══════════════════════════════════════════════════════════════════════════

mod greek_lowercase {
    use super::*;

    #[test]
    fn test_removes_tonos() {
        let f = GreekLowercaseTokenFilter::new();
        // Lucene standardizes both sigma forms to U+03C3 (standard sigma).
        check_filter(&f, "Άνθρωπος", "ανθρωποσ");
    }

    #[test]
    fn test_removes_dialytika() {
        let f = GreekLowercaseTokenFilter::new();
        check_filter(&f, "ΠΡΩΤΟΫΠΟΥΡΓΌΣ", "πρωτουπουργοσ");
    }

    #[test]
    fn test_final_sigma() {
        let f = GreekLowercaseTokenFilter::new();
        check_filter(&f, "ΣΩΜΑ", "σωμα");
    }

    #[test]
    fn test_empty_term() {
        let f = GreekLowercaseTokenFilter::new();
        check_filter(&f, "", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Scandinavian Normalization / Folding
// ═══════════════════════════════════════════════════════════════════════════

mod scandinavian {
    use super::*;

    // Canonical forms are å/æ/ø (Lucene ScandinavianNormalizer): Swedish
    // umlauts fold to them, digraphs collapse into them. Vectors from
    // TestScandinavianNormalizationFilter.
    #[test]
    fn test_normalization_ae() {
        let f = ScandinavianNormalizationTokenFilter::new();
        check_filter(&f, "ä", "æ");
        check_filter(&f, "ö", "ø");
        check_filter(&f, "æ", "æ"); // already canonical
        check_filter(&f, "ø", "ø");
        check_filter(&f, "räksmörgås", "ræksmørgås");
        check_filter(&f, "raeksmoergås", "ræksmørgås");
        check_filter(&f, "aeäaeeeae", "æææeeæ");
    }

    #[test]
    fn test_folding_aa() {
        let f = ScandinavianFoldingTokenFilter::new();
        check_filter(&f, "å", "a");
        check_filter(&f, "ä", "a");
        check_filter(&f, "ö", "o");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Serbian Normalization
// ═══════════════════════════════════════════════════════════════════════════

mod serbian_normalization {
    use super::*;

    #[test]
    fn test_cyrillic_to_latin() {
        let f = SerbianNormalizationTokenFilter::new();
        check_filter(&f, "Београд", "Beograd");
    }

    #[test]
    fn test_empty_term() {
        let f = SerbianNormalizationTokenFilter::new();
        check_filter(&f, "", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Apostrophe Filter
// ═══════════════════════════════════════════════════════════════════════════

mod apostrophe {
    use super::*;

    #[test]
    fn test_strips_after_apostrophe() {
        let f = ApostropheTokenFilter::new();
        check_filter(&f, "İstanbul'un", "İstanbul");
        check_filter(&f, "it's", "it");
    }

    #[test]
    fn test_no_apostrophe() {
        let f = ApostropheTokenFilter::new();
        check_filter(&f, "hello", "hello");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Decimal Digit
// ═══════════════════════════════════════════════════════════════════════════

mod decimal_digit {
    use super::*;

    #[test]
    fn test_arabic_digits() {
        let f = DecimalDigitTokenFilter::new();
        check_filter(&f, "٣٢١", "321");
    }

    #[test]
    fn test_devanagari_digits() {
        let f = DecimalDigitTokenFilter::new();
        check_filter(&f, "१२३", "123");
    }

    #[test]
    fn test_ascii_passthrough() {
        let f = DecimalDigitTokenFilter::new();
        check_filter(&f, "123", "123");
    }

    #[test]
    fn test_empty() {
        let f = DecimalDigitTokenFilter::new();
        check_filter(&f, "", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CJK Width
// ═══════════════════════════════════════════════════════════════════════════

mod cjk_width {
    use super::*;

    #[test]
    fn test_fullwidth_to_halfwidth() {
        let f = CjkWidthTokenFilter::new();
        check_filter(&f, "Ｔｅｓｔ", "Test");
    }

    #[test]
    fn test_halfwidth_katakana_to_fullwidth() {
        let f = CjkWidthTokenFilter::new();
        check_filter(&f, "ｶﾀｶﾅ", "カタカナ");
    }

    #[test]
    fn test_empty() {
        let f = CjkWidthTokenFilter::new();
        check_filter(&f, "", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Hindi Normalization
// ═══════════════════════════════════════════════════════════════════════════

mod hindi_normalization {
    use super::*;

    #[test]
    fn test_nukta_removal() {
        let f = HindiNormalizationTokenFilter::new();
        // Nukta composed characters should normalize
        check_filter(&f, "क़", "क");
    }

    #[test]
    fn test_chandrabindu() {
        let f = HindiNormalizationTokenFilter::new();
        // Chandrabindu → anusvara normalization
        check_filter(&f, "हँसना", "हंसना");
    }

    #[test]
    fn test_empty() {
        let f = HindiNormalizationTokenFilter::new();
        check_filter(&f, "", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Indic Normalization
// ═══════════════════════════════════════════════════════════════════════════

mod indic_normalization {
    use super::*;

    // Lucene's IndicNormalizer COMPOSES decomposed sequences into their
    // precomposed forms. Vectors from TestIndicNormalizer.testBasics.
    #[test]
    fn test_devanagari_nukta() {
        let f = IndicNormalizationTokenFilter::new();
        check_filter(&f, "\u{0915}\u{093C}", "\u{0958}"); // क + ़ → क़
        check_filter(&f, "अाअा", "आआ");
        check_filter(&f, "अाॅअाॅ", "ऑऑ"); // candra O
        check_filter(&f, "ত্\u{200D}", "ৎ"); // khanda-ta
    }

    #[test]
    fn test_empty() {
        let f = IndicNormalizationTokenFilter::new();
        check_filter(&f, "", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Persian Normalization
// ═══════════════════════════════════════════════════════════════════════════

mod persian_normalization {
    use super::*;

    // Lucene's PersianNormalizer maps Farsi forms to their Arabic
    // equivalents. Vectors from TestPersianNormalizationFilter (strict
    // mode: our default additionally folds diacritics/tatweel).
    #[test]
    fn test_yeh_normalization() {
        let f = PersianNormalizationTokenFilter::lucene_strict();
        // Farsi yeh → Arabic yeh
        check_filter(&f, "های", "هاي");
        // Yeh barree → Arabic yeh
        check_filter(&f, "هاے", "هاي");
    }

    #[test]
    fn test_keh_normalization() {
        let f = PersianNormalizationTokenFilter::lucene_strict();
        // Keheh → Arabic kaf
        check_filter(&f, "کشاندن", "كشاندن");
        // Heh+yeh → heh; heh+hamza above → heh (hamza deleted);
        // heh goal → heh
        check_filter(&f, "كتابۀ", "كتابه");
        check_filter(&f, "كتابهٔ", "كتابه");
        check_filter(&f, "زادہ", "زاده");
    }

    #[test]
    fn test_empty() {
        let f = PersianNormalizationTokenFilter::new();
        check_filter(&f, "", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Sorani Normalization
// ═══════════════════════════════════════════════════════════════════════════

mod sorani_normalization {
    use super::*;

    #[test]
    fn test_yeh_normalization() {
        let f = SoraniNormalizationTokenFilter::new();
        check_filter(&f, "ي", "ی");
    }

    #[test]
    fn test_keh_normalization() {
        let f = SoraniNormalizationTokenFilter::new();
        check_filter(&f, "ك", "ک");
    }

    #[test]
    fn test_empty() {
        let f = SoraniNormalizationTokenFilter::new();
        check_filter(&f, "", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Keyword Tokenizer
// ═══════════════════════════════════════════════════════════════════════════

mod keyword_tokenizer {
    use super::*;

    #[test]
    fn test_single_token() {
        let t = KeywordTokenizer::new();
        check_tokenizer(&t, "hello world", &["hello world"]);
    }

    #[test]
    fn test_empty() {
        let t = KeywordTokenizer::new();
        check_tokenizer(&t, "", &[""]);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Letter Tokenizer
// ═══════════════════════════════════════════════════════════════════════════

mod letter_tokenizer {
    use super::*;

    #[test]
    fn test_basic() {
        let t = LetterTokenizer::new();
        check_tokenizer(&t, "hello world", &["hello", "world"]);
    }

    #[test]
    fn test_numbers_split() {
        let t = LetterTokenizer::new();
        check_tokenizer(&t, "hello123world", &["hello", "world"]);
    }

    #[test]
    fn test_punctuation() {
        let t = LetterTokenizer::new();
        check_tokenizer(&t, "hello, world!", &["hello", "world"]);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Path Hierarchy Tokenizer
// ═══════════════════════════════════════════════════════════════════════════

mod path_hierarchy {
    use super::*;

    #[test]
    fn test_default() {
        let t = PathHierarchyTokenizer::default();
        check_tokenizer(
            &t,
            "/usr/local/bin",
            &["/usr", "/usr/local", "/usr/local/bin"],
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Classic Tokenizer
// ═══════════════════════════════════════════════════════════════════════════

mod classic_tokenizer {
    use super::*;

    #[test]
    fn test_basic() {
        let t = ClassicTokenizer::new();
        let tokens = t.tokenize("hello world");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, &["hello", "world"]);
    }

    #[test]
    fn test_acronyms() {
        let t = ClassicTokenizer::new();
        let tokens = t.tokenize("U.S.A.");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"U.S.A.") || terms.contains(&"U.S.A"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Dutch Stem (Snowball)
// ═══════════════════════════════════════════════════════════════════════════

mod dutch_stem {
    use super::*;

    fn check(input: &str, expected: &str) {
        let f = DutchStemTokenFilter::new();
        check_filter(&f, input, expected);
    }

    #[test]
    fn test_basic() {
        check("lichamen", "lichaam");
        check("lichamelijk", "lichamelijk");
        check("lichamelijke", "lichamelijk");
        check("lichamelijkheden", "lichamelijk");
    }

    #[test]
    fn test_licht_variants() {
        check("lichte", "licht");
        check("lichten", "licht");
        check("lichtende", "licht");
        check("lichtje", "licht");
        check("lichtjes", "licht");
        check("lichtste", "licht");
        check("lichtte", "licht");
        check("lichtten", "licht");
    }

    #[test]
    fn test_compound() {
        check("lichtkranten", "lichtkrant");
        check("lidstaten", "lidstaat");
    }

    #[test]
    fn test_op_prefix() {
        check("opgraven", "opgraaf");
        check("opgroeiende", "opgroeiend");
        check("ophalen", "ophaal");
        check("ophouden", "ophoud");
        check("opheffen", "ophef");
        check("opheffende", "ophef");
        check("opheffing", "ophef");
    }

    #[test]
    fn test_empty_term() {
        check("", "");
    }

    // Full vector set from Lucene's TestDutchAnalyzer (77 terms), including
    // compound/ge- words that exercise the prefix/infix removal paths.
    #[test]
    fn test_dutch_full_lucene_vectors() {
        let vectors: &[(&str, &str)] = &[
            ("lichaamsziek", "lichaamsziek"),
            ("lichamelijk", "lichamelijk"),
            ("lichamelijke", "lichamelijk"),
            ("lichamelijkheden", "lichamelijk"),
            ("lichamen", "lichaam"),
            ("lichere", "licher"),
            ("licht", "licht"),
            ("lichtbeeld", "lichtbeeld"),
            ("lichtbruin", "lichtbruin"),
            ("lichtdoorlatende", "lichtdoorlaat"),
            ("lichte", "licht"),
            ("lichten", "licht"),
            ("lichtende", "licht"),
            ("lichtenvoorde", "lichtenvoor"),
            ("lichter", "lichter"),
            ("lichtere", "lichter"),
            ("lichters", "lichter"),
            ("lichtgevoeligheid", "lichtvoel"),
            ("lichtgewicht", "lichtwicht"),
            ("lichtgrijs", "lichtgrijs"),
            ("lichthoeveelheid", "lichthoeveel"),
            ("lichtintensiteit", "lichtintens"),
            ("lichtje", "licht"),
            ("lichtjes", "licht"),
            ("lichtkranten", "lichtkrant"),
            ("lichtkring", "lichtkr"),
            ("lichtkringen", "lichtkr"),
            ("lichtregelsystemen", "lichtrelsysteem"),
            ("lichtste", "licht"),
            ("lichtstromende", "lichtstroom"),
            ("lichtte", "licht"),
            ("lichtten", "licht"),
            ("lichttoetreding", "lichttoetreed"),
            ("lichtverontreinigde", "lichtverontrein"),
            ("lichtzinnige", "lichtzin"),
            ("lid", "lid"),
            ("lidia", "lidia"),
            ("lidmaatschap", "lidmaatschap"),
            ("lidstaten", "lidstaat"),
            ("lidvereniging", "lidvereen"),
            ("opgingen", "opg"),
            ("opglanzing", "opglans"),
            ("opglanzingen", "opglans"),
            ("opglimlachten", "opglimlacht"),
            ("opglimpen", "opglimp"),
            ("opglimpende", "opglimp"),
            ("opglimping", "opglimp"),
            ("opglimpingen", "opglimp"),
            ("opgraven", "opgraaf"),
            ("opgrijnzen", "opgrijns"),
            ("opgrijzende", "opgrijs"),
            ("opgroeien", "opgroei"),
            ("opgroeiende", "opgroeiend"),
            ("opgroeiplaats", "opgroeiplaats"),
            ("ophaal", "ophaal"),
            ("ophaaldienst", "ophaaldienst"),
            ("ophaalkosten", "ophaalkost"),
            ("ophaalsystemen", "ophaalsysteem"),
            ("ophaalt", "ophaalt"),
            ("ophaaltruck", "ophaaltruck"),
            ("ophalen", "ophaal"),
            ("ophalend", "ophaal"),
            ("ophalers", "ophaler"),
            ("ophef", "ophef"),
            ("opheldering", "opheldeer"),
            ("ophemelde", "ophemel"),
            ("ophemelen", "ophemeel"),
            ("opheusden", "opheus"),
            ("ophief", "ophief"),
            ("ophield", "ophield"),
            ("ophieven", "ophief"),
            ("ophoepelt", "ophoepelt"),
            ("ophoog", "ophoog"),
            ("ophoogzand", "ophoogzand"),
            ("ophopen", "ophoop"),
            ("ophoping", "ophoop"),
            ("ophouden", "ophoud"),
        ];
        let f = DutchStemTokenFilter::new();
        for (input, expected) in vectors {
            check_filter(&f, input, expected);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Brazilian Stem
// ═══════════════════════════════════════════════════════════════════════════

mod brazilian_stem {
    use super::*;

    fn check(input: &str, expected: &str) {
        let f = BrazilianStemTokenFilter::new();
        check_filter(&f, input, expected);
    }

    // Vectors below are from Lucene's TestBrazilianAnalyzer
    // (testWithSnowballExamples / testNormalization). The old approximation
    // produced different stems for several of these (e.g. borracheiro →
    // "borrachei" instead of "borracheir"); expectations follow Lucene.
    #[test]
    fn test_basic_arias() {
        check("boataria", "boat");
        check("boatarias", "boat");
        check("boate", "boat");
        check("boates", "boat");
        check("boatos", "boat");
    }

    #[test]
    fn test_oes_plural() {
        check("bobalhões", "bobalho");
        check("bobalhona", "bobalhon");
        check("bobalhone", "bobalhon");
    }

    #[test]
    fn test_eiro_suffix() {
        check("borracheiro", "borracheir");
        check("borracheira", "borracheir");
    }

    #[test]
    fn test_ismo_suffix() {
        check("comunismo", "comun");
        check("comunista", "comun");
    }

    #[test]
    fn test_eza_suffix() {
        check("beleza", "belez");
        check("belezas", "belez");
    }

    #[test]
    fn test_metric() {
        check("quilômetros", "quilometr");
        check("quilômetro", "quilometr");
        check("quilométricas", "quilometr");
        check("quilométricos", "quilometr");
    }

    #[test]
    fn test_diacritic_normalization() {
        // removes diacritics: different from snowball portuguese
        check("bôas", "boas");
        check("boçal", "bocal");
        check("boêmio", "boemi");
        check("bóia", "boi");
        check("quinhão", "quinha");
        check("quintão", "quinta");
        // encia → ente (versus snowball portuguese 'quintessent')
        check("quintessência", "quintessente");
        // lowercase by default / remove diacritics
        check("Brasil", "brasil");
        check("Brasília", "brasil");
        // non-letter content: diacritic still removed, no stemming
        check("quimio5terápicos", "quimio5terapicos");
        // too short: diacritics are not removed
        check("áá", "áá");
        check("ááá", "aaa");
    }

    #[test]
    fn test_empty_term() {
        check("", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Norwegian Light Stem
// ═══════════════════════════════════════════════════════════════════════════

mod norwegian_stem {
    use super::*;

    fn check(input: &str, expected: &str) {
        let f = NorwegianLightStemTokenFilter::new();
        check_filter(&f, input, expected);
    }

    #[test]
    fn test_plural_er() {
        check("ansen", "ans");
    }

    #[test]
    fn test_empty_term() {
        check("", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Italian Light Stem
// ═══════════════════════════════════════════════════════════════════════════

mod italian_light_stem {
    use super::*;

    fn check(input: &str, expected: &str) {
        let f = ItalianLightStemTokenFilter::new();
        check_filter(&f, input, expected);
    }

    #[test]
    fn test_plural_i() {
        check("abbandonati", "abbandonat");
        check("abbandonato", "abbandonat");
    }

    #[test]
    fn test_plural_e() {
        // Lucene's light stemmer passes words shorter than 6 chars through.
        // Vectors from Lucene's itlighttestdata.zip vocabulary.
        check("casa", "casa");
        check("abbandonate", "abbandonat");
    }

    #[test]
    fn test_empty_term() {
        check("", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Spanish Light Stem
// ═══════════════════════════════════════════════════════════════════════════

mod spanish_light_stem {
    use super::*;

    fn check(input: &str, expected: &str) {
        let f = SpanishLightStemTokenFilter::new();
        check_filter(&f, input, expected);
    }

    #[test]
    fn test_plural() {
        // Vectors from Lucene's eslighttestdata.zip: the final vowel is
        // always stripped (torero -> torer).
        check("toreros", "torer");
        check("torero", "torer");
    }

    #[test]
    fn test_feminine() {
        check("española", "español");
        // ends in 'l' — not a stripping suffix
        check("español", "español");
    }

    #[test]
    fn test_empty_term() {
        check("", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Russian Light Stem
// ═══════════════════════════════════════════════════════════════════════════

mod russian_light_stem {
    use super::*;

    fn check(input: &str, expected: &str) {
        let f = RussianLightStemTokenFilter::new();
        check_filter(&f, input, expected);
    }

    #[test]
    fn test_nouns() {
        check("студента", "студент");
        check("студент", "студент");
    }

    #[test]
    fn test_adjectives() {
        check("новый", "нов");
        check("новая", "нов");
        check("новое", "нов");
    }

    #[test]
    fn test_empty_term() {
        check("", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// German Light Stem
// ═══════════════════════════════════════════════════════════════════════════

mod german_light_stem {
    use super::*;

    fn check(input: &str, expected: &str) {
        let f = GermanLightStemTokenFilter::new();
        check_filter(&f, input, expected);
    }

    #[test]
    fn test_basic() {
        check("häuser", "haus");
        check("haus", "haus");
    }

    #[test]
    fn test_empty_term() {
        check("", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Bengali Stem
// ═══════════════════════════════════════════════════════════════════════════

mod bengali_stem {
    use super::*;

    fn check(input: &str, expected: &str) {
        let f = BengaliStemTokenFilter::new();
        check_filter(&f, input, expected);
    }

    #[test]
    fn test_basic() {
        check("বাংলাদেশ", "বাংলাদেশ");
    }

    #[test]
    fn test_empty_term() {
        check("", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Fingerprint Filter
// ═══════════════════════════════════════════════════════════════════════════

mod fingerprint {
    use super::*;

    // Lucene's FingerprintFilter emits ONE token for the whole field: the
    // sorted unique terms joined by a separator (ES example: "the quick
    // brown fox" → "brown fox quick the"). Our per-token API accumulates
    // through filter() and yields the fingerprint via take_fingerprint().
    #[test]
    fn test_basic() {
        let f = FingerprintTokenFilter::new();
        let t = StandardTokenizer::new();
        let mut tokens = t.tokenize("the quick brown fox");
        for token in &mut tokens {
            let (deleted, _) = f.filter(token);
            assert!(deleted, "fingerprint removes the original token");
        }
        assert_eq!(f.take_fingerprint().unwrap(), "brown fox quick the");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Trim Filter
// ═══════════════════════════════════════════════════════════════════════════

mod trim_filter {
    use super::*;

    #[test]
    fn test_whitespace_trimming() {
        let f = TrimTokenFilter::new();
        check_filter(&f, "  hello  ", "hello");
        check_filter(&f, "\thello\t", "hello");
    }

    #[test]
    fn test_no_trim_needed() {
        let f = TrimTokenFilter::new();
        check_filter(&f, "hello", "hello");
    }

    #[test]
    fn test_empty() {
        let f = TrimTokenFilter::new();
        check_filter(&f, "", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Truncate Filter
// ═══════════════════════════════════════════════════════════════════════════

mod truncate_filter {
    use super::*;

    #[test]
    fn test_truncate() {
        let f = TruncateTokenFilter::new(5);
        check_filter(&f, "abcdefgh", "abcde");
    }

    #[test]
    fn test_no_truncation() {
        let f = TruncateTokenFilter::new(10);
        check_filter(&f, "hello", "hello");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Reverse Filter
// ═══════════════════════════════════════════════════════════════════════════

mod reverse_filter {
    use super::*;

    #[test]
    fn test_reverse() {
        let f = ReverseTokenFilter::new();
        check_filter(&f, "hello", "olleh");
    }

    #[test]
    fn test_empty() {
        let f = ReverseTokenFilter::new();
        check_filter(&f, "", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Length Filter
// ═══════════════════════════════════════════════════════════════════════════

mod length_filter {
    use super::*;

    #[test]
    fn test_too_short() {
        let f = LengthTokenFilter::new(3, 10);
        check_filter_delete(&f, "ab");
    }

    #[test]
    fn test_too_long() {
        let f = LengthTokenFilter::new(1, 5);
        check_filter_delete(&f, "abcdefg");
    }

    #[test]
    fn test_within_range() {
        let f = LengthTokenFilter::new(2, 10);
        check_filter(&f, "hello", "hello");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Lowercase Filter
// ═══════════════════════════════════════════════════════════════════════════

mod lowercase_filter {
    use super::*;

    #[test]
    fn test_basic() {
        let f = LowercaseTokenFilter::new();
        check_filter(&f, "Hello", "hello");
        check_filter(&f, "WORLD", "world");
    }

    #[test]
    fn test_already_lowercase() {
        let f = LowercaseTokenFilter::new();
        check_filter(&f, "hello", "hello");
    }

    #[test]
    fn test_empty() {
        let f = LowercaseTokenFilter::new();
        check_filter(&f, "", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Uppercase Filter
// ═══════════════════════════════════════════════════════════════════════════

mod uppercase_filter {
    use super::*;

    #[test]
    fn test_basic() {
        let f = UppercaseTokenFilter::new();
        check_filter(&f, "Hello", "HELLO");
        check_filter(&f, "world", "WORLD");
    }

    #[test]
    fn test_empty() {
        let f = UppercaseTokenFilter::new();
        check_filter(&f, "", "");
    }
}
