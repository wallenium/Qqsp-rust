//! C-Schnittstelle (für die SwiftUI-Shell). Alle Strings sind UTF-8, NULL-terminiert.
//! Von der Bibliothek gelieferte `char*` müssen mit `qsp_string_free` freigegeben werden.
#![allow(clippy::missing_safety_doc)]
use crate::engine::Engine;
use crate::host::{Host, Win};
use crate::value::Val;
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::rc::Rc;

type E = *mut Engine;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct QspHost {
    pub user: *mut c_void,
    pub debug: Option<extern "C" fn(*mut c_void, E, *const c_char)>,
    pub is_playing_file: Option<extern "C" fn(*mut c_void, E, *const c_char) -> c_int>,
    pub play_file: Option<extern "C" fn(*mut c_void, E, *const c_char, c_int)>,
    /// file == NULL: alle schließen
    pub close_file: Option<extern "C" fn(*mut c_void, E, *const c_char)>,
    /// file == NULL: Bild ausblenden
    pub show_image: Option<extern "C" fn(*mut c_void, E, *const c_char)>,
    pub show_window: Option<extern "C" fn(*mut c_void, E, c_int, c_int)>,
    pub delete_menu: Option<extern "C" fn(*mut c_void, E)>,
    pub add_menu_item: Option<extern "C" fn(*mut c_void, E, *const c_char, *const c_char)>,
    pub show_menu: Option<extern "C" fn(*mut c_void, E) -> c_int>,
    pub show_message: Option<extern "C" fn(*mut c_void, E, *const c_char)>,
    pub refresh_int: Option<extern "C" fn(*mut c_void, E, c_int)>,
    pub set_timer: Option<extern "C" fn(*mut c_void, E, c_int)>,
    pub set_input_str_text: Option<extern "C" fn(*mut c_void, E, *const c_char)>,
    pub system: Option<extern "C" fn(*mut c_void, E, *const c_char)>,
    /// file == NULL: Auswahldialog
    pub open_game_status: Option<extern "C" fn(*mut c_void, E, *const c_char)>,
    pub save_game_status: Option<extern "C" fn(*mut c_void, E, *const c_char)>,
    pub sleep: Option<extern "C" fn(*mut c_void, E, c_int)>,
    pub ms_count: Option<extern "C" fn(*mut c_void, E) -> c_int>,
    /// Rückgabe: mit malloc angelegter UTF-8-String (wird per free() freigegeben) oder NULL.
    pub input_box: Option<extern "C" fn(*mut c_void, E, *const c_char) -> *mut c_char>,
}

fn cs(s: &str) -> CString {
    CString::new(s.replace('\0', "")).unwrap()
}
fn opt(s: Option<&str>) -> Option<CString> {
    s.map(cs)
}
fn p(c: &Option<CString>) -> *const c_char {
    c.as_ref().map_or(std::ptr::null(), |c| c.as_ptr())
}
fn st(s: *const c_char) -> String {
    if s.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(s) }.to_string_lossy().into_owned()
    }
}
fn out(s: &str) -> *mut c_char {
    cs(s).into_raw()
}

struct CHost(QspHost);
impl Host for CHost {
    fn debug(&self, e: &mut Engine, t: &str) {
        if let Some(f) = self.0.debug { f(self.0.user, e, cs(t).as_ptr()) }
    }
    fn is_playing_file(&self, e: &mut Engine, t: &str) -> bool {
        self.0.is_playing_file.is_some_and(|f| f(self.0.user, e, cs(t).as_ptr()) != 0)
    }
    fn play_file(&self, e: &mut Engine, t: &str, v: i32) {
        if let Some(f) = self.0.play_file { f(self.0.user, e, cs(t).as_ptr(), v) }
    }
    fn close_file(&self, e: &mut Engine, t: Option<&str>) {
        if let Some(f) = self.0.close_file { f(self.0.user, e, p(&opt(t))) }
    }
    fn show_image(&self, e: &mut Engine, t: Option<&str>) {
        if let Some(f) = self.0.show_image { f(self.0.user, e, p(&opt(t))) }
    }
    fn show_window(&self, e: &mut Engine, w: Win, s: bool) {
        if let Some(f) = self.0.show_window { f(self.0.user, e, w as c_int, s as c_int) }
    }
    fn delete_menu(&self, e: &mut Engine) {
        if let Some(f) = self.0.delete_menu { f(self.0.user, e) }
    }
    fn add_menu_item(&self, e: &mut Engine, n: &str, i: Option<&str>) {
        if let Some(f) = self.0.add_menu_item { f(self.0.user, e, cs(n).as_ptr(), p(&opt(i))) }
    }
    fn show_menu(&self, e: &mut Engine) -> i32 {
        self.0.show_menu.map_or(-1, |f| f(self.0.user, e))
    }
    fn show_message(&self, e: &mut Engine, t: &str) {
        if let Some(f) = self.0.show_message { f(self.0.user, e, cs(t).as_ptr()) }
    }
    fn refresh_int(&self, e: &mut Engine, r: bool) {
        if let Some(f) = self.0.refresh_int { f(self.0.user, e, r as c_int) }
    }
    fn set_timer(&self, e: &mut Engine, ms: i32) {
        if let Some(f) = self.0.set_timer { f(self.0.user, e, ms) }
    }
    fn set_input_str_text(&self, e: &mut Engine, t: Option<&str>) {
        if let Some(f) = self.0.set_input_str_text { f(self.0.user, e, p(&opt(t))) }
    }
    fn system(&self, e: &mut Engine, t: &str) {
        if let Some(f) = self.0.system { f(self.0.user, e, cs(t).as_ptr()) }
    }
    fn open_game_status(&self, e: &mut Engine, t: Option<&str>) {
        if let Some(f) = self.0.open_game_status { f(self.0.user, e, p(&opt(t))) }
    }
    fn save_game_status(&self, e: &mut Engine, t: Option<&str>) {
        if let Some(f) = self.0.save_game_status { f(self.0.user, e, p(&opt(t))) }
    }
    fn sleep(&self, e: &mut Engine, ms: i32) {
        if let Some(f) = self.0.sleep { f(self.0.user, e, ms) }
    }
    fn ms_count(&self, e: &mut Engine) -> i32 {
        self.0.ms_count.map_or(0, |f| f(self.0.user, e))
    }
    fn input_box(&self, e: &mut Engine, t: &str) -> String {
        let Some(f) = self.0.input_box else { return String::new() };
        let r = f(self.0.user, e, cs(t).as_ptr());
        if r.is_null() {
            return String::new();
        }
        let s = st(r);
        unsafe { libc_free(r as *mut c_void) };
        s
    }
}
extern "C" {
    #[link_name = "free"]
    fn libc_free(p: *mut c_void);
}

