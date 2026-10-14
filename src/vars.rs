//! Variablentabelle (identische Hash-Struktur wie libqsp: 256 Buckets à 50 Slots).
use crate::engine::*;
use crate::errors::*;
use crate::text::*;
use crate::value::*;

const RAND8: [u8; 256] = [
    0x0C, 0x6B, 0x35, 0x24, 0x85, 0x05, 0x2B, 0xAC, 0x32, 0x26, 0xF3, 0x4C, 0x8F, 0xF4, 0x8E, 0xBC,
    0xEC, 0x69, 0x4D, 0x95, 0x77, 0x68, 0xC3, 0xDB, 0xC2, 0x71, 0x1F, 0xD1, 0x14, 0xAA, 0x0A, 0x09,
    0x0D, 0x06, 0xD3, 0x51, 0xE9, 0x31, 0x36, 0x9E, 0x9D, 0x80, 0x25, 0xEF, 0xE2, 0x55, 0xEE, 0x90,
    0x5A, 0xB4, 0xE7, 0x29, 0x04, 0xC1, 0x67, 0x00, 0xD4, 0xD2, 0x75, 0xD0, 0xF8, 0x74, 0x84, 0x46,
    0xC8, 0x44, 0xE6, 0x63, 0x3D, 0xD8, 0x9C, 0xDA, 0x07, 0xB5, 0x39, 0x6A, 0xA7, 0xDE, 0x50, 0xF9,
    0x66, 0xA8, 0xBD, 0xC9, 0x19, 0xCE, 0x7D, 0xEB, 0xE4, 0xCD, 0xFD, 0xA5, 0x21, 0x83, 0xA3, 0xD9,
    0x97, 0x10, 0xBF, 0x8B, 0xD5, 0x81, 0x41, 0x1E, 0x6E, 0x11, 0x4E, 0xAE, 0x57, 0x92, 0xC4, 0xA1,
    0x3F, 0x7C, 0x4A, 0x18, 0x23, 0x6D, 0x3B, 0x96, 0xAF, 0xE0, 0x4F, 0xF5, 0x7E, 0x22, 0xB7, 0x30,
    0x59, 0x15, 0x47, 0xDC, 0xE1, 0x65, 0xA6, 0x20, 0x1B, 0x42, 0xCC, 0x1D, 0x94, 0xCF, 0xCA, 0x53,
    0x9A, 0x28, 0x87, 0x3C, 0x8C, 0x78, 0x2D, 0x93, 0x8D, 0x38, 0x03, 0xA2, 0xDD, 0x49, 0x62, 0xF0,
    0xDF, 0xA0, 0xF2, 0x48, 0x72, 0x6F, 0x7F, 0xC6, 0x73, 0x1A, 0x76, 0xAD, 0x0B, 0xFE, 0x82, 0x6C,
    0xBA, 0x0F, 0x3A, 0x60, 0x12, 0x7B, 0x33, 0xBE, 0x9F, 0x5D, 0x01, 0x64, 0xB6, 0x17, 0xD7, 0x98,
    0x02, 0xB9, 0x4B, 0xFF, 0xAB, 0xB0, 0x5B, 0xB3, 0x16, 0xF7, 0xCB, 0xFC, 0xC5, 0x0E, 0x52, 0x5C,
    0xE8, 0x2A, 0x86, 0x61, 0xC7, 0x2E, 0xE5, 0xA4, 0xFA, 0x79, 0x27, 0xFB, 0xC0, 0x7A, 0x8A, 0x37,
    0xB2, 0xED, 0xA9, 0x5F, 0xBB, 0x3E, 0x45, 0x2F, 0x54, 0x58, 0x2C, 0x70, 0x40, 0xE3, 0x56, 0xB8,
    0xEA, 0x91, 0x34, 0xF6, 0x88, 0x43, 0x99, 0xD6, 0x89, 0x9B, 0x08, 0xF1, 0x5E, 0x1C, 0xB1, 0x13,
];

#[derive(Default, Clone, Debug)]
pub struct VarValue {
    pub num: i32,
    pub str: Option<String>,
}
#[derive(Default, Clone, Debug)]
pub struct VarIndex {
    pub index: i32,
    pub str: String,
}
#[derive(Default, Clone, Debug)]
pub struct VarData {
    pub values: Vec<VarValue>,
    pub indices: Vec<VarIndex>,
}
#[derive(Default)]
pub struct Slot {
    pub name: Option<String>,
    pub data: VarData,
}
pub struct LocalVar {
    pub name: String,
    pub data: VarData,
}


