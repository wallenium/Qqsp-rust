use qsp_core::*;
use std::rc::Rc;
fn out(s: &str) -> String {
    s.encode_utf16().map(|u| if u < 128 { (u as u8 as char).to_string() } else { format!("\\u{:04x}", u) }).collect()
}
struct H;
impl Host for H {
    fn show_window(&self, _e: &mut Engine, w: Win, on: bool) {
        println!("WIN:{}:{}", w as i32, on as i32);
    }
    fn show_message(&self, _e: &mut Engine, t: &str) {
        println!("MSG:{}", out(t));
    }
}
fn main() {
    let d = std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let mut e = Engine::new(Rc::new(H));
    if !e.load_game_world_from_data(&d, "t.qsp") {
        println!("LOADFAIL");
        return;
    }
    if !e.restart_game(true) {
        println!("ERR:{}", e.last_error().num);
    }
    println!("DESC:{}", out(e.main_desc().0));
}
