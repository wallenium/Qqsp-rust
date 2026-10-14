//! Objekte, Playlist, Spiel laden/speichern, Includes – Port von objects.c / playlist.c / game.c.
use crate::code::preprocess;
use crate::engine::*;
use crate::errors::*;
use crate::gamefile::*;
use crate::host::Win;
use crate::text::*;
use crate::value::*;
use crate::vars::{VarData, VarIndex, VarValue};
use std::rc::Rc;

const GAMEID: &str = "QSPGAME";
const SAVEDGAMEID: &str = "QSPSAVEDGAME";
const GAMEMINVER: &str = "5.7.0";
const PATHDELIMS: &[u8] = b"/";

impl Engine {
    // ---------------------------------------------------------------- Objekte
    pub(crate) fn clear_objects(&mut self) {
        if !self.cur_objects.is_empty() {
            self.cur_objects.clear();
            self.objects_changed = true;
        }
        self.cur_sel_object = -1;
    }
    pub(crate) fn clear_objects_with_notify(&mut self) -> R<()> {
        let old: Vec<String> = self.cur_objects.iter().map(|o| o.desc.clone()).collect();
        if old.is_empty() {
            return Ok(());
        }
        self.clear_objects();
        for d in old {
            self.exec_loc_by_var_name_with_args("ONOBJDEL", &[Val::Str(d)])?;
        }
        Ok(())
    }
    fn remove_object(&mut self, index: i32) -> R<()> {
        if index < 0 || index as usize >= self.cur_objects.len() {
            return Ok(());
        }
        if self.cur_sel_object >= index {
            self.cur_sel_object = -1;
        }
        let o = self.cur_objects.remove(index as usize);
        self.objects_changed = true;
        self.exec_loc_by_var_name_with_args("ONOBJDEL", &[Val::Str(o.desc)])
    }
    pub(crate) fn stat_add_object(&mut self, args: &[Val]) -> R<()> {
        let count = self.cur_objects.len() as i32;
        let ind = if args.len() == 3 {
            let i = args[2].num().wrapping_sub(1);
            if i < 0 || i > count {
                return Ok(());
            }
            i
        } else {
            count
        };
        if self.cur_objects.len() == MAXOBJECTS {
            return self.fail(CANTADDOBJECT);
        }
        if self.cur_sel_object >= ind {
            self.cur_sel_object = -1;
        }
        let image = if args.len() >= 2 && is_any_string(args[1].s()) { Some(args[1].s().to_string()) } else { None };
        self.cur_objects.insert(ind as usize, Obj { image, desc: args[0].s().to_string() });
        self.objects_changed = true;
        let n = args.len().min(2);
        self.exec_loc_by_var_name_with_args("ONOBJADD", &args[..n])
    }
    pub(crate) fn stat_del_obj(&mut self, ext: i32, args: &[Val]) -> R<()> {
        if ext == 0 {
            let i = self.obj_index(args[0].s());
            self.remove_object(i)
        } else if !args.is_empty() {
            self.remove_object(args[0].num().wrapping_sub(1))
        } else {
            self.clear_objects_with_notify()
        }
    }
    pub(crate) fn stat_del_act(&mut self, args: &[Val]) {
        if let Some(i) = self.act_index(args[0].s()) {
            if self.cur_sel_action >= i as i32 {
                self.cur_sel_action = -1;
            }
            self.cur_actions.remove(i);
            self.actions_changed = true;
        }
    }

