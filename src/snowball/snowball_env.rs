use std::borrow::Cow;
use snowball::Among;

#[derive(Debug)]
pub struct SnowballEnv<'a> {
    pub current: Cow<'a, str>,
    pub cursor: usize,
    pub limit: usize,
    pub limit_backward: usize,
    pub bra: usize,
    pub ket: usize,
}


impl<'a> SnowballEnv<'a> {
    pub fn create(value: &'a str) -> Self {
        let len = value.len();
        SnowballEnv {
            current: Cow::from(value),
            cursor: 0,
            limit: len,
            limit_backward: 0,
            bra: 0,
            ket: len,
        }
    }

    pub fn get_current(self) -> Cow<'a, str> {
        self.current
    }

    fn replace_s(&mut self, bra: usize, ket: usize, s: &str) -> i32 {
        let adjustment = s.len() as i32 - (ket as i32 - bra as i32);
        let mut result = String::with_capacity(self.current.len());
        {
            let (lhs, _) = self.current.split_at(bra);
            let (_, rhs) = self.current.split_at(ket);
            result.push_str(lhs);
            result.push_str(s);
            result.push_str(rhs);
        }
        // ... not very nice...
        let new_lim = self.limit as i32 + adjustment;
        self.limit = new_lim as usize;
        if self.cursor >= ket {
            let new_cur = self.cursor as i32 + adjustment;
            self.cursor = new_cur as usize;
        } else if self.cursor > bra {
            self.cursor = bra
        }
        self.current = Cow::from(result);
        adjustment
    }

    /// Check if s is after cursor.
    /// If so, move cursor to the end of s
    pub fn eq_s(&mut self, s: &str) -> bool {
        if self.cursor >= self.limit {
            return false;
        }
        if self.current[self.cursor..].starts_with(s) {
            self.cursor += s.len();
            while !self.current.is_char_boundary(self.cursor) {
                self.cursor += 1;
            }
            true
        } else {
            false
        }
    }

    /// Check if 's' is before cursor
    /// If so, move cursor to the beginning of s
    pub fn eq_s_b(&mut self, s: &str) -> bool {
        if (self.cursor as i32 - self.limit_backward as i32) < s.len() as i32 {
            false
            // Check if cursor -s.len is a char boundry. if not well... return false obv
        } else if !self.current.is_char_boundary(self.cursor - s.len()) ||
                  !self.current[self.cursor - s.len()..].starts_with(s) {
            false
        } else {
            self.cursor -= s.len();
            true
        }
    }

    /// Replace string between `bra` and `ket` with s
    pub fn slice_from(&mut self, s: &str) -> bool {
        let (bra, ket) = (self.bra, self.ket);
        self.replace_s(bra, ket, s);
        true
    }

    /// Move cursor to next charater
    pub fn next_char(&mut self) {
        self.cursor += 1;
        while !self.current.is_char_boundary(self.cursor) {
            self.cursor += 1;
        }
    }

    /// Move cursor to previous character
    pub fn previous_char(&mut self) {
        self.cursor -= 1;
        while !self.current.is_char_boundary(self.cursor) {
            self.cursor -= 1;
        }
    }

    pub fn byte_index_for_hop(&self, mut delta: i32) -> i32 {
        if delta > 0 {
            let mut res = self.cursor;
            while delta > 0 {
                res += 1;
                delta -= 1;
                while res <= self.current.len() && !self.current.is_char_boundary(res) {
                    res += 1;
                }
            }
            return res as i32;
        } else if delta < 0 {
            let mut res: i32 = self.cursor as i32;
            while delta < 0 {
                res -= 1;
                delta += 1;
                while res >= 0 && !self.current.is_char_boundary(res as usize) {
                    res -= 1;
                }
            }
            return res as i32;
        } else {
            return self.cursor as i32;
        }
    }

    /// Check if the char the cursor points to is in the grouping
    /// This is determined by weird magic stuff. I have no idea how it works
    pub fn in_grouping(&mut self, chars: &[u8], min: u32, max: u32) -> bool {
        if self.cursor >= self.limit {
            return false;
        }
        if let Some(chr) = self.current[self.cursor..].chars().next() {
            let mut ch = chr as u32; //codepoint as integer
            if ch > max || ch < min {
                return false;
            }
            ch -= min;
            if (chars[(ch >> 3) as usize] & (0x1 << (ch & 0x7))) == 0 {
                return false;
            }
            self.next_char();
            return true;
        }
        return false;
    }

    pub fn in_grouping_b(&mut self, chars: &[u8], min: u32, max: u32) -> bool {
        if self.cursor <= self.limit_backward {
            return false;
        }
        self.previous_char();
        if let Some(chr) = self.current[self.cursor..].chars().next() {
            let mut ch = chr as u32; //codepoint as integer
            self.next_char();
            if ch > max || ch < min {
                return false;
            }
            ch -= min;
            if (chars[(ch >> 3) as usize] & (0x1 << (ch & 0x7))) == 0 {
                return false;
            }
            self.previous_char();
            return true;
        }
        return false;
    }

    pub fn out_grouping(&mut self, chars: &[u8], min: u32, max: u32) -> bool {
        if self.cursor >= self.limit {
            return false;
        }
        if let Some(chr) = self.current[self.cursor..].chars().next() {
            let mut ch = chr as u32; //codepoint as integer
            if ch > max || ch < min {
                self.next_char();
                return true;
            }
            ch -= min;
            if (chars[(ch >> 3) as usize] & (0x1 << (ch & 0x7))) == 0 {
                self.next_char();
                return true;
            }
        }
        return false;
    }

    pub fn out_grouping_b(&mut self, chars: &[u8], min: u32, max: u32) -> bool {
        if self.cursor <= self.limit_backward {
            return false;
        }
        self.previous_char();
        if let Some(chr) = self.current[self.cursor..].chars().next() {
            let mut ch = chr as u32; //codepoint as integer
            self.next_char();
            if ch > max || ch < min {
                self.previous_char();
                return true;
            }
            ch -= min;
            if (chars[(ch >> 3) as usize] & (0x1 << (ch & 0x7))) == 0 {
                self.previous_char();
                return true;
            }
        }
        return false;

    }


    /// Helper function that removes the string slice between `bra` and `ket`
    pub fn slice_del(&mut self) -> bool {
        self.slice_from("")
    }

    pub fn get_next_char_boundry(data: &str, mut index: usize) -> usize {
        loop {
            if index > data.len() || data.is_char_boundary(index) {
                return index;
            }
            index += 1;
        }
    }

    pub fn get_next_char_boundry_b(data: &str, mut index: usize) -> usize {
        loop {
            if data.is_char_boundary(index) {
                return index;
            }
            index -= 1;
        }
    }

    pub fn insert(&mut self, bra: usize, ket: usize, s: &str) {
        let adjustment = self.replace_s(bra, ket, s);
        if bra <= self.bra {
            self.bra = (self.bra as i32 + adjustment) as usize;
        }
        if bra <= self.ket {
            self.ket = (self.ket as i32 + adjustment) as usize;
        }
    }


    pub fn find_among(&mut self, amongs: &[Among]) -> i32 {
        use std::cmp::min;
        let mut i: i32 = 0;
        let mut j: i32 = amongs.len() as i32;

        let c = self.cursor;
        let l = self.limit;

        let mut common_i = 0;
        let mut common_j = 0;

        let mut first_key_inspected = false;
        loop {
            let k = i + ((j - i) >> 1);
            let mut diff: i32 = 0;
            let mut common = min(common_i, common_j);
            let w = &amongs[k as usize];
            for lvar in common..w.0.len() {
                if c + common == l {
                    diff = -1;
                    break;
                }
                diff = self.current.as_bytes()[c + common] as i32 - w.0.as_bytes()[lvar] as i32;
                if diff != 0 {
                    break;
                }
                common += 1;
            }
            if diff < 0 {
                j = k;
                common_j = common;
            } else {
                i = k;
                common_i = common;
            }
            if j - i <= 1 {
                if i > 0 {
                    break;
                }
                if j == i {
                    break;
                }
                if first_key_inspected {
                    break;
                }
                first_key_inspected = true;
            }
        }

        loop {
            let w = &amongs[i as usize];
            if common_i >= w.0.len() {
                self.cursor = c + w.0.len();
                if let Some(ref method) = w.3 {
                    let res = method(self);
                    self.cursor = c + w.0.len();
                    if res {
                        return w.2;
                    };
                } else {
                    return w.2;
                }
            }
            i = w.1;
            if i < 0 {
                return 0;
            };
        }
    }

    pub fn find_among_b(&mut self, amongs: &[Among]) -> i32 {
        let mut i: i32 = 0;
        let mut j: i32 = amongs.len() as i32;

        let c = self.cursor;
        let lb = self.limit_backward;

        let mut common_i = 0;
        let mut common_j = 0;

        let mut first_key_inspected = false;

        loop {
            let k = i + ((j - i) >> 1);
            let mut diff: i32 = 0;
            let mut common = if common_i < common_j {
                common_i
            } else {
                common_j
            };
            let w = &amongs[k as usize];
            for lvar in (0..w.0.len() - common).rev() {
                if c - common == lb {
                    diff = -1;
                    break;
                }
                // What we do here, I can only assume.
                // We do not check bytewise, as in find_among, but characterwise
                // And probably we check some characters at least twice...
                // I tried fixing that by counting up common by the number of matched bytes.
                // That did not work...

                // Get the char boundries for self.current and w
                let char_bound = Self::get_next_char_boundry_b(&self.current, c - 1 - common);
                let w_char_bound = Self::get_next_char_boundry_b(&w.0, lvar);
                // Get the chars
                let cur_char = self.current[char_bound..].chars().next();
                let w_char = w.0[w_char_bound..].chars().next();
                // Diff them!
                if cur_char.is_none() || w_char.is_none() {
                    break;
                }
                diff = cur_char.unwrap() as i32 - w_char.unwrap() as i32;
                if diff != 0 {
                    break;
                }
                // Count up commons. But not one character but the byte width of that char
                common += 1;
            }
            if diff < 0 {
                j = k;
                common_j = common;
            } else {
                i = k;
                common_i = common;
            }
            if j - i <= 1 {
                if i > 0 {
                    break;
                }
                if j == i {
                    break;
                }
                if first_key_inspected {
                    break;
                }
                first_key_inspected = true;
            }
        }
        loop {
            let w = &amongs[i as usize];
            if common_i >= w.0.len() {
                self.cursor = c - w.0.len();
                if let Some(ref method) = w.3 {
                    let res = method(self);
                    self.cursor = c - w.0.len();
                    if res {
                        return w.2;
                    };
                } else {
                    return w.2;
                }
            }
            i = w.1;
            if i < 0 {
                return 0;
            };
        }
    }
}
