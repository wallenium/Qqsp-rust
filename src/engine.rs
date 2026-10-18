//! Zentraler Zustand der Engine.
use crate::errors::*;
use crate::host::{Host, Win};
use crate::code::Line;
use crate::vars::{Slot, LocalVar};
use std::rc::Rc;

pub const MAXACTIONS: usize = 50;
pub const MAXOBJECTS: usize = 1000;
pub const VARSSEEK: usize = 50;
pub const VARSCOUNT: usize = 256 * VARSSEEK;
pub const MAXMENUITEMS: usize = 100;
pub const MAXPLFILES: usize = 500;
pub const MAXINCFILES: usize = 100;
pub const DEFTIMERINTERVAL: i32 = 500;
pub const QSP_VER: &str = "5.7.0";
pub const STRSDELIM: &str = "\r\n";

pub type Code = Vec<Rc<Line>>;

pub struct LocAct {
    pub image: Option<String>,
    pub desc: String,
    pub code: Code,
}
pub struct Location {
    pub name: String,
    pub desc: String,
    pub on_visit: Code,
    pub actions: Vec<LocAct>,
}

pub struct CurAct {
    pub image: Option<String>,
    pub desc: String,
    pub code: Code,
    pub location: i32,
    pub act_index: i32,
    pub start_line: i32,
    pub is_manage_lines: bool,
}
pub struct Obj {
    pub image: Option<String>,
    pub desc: String,
}

#[derive(Default, Clone, Debug)]
pub struct ErrState {
    pub num: i32,
    pub loc: i32,
    pub line: i32,
    pub act_index: i32,
}

/// Port des libqsp-Zufallsgenerators (für identisches Verhalten).
pub struct Rand {
    x: [u32; 55],
    y: [u32; 256],
    z: u32,
    i: i32,
    j: i32,
}
impl Rand {
    pub fn new(seed: u32) -> Rand {
        let mut r = Rand { x: [0; 55], y: [0; 256], z: 0, i: 0, j: 0 };
        r.seed(seed);
        r
    }
    pub fn seed(&mut self, seed: u32) {
        self.x[0] = 1;
        self.x[1] = seed;
        for i in 2..55 {
            self.x[i] = self.x[i - 1].wrapping_add(self.x[i - 2]);
        }
        self.i = 23;
        self.j = 54;
        for _ in 0..256 {
            self.urand();
        }
        for i in (0..256).rev() {
            self.y[i] = self.urand();
        }
        self.z = self.urand();
    }
    fn urand(&mut self) -> u32 {
        self.i -= 1;
        if self.i < 0 { self.i = 54; }
        self.j -= 1;
        if self.j < 0 { self.j = 54; }
        let v = self.x[self.j as usize].wrapping_add(self.x[self.i as usize]);
        self.x[self.j as usize] = v;
        v
    }
    pub fn next(&mut self) -> i32 {
        let i = (self.z >> 24) as usize;
        self.z = self.y[i];
        self.i -= 1;
        if self.i < 0 { self.i = 54; }
        self.j -= 1;
        if self.j < 0 { self.j = 54; }
        let v = self.x[self.j as usize].wrapping_add(self.x[self.i as usize]);
        self.x[self.j as usize] = v;
        self.y[i] = v;
        (self.z & 0x7FFF_FFFF) as i32
    }
}