    // ---------------------------------------------------------------- Playlist
    pub(crate) fn play_file(&mut self, s: &str, volume: i32, add: bool) {
        if !is_any_string(s) {
            return;
        }
        let volume = volume.clamp(0, 100);
        self.call_play_file(s, volume);
        if add {
            if self.pl_files.len() == MAXPLFILES {
                self.refresh_play_list();
                if self.pl_files.len() == MAXPLFILES {
                    return;
                }
            }
            let mut f = s.to_string();
            if volume != 100 {
                f.push('*');
                f.push_str(&volume.to_string());
            }
            self.pl_files.push(f);
        }
    }
    fn search_play_list(&self, file: &str) -> Option<usize> {
        let u = upper(file);
        self.pl_files.iter().position(|p| {
            let pu = upper(p);
            pu.starts_with(&u) && is_in_list_eol(b"*", at(pu.as_bytes(), u.len()))
        })
    }
    pub(crate) fn play_pl_files(&mut self) {
        let list = self.pl_files.clone();
        for p in list {
            match p.find('*') {
                Some(pos) => self.play_file(&p[..pos], str_to_num_raw(&p[pos + 1..]), false),
                None => self.play_file(&p, 100, false),
            }
        }
    }
    pub(crate) fn refresh_play_list(&mut self) {
        if self.pl_files.is_empty() {
            return;
        }
        let list = std::mem::take(&mut self.pl_files);
        for item in list.iter().rev() {
            let name = match item.find('*') {
                Some(p) => &item[..p],
                None => item.as_str(),
            };
            if is_any_string(name) && self.search_play_list(name).is_none() && self.call_is_playing(name) {
                self.pl_files.push(item.clone());
            }
        }
    }
    pub(crate) fn stat_close_file(&mut self, args: &[Val]) {
        if self.pl_files.is_empty() {
            return;
        }
        if !args.is_empty() {
            let name = args[0].s();
            if is_any_string(name) {
                if let Some(mut pos) = self.search_play_list(name) {
                    self.call_close_file(Some(name));
                    loop {
                        self.pl_files.remove(pos);
                        match self.search_play_list(name) {
                            Some(p) => pos = p,
                            None => break,
                        }
                    }
                }
            }
        } else {
            self.pl_files.clear();
            self.call_close_file(None);
        }
    }

    // ---------------------------------------------------------------- Speicher zurücksetzen
    pub(crate) fn mem_clear(&mut self) {
        self.clear_includes();
        self.clear_vars();
        self.clear_objects();
        self.clear_actions();
        self.pl_files.clear();
        if !self.cur_desc.is_empty() {
            self.main_desc_changed = true;
        }
        self.cur_desc.clear();
        if !self.cur_vars.is_empty() {
            self.vars_desc_changed = true;
        }
        self.cur_vars.clear();
        self.cur_input.clear();
        self.view_path = None;
        self.groups.clear();
    }
    pub(crate) fn prepare_execution(&mut self) {
        self.reset_error();
        self.main_desc_changed = false;
        self.vars_desc_changed = false;
        self.objects_changed = false;
        self.actions_changed = false;
    }

    // ---------------------------------------------------------------- Includes / Pfade
    pub(crate) fn abs_from_rel_path(&self, p: &str) -> String {
        format!("{}{}", self.qst_path, p)
    }
    pub(crate) fn clear_includes(&mut self) {
        self.inc_files.clear();
        if self.cur_inc_locs_count > 0 {
            let keep = self.locs.len().saturating_sub(self.cur_inc_locs_count);
            self.locs.truncate(keep);
            self.prepare_locs();
        }
        self.cur_inc_locs_count = 0;
    }
    pub(crate) fn include_file(&mut self, s: &str) -> R<()> {
        if !is_any_string(s) {
            return Ok(());
        }
        if self.inc_files.len() == MAXINCFILES {
            return self.fail(CANTINCFILE);
        }
        let old = self.cur_inc_locs_count;
        let f = self.abs_from_rel_path(s);
        self.open_quest(&f, true)?;
        if self.cur_inc_locs_count != old {
            self.inc_files.push(s.to_string());
        }
        Ok(())
    }
    fn open_includes(&mut self) -> R<()> {
        for f in self.inc_files.clone() {
            let p = self.abs_from_rel_path(&f);
            self.open_quest(&p, true)?;
        }
        Ok(())
    }