impl Engine {
    pub(crate) fn clear_vars(&mut self) {
        for s in self.vars.iter_mut() {
            s.name = None;
            s.data = VarData::default();
        }
    }
    pub(crate) fn take_var(&mut self, vi: usize) -> VarData {
        std::mem::take(&mut self.vars[vi].data)
    }
    pub(crate) fn put_var(&mut self, vi: usize, d: VarData) {
        self.vars[vi].data = d;
    }
    pub(crate) fn empty_var(&mut self, vi: usize) {
        self.vars[vi].data = VarData::default();
    }

    /// qspVarReference: liefert den Slot-Index.
    pub(crate) fn var_ref(&mut self, name: &str, create: bool) -> R<usize> {
        let name = name.strip_prefix('$').unwrap_or(name);
        let nb = name.as_bytes();
        if nb.is_empty() || is_digit(nb[0]) || str_p_brk(name, DELIMS).is_some() {
            return self.fail(NOTCORRECTNAME);
        }
        let mut b: u8 = 0;
        if name.is_ascii() {
            for &c in nb {
                b = RAND8[(b ^ c.to_ascii_uppercase()) as usize];
            }
        } else {
            for u in upper(name).encode_utf16() {
                b = RAND8[(b ^ (u as u8)) as usize];
            }
        }
        let base = VARSSEEK * b as usize;
        for i in 0..VARSSEEK {
            let slot = &mut self.vars[base + i];
            match &slot.name {
                None => {
                    if create {
                        slot.name = Some(upper(name));
                    }
                    return Ok(base + i);
                }
                Some(n) if n.chars().eq(name.chars().map(up)) => return Ok(base + i),
                _ => {}
            }
        }
        self.fail(TOOMANYVARS)
    }
    /// Wie var_ref, nach Trimmen; liefert zusätzlich, ob der Name mit `$` beginnt.
    pub(crate) fn var_ref_typed(&mut self, name: &str, create: bool) -> R<(usize, bool)> {
        let n = del_spc(name);
        let is_str = n.starts_with('$');
        Ok((self.var_ref(n, create)?, is_str))
    }

    fn var_text_index(&mut self, vi: usize, s: &str, create: bool) -> i32 {
        let u = upper(s);
        let found = {
            let inds = &self.vars[vi].data.indices;
            bsearch(inds, |x| cmp_str(&u, &x.str))
        };
        if let Some(i) = found {
            return self.vars[vi].data.indices[i].index;
        }
        if create {
            let d = &mut self.vars[vi].data;
            let n = d.values.len() as i32;
            let mut i = d.indices.len();
            while i > 0 && cmp_str(&d.indices[i - 1].str, &u) > 0 {
                i -= 1;
            }
            d.indices.insert(i, VarIndex { index: n, str: u });
            return n;
        }
        -1
    }

    /// qspGetVarData: `name[index]` auflösen.
    pub(crate) fn var_data(&mut self, s: &str, is_set: bool) -> R<(usize, i32)> {
        if let Some(l) = s.find('[') {
            let vi = self.var_ref(&s[..l], is_set)?;
            let r = match str_pos(&s[l..], "]", false) {
                Some(r) => l + r,
                None => return self.fail(BRACKNOTFOUND),
            };
            let inner_start = l + 1 + skip_spaces(&s[l + 1..]);
            let n = self.vars[vi].data.values.len() as i32;
            if inner_start >= r {
                let idx = if is_set { n } else if n > 0 { n - 1 } else { 0 };
                return Ok((vi, idx));
            }
            let old = self.refresh_count;
            let v = self.expr_value(&s[inner_start..r])?;
            if self.refresh_count != old {
                return Err(Abort::Refresh);
            }
            let idx = match v {
                Val::Str(t) => self.var_text_index(vi, &t, is_set),
                Val::Num(n) => n,
            };
            return Ok((vi, idx));
        }
        Ok((self.var_ref(s, is_set)?, 0))
    }

