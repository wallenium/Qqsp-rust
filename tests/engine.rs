use qsp_core::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
struct Rec(RefCell<Vec<String>>);
impl Host for Rec {
    fn show_message(&self, _e: &mut Engine, t: &str) {
        self.0.borrow_mut().push(t.to_string());
    }
}

fn enc(s: &str) -> Vec<u16> {
    s.encode_utf16().map(|x| if x == 5 { 0xFFFB } else { x.wrapping_sub(5) }).collect()
}
/// Baut eine UCS2-.qsp-Datei: locs = (name, code, actions[(desc, code)])
fn build(locs: &[(&str, &str, &[(&str, &str)])]) -> Vec<u8> {
    let mut u: Vec<u16> = Vec::new();
    let mut line = |u: &mut Vec<u16>, v: Vec<u16>| {
        u.extend(v);
        u.extend([13, 10]);
    };
    line(&mut u, "QSPGAME".encode_utf16().collect());
    line(&mut u, "5.7.0".encode_utf16().collect());
    line(&mut u, enc("")); // Passwort
    line(&mut u, enc(&locs.len().to_string()));
    for (n, c, acts) in locs {
        line(&mut u, enc(n));
        line(&mut u, enc(""));
        line(&mut u, enc(&c.replace("\n", "\r\n")));
        line(&mut u, enc(&acts.len().to_string()));
        for (d, ac) in *acts {
            line(&mut u, enc(""));
            line(&mut u, enc(d));
            line(&mut u, enc(&ac.replace("\n", "\r\n")));
        }
    }
    u.iter().flat_map(|x| x.to_le_bytes()).collect()
}

fn run(code: &str) -> Vec<String> {
    let h = Rc::new(Rec::default());
    let mut e = Engine::new(h.clone());
    let data = build(&[("start", code, &[])]);
    assert!(e.load_game_world_from_data(&data, "t.qsp"), "{}", e.error_description());
    assert!(e.restart_game(true), "{}", e.error_description());
    let r = h.0.borrow().clone();
    r
}

#[test]
fn arithmetic_and_strings() {
    let r = run("msg 1+2*3\nmsg 'a'+'b'\nmsg 7/2\nmsg 7 mod 3\nmsg -5\nmsg 'x' + 1\nmsg len('hello')\nmsg mid('abcdef',2,3)\nmsg instr('hello','ll')");
    assert_eq!(r, ["7", "ab", "3", "1", "-5", "x1", "5", "bcd", "3"]);
}

#[test]
fn control_flow_and_vars() {
    let r = run(
        "x=0\n:loop\nx=x+1\nif x<3: jump 'loop'\nmsg x\n\
         for i=1 to 3: s=s+i\nmsg s\n\
         $t='q'\nmsg $t + '<<x>>'\n\
         if x=3:\n msg 'yes'\nelse\n msg 'no'\nend\n\
         a[0]=5\na[1]=6\nmsg arrsize('a')",
    );
    assert_eq!(r, ["3", "6", "q3", "yes", "2"]);
}

#[test]
fn locals_and_gosub() {
    let h = Rc::new(Rec::default());
    let mut e = Engine::new(h.clone());
    let data = build(&[("start", "x=1\ngosub 'f',5\nmsg x\nmsg res", &[]), ("f", "local x\nx=args[0]\nres=x*2", &[])]);
    assert!(e.load_game_world_from_data(&data, "t.qsp"));
    assert!(e.restart_game(true));
    assert_eq!(*h.0.borrow(), ["1", "10"]);
}

#[test]
fn actions_and_savegame() {
    let h = Rc::new(Rec::default());
    let mut e = Engine::new(h.clone());
    let data = build(&[("start", "n=0\nact 'inc': n=n+1\nmsg 'go'", &[("two", "n=n+2")])]);
    assert!(e.load_game_world_from_data(&data, "t.qsp"));
    assert!(e.restart_game(true));
    let (acts, _) = e.actions();
    assert_eq!(acts.len(), 2);
    e.set_sel_action_index(1, true);
    assert!(e.exec_selected_action_code(true));
    assert_eq!(e.var_num("n"), 1);
    let save = e.save_game_as_data(false).expect("save");
    e.set_sel_action_index(0, true);
    assert!(e.exec_selected_action_code(true));
    assert_eq!(e.var_num("n"), 3);
    assert!(e.open_saved_game_from_data(&save, true), "{}", e.error_description());
    assert_eq!(e.var_num("n"), 1);
}

#[test]
fn errors_are_reported() {
    let h = Rc::new(Rec::default());
    let mut e = Engine::new(h);
    let data = build(&[("start", "x=1/0", &[])]);
    assert!(e.load_game_world_from_data(&data, "t.qsp"));
    assert!(!e.restart_game(true));
    assert_eq!(e.last_error().num, 100);
}