    // ---------------------------------------------------------------- Neues Spiel
    pub(crate) fn new_game(&mut self, is_reset: bool) -> R<()> {
        if self.locs.is_empty() {
            return self.fail(GAMENOTLOADED);
        }
        self.cur_loc = 0;
        if is_reset {
            let t = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as u32)
                .unwrap_or(0);
            self.rand.seed(t);
            self.timer_interval = DEFTIMERINTERVAL;
            self.show_objs = true;
            self.show_acts = true;
            self.show_vars = true;
            self.show_input = true;
            self.mem_clear();
            self.reset_time(0);
            self.call_show_window(Win::Acts, true);
            self.call_show_window(Win::Objs, true);
            self.call_show_window(Win::Vars, true);
            self.call_show_window(Win::Input, true);
            self.call_set_input_str_text(None);
            self.call_show_picture(None);
            self.call_close_file(None);
            self.call_set_timer(DEFTIMERINTERVAL);
        }
        self.refresh_cur_loc(true, &[])
    }

    // ---------------------------------------------------------------- Spieldatei laden
    pub(crate) fn open_quest(&mut self, file: &str, add_locs: bool) -> R<()> {
        match std::fs::read(file) {
            Ok(data) => self.open_quest_from_data(&data, file, add_locs),
            Err(_) => self.fail(FILENOTFOUND),
        }
    }

    pub(crate) fn open_quest_from_data(&mut self, data: &[u8], file_name: &str, add_locs: bool) -> R<()> {
        let mut padded = data.to_vec();
        padded.extend_from_slice(&[0, 0, 0]);
        if padded.len() < 2 {
            return self.fail(CANTLOADFILE);
        }
        let file_crc = if add_locs { 0 } else { crc(&padded) };
        let ucs2 = padded[1] == 0;
        let strs = split_game(&padded, ucs2);
        let locs = match parse_game(&strs, ucs2) {
            Some(l) => l,
            None => return self.fail(CANTLOADFILE),
        };
        if add_locs {
            let mut added = 0;
            for l in locs {
                if self.loc_index(&l.name) < 0 {
                    self.locs.push(l);
                    added += 1;
                }
            }
            if added > 0 {
                self.prepare_locs();
            }
            self.cur_inc_locs_count += added;
        } else {
            self.clear_includes();
            self.locs = locs;
            self.prepare_locs();
            self.qst_full_path = file_name.to_string();
            let plen = in_str_r_chars(file_name, PATHDELIMS, None).map_or(0, |p| p + 1);
            self.qst_path = file_name[..plen].to_string();
            self.qst_crc = file_crc;
            self.cur_loc = -1;
        }
        Ok(())
    }

    // ---------------------------------------------------------------- Spielstand speichern
    pub(crate) fn save_game_status_to_bytes(&mut self) -> R<Vec<u8>> {
        let old = self.refresh_count;
        self.exec_loc_by_var_name_with_args("ONGSAVE", &[])?;
        if self.refresh_count != old {
            return Err(Abort::Refresh);
        }
        let saved = self.prepare_local_vars()?;
        self.refresh_play_list();
        let mut w = SaveWriter::default();
        let loc_name = if self.cur_loc >= 0 { Some(self.locs[self.cur_loc as usize].name.clone()) } else { None };
        w.val(Some(SAVEDGAMEID), false);
        w.val(Some(QSP_VER), false);
        w.int(self.qst_crc, true);
        let t = self.get_time();
        w.int(t, true);
        w.int(self.cur_sel_action, true);
        w.int(self.cur_sel_object, true);
        w.val(self.view_path.as_deref(), true);
        w.val(Some(&self.cur_input), true);
        w.val(Some(&self.cur_desc), true);
        w.val(Some(&self.cur_vars), true);
        w.val(loc_name.as_deref(), true);
        w.int(self.show_acts as i32, true);
        w.int(self.show_objs as i32, true);
        w.int(self.show_vars as i32, true);
        w.int(self.show_input as i32, true);
        w.int(self.timer_interval, true);
        w.int(self.pl_files.len() as i32, true);
        for p in &self.pl_files {
            w.val(Some(p), true);
        }
        w.int(self.inc_files.len() as i32, true);
        for p in &self.inc_files {
            w.val(Some(p), true);
        }
        w.int(self.cur_actions.len() as i32, true);
        for a in &self.cur_actions {
            match &a.image {
                Some(i) => w.val(Some(i), true),
                None => w.val(None, false),
            }
            w.val(Some(&a.desc), true);
            w.int(a.code.len() as i32, true);
            for l in &a.code {
                w.val(Some(&l.text), true);
                w.int(l.line_num, true);
            }
            w.int(a.location, true);
            w.int(a.act_index, true);
            w.int(a.start_line, true);
            w.int(a.is_manage_lines as i32, true);
        }
        w.int(self.cur_objects.len() as i32, true);
        for o in &self.cur_objects {
            match &o.image {
                Some(i) => w.val(Some(i), true),
                None => w.val(None, false),
            }
            w.val(Some(&o.desc), true);
        }
        w.int(self.vars_count() as i32, true);
        for (i, s) in self.vars.iter().enumerate() {
            if let Some(name) = &s.name {
                w.int(i as i32, true);
                w.val(Some(name), true);
                w.int(s.data.values.len() as i32, true);
                for v in &s.data.values {
                    w.int(v.num, true);
                    w.val(v.str.as_deref(), true);
                }
                w.int(s.data.indices.len() as i32, true);
                for ind in &s.data.indices {
                    w.int(ind.index, true);
                    w.val(Some(&ind.str), true);
                }
            }
        }
        let mut g = std::mem::take(&mut self.groups);
        let r = self.restore_local_vars(saved, &mut g);
        self.groups = g;
        r?;
        self.chk()?;
        Ok(w.into_bytes())
    }

    // ---------------------------------------------------------------- Spielstand laden
    fn check_game_status(&mut self, s: &[Vec<u16>]) -> bool {
        let n = s.len();
        let plain = |i: usize| String::from_utf16_lossy(&s[i]);
        let int = |i: usize| str_to_num_raw(&String::from_utf16_lossy(&decode_units(&s[i])));
        let mut ind = 17usize;
        if ind > n {
            return false;
        }
        if plain(0) != SAVEDGAMEID || cmp_str(&plain(1), GAMEMINVER) < 0 || cmp_str(&plain(1), QSP_VER) > 0 {
            return false;
        }
        if self.get_var_num("DEBUG") == 0 && int(2) != self.qst_crc {
            return false;
        }
        let sel_action = int(4);
        let sel_object = int(5);
        if int(15) < 0 {
            return false;
        }
        let mut temp = int(16);
        if temp < 0 || temp as usize > MAXPLFILES {
            return false;
        }
        ind += temp as usize;
        if ind > n || ind + 1 > n {
            return false;
        }
        temp = int(ind);
        ind += 1;
        if temp < 0 || temp as usize > MAXINCFILES {
            return false;
        }
        ind += temp as usize;
        if ind > n || ind + 1 > n {
            return false;
        }
        let count = int(ind);
        ind += 1;
        if count < 0 || count as usize > MAXACTIONS || sel_action >= count {
            return false;
        }
        for _ in 0..count {
            ind += 2;
            if ind > n || ind + 1 > n {
                return false;
            }
            let lines = int(ind);
            ind += 1;
            if lines < 0 || ind + 2 * lines as usize > n {
                return false;
            }
            for _ in 0..lines {
                ind += 1;
                let l = int(ind);
                ind += 1;
                if l < 0 {
                    return false;
                }
            }
            if ind + 1 > n {
                return false;
            }
            let v = int(ind);
            ind += 1;
            if v < 0 {
                return false;
            }
            ind += 1;
            if ind > n || ind + 1 > n {
                return false;
            }
            let v = int(ind);
            ind += 1;
            if v < 0 {
                return false;
            }
            ind += 1;
            if ind > n {
                return false;
            }
        }
        if ind + 1 > n {
            return false;
        }
        temp = int(ind);
        ind += 1;
        if temp < 0 || temp as usize > MAXOBJECTS || sel_object >= temp {
            return false;
        }
        ind += 2 * temp as usize;
        if ind > n || ind + 1 > n {
            return false;
        }
        let count = int(ind);
        ind += 1;
        if count < 0 {
            return false;
        }
        let mut last = -1;
        for _ in 0..count {
            if ind + 1 > n {
                return false;
            }
            let t = int(ind);
            ind += 1;
            if t <= last || t as usize >= VARSCOUNT {
                return false;
            }
            last = t;
            ind += 1;
            if ind > n || ind + 1 > n {
                return false;
            }
            let t = int(ind);
            ind += 1;
            if t < 0 {
                return false;
            }
            ind += 2 * t as usize;
            if ind > n || ind + 1 > n {
                return false;
            }
            let t = int(ind);
            ind += 1;
            if t < 0 {
                return false;
            }
            ind += 2 * t as usize;
            if ind > n {
                return false;
            }
        }
        true
    }

    pub(crate) fn open_game_status_from_units(&mut self, units: &[u16]) -> R<()> {
        let s = split_units(units);
        if !self.check_game_status(&s) {
            return self.fail(CANTLOADFILE);
        }
        let dec = |i: usize| String::from_utf16_lossy(&decode_units(&s[i]));
        let int = |i: usize| str_to_num_raw(&dec(i));
        self.refresh_count += 1;
        self.full_refresh_count += 1;
        self.mem_clear();
        self.reset_time(int(3));
        self.cur_sel_action = int(4);
        self.cur_sel_object = int(5);
        if !s[6].is_empty() {
            self.view_path = Some(dec(6));
        }
        if !s[7].is_empty() {
            self.cur_input = dec(7);
        }
        if !s[8].is_empty() {
            self.cur_desc = dec(8);
        }
        if !s[9].is_empty() {
            self.cur_vars = dec(9);
        }
        let loc_name = dec(10);
        self.show_acts = int(11) != 0;
        self.show_objs = int(12) != 0;
        self.show_vars = int(13) != 0;
        self.show_input = int(14) != 0;
        self.timer_interval = int(15);
        let pl = int(16);
        let mut ind = 17usize;
        for _ in 0..pl {
            self.pl_files.push(dec(ind));
            ind += 1;
        }
        let inc = int(ind);
        ind += 1;
        for _ in 0..inc {
            self.inc_files.push(dec(ind));
            ind += 1;
        }
        let acts = int(ind);
        ind += 1;
        for _ in 0..acts {
            let image = if !s[ind].is_empty() { Some(dec(ind)) } else { None };
            ind += 1;
            let desc = dec(ind);
            ind += 1;
            let nlines = int(ind);
            ind += 1;
            let mut code = Vec::new();
            for _ in 0..nlines {
                let mut line = crate::code::init_line(dec(ind), 0);
                ind += 1;
                line.line_num = int(ind);
                ind += 1;
                code.push(Rc::new(line));
            }
            let location = int(ind);
            let act_index = int(ind + 1);
            let start_line = int(ind + 2);
            let is_manage_lines = int(ind + 3) != 0;
            ind += 4;
            self.cur_actions.push(CurAct { image, desc, code, location, act_index, start_line, is_manage_lines });
        }
        let objs = int(ind);
        ind += 1;
        for _ in 0..objs {
            let image = if !s[ind].is_empty() { Some(dec(ind)) } else { None };
            ind += 1;
            let desc = dec(ind);
            ind += 1;
            self.cur_objects.push(Obj { image, desc });
        }
        let nvars = int(ind);
        ind += 1;
        for _ in 0..nvars {
            let vi = int(ind) as usize;
            ind += 1;
            self.vars[vi].name = Some(dec(ind));
            ind += 1;
            let nvals = int(ind);
            ind += 1;
            let mut data = VarData::default();
            for _ in 0..nvals {
                let num = int(ind);
                ind += 1;
                let str = if !s[ind].is_empty() { Some(dec(ind)) } else { None };
                ind += 1;
                data.values.push(VarValue { num, str });
            }
            let ninds = int(ind);
            ind += 1;
            for _ in 0..ninds {
                let index = int(ind);
                ind += 1;
                let str = dec(ind);
                ind += 1;
                data.indices.push(VarIndex { index, str });
            }
            self.vars[vi].data = data;
        }
        self.main_desc_changed = true;
        self.vars_desc_changed = true;
        self.objects_changed = true;
        self.actions_changed = true;
        self.open_includes()?;
        self.cur_loc = self.loc_index(&loc_name);
        self.chk()?;
        let (a, o, v, i) = (self.show_acts, self.show_objs, self.show_vars, self.show_input);
        self.call_show_window(Win::Acts, a);
        self.call_show_window(Win::Objs, o);
        self.call_show_window(Win::Vars, v);
        self.call_show_window(Win::Input, i);
        let input = self.cur_input.clone();
        self.call_set_input_str_text(Some(&input));
        match self.view_path.clone() {
            Some(p) => self.call_show_picture(Some(&p)),
            None => self.call_show_picture(None),
        }
        self.call_close_file(None);
        self.play_pl_files();
        let t = self.timer_interval;
        self.call_set_timer(t);
        self.exec_loc_by_var_name_with_args("ONGLOAD", &[])
    }
}