#[no_mangle]
pub unsafe extern "C" fn qsp_new(host: *const QspHost) -> E {
    Box::into_raw(Box::new(Engine::new(Rc::new(CHost(*host)))))
}
#[no_mangle]
pub unsafe extern "C" fn qsp_free(e: E) {
    if !e.is_null() {
        drop(Box::from_raw(e));
    }
}
#[no_mangle]
pub unsafe extern "C" fn qsp_string_free(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}
#[no_mangle]
pub unsafe extern "C" fn qsp_version() -> *mut c_char {
    out(Engine::version())
}
#[no_mangle]
pub unsafe extern "C" fn qsp_enable_debug(e: E, on: c_int) {
    (*e).enable_debug_mode(on != 0)
}

// ---- Laden / Ausführen
#[no_mangle]
pub unsafe extern "C" fn qsp_load_game(e: E, path: *const c_char) -> c_int {
    (*e).load_game_world(&st(path)) as c_int
}
#[no_mangle]
pub unsafe extern "C" fn qsp_load_game_data(e: E, data: *const u8, len: usize, name: *const c_char) -> c_int {
    (*e).load_game_world_from_data(std::slice::from_raw_parts(data, len), &st(name)) as c_int
}
#[no_mangle]
pub unsafe extern "C" fn qsp_restart(e: E, refresh: c_int) -> c_int {
    (*e).restart_game(refresh != 0) as c_int
}
#[no_mangle]
pub unsafe extern "C" fn qsp_exec_string(e: E, s: *const c_char, refresh: c_int) -> c_int {
    (*e).exec_code_string(&st(s), refresh != 0) as c_int
}
#[no_mangle]
pub unsafe extern "C" fn qsp_exec_location(e: E, s: *const c_char, refresh: c_int) -> c_int {
    (*e).exec_location_code(&st(s), refresh != 0) as c_int
}
#[no_mangle]
pub unsafe extern "C" fn qsp_exec_counter(e: E, refresh: c_int) -> c_int {
    (*e).exec_counter(refresh != 0) as c_int
}
#[no_mangle]
pub unsafe extern "C" fn qsp_exec_user_input(e: E, refresh: c_int) -> c_int {
    (*e).exec_user_input(refresh != 0) as c_int
}
#[no_mangle]
pub unsafe extern "C" fn qsp_set_sel_action(e: E, ind: c_int, refresh: c_int) -> c_int {
    (*e).set_sel_action_index(ind, refresh != 0) as c_int
}
#[no_mangle]
pub unsafe extern "C" fn qsp_exec_sel_action(e: E, refresh: c_int) -> c_int {
    (*e).exec_selected_action_code(refresh != 0) as c_int
}
#[no_mangle]
pub unsafe extern "C" fn qsp_set_sel_object(e: E, ind: c_int, refresh: c_int) -> c_int {
    (*e).set_sel_object_index(ind, refresh != 0) as c_int
}
/// Ausdruck auswerten. `*is_str`=1: Ergebnis in `*str_val` (freigeben), sonst `*num_val`.
#[no_mangle]
pub unsafe extern "C" fn qsp_expr_value(e: E, expr: *const c_char, is_str: *mut c_int, num_val: *mut c_int, str_val: *mut *mut c_char) -> c_int {
    match (*e).get_expr_value(&st(expr)) {
        Some(Val::Num(n)) => {
            *is_str = 0;
            *num_val = n;
            1
        }
        Some(Val::Str(s)) => {
            *is_str = 1;
            *str_val = out(&s);
            1
        }
        None => 0,
    }
}