    pub(crate) fn set_var_value_by_ref(&mut self, vi: usize, ind: i32, val: &Val) {
        let d = &mut self.vars[vi].data;
        let old = d.values.len();
        if ind >= 0 && (ind as usize) >= old {
            d.values.resize(ind as usize + 1, VarValue::default());
        }
        if ind >= 0 {
            let slot = &mut d.values[ind as usize];
            match val {
                Val::Str(s) => slot.str = Some(s.clone()),
                Val::Num(n) => slot.num = *n,
            }
        }
    }
    pub(crate) fn get_var_value_by_ref(&self, vi: usize, ind: i32, is_str: bool) -> Val {
        let d = &self.vars[vi].data;
        if ind >= 0 && (ind as usize) < d.values.len() {
            let v = &d.values[ind as usize];
            if is_str {
                Val::Str(v.str.clone().unwrap_or_default())
            } else {
                Val::Num(v.num)
            }
        } else {
            Val::empty(is_str)
        }
    }

    /// qspSetVar: Zuweisung `name op= val`.
    fn set_var(&mut self, name: &str, val: &mut Val, op: u8) -> R<()> {
        let (vi, index) = self.var_data(name, true)?;
        let is_str_var = name.starts_with('$');
        match op {
            b'=' => {
                if val.conv_to(is_str_var) {
                    return self.fail(TYPEMISMATCH);
                }
                self.set_var_value_by_ref(vi, index, val);
            }
            b'+' => {
                let mut old = self.get_var_value_by_ref(vi, index, is_str_var);
                if old.is_str() && val.is_str() {
                    if let (Val::Str(a), Val::Str(b)) = (&mut old, &*val) {
                        a.push_str(b);
                    }
                } else if old.can_num() && val.can_num() {
                    old.conv_to(false);
                    val.conv_to(false);
                    old = Val::Num(old.num().wrapping_add(val.num()));
                    old.conv_to(is_str_var);
                } else {
                    if !old.is_str() {
                        return self.fail(TYPEMISMATCH);
                    }
                    val.conv_to(true);
                    if let Val::Str(a) = &mut old {
                        a.push_str(val.s());
                    }
                }
                self.set_var_value_by_ref(vi, index, &old);
            }
            b'-' | b'/' | b'*' => {
                if val.conv_to(false) {
                    return self.fail(TYPEMISMATCH);
                }
                let mut old = self.get_var_value_by_ref(vi, index, is_str_var);
                if old.conv_to(false) {
                    return self.fail(TYPEMISMATCH);
                }
                let (a, b) = (old.num(), val.num());
                let r = match op {
                    b'-' => a.wrapping_sub(b),
                    b'/' => {
                        if b == 0 {
                            return self.fail(DIVBYZERO);
                        }
                        a.wrapping_div(b)
                    }
                    _ => a.wrapping_mul(b),
                };
                old = Val::Num(r);
                old.conv_to(is_str_var);
                self.set_var_value_by_ref(vi, index, &old);
            }
            _ => {}
        }
        Ok(())
    }

    pub(crate) fn get_var(&mut self, name: &str) -> R<Val> {
        let (vi, index) = self.var_data(name, false)?;
        Ok(self.get_var_value_by_ref(vi, index, name.starts_with('$')))
    }
    /// qspGetVarNumValue (Fehler werden verschluckt)
    pub(crate) fn get_var_num(&mut self, name: &str) -> i32 {
        match self.var_ref(name, false) {
            Ok(vi) => self.vars[vi].data.values.first().map(|v| v.num).unwrap_or(0),
            Err(_) => {
                self.reset_error();
                0
            }
        }
    }