pub struct Engine {
    pub(crate) host: Rc<dyn Host>,
    pub err: ErrState,
    pub(crate) real_cur_loc: i32,
    pub(crate) real_act_index: i32,
    pub(crate) real_line: i32,
    pub(crate) in_callback: bool,
    pub(crate) disable_code_exec: bool,
    pub(crate) exit_on_error: bool,
    pub(crate) rand: Rand,
    pub is_debug: bool,
    /// Modus „5.9-Funktionen“ (vor dem Laden eines Spiels setzen).
    pub(crate) ext59: bool,
    pub(crate) cur_desc: String,
    pub(crate) cur_vars: String,
    pub(crate) cur_input: String,
    pub(crate) view_path: Option<String>,
    pub(crate) timer_interval: i32,
    pub(crate) main_desc_changed: bool,
    pub(crate) vars_desc_changed: bool,
    pub(crate) objects_changed: bool,
    pub(crate) actions_changed: bool,
    pub(crate) show_acts: bool,
    pub(crate) show_objs: bool,
    pub(crate) show_vars: bool,
    pub(crate) show_input: bool,
    pub(crate) vars: Vec<Slot>,
    pub(crate) groups: Vec<Vec<LocalVar>>,
    pub(crate) locs: Vec<Location>,
    pub(crate) loc_names: Vec<(String, usize)>,
    pub(crate) cur_loc: i32,
    pub(crate) refresh_count: i32,
    pub(crate) full_refresh_count: i32,
    pub(crate) cur_actions: Vec<CurAct>,
    pub(crate) cur_sel_action: i32,
    pub(crate) cur_objects: Vec<Obj>,
    pub(crate) cur_sel_object: i32,
    pub(crate) pl_files: Vec<String>,
    pub(crate) inc_files: Vec<String>,
    pub(crate) cur_inc_locs_count: usize,
    pub(crate) qst_path: String,
    pub(crate) qst_full_path: String,
    pub(crate) qst_crc: i32,
    pub(crate) ms_count: i32,
}

impl Engine {
    pub fn new(host: Rc<dyn Host>) -> Engine {
        let mut vars = Vec::with_capacity(VARSCOUNT);
        vars.resize_with(VARSCOUNT, Slot::default);
        Engine {
            host,
            err: ErrState { num: 0, loc: -1, line: 0, act_index: -1 },
            real_cur_loc: -1,
            real_act_index: -1,
            real_line: 0,
            in_callback: false,
            disable_code_exec: false,
            exit_on_error: false,
            rand: Rand::new(0),
            is_debug: false,
            ext59: false,
            cur_desc: String::new(),
            cur_vars: String::new(),
            cur_input: String::new(),
            view_path: None,
            timer_interval: 0,
            main_desc_changed: false,
            vars_desc_changed: false,
            objects_changed: false,
            actions_changed: false,
            show_acts: true,
            show_objs: true,
            show_vars: true,
            show_input: true,
            vars,
            groups: Vec::new(),
            locs: Vec::new(),
            loc_names: Vec::new(),
            cur_loc: -1,
            refresh_count: 0,
            full_refresh_count: 0,
            cur_actions: Vec::new(),
            cur_sel_action: -1,
            cur_objects: Vec::new(),
            cur_sel_object: -1,
            pl_files: Vec::new(),
            inc_files: Vec::new(),
            cur_inc_locs_count: 0,
            qst_path: String::new(),
            qst_full_path: String::new(),
            qst_crc: 0,
            ms_count: 0,
        }
    }

    // ---------------- Fehler ----------------
    pub(crate) fn set_error(&mut self, num: i32) {
        if self.err.num == 0 {
            self.err = ErrState {
                num,
                loc: self.real_cur_loc,
                act_index: self.real_act_index,
                line: self.real_line,
            };
        }
    }
    pub(crate) fn reset_error(&mut self) {
        self.err = ErrState { num: 0, loc: -1, line: 0, act_index: -1 };
    }
    /// Fehler setzen und als `Err` zurückgeben.
    pub(crate) fn fail<T>(&mut self, num: i32) -> R<T> {
        self.set_error(num);
        Err(Abort::Error)
    }
    /// Bricht ab, falls (z. B. durch einen Callback) ein Fehler gesetzt wurde.
    pub(crate) fn chk(&self) -> R<()> {
        if self.err.num != 0 { Err(Abort::Error) } else { Ok(()) }
    }

