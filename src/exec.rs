//! Ausführung von Code: Statements, Kontrollfluss, Lokale – Port von statements.c / locations.c / actions.c.
use crate::code::{get_line_label, preprocess, st, stat_spec, Line, Stat};
use crate::engine::*;
use crate::errors::*;
use crate::host::Win;
use crate::text::*;
use crate::value::*;
use std::rc::Rc;

#[derive(PartialEq, Clone, Copy)]
enum Flow {
    Execute,
    Skip,
    Continue,
}

fn search_else(s: &[Rc<Line>], start: usize, end: usize) -> Option<usize> {
    let mut c = 1;
    for i in start..end {
        let l = &s[i];
        match l.stats[0].kind {
            st::ACT | st::FOR | st::IF => {
                if l.is_multiline {
                    c += 1
                }
            }
            st::ELSE | st::ELSEIF => {
                if c == 1 {
                    return Some(i);
                }
            }
            st::END => {
                c -= 1;
                if c == 0 {
                    return None;
                }
            }
            _ => {}
        }
    }
    None
}
fn search_end(s: &[Rc<Line>], start: usize, end: usize) -> Option<usize> {
    let mut c = 1;
    for i in start..end {
        let l = &s[i];
        match l.stats[0].kind {
            st::ACT | st::FOR | st::IF => {
                if l.is_multiline {
                    c += 1
                }
            }
            st::END => {
                c -= 1;
                if c == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}
fn search_label(s: &[Rc<Line>], start: usize, end: usize, label: &str) -> Option<usize> {
    (start..end).find(|&i| s[i].label.as_deref() == Some(label))
}

/// Text eines Statements (bis zum nächsten `&` bzw. Zeilenende).
fn stat_text(line: &Line, i: usize) -> &str {
    let s = &line.stats[i];
    let end = if i + 1 < line.stats.len() { s.end } else { line.text.len() };
    &line.text[s.param..end]
}
/// Bedingungstext von IF/ELSEIF/ACT/FOR (bis zum Doppelpunkt).
fn head_text(line: &Line, i: usize) -> &str {
    let s = &line.stats[i];
    &line.text[s.param..s.end]
}

impl Engine {
    // ---------------------------------------------------------------- Argumente
    pub(crate) fn get_stat_args(&mut self, s: &str, code: u8) -> R<Vec<Val>> {
        let spec = stat_spec(code);
        let mut text = &s[skip_spaces(s)..];
        if text.starts_with('(') {
            let b = match str_pos(text, ")", false) {
                Some(b) => b,
                None => return self.fail(BRACKNOTFOUND),
            };
            if !is_any_string(&text[b + 1..]) {
                text = &text[1..b];
                text = &text[skip_spaces(text)..];
            }
        }
        let mut args: Vec<Val> = Vec::new();
        if !text.is_empty() {
            let old = self.refresh_count;
            loop {
                if args.len() == spec.max {
                    return self.fail(ARGSCOUNT);
                }
                let pos = str_pos(text, ",", false);
                let expr = match pos {
                    Some(p) => &text[..p],
                    None => text,
                };
                let mut v = self.expr_value(expr)?;
                if self.refresh_count != old {
                    return Err(Abort::Refresh);
                }
                let t = spec.types[args.len()];
                if t != 0 && v.conv_to(t == 1) {
                    return self.fail(TYPEMISMATCH);
                }
                args.push(v);
                match pos {
                    None => break,
                    Some(p) => {
                        text = &text[p + 1..];
                        text = &text[skip_spaces(text)..];
                        if text.is_empty() {
                            return self.fail(SYNTAX);
                        }
                    }
                }
            }
        }
        if args.len() < spec.min {
            return self.fail(ARGSCOUNT);
        }
        Ok(args)
    }

    fn cond_value(&mut self, text: &str, code: u8) -> R<bool> {
        let args = self.get_stat_args(text, code)?;
        Ok(args.first().map_or(false, |a| a.num() != 0))
    }

    // ---------------------------------------------------------------- Zeilen
    pub(crate) fn exec_string(&mut self, line: &Line, start: usize, end: usize, jump: &mut String) -> R<bool> {
        let old = self.refresh_count;
        for i in start..end {
            let stat = line.stats[i].kind;
            match stat {
                st::UNKNOWN | st::LABEL | st::ELSE | st::END => {}
                st::COMMENT | st::ELSEIF => return Ok(false),
                st::ACT => {
                    self.stat_singleline_add_act(line, i, end)?;
                    return Ok(false);
                }
                st::IF => return self.stat_if(line, i, end, jump),
                st::FOR => return self.stat_singleline_for(line, i, end, jump),
                st::LOCAL => {
                    let t = stat_text(line, i);
                    self.stat_local(t)?;
                }
                st::SET => {
                    let t = stat_text(line, i);
                    self.stat_set_var_value(t)?;
                }
                _ => {
                    let t = stat_text(line, i);
                    let args = self.get_stat_args(t, stat)?;
                    if self.refresh_count != old {
                        return Err(Abort::Refresh);
                    }
                    let is_exit = self.exec_stat(stat, &args, jump)?;
                    if is_exit {
                        return Ok(true);
                    }
                    if self.refresh_count != old {
                        return Err(Abort::Refresh);
                    }
                    if !jump.is_empty() {
                        return Ok(false);
                    }
                }
            }
        }
        Ok(false)
    }

    fn exec_multiline(&mut self, s: &[Rc<Line>], end_line: usize, off: i32, jump: &mut String, ind: &mut usize, action: &mut Flow) -> R<bool> {
        let i = *ind;
        let end = match search_end(s, i + 1, end_line) {
            Some(e) => e,
            None => return self.fail(ENDNOTFOUND),
        };
        let line = &s[i];
        let stat = line.stats[0].kind;
        match stat {
            st::IF | st::ELSEIF => {
                let c = self.cond_value(head_text(line, 0), stat)?;
                let else_pos = search_else(s, i + 1, end);
                if c {
                    *ind = end;
                    *action = Flow::Skip;
                    return self.exec_block_locals(s, i + 1, else_pos.unwrap_or(end), off, Some(&mut *jump));
                } else {
                    *ind = else_pos.unwrap_or(end);
                    *action = Flow::Continue;
                }
            }
            st::ACT => {
                *ind = end;
                *action = Flow::Continue;
                self.stat_multiline_add_act(s, end, i, off > 0)?;
            }
            st::FOR => {
                *ind = end;
                *action = Flow::Skip;
                return self.stat_multiline_for(s, end, i, off, jump);
            }
            _ => {}
        }
        Ok(false)
    }

    fn exec_singleline(&mut self, s: &[Rc<Line>], end_line: usize, off: i32, jump: &mut String, ind: &mut usize, action: &mut Flow) -> R<bool> {
        let i = *ind;
        let line = &s[i];
        match line.stats[0].kind {
            st::ELSEIF => {
                if at(line.text.as_bytes(), line.stats[0].end) != b':' {
                    return self.fail(COLONNOTFOUND);
                }
                let end = match search_end(s, i + 1, end_line) {
                    Some(e) => e,
                    None => return self.fail(ENDNOTFOUND),
                };
                let c = self.cond_value(head_text(line, 0), st::ELSEIF)?;
                if c {
                    *ind = end;
                    *action = Flow::Skip;
                    return self.exec_string_locals(line, 1, line.stats.len(), jump);
                } else {
                    let else_pos = search_else(s, i + 1, end);
                    *ind = else_pos.unwrap_or(end);
                    *action = Flow::Continue;
                }
            }
            st::ELSE => {
                let end = match search_end(s, i + 1, end_line) {
                    Some(e) => e,
                    None => return self.fail(ENDNOTFOUND),
                };
                *ind = end;
                *action = Flow::Skip;
                if line.stats.len() > 1 {
                    return self.exec_string_locals(line, 1, line.stats.len(), jump);
                }
                let else_pos = search_else(s, i + 1, end);
                return self.exec_block_locals(s, i + 1, else_pos.unwrap_or(end), off, Some(&mut *jump));
            }
            _ => {}
        }
        Ok(false)
    }

    pub(crate) fn exec_code(&mut self, s: &[Rc<Line>], start: usize, end: usize, off: i32, jump: Option<&mut String>) -> R<bool> {
        let old = self.refresh_count;
        let mut fake = String::new();
        let (jump, u_level) = match jump {
            Some(j) => (j, false),
            None => (&mut fake, true),
        };
        let mut action = Flow::Execute;
        let mut i = start;
        let mut is_exit = false;
        while i < end {
            let line = &s[i];
            if off > 0 {
                self.real_line = line.line_num + off;
                if self.is_debug && !line.text.is_empty() {
                    self.call_debug(&line.text);
                    if self.refresh_count != old {
                        return Err(Abort::Refresh);
                    }
                }
            }
            if line.is_multiline {
                is_exit = self.exec_multiline(s, end, off, jump, &mut i, &mut action)?;
            } else {
                is_exit = self.exec_singleline(s, end, off, jump, &mut i, &mut action)?;
            }
            if is_exit {
                break;
            }
            if self.refresh_count != old {
                return Err(Abort::Refresh);
            }
            if action == Flow::Continue {
                action = Flow::Execute;
                continue;
            }
            if action == Flow::Skip {
                action = Flow::Execute;
            } else {
                is_exit = self.exec_string(line, 0, line.stats.len(), jump)?;
                if is_exit {
                    break;
                }
                i += 1;
            }
            if !jump.is_empty() {
                match search_label(s, start, end, jump) {
                    Some(l) => {
                        i = l;
                        jump.clear();
                    }
                    None => {
                        if u_level {
                            return self.fail(LABELNOTFOUND);
                        }
                        break;
                    }
                }
            }
        }
        Ok(is_exit)
    }

    pub(crate) fn exec_block_locals(&mut self, s: &[Rc<Line>], start: usize, end: usize, off: i32, jump: Option<&mut String>) -> R<bool> {
        self.groups.push(Vec::new());
        let r = self.exec_code(s, start, end, off, jump);
        self.pop_group(r)
    }
    fn exec_string_locals(&mut self, line: &Line, start: usize, end: usize, jump: &mut String) -> R<bool> {
        self.groups.push(Vec::new());
        let r = self.exec_string(line, start, end, jump);
        self.pop_group(r)
    }
    fn pop_group(&mut self, r: R<bool>) -> R<bool> {
        let g = self.groups.pop().unwrap_or_default();
        match r {
            Err(e) => Err(e),
            Ok(x) => {
                self.restore_vars_list(g)?;
                Ok(x)
            }
        }
    }

    /// qspExecTopCodeWithLocals
    pub(crate) fn exec_top_code(&mut self, code: &[Rc<Line>], off: i32, is_new_loc: bool) -> R<bool> {
        let mut saved_local: Vec<crate::vars::VarData> = Vec::new();
        if is_new_loc {
            self.prepare_global_vars()?;
        } else {
            saved_local = self.prepare_local_vars()?;
        }
        let mut outer = std::mem::take(&mut self.groups);
        self.groups = vec![Vec::new()];
        let r = self.exec_code(code, 0, code.len(), off, None);
        let g0 = self.groups.pop().unwrap_or_default();
        let mut result = r;
        match &r {
            Err(_) => { /* g0 und gesicherte Lokale verwerfen */ }
            Ok(_) => {
                if let Err(e) = self.restore_vars_list(g0) {
                    result = Err(e);
                }
                if !is_new_loc {
                    if result.is_err() || self.err.num != 0 {
                        // gesicherte Werte verwerfen
                    } else if let Err(e) = self.restore_local_vars(saved_local, &mut outer) {
                        result = Err(e);
                    }
                }
            }
        }
        self.groups = outer;
        result
    }

    pub(crate) fn exec_string_as_code_with_args(&mut self, code: &str, args: &[Val]) -> R<()> {
        let vi = self.var_ref("ARGS", true)?;
        let local = self.take_var(vi);
        self.set_args(vi, args);
        let lines: Vec<Rc<Line>> = preprocess(code).into_iter().map(Rc::new).collect();
        self.exec_block_locals(&lines, 0, lines.len(), 0, None)?;
        let vi = self.var_ref("ARGS", true)?;
        self.put_var(vi, local);
        Ok(())
    }

    // ---------------------------------------------------------------- IF / FOR / LOCAL
    fn stat_if(&mut self, line: &Line, start: usize, end: usize, jump: &mut String) -> R<bool> {
        if at(line.text.as_bytes(), line.stats[start].end) != b':' {
            return self.fail(COLONNOTFOUND);
        }
        let mut else_stat = 0usize;
        let mut c = 1;
        for i in start + 1..end {
            match line.stats[i].kind {
                st::IF => c += 1,
                st::ELSE => c -= 1,
                _ => {}
            }
            if c == 0 {
                else_stat = i;
                break;
            }
        }
        if else_stat != 0 {
            if else_stat == start + 1 || else_stat == end - 1 {
                return self.fail(CODENOTFOUND);
            }
        } else if start == end - 1 {
            return self.fail(CODENOTFOUND);
        }
        if self.cond_value(head_text(line, start), st::IF)? {
            self.exec_string_locals(line, start + 1, if else_stat != 0 { else_stat } else { end }, jump)
        } else if else_stat != 0 {
            self.exec_string_locals(line, else_stat + 1, end, jump)
        } else {
            Ok(false)
        }
    }

    /// FOR-Initialisierung: liefert (Variablenname, gesicherte Daten, to-Offset, step-Offset).
    fn prepare_for(&mut self, p: &str) -> R<(String, crate::vars::VarData, usize, Option<usize>)> {
        let eq = match str_pos(p, "=", false) {
            Some(e) => e,
            None => return self.fail(EQNOTFOUND),
        };
        let name = del_spc(&p[..eq]).to_string();
        if name.contains('[') {
            return self.fail(NOTCORRECTNAME);
        }
        let u = upper(&p[eq..]);
        let to_rel = match str_pos(&u[1..], "TO", true) {
            Some(t) => t + 1,
            None => return self.fail(TONOTFOUND),
        };
        let to = eq + to_rel;
        let step = str_pos(&u[to_rel + 2..], "STEP", true).map(|s| eq + to_rel + 2 + s);
        let old = self.refresh_count;
        let mut init = self.expr_value(&p[eq + 1..to])?;
        if self.refresh_count != old {
            return Err(Abort::Refresh);
        }
        if init.conv_to(false) {
            return self.fail(TYPEMISMATCH);
        }
        let vi = self.var_ref(&name, true)?;
        let local = self.take_var(vi);
        init.conv_to(name.starts_with('$'));
        self.set_var_value_by_ref(vi, 0, &init);
        Ok((name, local, to, step))
    }
    /// `true` = Schleife beendet.
    fn check_for(&mut self, name: &str, p: &str, to: usize, step: Option<usize>, cur_step: &mut i32) -> R<bool> {
        let old = self.refresh_count;
        let mut to_val;
        if let Some(sp) = step {
            let mut sv = self.expr_value(&p[sp + 4..])?;
            if self.refresh_count != old {
                return Err(Abort::Refresh);
            }
            if sv.conv_to(false) {
                return self.fail(TYPEMISMATCH);
            }
            *cur_step = sv.num();
            to_val = self.expr_value(&p[to + 2..sp])?;
        } else {
            to_val = self.expr_value(&p[to + 2..])?;
        }
        if self.refresh_count != old {
            return Err(Abort::Refresh);
        }
        if to_val.conv_to(false) {
            return self.fail(TYPEMISMATCH);
        }
        let vi = self.var_ref(name, true)?;
        let mut cur = self.get_var_value_by_ref(vi, 0, name.starts_with('$'));
        if cur.conv_to(false) {
            return self.fail(TYPEMISMATCH);
        }
        Ok(if *cur_step >= 0 { cur.num() > to_val.num() } else { cur.num() < to_val.num() })
    }
    fn end_for(&mut self, name: &str, cur_step: i32) -> R<()> {
        let vi = self.var_ref(name, true)?;
        let is_str = name.starts_with('$');
        let mut cur = self.get_var_value_by_ref(vi, 0, is_str);
        if cur.conv_to(false) {
            return self.fail(TYPEMISMATCH);
        }
        let mut n = Val::Num(cur.num().wrapping_add(cur_step));
        n.conv_to(is_str);
        self.set_var_value_by_ref(vi, 0, &n);
        Ok(())
    }
    fn finish_for(&mut self, name: &str, local: crate::vars::VarData) -> R<bool> {
        let vi = self.var_ref(name, true)?;
        self.put_var(vi, local);
        Ok(false)
    }

    fn stat_singleline_for(&mut self, line: &Line, start: usize, end: usize, jump: &mut String) -> R<bool> {
        if at(line.text.as_bytes(), line.stats[start].end) != b':' {
            return self.fail(COLONNOTFOUND);
        }
        let p = head_text(line, start);
        let (name, local, to, step) = self.prepare_for(p)?;
        let mut cur_step = 1;
        loop {
            if self.check_for(&name, p, to, step, &mut cur_step)? {
                break;
            }
            let is_exit = self.exec_string_locals(line, start + 1, end, jump)?;
            if is_exit || !jump.is_empty() {
                break;
            }
            self.end_for(&name, cur_step)?;
        }
        self.finish_for(&name, local)
    }
    fn stat_multiline_for(&mut self, s: &[Rc<Line>], end_line: usize, ind: usize, off: i32, jump: &mut String) -> R<bool> {
        let line = &s[ind];
        let p = head_text(line, 0);
        let (name, local, to, step) = self.prepare_for(p)?;
        let mut cur_step = 1;
        let old = self.refresh_count;
        loop {
            if self.check_for(&name, p, to, step, &mut cur_step)? {
                break;
            }
            let is_exit = self.exec_block_locals(s, ind + 1, end_line, off, Some(&mut *jump))?;
            if is_exit || !jump.is_empty() {
                break;
            }
            if off > 0 {
                self.real_line = line.line_num + off;
                if self.is_debug {
                    self.call_debug(&line.text);
                    if self.refresh_count != old {
                        return Err(Abort::Refresh);
                    }
                }
            }
            self.end_for(&name, cur_step)?;
        }
        self.finish_for(&name, local)
    }

    fn stat_local(&mut self, s: &str) -> R<()> {
        let list = self.get_vars_list(s)?;
        if !list.is_empty() {
            let gi = self.groups.len() - 1;
            self.groups[gi].extend(list);
        }
        Ok(())
    }

    // ---------------------------------------------------------------- ACT
    pub(crate) fn act_index(&self, name: &str) -> Option<usize> {
        let u = upper(name);
        self.cur_actions.iter().position(|a| upper(&a.desc) == u)
    }
    pub(crate) fn add_action(&mut self, args: &[Val], code: Vec<Rc<Line>>, is_manage_lines: bool) -> R<()> {
        if self.act_index(args[0].s()).is_some() {
            return Ok(());
        }
        if self.cur_actions.len() == MAXACTIONS {
            return self.fail(CANTADDACTION);
        }
        let image = if args.len() == 2 && is_any_string(args[1].s()) { Some(args[1].s().to_string()) } else { None };
        self.cur_actions.push(CurAct {
            image,
            desc: args[0].s().to_string(),
            code,
            location: self.real_cur_loc,
            act_index: self.real_act_index,
            start_line: self.real_line,
            is_manage_lines,
        });
        self.actions_changed = true;
        Ok(())
    }
    fn stat_singleline_add_act(&mut self, line: &Line, stat_pos: usize, end_pos: usize) -> R<()> {
        let colon = line.stats[stat_pos].end;
        if at(line.text.as_bytes(), colon) != b':' {
            return self.fail(COLONNOTFOUND);
        }
        if stat_pos == end_pos - 1 {
            return self.fail(CODENOTFOUND);
        }
        let old = self.refresh_count;
        let args = self.get_stat_args(head_text(line, stat_pos), st::ACT)?;
        if self.refresh_count != old {
            return Err(Abort::Refresh);
        }
        let start = colon + 1;
        let mut cut = line.stats[end_pos - 1].end;
        if at(line.text.as_bytes(), cut) == b':' {
            cut += 1;
        }
        let cut = cut.min(line.text.len());
        let stats: Vec<Stat> = (stat_pos + 1..end_pos)
            .map(|k| {
                let s = &line.stats[k];
                Stat { kind: s.kind, end: s.end.saturating_sub(start), param: s.param.saturating_sub(start) }
            })
            .collect();
        let sub = Line {
            text: line.text[start..cut].to_string(),
            line_num: 0,
            is_multiline: false,
            label: get_line_label(&line.text[start..]),
            stats,
        };
        self.add_action(&args, vec![Rc::new(sub)], false)
    }
    fn stat_multiline_add_act(&mut self, s: &[Rc<Line>], end_line: usize, ind: usize, manage: bool) -> R<()> {
        let line = &s[ind];
        let old = self.refresh_count;
        let args = self.get_stat_args(head_text(line, 0), st::ACT)?;
        if self.refresh_count != old {
            return Err(Abort::Refresh);
        }
        self.add_action(&args, s[ind + 1..end_line].to_vec(), manage)
    }
    pub(crate) fn clear_actions(&mut self) {
        if !self.cur_actions.is_empty() {
            self.cur_actions.clear();
            self.actions_changed = true;
        }
        self.cur_sel_action = -1;
    }
    pub(crate) fn exec_action(&mut self, ind: usize) -> R<()> {
        let (ol, oa, oline) = (self.real_cur_loc, self.real_act_index, self.real_line);
        let (loc, ai, start, manage, code) = {
            let a = &self.cur_actions[ind];
            (a.location, a.act_index, a.start_line, a.is_manage_lines, a.code.clone())
        };
        self.real_cur_loc = loc;
        self.real_act_index = ai;
        let r = if manage {
            self.exec_top_code(&code, start + 1, false)
        } else {
            self.real_line = start;
            self.exec_top_code(&code, 0, false)
        };
        self.real_line = oline;
        self.real_act_index = oa;
        self.real_cur_loc = ol;
        r.map(|_| ())
    }
    /// CURACTS: alle Aktionen als QSP-Code.
    pub(crate) fn get_all_actions_as_code(&self) -> String {
        let mut res = String::new();
        for a in &self.cur_actions {
            res.push_str("ACT '");
            res.push_str(&a.desc.replace('\'', "''"));
            if let Some(img) = &a.image {
                res.push_str("','");
                res.push_str(&img.replace('\'', "''"));
            }
            res.push_str("':");
            let n = a.code.len();
            if n == 1 && is_any_string(&a.code[0].text) {
                res.push_str(&a.code[0].text);
            } else {
                if n >= 2 {
                    res.push_str(STRSDELIM);
                    res.push_str(&crate::code::join_lines(&a.code, STRSDELIM));
                }
                res.push_str(STRSDELIM);
                res.push_str("END");
            }
            res.push_str(STRSDELIM);
        }
        res
    }

    // ---------------------------------------------------------------- Locations
    pub(crate) fn prepare_locs(&mut self) {
        let mut v: Vec<(String, usize)> = self.locs.iter().enumerate().map(|(i, l)| (upper(&l.name), i)).collect();
        v.sort_by(|a, b| a.0.cmp(&b.0)); // stabil wie glibc-qsort (Merge-Sort)
        self.loc_names = v;
    }
    pub(crate) fn loc_index(&self, name: &str) -> i32 {
        if self.locs.is_empty() || name.is_empty() {
            return -1;
        }
        let u = upper(del_spc(name));
        match bsearch(&self.loc_names, |e| cmp_str(&u, &e.0)) {
            Some(i) => self.loc_names[i].1 as i32,
            None => -1,
        }
    }
    pub(crate) fn obj_index(&self, name: &str) -> i32 {
        let u = upper(name);
        self.cur_objects.iter().position(|o| upper(&o.desc) == u).map_or(-1, |i| i as i32)
    }

    pub(crate) fn exec_loc_by_index(&mut self, ind: usize, change_desc: bool, is_new_loc: bool) -> R<()> {
        let (ol, oa, oline) = (self.real_cur_loc, self.real_act_index, self.real_line);
        self.real_cur_loc = ind as i32;
        self.real_act_index = -1;
        self.real_line = 0;
        let r = self.exec_loc_inner(ind, change_desc, is_new_loc);
        self.real_line = oline;
        self.real_act_index = oa;
        self.real_cur_loc = ol;
        r
    }
    fn exec_loc_inner(&mut self, ind: usize, change_desc: bool, is_new_loc: bool) -> R<()> {
        let desc = self.locs[ind].desc.clone();
        let s = self.format_text(&desc)?;
        if change_desc {
            self.cur_desc = s;
            self.main_desc_changed = true;
        } else if !s.is_empty() {
            self.cur_desc.push_str(&s);
            self.main_desc_changed = true;
        }
        for i in 0..MAXACTIONS {
            let (adesc, aimg, acode) = match self.locs[ind].actions.get(i) {
                Some(a) if !a.desc.is_empty() => (a.desc.clone(), a.image.clone(), a.code.clone()),
                _ => break,
            };
            let d = self.format_text(&adesc)?;
            self.real_act_index = i as i32;
            let mut args = vec![Val::Str(d)];
            if let Some(img) = aimg {
                if !img.is_empty() {
                    args.push(Val::Str(img));
                }
            }
            self.add_action(&args, acode, true)?;
        }
        self.real_act_index = -1;
        let code = self.locs[ind].on_visit.clone();
        self.exec_top_code(&code, 1, is_new_loc)?;
        Ok(())
    }
    pub(crate) fn exec_loc_by_name(&mut self, name: &str, change_desc: bool) -> R<()> {
        let i = self.loc_index(name);
        if i < 0 {
            return self.fail(LOCNOTFOUND);
        }
        self.exec_loc_by_index(i as usize, change_desc, false)
    }
    pub(crate) fn exec_loc_by_name_with_args(&mut self, name: &str, args: &[Val]) -> R<()> {
        let vi = self.var_ref("ARGS", true)?;
        let local = self.take_var(vi);
        self.set_args(vi, args);
        self.exec_loc_by_name(name, false)?;
        let vi = self.var_ref("ARGS", true)?;
        self.put_var(vi, local);
        Ok(())
    }
    /// Führt alle Orte aus, deren Namen in der Variable stehen (ONNEWLOC, COUNTER, …).
    pub(crate) fn exec_loc_by_var_name_with_args(&mut self, name: &str, args: &[Val]) -> R<()> {
        let mut ind = 0usize;
        loop {
            let vi = self.var_ref(name, false)?;
            let loc = match self.vars[vi].data.values.get(ind) {
                Some(v) => v.str.clone(),
                None => break,
            };
            match loc {
                Some(l) if is_any_string(&l) => self.exec_loc_by_name_with_args(&l, args)?,
                _ => break,
            }
            ind += 1;
        }
        Ok(())
    }
    /// Wechsel zur aktuellen Location (GOTO/XGOTO/Spielstart). Endet immer mit `Err`
    /// (Refresh = Erfolg), damit alle Aufrufer abbrechen.
    pub(crate) fn refresh_cur_loc(&mut self, change_desc: bool, args: &[Val]) -> R<()> {
        let vi = self.var_ref("ARGS", true)?;
        self.empty_var(vi);
        self.set_args(vi, args);
        self.clear_actions();
        self.refresh_count += 1;
        if change_desc {
            self.full_refresh_count += 1;
        }
        let old = self.refresh_count;
        match self.exec_loc_by_index(self.cur_loc as usize, change_desc, true) {
            Err(Abort::Error) => return Err(Abort::Error),
            _ => {}
        }
        self.chk()?;
        if self.refresh_count == old {
            if let Err(Abort::Error) = self.exec_loc_by_var_name_with_args("ONNEWLOC", args) {
                return Err(Abort::Error);
            }
        }
        Err(Abort::Refresh)
    }

    // ---------------------------------------------------------------- Statement-Dispatch
    fn exec_stat(&mut self, code: u8, args: &[Val], jump: &mut String) -> R<bool> {
        let ext = stat_spec(code).ext;
        match code {
            st::ADDOBJ => self.stat_add_object(args)?,
            st::CLA | st::CLEAR | st::CLS | st::CMDCLEAR | st::MCLEAR | st::KILLALL | st::FREELIB => self.stat_clear(code)?,
            st::CLOSEALL | st::CLOSE => self.stat_close_file(args),
            st::COPYARR => self.stat_copy_arr(args)?,
            st::KILLVAR => self.stat_kill_var(args)?,
            st::DELACT => self.stat_del_act(args),
            st::DELOBJ | st::KILLOBJ => self.stat_del_obj(ext, args)?,
            st::DYNAMIC => self.exec_string_as_code_with_args(args[0].s(), &args[1..])?,
            st::EXEC => self.call_system(args[0].s()),
            st::EXIT => return Ok(true),
            st::GOSUB => self.exec_loc_by_name_with_args(args[0].s(), &args[1..])?,
            st::GOTO | st::XGOTO => {
                let i = self.loc_index(args[0].s());
                if i < 0 {
                    return self.fail(LOCNOTFOUND);
                }
                self.cur_loc = i;
                self.refresh_cur_loc(ext != 0, &args[1..])?;
            }
            st::INCLIB => self.include_file(args[0].s())?,
            st::OPENQST => {
                if is_any_string(args[0].s()) {
                    let f = self.abs_from_rel_path(args[0].s());
                    self.open_quest(&f, false)?;
                    self.new_game(false)?;
                }
            }
            st::JUMP => {
                *jump = upper(del_spc(args[0].s()));
            }
            st::MENU => self.stat_show_menu(args)?,
            st::MNL | st::MPL | st::MP | st::NL | st::PL | st::P => self.stat_add_text(ext, args),
            st::MSG => self.call_show_message(args[0].s()),
            st::OPENGAME => {
                if !args.is_empty() && is_any_string(args[0].s()) {
                    let f = self.abs_from_rel_path(args[0].s());
                    self.call_open_game(Some(&f));
                } else {
                    self.call_open_game(None);
                }
                self.chk()?;
            }
            st::SAVEGAME => {
                if !args.is_empty() && is_any_string(args[0].s()) {
                    let f = self.abs_from_rel_path(args[0].s());
                    self.call_save_game(Some(&f));
                } else {
                    self.call_save_game(None);
                }
                self.chk()?;
            }
            st::PLAY => self.play_file(args[0].s(), if args.len() == 2 { args[1].num() } else { 100 }, true),
            st::REFINT => self.call_refresh_int(true),
            st::SETTIMER => {
                let n = args[0].num().max(0);
                self.timer_interval = n;
                self.call_set_timer(n);
            }
            st::SHOWACTS | st::SHOWOBJS | st::SHOWVARS | st::SHOWINPUT => {
                let v = args[0].num() != 0;
                match code {
                    st::SHOWACTS => {
                        self.show_acts = v;
                        self.call_show_window(Win::Acts, v)
                    }
                    st::SHOWOBJS => {
                        self.show_objs = v;
                        self.call_show_window(Win::Objs, v)
                    }
                    st::SHOWVARS => {
                        self.show_vars = v;
                        self.call_show_window(Win::Vars, v)
                    }
                    _ => {
                        self.show_input = v;
                        self.call_show_window(Win::Input, v)
                    }
                }
            }
            st::UNSELECT => self.cur_sel_object = -1,
            st::VIEW => self.stat_view(args),
            st::WAIT => {
                self.call_refresh_int(true);
                self.call_sleep(args[0].num().max(0));
            }
            _ => {}
        }
        Ok(false)
    }

    fn stat_add_text(&mut self, ext: i32, args: &[Val]) {
        let text = args.first().map(|a| a.s()).unwrap_or("");
        match ext {
            0 => {
                if !text.is_empty() {
                    self.cur_vars.push_str(text);
                    self.vars_desc_changed = true;
                }
            }
            1 => {
                if !text.is_empty() {
                    self.cur_desc.push_str(text);
                    self.main_desc_changed = true;
                }
            }
            2 => {
                self.cur_vars.push_str(text);
                self.cur_vars.push_str(STRSDELIM);
                self.vars_desc_changed = true;
            }
            3 => {
                self.cur_desc.push_str(text);
                self.cur_desc.push_str(STRSDELIM);
                self.main_desc_changed = true;
            }
            4 => {
                self.cur_vars.push_str(STRSDELIM);
                self.cur_vars.push_str(text);
                self.vars_desc_changed = true;
            }
            5 => {
                self.cur_desc.push_str(STRSDELIM);
                self.cur_desc.push_str(text);
                self.main_desc_changed = true;
            }
            _ => {}
        }
    }

    fn stat_clear(&mut self, code: u8) -> R<()> {
        match code {
            st::CLEAR => {
                if !self.cur_vars.is_empty() {
                    self.cur_vars.clear();
                    self.vars_desc_changed = true;
                }
            }
            st::MCLEAR => {
                if !self.cur_desc.is_empty() {
                    self.cur_desc.clear();
                    self.main_desc_changed = true;
                }
            }
            st::CMDCLEAR => {
                self.cur_input.clear();
                self.call_set_input_str_text(None);
            }
            st::CLA => self.clear_actions(),
            st::CLS => {
                if !self.cur_vars.is_empty() {
                    self.cur_vars.clear();
                    self.vars_desc_changed = true;
                }
                if !self.cur_desc.is_empty() {
                    self.cur_desc.clear();
                    self.main_desc_changed = true;
                }
                self.cur_input.clear();
                self.clear_actions();
                self.call_set_input_str_text(None);
            }
            st::KILLALL => {
                self.clear_vars();
                self.clear_objects_with_notify()?;
            }
            st::FREELIB => {
                self.clear_includes();
                if self.cur_loc >= self.locs.len() as i32 {
                    self.cur_loc = -1;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn stat_view(&mut self, args: &[Val]) {
        if !args.is_empty() && is_any_string(args[0].s()) {
            let p = args[0].s().to_string();
            self.view_path = Some(p.clone());
            self.call_show_picture(Some(&p));
        } else {
            self.view_path = None;
            self.call_show_picture(None);
        }
    }

    fn stat_show_menu(&mut self, args: &[Val]) -> R<()> {
        let (vi, _) = self.var_ref_typed(args[0].s(), false)?;
        self.call_delete_menu();
        let (mut ind, max_items) = match args.len() {
            1 => (0i32, MAXMENUITEMS as i32),
            2 => (args[1].num().max(0), MAXMENUITEMS as i32),
            _ => (args[1].num().max(0), args[2].num().max(0)),
        };
        let mut locs: Vec<String> = Vec::new();
        loop {
            if ind as usize >= self.vars[vi].data.values.len() {
                break;
            }
            if locs.len() as i32 == max_items {
                break;
            }
            let s = match &self.vars[vi].data.values[ind as usize].str {
                Some(s) if is_any_string(s) => s.clone(),
                _ => break,
            };
            let pos2 = match in_str_r_chars(&s, b":", None) {
                Some(p) => p,
                None => return self.fail(COLONNOTFOUND),
            };
            if locs.len() == MAXMENUITEMS {
                return self.fail(CANTADDMENUITEM);
            }
            let (name, loc, img): (&str, &str, Option<&str>);
            if let Some(pos) = in_str_r_chars(&s, b":", Some(pos2)) {
                let rest = &s[pos2 + 1..];
                img = if is_any_string(rest) { Some(rest) } else { None };
                loc = &s[pos + 1..pos2];
                name = &s[..pos];
            } else {
                img = None;
                loc = &s[pos2 + 1..];
                name = &s[..pos2];
            }
            locs.push(loc.to_string());
            self.call_add_menu_item(name, img);
            ind += 1;
        }
        if !locs.is_empty() {
            let sel = self.call_show_menu();
            self.chk()?;
            if sel >= 0 && (sel as usize) < locs.len() {
                self.exec_loc_by_name_with_args(&locs[sel as usize], &[Val::Num(sel + 1)])?;
            }
        }
        Ok(())
    }
}
