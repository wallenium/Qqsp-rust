//! Der Modus „5.9-Funktionen“ ist optional; ohne ihn verhalten sich die neuen Namen wie in 5.7 (Fehler).
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


/// (Erfolg, Meldungen)
fn run(ext: bool, code: &str) -> (bool, Vec<String>) {
    let h = Rc::new(Rec::default());
    let mut e = Engine::new(h.clone());
    e.set_ext59(ext);
    assert!(e.load_game_world_from_data(&build(&[("start", code, &[])]), "t.qsp"));
    let ok = e.restart_game(true);
    let r = h.0.borrow().clone();
    (ok, r)
}

#[test]
fn neue_anweisungen_nur_im_modus() {
    let code = "a[0]=3\na[1]=1\nsortarr 'a'\nmsg a[0]";
    assert!(!run(false, code).0, "5.7: SORTARR ist unbekannt");
    assert_eq!(run(true, code), (true, vec!["1".to_string()]));
}
#[test]
fn neue_funktionen() {
    assert_eq!(run(true, "msg replace('aaaa','a','b',2)").1, ["bbaa"]);
    assert!(!run(false, "msg replace('aaaa','a','b',2)").0);
    assert_eq!(run(true, "addobj 'x'\nmsg countobj('X')").1, ["1"]);
    assert_eq!(run(true, "setvar 'a', 7, 2\nmsg arritem('a', 2)").1, ["7"]);
}
