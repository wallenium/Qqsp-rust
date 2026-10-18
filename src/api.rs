//! Öffentliche API (entspricht der libqsp-Schnittstelle).
use crate::engine::*;
use crate::errors::*;
use crate::host::Win;
use crate::value::Val;

pub struct ErrorInfo {
    pub num: i32,
    pub loc: Option<String>,
    pub act_index: i32,
    pub line: i32,
}

impl Engine {
    /// Gemeinsamer Rahmen: Fehlerzustand zurücksetzen, ausführen, `Refresh` = Erfolg.
    fn run(&mut self, refresh: bool, f: impl FnOnce(&mut Self) -> R<()>) -> bool {
        if self.exit_on_error && self.err.num != 0 {
            return false;
        }
        self.prepare_execution();
        if self.disable_code_exec {
            return false;
        }
        let r = f(self);
        let ok = !matches!(r, Err(Abort::Error)) && self.err.num == 0;
        if ok && refresh {
            self.call_refresh_int(false);
        }
        ok
    }

    pub fn version() -> &'static str {
        QSP_VER
    }
    pub fn enable_debug_mode(&mut self, on: bool) {
        self.is_debug = on;
    }
    /// Schaltet die additiven Erweiterungen aus QSP 5.9 ein (vor dem Laden eines Spiels aufrufen:
    /// Zeilen werden beim Laden vorverarbeitet).
    pub fn set_ext59(&mut self, on: bool) {
        self.ext59 = on;
    }
    pub fn ext59(&self) -> bool {
        self.ext59
    }
    pub fn disable_code_exec(&mut self, on: bool) {
        self.disable_code_exec = on;
    }

    // ---- Spiel laden / starten
    pub fn load_game_world(&mut self, path: &str) -> bool {
        self.run(false, |e| e.open_quest(path, false))
    }
    pub fn load_game_world_from_data(&mut self, data: &[u8], name: &str) -> bool {
        self.run(false, |e| e.open_quest_from_data(data, name, false))
    }
    pub fn restart_game(&mut self, refresh: bool) -> bool {
        self.run(refresh, |e| e.new_game(true))
    }

    // ---- Code ausführen
    pub fn exec_code_string(&mut self, s: &str, refresh: bool) -> bool {
        let s = s.to_string();
        self.run(refresh, |e| e.exec_string_as_code_with_args(&s, &[]))
    }
    pub fn exec_location_code(&mut self, name: &str, refresh: bool) -> bool {
        let n = name.to_string();
        self.run(refresh, |e| e.exec_loc_by_name(&n, false))
    }
    pub fn exec_counter(&mut self, refresh: bool) -> bool {
        if self.in_callback {
            return true;
        }
        self.run(refresh, |e| e.exec_loc_by_var_name_with_args("COUNTER", &[]))
    }
    pub fn exec_user_input(&mut self, refresh: bool) -> bool {
        self.run(refresh, |e| e.exec_loc_by_var_name_with_args("USERCOM", &[]))
    }
    pub fn set_sel_action_index(&mut self, ind: i32, refresh: bool) -> bool {
        if ind >= 0 && (ind as usize) < self.cur_actions.len() && ind != self.cur_sel_action {
            self.cur_sel_action = ind;
            return self.run(refresh, |e| e.exec_loc_by_var_name_with_args("ONACTSEL", &[]));
        }
        true
    }
    pub fn exec_selected_action_code(&mut self, refresh: bool) -> bool {
        self.run(refresh, |e| {
            if e.cur_sel_action >= 0 && (e.cur_sel_action as usize) < e.cur_actions.len() {
                e.exec_action(e.cur_sel_action as usize)
            } else {
                Ok(())
            }
        })
    }
    pub fn set_sel_object_index(&mut self, ind: i32, refresh: bool) -> bool {
        if ind >= 0 && (ind as usize) < self.cur_objects.len() && ind != self.cur_sel_object {
            self.cur_sel_object = ind;
            return self.run(refresh, |e| e.exec_loc_by_var_name_with_args("ONOBJSEL", &[]));
        }
        true
    }
    pub fn get_expr_value(&mut self, expr: &str) -> Option<Val> {
        let ex = expr.to_string();
        let mut out = None;
        if self.run(false, |e| {
            out = Some(e.expr_value(&ex)?);
            Ok(())
        }) {
            out
        } else {
            None
        }
    }

    // ---- Spielstände
    pub fn save_game_as_data(&mut self, refresh: bool) -> Option<Vec<u8>> {
        let mut out = None;
        if self.run(refresh, |e| {
            out = Some(e.save_game_status_to_bytes()?);
            Ok(())
        }) {
            out
        } else {
            None
        }
    }
    pub fn open_saved_game_from_data(&mut self, data: &[u8], refresh: bool) -> bool {
        let units: Vec<u16> = data.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        self.run(refresh, |e| e.open_game_status_from_units(&units))
    }
    pub fn save_game(&mut self, path: &str, refresh: bool) -> bool {
        match self.save_game_as_data(refresh) {
            Some(d) => std::fs::write(path, d).is_ok() || self.fail_io(),
            None => false,
        }
    }
    pub fn open_saved_game(&mut self, path: &str, refresh: bool) -> bool {
        match std::fs::read(path) {
            Ok(d) => self.open_saved_game_from_data(&d, refresh),
            Err(_) => self.fail_io(),
        }
    }
    fn fail_io(&mut self) -> bool {
        self.set_error(CANTLOADFILE);
        false
    }

    // ---- Zustand für die Oberfläche
    pub fn set_input_str_text(&mut self, s: &str) {
        self.cur_input = s.to_string();
    }
    pub fn show_window(&self, w: Win) -> bool {
        match w {
            Win::Acts => self.show_acts,
            Win::Objs => self.show_objs,
            Win::Vars => self.show_vars,
            Win::Input => self.show_input,
        }
    }
    pub fn main_desc(&self) -> (&str, bool) {
        (&self.cur_desc, self.main_desc_changed)
    }
    pub fn vars_desc(&self) -> (&str, bool) {
        (&self.cur_vars, self.vars_desc_changed)
    }
    pub fn actions(&self) -> (Vec<(String, Option<String>)>, bool) {
        (self.cur_actions.iter().map(|a| (a.desc.clone(), a.image.clone())).collect(), self.actions_changed)
    }
    pub fn objects(&self) -> (Vec<(String, Option<String>)>, bool) {
        (self.cur_objects.iter().map(|o| (o.desc.clone(), o.image.clone())).collect(), self.objects_changed)
    }
    pub fn sel_action_index(&self) -> i32 {
        self.cur_sel_action
    }
    pub fn sel_object_index(&self) -> i32 {
        self.cur_sel_object
    }
    pub fn cur_loc_name(&self) -> Option<&str> {
        self.locs.get(usize::try_from(self.cur_loc).ok()?).map(|l| l.name.as_str())
    }
    pub fn qst_full_path(&self) -> &str {
        &self.qst_full_path
    }
    pub fn full_refresh_count(&self) -> i32 {
        self.full_refresh_count
    }
    pub fn view_path(&self) -> Option<&str> {
        self.view_path.as_deref()
    }
    /// Reiner Lesezugriff auf `name[ind]` (auch innerhalb von Callbacks erlaubt, führt keinen Code aus).
    pub fn var_values(&mut self, name: &str, ind: usize) -> Option<(i32, Option<String>)> {
        let vi = self.var_ref(name, false).ok()?;
        self.vars[vi].data.values.get(ind).map(|v| (v.num, v.str.clone()))
    }
    /// Alle belegten Variablen als JSON (für das Debug-Fenster):
    /// `[{"n":"NAME","v":[[num,"text"|null],...],"k":{"schluessel":index,...}},...]`, nach Namen sortiert.
    pub fn vars_dump_json(&self) -> String {
        fn esc(s: &str, out: &mut String) {
            out.push('"');
            for c in s.chars() {
                match c {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
                    c => out.push(c),
                }
            }
            out.push('"');
        }
        let mut slots: Vec<&crate::vars::Slot> = self
            .vars
            .iter()
            .filter(|s| s.name.is_some() && (!s.data.values.is_empty() || !s.data.indices.is_empty()))
            .collect();
        slots.sort_by(|a, b| a.name.cmp(&b.name));
        let mut o = String::from("[");
        for (i, s) in slots.iter().enumerate() {
            if i > 0 {
                o.push(',');
            }
            o.push_str("{\"n\":");
            esc(s.name.as_deref().unwrap_or(""), &mut o);
            o.push_str(",\"v\":[");
            for (j, v) in s.data.values.iter().enumerate() {
                if j > 0 {
                    o.push(',');
                }
                o.push_str(&format!("[{},", v.num));
                match &v.str {
                    Some(t) => esc(t, &mut o),
                    None => o.push_str("null"),
                }
                o.push(']');
            }
            o.push_str("],\"k\":{");
            for (j, k) in s.data.indices.iter().enumerate() {
                if j > 0 {
                    o.push(',');
                }
                esc(&k.str, &mut o);
                o.push_str(&format!(":{}", k.index));
            }
            o.push_str("}}");
        }
        o.push(']');
        o
    }
    pub fn var_num(&mut self, name: &str) -> i32 {
        self.get_var_num(name)
    }
    pub fn var_value(&mut self, name: &str) -> Option<Val> {
        self.get_var(name).ok()
    }
    pub fn last_error(&self) -> ErrorInfo {
        let loc = usize::try_from(self.err.loc).ok().and_then(|i| self.locs.get(i)).map(|l| l.name.clone());
        ErrorInfo { num: self.err.num, loc, act_index: self.err.act_index, line: self.err.line }
    }
    pub fn error_description(&self) -> String {
        let e = self.last_error();
        format!(
            "Location: {}, Action: {}, Line: {}, Error: {} - {}",
            e.loc.unwrap_or_default(),
            e.act_index + 1,
            e.line,
            e.num,
            describe(e.num)
        )
    }
}