// ---- Spielstände
#[no_mangle]
pub unsafe extern "C" fn qsp_save_game(e: E, path: *const c_char, refresh: c_int) -> c_int {
    (*e).save_game(&st(path), refresh != 0) as c_int
}
#[no_mangle]
pub unsafe extern "C" fn qsp_open_saved_game(e: E, path: *const c_char, refresh: c_int) -> c_int {
    (*e).open_saved_game(&st(path), refresh != 0) as c_int
}

// ---- Zustand
#[no_mangle]
pub unsafe extern "C" fn qsp_set_input_text(e: E, s: *const c_char) {
    (*e).set_input_str_text(&st(s))
}
#[no_mangle]
pub unsafe extern "C" fn qsp_window_visible(e: E, w: c_int) -> c_int {
    let w = match w { 0 => Win::Acts, 1 => Win::Objs, 2 => Win::Vars, _ => Win::Input };
    (*e).show_window(w) as c_int
}
#[no_mangle]
pub unsafe extern "C" fn qsp_main_desc(e: E, changed: *mut c_int) -> *mut c_char {
    let (s, c) = (*e).main_desc();
    *changed = c as c_int;
    out(s)
}
#[no_mangle]
pub unsafe extern "C" fn qsp_vars_desc(e: E, changed: *mut c_int) -> *mut c_char {
    let (s, c) = (*e).vars_desc();
    *changed = c as c_int;
    out(s)
}
#[no_mangle]
pub unsafe extern "C" fn qsp_actions_count(e: E) -> c_int {
    (*e).actions().0.len() as c_int
}
#[no_mangle]
pub unsafe extern "C" fn qsp_actions_changed(e: E) -> c_int {
    (*e).actions().1 as c_int
}
#[no_mangle]
pub unsafe extern "C" fn qsp_action_data(e: E, i: c_int, image: *mut *mut c_char, desc: *mut *mut c_char) {
    let (v, _) = (*e).actions();
    let (d, im) = &v[i as usize];
    *desc = out(d);
    *image = im.as_deref().map_or(std::ptr::null_mut(), out);
}
#[no_mangle]
pub unsafe extern "C" fn qsp_objects_count(e: E) -> c_int {
    (*e).objects().0.len() as c_int
}
#[no_mangle]
pub unsafe extern "C" fn qsp_objects_changed(e: E) -> c_int {
    (*e).objects().1 as c_int
}
#[no_mangle]
pub unsafe extern "C" fn qsp_object_data(e: E, i: c_int, image: *mut *mut c_char, desc: *mut *mut c_char) {
    let (v, _) = (*e).objects();
    let (d, im) = &v[i as usize];
    *desc = out(d);
    *image = im.as_deref().map_or(std::ptr::null_mut(), out);
}
#[no_mangle]
pub unsafe extern "C" fn qsp_sel_action(e: E) -> c_int {
    (*e).sel_action_index()
}
#[no_mangle]
pub unsafe extern "C" fn qsp_sel_object(e: E) -> c_int {
    (*e).sel_object_index()
}
#[no_mangle]
pub unsafe extern "C" fn qsp_cur_loc(e: E) -> *mut c_char {
    out((*e).cur_loc_name().unwrap_or(""))
}
#[no_mangle]
pub unsafe extern "C" fn qsp_qst_full_path(e: E) -> *mut c_char {
    out((*e).qst_full_path())
}
#[no_mangle]
pub unsafe extern "C" fn qsp_full_refresh_count(e: E) -> c_int {
    (*e).full_refresh_count()
}
#[no_mangle]
pub unsafe extern "C" fn qsp_var_num(e: E, name: *const c_char) -> c_int {
    (*e).var_num(&st(name))
}
#[no_mangle]
pub unsafe extern "C" fn qsp_last_error_num(e: E) -> c_int {
    (*e).last_error().num
}
#[no_mangle]
pub unsafe extern "C" fn qsp_error_description(e: E) -> *mut c_char {
    out(&(*e).error_description())
}
/// Liest `name[ind]` ohne Code auszuführen (in Callbacks erlaubt). Rückgabe 0, wenn der Wert nicht existiert.
/// `*str_val` ist NULL, wenn kein Textwert gesetzt ist, sonst freizugeben.
#[no_mangle]
pub unsafe extern "C" fn qsp_var_values(e: E, name: *const c_char, ind: c_int, num_val: *mut c_int, str_val: *mut *mut c_char) -> c_int {
    match (*e).var_values(&st(name), ind.max(0) as usize) {
        Some((n, s)) => {
            *num_val = n;
            *str_val = s.as_deref().map_or(std::ptr::null_mut(), out);
            1
        }
        None => 0,
    }
}
