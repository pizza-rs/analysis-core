//! Minimal Snowball runtime shared by the generated-stemmer ports
//! (Basque, Catalan). Mirrors Lucene's `SnowballProgram` semantics:
//! a UTF-32 char buffer with cursor/limit/limit_backward plus bra/ket
//! slice markers, among tables matched longest-first, and the Java
//! runtime's cursor-adjustment rules for splice operations.

/// Character-group bitset, verbatim from the generated Java. A character is
/// in the group when bit `(ch - min) % 8` of byte `(ch - min) / 8` is set.
pub(crate) struct Grouping {
    pub(crate) bits: &'static [u8],
    pub(crate) min: u32,
    pub(crate) max: u32,
}

/// Among table: (string, result id). Longest match at the cursor wins.
pub(crate) type Among = &'static [(&'static str, i32)];

pub(crate) struct SnowballCore {
    pub(crate) text: Vec<char>,
    pub(crate) cursor: usize,
    pub(crate) limit: usize,
    pub(crate) limit_backward: usize,
    pub(crate) bra: usize,
    pub(crate) ket: usize,
}

impl SnowballCore {
    pub(crate) fn new(word: &str) -> Self {
        let text: Vec<char> = word.chars().collect();
        let limit = text.len();
        SnowballCore {
            text,
            cursor: 0,
            limit,
            limit_backward: 0,
            bra: 0,
            ket: limit,
        }
    }

    pub(crate) fn result(&self) -> String {
        self.text.iter().collect()
    }

    /// Save/restore a cursor position the way the generated code does:
    /// relative to the (possibly shifted) current limit.
    pub(crate) fn save(&self) -> isize {
        self.limit as isize - self.cursor as isize
    }

    pub(crate) fn restore(&mut self, v: isize) {
        self.cursor = (self.limit as isize - v) as usize;
    }

    fn in_group_set(&self, ch: u32, g: &Grouping) -> bool {
        ch <= g.max && ch >= g.min && {
            let b = (ch - g.min) as usize;
            g.bits[b >> 3] & (1 << (b & 7)) != 0
        }
    }

    pub(crate) fn in_grouping(&mut self, g: &Grouping) -> bool {
        if self.cursor >= self.limit {
            return false;
        }
        let ch = self.text[self.cursor] as u32;
        if !self.in_group_set(ch, g) {
            return false;
        }
        self.cursor += 1;
        true
    }

    pub(crate) fn in_grouping_b(&mut self, g: &Grouping) -> bool {
        if self.cursor <= self.limit_backward {
            return false;
        }
        let ch = self.text[self.cursor - 1] as u32;
        if !self.in_group_set(ch, g) {
            return false;
        }
        self.cursor -= 1;
        true
    }

    pub(crate) fn out_grouping(&mut self, g: &Grouping) -> bool {
        if self.cursor >= self.limit {
            return false;
        }
        let ch = self.text[self.cursor] as u32;
        if self.in_group_set(ch, g) {
            return false;
        }
        self.cursor += 1;
        true
    }

    pub(crate) fn out_grouping_b(&mut self, g: &Grouping) -> bool {
        if self.cursor <= self.limit_backward {
            return false;
        }
        let ch = self.text[self.cursor - 1] as u32;
        if self.in_group_set(ch, g) {
            return false;
        }
        self.cursor -= 1;
        true
    }

    /// Advance while the grouping holds; true if it stopped on a char
    /// outside the group (false at the limit).
    pub(crate) fn go_in_grouping(&mut self, g: &Grouping) -> bool {
        while self.cursor < self.limit {
            let ch = self.text[self.cursor] as u32;
            if !self.in_group_set(ch, g) {
                return true;
            }
            self.cursor += 1;
        }
        false
    }

    pub(crate) fn go_out_grouping(&mut self, g: &Grouping) -> bool {
        while self.cursor < self.limit {
            let ch = self.text[self.cursor] as u32;
            if self.in_group_set(ch, g) {
                return true;
            }
            self.cursor += 1;
        }
        false
    }

    pub(crate) fn eq_s(&mut self, s: &str) -> bool {
        let l = s.chars().count();
        if self.limit - self.cursor < l {
            return false;
        }
        if !self.text[self.cursor..self.cursor + l]
            .iter()
            .copied()
            .eq(s.chars())
        {
            return false;
        }
        self.cursor += l;
        true
    }

    pub(crate) fn eq_s_b(&mut self, s: &str) -> bool {
        let l = s.chars().count();
        if self.cursor - self.limit_backward < l {
            return false;
        }
        let start = self.cursor - l;
        if !self.text[start..self.cursor].iter().copied().eq(s.chars()) {
            return false;
        }
        self.cursor = start;
        true
    }

    pub(crate) fn find_among(&mut self, v: Among) -> i32 {
        let c = self.cursor;
        let mut best: Option<(usize, i32)> = None;
        for &(s, result) in v {
            let l = s.chars().count();
            if l <= self.limit - c
                && self.text[c..c + l].iter().copied().eq(s.chars())
                && best.is_none_or(|(bl, _)| l > bl)
            {
                best = Some((l, result));
            }
        }
        match best {
            Some((l, r)) => {
                self.cursor = c + l;
                r
            }
            None => 0,
        }
    }

    pub(crate) fn find_among_b(&mut self, v: Among) -> i32 {
        let c = self.cursor;
        let mut best: Option<(usize, i32)> = None;
        for &(s, result) in v {
            let l = s.chars().count();
            if l <= c - self.limit_backward
                && self.text[c - l..c].iter().copied().eq(s.chars())
                && best.is_none_or(|(bl, _)| l > bl)
            {
                best = Some((l, result));
            }
        }
        match best {
            Some((l, r)) => {
                self.cursor = c - l;
                r
            }
            None => 0,
        }
    }

    /// Port of `replace_s`: splice `s` over `[c_bra, c_ket)` and shift the
    /// cursor per the Java runtime's adjustment rules.
    fn replace_s(&mut self, c_bra: usize, c_ket: usize, s: &[char]) -> isize {
        let adjustment = s.len() as isize - (c_ket - c_bra) as isize;
        self.text.splice(c_bra..c_ket, s.iter().copied());
        self.limit = (self.limit as isize + adjustment) as usize;
        if self.cursor as isize >= c_ket as isize {
            self.cursor = (self.cursor as isize + adjustment) as usize;
        } else if self.cursor > c_bra {
            self.cursor = c_bra;
        }
        adjustment
    }

    pub(crate) fn slice_from_str(&mut self, s: &str) {
        let s: Vec<char> = s.chars().collect();
        let adjustment = self.replace_s(self.bra, self.ket, &s);
        self.ket = (self.ket as isize + adjustment) as usize;
    }

    pub(crate) fn slice_del(&mut self) {
        self.slice_from_str("");
    }

    /// Port of the Java runtime's `insert`: splice `s` in at
    /// `[c_bra, c_ket)` and adjust the active slice markers.
    pub(crate) fn insert_at(&mut self, c_bra: usize, c_ket: usize, s: &[char]) {
        let adjustment = self.replace_s(c_bra, c_ket, s);
        if c_bra <= self.bra {
            self.bra = (self.bra as isize + adjustment) as usize;
        }
        if c_bra <= self.ket {
            self.ket = (self.ket as isize + adjustment) as usize;
        }
    }
}