    // ---------------- Callbacks mit Zustandssicherung ----------------
    pub(crate) fn callback<T>(
        &mut self,
        disable_code_exec: bool,
        exit_on_error: bool,
        f: impl FnOnce(&Rc<dyn Host>, &mut Engine) -> T,
    ) -> T {
        let saved = (
            self.in_callback,
            self.disable_code_exec,
            self.exit_on_error,
            self.main_desc_changed,
            self.vars_desc_changed,
            self.objects_changed,
            self.actions_changed,
        );
        self.in_callback = true;
        self.disable_code_exec = disable_code_exec;
        self.exit_on_error = exit_on_error;
        let host = self.host.clone();
        let r = f(&host, self);
        if !self.exit_on_error {
            self.reset_error();
        }
        self.exit_on_error = saved.2;
        self.disable_code_exec = saved.1;
        self.in_callback = saved.0;
        if saved.6 { self.actions_changed = true; }
        if saved.5 { self.objects_changed = true; }
        if saved.4 { self.vars_desc_changed = true; }
        if saved.3 { self.main_desc_changed = true; }
        r
    }
    pub(crate) fn call_debug(&mut self, s: &str) {
        self.callback(false, false, |h, e| h.debug(e, s))
    }
    pub(crate) fn call_set_timer(&mut self, ms: i32) {
        self.callback(true, false, |h, e| h.set_timer(e, ms))
    }
    pub(crate) fn call_refresh_int(&mut self, redraw: bool) {
        self.callback(true, false, |h, e| h.refresh_int(e, redraw))
    }
    pub(crate) fn call_set_input_str_text(&mut self, t: Option<&str>) {
        self.callback(true, false, |h, e| h.set_input_str_text(e, t))
    }
    pub(crate) fn call_add_menu_item(&mut self, name: &str, img: Option<&str>) {
        self.callback(true, false, |h, e| h.add_menu_item(e, name, img))
    }
    pub(crate) fn call_system(&mut self, cmd: &str) {
        self.callback(false, false, |h, e| h.system(e, cmd))
    }
    pub(crate) fn call_open_game(&mut self, f: Option<&str>) {
        self.callback(false, true, |h, e| h.open_game_status(e, f))
    }
    pub(crate) fn call_save_game(&mut self, f: Option<&str>) {
        self.callback(false, true, |h, e| h.save_game_status(e, f))
    }
    pub(crate) fn call_show_message(&mut self, t: &str) {
        self.callback(true, false, |h, e| h.show_message(e, t))
    }
    pub(crate) fn call_show_menu(&mut self) -> i32 {
        self.callback(false, true, |h, e| h.show_menu(e))
    }
    pub(crate) fn call_show_picture(&mut self, f: Option<&str>) {
        self.callback(true, false, |h, e| h.show_image(e, f))
    }
    pub(crate) fn call_show_window(&mut self, w: Win, show: bool) {
        self.callback(true, false, |h, e| h.show_window(e, w, show))
    }
    pub(crate) fn call_play_file(&mut self, f: &str, vol: i32) {
        self.callback(true, false, |h, e| h.play_file(e, f, vol))
    }
    pub(crate) fn call_is_playing(&mut self, f: &str) -> bool {
        self.callback(true, false, |h, e| h.is_playing_file(e, f))
    }
    pub(crate) fn call_sleep(&mut self, ms: i32) {
        self.callback(true, false, |h, e| h.sleep(e, ms))
    }
    pub(crate) fn call_ms_count(&mut self) -> i32 {
        self.callback(true, false, |h, e| h.ms_count(e))
    }
    pub(crate) fn call_close_file(&mut self, f: Option<&str>) {
        self.callback(true, false, |h, e| h.close_file(e, f))
    }
    pub(crate) fn call_delete_menu(&mut self) {
        self.callback(true, false, |h, e| h.delete_menu(e))
    }
    pub(crate) fn call_input_box(&mut self, t: &str) -> String {
        let mut s = self.callback(true, false, |h, e| h.input_box(e, t));
        // libqsp: Puffer von 511 Zeichen
        if s.encode_utf16().count() > 511 {
            s = String::from_utf16_lossy(&s.encode_utf16().take(511).collect::<Vec<_>>());
        }
        s
    }

    // ---------------- Zeit ----------------
    pub(crate) fn reset_time(&mut self, ms: i32) {
        self.ms_count = ms;
        self.call_ms_count();
    }
    pub(crate) fn get_time(&mut self) -> i32 {
        let d = self.call_ms_count();
        self.ms_count = self.ms_count.wrapping_add(d);
        if self.ms_count < 0 {
            self.ms_count = 0;
        }
        self.ms_count
    }
}