/// qspCheckQuest + Parser: liefert die Locations oder `None` bei ungültigen Daten.
fn parse_game(strs: &[Vec<u16>], ucs2: bool) -> Option<Vec<Location>> {
    let count = strs.len();
    let plain = |i: usize| game_to_string(&strs[i], ucs2, false);
    let coded = |i: usize| game_to_string(&strs[i], ucs2, true);
    if count == 0 {
        return None;
    }
    let old = plain(0) != GAMEID;
    let mut ind = if old { 30 } else { 4 };
    if ind > count {
        return None;
    }
    let locs_count = str_to_num_raw(&if old { plain(0) } else { coded(3) });
    if locs_count <= 0 {
        return None;
    }
    // --- Prüfung ---
    let mut chk = ind;
    for _ in 0..locs_count {
        chk += 3;
        if chk > count {
            return None;
        }
        let acts = if old {
            20
        } else {
            if chk + 1 > count {
                return None;
            }
            let a = str_to_num_raw(&coded(chk));
            chk += 1;
            if a < 0 || a as usize > MAXACTIONS {
                return None;
            }
            a as usize
        };
        chk += acts * if old { 2 } else { 3 };
        if chk > count {
            return None;
        }
    }
    // --- Einlesen ---
    let mut locs = Vec::with_capacity(locs_count as usize);
    for _ in 0..locs_count {
        let name = coded(ind);
        ind += 1;
        let desc = coded(ind);
        ind += 1;
        let code = coded(ind);
        ind += 1;
        let on_visit: Code = preprocess(&code).into_iter().map(Rc::new).collect();
        let acts = if old {
            20
        } else {
            let a = str_to_num_raw(&coded(ind));
            ind += 1;
            a as usize
        };
        let mut actions = Vec::with_capacity(acts);
        for _ in 0..acts {
            let image = if old {
                None
            } else {
                let i = coded(ind);
                ind += 1;
                Some(i)
            };
            let adesc = coded(ind);
            ind += 1;
            let acode = coded(ind);
            ind += 1;
            actions.push(LocAct { image, desc: adesc, code: preprocess(&acode).into_iter().map(Rc::new).collect() });
        }
        locs.push(Location { name, desc, on_visit, actions });
    }
    Some(locs)
}