    // ---------------- Lokale Variablen ----------------
    pub(crate) fn prepare_global_vars(&mut self) -> R<()> {
        let mut groups = std::mem::take(&mut self.groups);
        let mut res = Ok(());
        'outer: for g in groups.iter_mut().rev() {
            for lv in g.iter_mut().rev() {
                match self.var_ref(&lv.name, true) {
                    Ok(vi) => {
                        let d = std::mem::take(&mut lv.data);
                        self.put_var(vi, d);
                    }
                    Err(e) => {
                        res = Err(e);
                        break 'outer;
                    }
                }
            }
        }
        self.groups = groups;
        res
    }
    /// qspPrepareLocalVars: gibt die gesicherten (aktuell globalen) Werte zurück.
    pub(crate) fn prepare_local_vars(&mut self) -> R<Vec<VarData>> {
        let mut groups = std::mem::take(&mut self.groups);
        let mut saved: Vec<VarData> = Vec::new();
        let mut res = Ok(());
        'outer: for g in groups.iter_mut().rev() {
            for lv in g.iter_mut().rev() {
                match self.var_ref(&lv.name, true) {
                    Ok(vi) => {
                        saved.push(self.take_var(vi));
                        let d = std::mem::take(&mut lv.data);
                        self.put_var(vi, d);
                    }
                    Err(e) => {
                        res = Err(e);
                        break 'outer;
                    }
                }
            }
        }
        self.groups = groups;
        match res {
            Ok(()) => Ok(saved),
            Err(e) => Err(e),
        }
    }
    pub(crate) fn restore_local_vars(&mut self, saved: Vec<VarData>, groups: &mut Vec<Vec<LocalVar>>) -> R<()> {
        let mut it = saved.into_iter();
        for g in groups.iter_mut().rev() {
            for lv in g.iter_mut().rev() {
                let vi = self.var_ref(&lv.name, true)?;
                let cur = self.take_var(vi);
                lv.data = cur;
                if let Some(d) = it.next() {
                    self.put_var(vi, d);
                }
            }
        }
        Ok(())
    }
    /// qspGetVarsList (LOCAL): nimmt die Variablen aus der globalen Tabelle.
    pub(crate) fn get_vars_list(&mut self, s: &str) -> R<Vec<LocalVar>> {
        let mut s = &s[skip_spaces(s)..];
        if s.is_empty() {
            return self.fail(SYNTAX);
        }
        let mut out: Vec<LocalVar> = Vec::new();
        let gi = self.groups.len() - 1;
        let old_count = self.groups[gi].len();
        loop {
            let pos = str_pos(s, ",", false);
            let name = upper(del_spc(match pos { Some(p) => &s[..p], None => s }));
            let found = self.groups[gi][..old_count].iter().any(|v| v.name == name);
            if !found {
                let vi = self.var_ref(&name, false)?;
                let data = self.take_var(vi);
                out.push(LocalVar { name, data });
            }
            match pos {
                None => break,
                Some(p) => {
                    s = &s[p + 1..];
                    s = &s[skip_spaces(s)..];
                    if s.is_empty() {
                        return self.fail(SYNTAX);
                    }
                }
            }
        }
        Ok(out)
    }
    pub(crate) fn restore_vars_list(&mut self, vars: Vec<LocalVar>) -> R<()> {
        for lv in vars {
            let vi = self.var_ref(&lv.name, true)?;
            self.put_var(vi, lv.data);
        }
        Ok(())
    }

    // ---------------- Arrays ----------------
    pub(crate) fn array_size(&mut self, name: &str) -> i32 {
        match self.var_ref_typed(name, false) {
            Ok((vi, _)) => self.vars[vi].data.values.len() as i32,
            Err(_) => 0,
        }
    }
    pub(crate) fn array_pos(&mut self, name: &str, val: &mut Val, ind: i32, is_regexp: bool) -> i32 {
        let (vi, is_string) = match self.var_ref_typed(name, false) {
            Ok(x) => x,
            Err(_) => return -1,
        };
        if val.conv_to(is_regexp || is_string) {
            self.set_error(TYPEMISMATCH);
            return -1;
        }
        let re = if is_regexp {
            match crate::expr::compile_anchored(val.s()) {
                Some(r) => Some(r),
                None => {
                    self.set_error(INCORRECTREGEXP);
                    return -1;
                }
            }
        } else {
            None
        };
        let count = self.vars[vi].data.values.len() as i32;
        let mut ind = ind.clamp(0, count);
        while ind <= count {
            let d = &self.vars[vi].data;
            if val.is_str() {
                let s: &str = if ind < count { d.values[ind as usize].str.as_deref().unwrap_or("") } else { "" };
                if let Some(re) = &re {
                    if crate::expr::full_match(re, s) {
                        return ind;
                    }
                } else if s == val.s() {
                    return ind;
                }
            } else {
                let n = if ind < count { d.values[ind as usize].num } else { 0 };
                if n == val.num() {
                    return ind;
                }
            }
            ind += 1;
        }
        -1
    }
    pub(crate) fn array_min_max(&mut self, name: &str, is_min: bool) -> Val {
        let (vi, is_string) = match self.var_ref_typed(name, false) {
            Ok(x) => x,
            Err(_) => return Val::Num(0),
        };
        let d = &self.vars[vi].data;
        let mut cur: i32 = -1;
        let mut count = d.values.len() as i32;
        while count > 0 {
            count -= 1;
            let v = &d.values[count as usize];
            if is_string {
                let s = v.str.as_deref().unwrap_or("");
                if !s.is_empty() {
                    if cur >= 0 {
                        let c = cmp_str(s, d.values[cur as usize].str.as_deref().unwrap_or(""));
                        if (is_min && c < 0) || (!is_min && c > 0) {
                            cur = count;
                        }
                    } else {
                        cur = count;
                    }
                }
            } else if cur >= 0 {
                let c = v.num;
                let o = d.values[cur as usize].num;
                if (is_min && c < o) || (!is_min && c > o) {
                    cur = count;
                }
            } else {
                cur = count;
            }
        }
        if cur < 0 {
            return Val::empty(is_string);
        }
        let v = &d.values[cur as usize];
        if is_string { Val::Str(v.str.clone().unwrap_or_default()) } else { Val::Num(v.num) }
    }
    pub(crate) fn vars_count(&self) -> usize {
        self.vars.iter().filter(|s| s.name.is_some()).count()
    }
    pub(crate) fn set_args(&mut self, vi: usize, args: &[Val]) {
        for (i, a) in args.iter().enumerate().rev() {
            self.set_var_value_by_ref(vi, i as i32, a);
        }
    }

    // ---------------- Statements ----------------
    /// `SET`/`LET`/implizite Zuweisung
    pub(crate) fn stat_set_var_value(&mut self, s: &str) -> R<()> {
        let mut pos = match str_pos(s, "=", false) {
            Some(p) => p,
            None => return self.fail(EQNOTFOUND),
        };
        let old = self.refresh_count;
        let mut v = self.expr_value(&s[pos + 1..])?;
        if self.refresh_count != old {
            return Err(Abort::Refresh);
        }
        if pos != 0 && is_in_list(b"+-/*", s.as_bytes()[pos - 1]) {
            pos -= 1;
        }
        let ch = s.as_bytes()[pos];
        let name = del_spc(&s[..pos]);
        self.set_var(name, &mut v, ch)
    }
    /// COPYARR
    pub(crate) fn stat_copy_arr(&mut self, args: &[Val]) -> R<()> {
        let (dest, _) = self.var_ref_typed(args[0].s(), true)?;
        let (src, _) = self.var_ref_typed(args[1].s(), false)?;
        if dest != src {
            let count = self.vars[src].data.values.len() as i32;
            let start = if args.len() >= 3 { args[2].num() } else { 0 };
            let num = if args.len() == 4 { args[3].num() } else { count };
            self.empty_var(dest);
            let copy = copy_var(&self.vars[src].data, start, num);
            self.put_var(dest, copy);
        }
        Ok(())
    }
    /// KILLVAR
    pub(crate) fn stat_kill_var(&mut self, args: &[Val]) -> R<()> {
        match args.len() {
            1 => {
                if let Ok((vi, _)) = self.var_ref_typed(args[0].s(), false) {
                    self.empty_var(vi);
                } else {
                    // C ignoriert den Fehler nicht: er bleibt gesetzt
                    return Err(Abort::Error);
                }
            }
            2 => {
                let (vi, _) = self.var_ref_typed(args[0].s(), false)?;
                self.remove_array_item(vi, args[1].num());
            }
            _ => self.clear_vars(),
        }
        Ok(())
    }
    fn remove_array_item(&mut self, vi: usize, index: i32) {
        let d = &mut self.vars[vi].data;
        if index < 0 || index as usize >= d.values.len() {
            return;
        }
        d.values.remove(index as usize);
        let mut out: Vec<VarIndex> = Vec::with_capacity(d.indices.len());
        for mut ind in d.indices.drain(..) {
            if ind.index == index {
                continue;
            }
            if ind.index > index {
                ind.index -= 1;
            }
            out.push(ind);
        }
        d.indices = out;
    }
}

fn copy_var(src: &VarData, start: i32, count: i32) -> VarData {
    let start = start.max(0);
    let max_count = src.values.len() as i32 - start;
    if count <= 0 || max_count <= 0 {
        return VarData::default();
    }
    let n = count.min(max_count);
    let values = src.values[start as usize..(start + n) as usize].to_vec();
    let indices = src
        .indices
        .iter()
        .filter_map(|i| {
            let ni = i.index - start;
            if ni >= 0 && ni < n { Some(VarIndex { index: ni, str: i.str.clone() }) } else { None }
        })
        .collect();
    VarData { values, indices }
}
