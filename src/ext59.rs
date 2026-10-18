// Erweiterungen aus QSP 5.9 (Modus „5.9-Funktionen“).
//
// Copyright (C) 2001-2025 Val Argunov (byte AT qsp DOT org) – ursprüngliche Semantik (libqsp 5.9)
// Rust-Neuimplementierung: Qqsp-rust-Mitwirkende
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Nur additive Neuerungen (vorher unbekannte Schlüsselwörter bzw. zusätzliche optionale
// Argumente). Das Verhalten von 5.7 bleibt im Standardmodus unverändert.
use crate::engine::*;
use crate::errors::*;
use crate::text::*;
use crate::value::*;
use crate::vars::VarValue;

/// Neue Anweisungen (Codes liegen hinter den 5.7-Codes).
pub mod st {
    pub const SORTARR: u8 = 60;
    pub const SCANSTR: u8 = 61;
    pub const SETVAR: u8 = 62;
}
pub const OP_ARRITEM: u8 = 63;

const STAT_NAMES: [(&str, u8); 3] = [("SORTARR", st::SORTARR), ("SCANSTR", st::SCANSTR), ("SETVAR", st::SETVAR)];

/// Anweisungsname am Anfang von `s` (nur im 5.9-Modus aufgerufen).
pub fn stat_code(s: &str) -> Option<(u8, usize)> {
    let b = s.as_bytes();
    for (name, code) in STAT_NAMES {
        let n = name.len();
        if b.len() >= n && b[..n].eq_ignore_ascii_case(name.as_bytes()) && is_in_list_eol(DELIMS, at(b, n)) {
            return Some((code, n));
        }
    }
    None
}

pub fn is_arritem(name: &str) -> bool {
    let n = name.strip_prefix('$').unwrap_or(name);
    n.eq_ignore_ascii_case("ARRITEM")
}

impl Engine {
    /// Wertart eines Arrays: Some(true) = Text, Some(false) = Zahl, None = leer/Lücken.
    fn array_kind(&self, vi: usize) -> Result<Option<bool>, ()> {
        let mut kind: Option<bool> = None;
        for v in &self.vars[vi].data.values {
            let k = if v.str.is_some() { Some(true) } else if v.num != 0 { Some(false) } else { None };
            if let Some(k) = k {
                match kind {
                    None => kind = Some(k),
                    Some(o) if o != k => return Err(()),
                    _ => {}
                }
            }
        }
        Ok(kind)
    }

    /// SORTARR 'name'[, aufsteigend]
    pub(crate) fn stat_sort_arr(&mut self, args: &[Val]) -> R<()> {
        let (vi, prefixed) = self.var_ref_typed(args[0].s(), false)?;
        let n = self.vars[vi].data.values.len();
        if n < 2 {
            return Ok(());
        }
        let kind = match self.array_kind(vi) {
            Ok(k) => k,
            Err(()) => return self.fail(TYPEMISMATCH),
        };
        let Some(is_str) = kind else { return Ok(()) };
        if prefixed && !is_str {
            return self.fail(TYPEMISMATCH);
        }
        // zweites Argument: absteigend (wie in 5.9)
        let asc = args.len() < 2 || args[1].num() == 0;
        let mut order: Vec<usize> = (0..n).collect();
        {
            let vals = &self.vars[vi].data.values;
            order.sort_by(|&a, &b| {
                let c = if is_str {
                    cmp_str(vals[a].str.as_deref().unwrap_or(""), vals[b].str.as_deref().unwrap_or(""))
                } else {
                    (vals[a].num > vals[b].num) as i32 - (vals[a].num < vals[b].num) as i32
                };
                let c = if asc { c } else { -c };
                c.cmp(&0)
            });
        }
        let d = &mut self.vars[vi].data;
        let mut mapping = vec![0usize; n];
        for (newp, &old) in order.iter().enumerate() {
            mapping[old] = newp;
        }
        for ix in d.indices.iter_mut() {
            ix.index = mapping[ix.index as usize] as i32;
        }
        let old = std::mem::take(&mut d.values);
        d.values = order.iter().map(|&o| old[o].clone()).collect();
        Ok(())
    }

    /// SCANSTR 'name', 'text', 'regexp'[, gruppe]
    pub(crate) fn stat_scan_str(&mut self, args: &[Val]) -> R<()> {
        let (vi, _) = self.var_ref_typed(args[0].s(), true)?;
        let re = match fancy_regex::Regex::new(args[2].s()) {
            Ok(r) => r,
            Err(_) => return self.fail(INCORRECTREGEXP),
        };
        let group = if args.len() == 4 { args[3].num().max(0) as usize } else { 0 };
        let text = args[1].s();
        self.empty_var(vi);
        let mut found: Vec<VarValue> = Vec::new();
        let mut pos = 0usize;
        while pos <= text.len() {
            let Ok(Some(c)) = re.captures_from_pos(text, pos) else { break };
            let Some(m) = c.get(group) else { break };
            if m.as_str().is_empty() {
                break;
            }
            found.push(VarValue { num: 0, str: Some(m.as_str().to_string()) });
            pos = m.end();
        }
        self.vars[vi].data.values = found;
        Ok(())
    }

    /// SETVAR 'name', wert[, index]
    pub(crate) fn stat_set_var_ext(&mut self, args: &[Val]) -> R<()> {
        let (vi, prefixed) = self.var_ref_typed(args[0].s(), true)?;
        let mut val = args[1].clone();
        let ind = if args.len() == 3 {
            match &args[2] {
                Val::Str(t) => self.var_text_index(vi, t, true),
                Val::Num(n) => *n,
            }
        } else {
            0
        };
        // ohne Typpräfix bleibt der Typ des Werts erhalten
        if prefixed && val.conv_to(true) {
            return self.fail(TYPEMISMATCH);
        }
        self.set_var_value_by_ref(vi, ind, &val);
        Ok(())
    }

    /// ARRITEM('name'[, index])
    pub(crate) fn func_arr_item(&mut self, args: &[Val]) -> R<Val> {
        let (vi, prefixed) = self.var_ref_typed(args[0].s(), false)?;
        let ind = if args.len() == 2 {
            match &args[1] {
                Val::Str(t) => self.var_text_index(vi, t, false),
                Val::Num(n) => *n,
            }
        } else {
            0
        };
        let d = &self.vars[vi].data;
        if ind >= 0 && (ind as usize) < d.values.len() {
            let v = &d.values[ind as usize];
            if prefixed || v.str.is_some() {
                return Ok(Val::Str(v.str.clone().unwrap_or_default()));
            }
            return Ok(Val::Num(v.num));
        }
        Ok(Val::Str(String::new()))
    }

    /// COUNTOBJ('name')
    pub(crate) fn count_objects_by_name(&self, name: &str) -> i32 {
        let u = upper(name);
        self.cur_objects.iter().filter(|o| upper(&o.desc) == u).count() as i32
    }
}
