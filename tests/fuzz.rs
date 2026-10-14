//! Robustheit: beschädigte Spieldateien/Spielstände und zufälliger Code dürfen nie zu einem Panic führen.
use qsp_core::*;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;

struct Quiet;
impl Host for Quiet {}

fn iters(n: usize) -> usize {
    std::env::var("FUZZ_N").ok().and_then(|v| v.parse().ok()).unwrap_or(n)
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

fn enc(s: &str) -> Vec<u16> {
    s.encode_utf16().map(|x| if x == 5 { 0xFFFB } else { x.wrapping_sub(5) }).collect()
}
fn build(code: &str, act: &str) -> Vec<u8> {
    let mut u: Vec<u16> = Vec::new();
    let mut line = |v: Vec<u16>| {
        u.extend(v);
        u.extend([13, 10]);
    };
    line("QSPGAME".encode_utf16().collect());
    line("5.7.0".encode_utf16().collect());
    line(enc(""));
    line(enc("1"));
    line(enc("start"));
    line(enc(""));
    line(enc(&code.replace('\n', "\r\n")));
    line(enc("1"));
    line(enc(""));
    line(enc("go"));
    line(enc(&act.replace('\n', "\r\n")));
    u.iter().flat_map(|x| x.to_le_bytes()).collect()
}

fn mutate(r: &mut Rng, src: &[u8]) -> Vec<u8> {
    let mut d = src.to_vec();
    for _ in 0..1 + r.below(6) {
        match r.below(4) {
            0 if !d.is_empty() => { let i = r.below(d.len()); d[i] = r.next() as u8; }
            1 if !d.is_empty() => { let i = r.below(d.len()); d.truncate(i); }
            2 => { let i = r.below(d.len() + 1); d.insert(i, r.next() as u8); }
            _ if d.len() > 2 => { let i = r.below(d.len() - 1); d.remove(i); }
            _ => {}
        }
    }
    d
}

const GAME: &str = "x=5\n$s='abc'\nact 'k': msg 'k'\nif x>3: msg $s else msg 'no'\n*pl 'hi'";

#[test]
fn corrupted_game_files_do_not_panic() {
    let base = build(GAME, "x=x+1\nmsg x");
    let mut r = Rng(0x9E3779B97F4A7C15);
    for _ in 0..iters(20000) {
        let data = mutate(&mut r, &base);
        let res = catch_unwind(AssertUnwindSafe(|| {
            let mut e = Engine::new(Rc::new(Quiet));
            if e.load_game_world_from_data(&data, "f.qsp") {
                e.restart_game(true);
                e.set_sel_action_index(0, true);
                e.exec_selected_action_code(true);
            }
        }));
        assert!(res.is_ok(), "Panic bei Spieldatei {:?}", data);
    }
}

#[test]
fn corrupted_savegames_do_not_panic() {
    let mut e = Engine::new(Rc::new(Quiet));
    assert!(e.load_game_world_from_data(&build(GAME, "x=x+1"), "f.qsp"));
    e.restart_game(true);
    let save = e.save_game_as_data(true).expect("save");
    let mut r = Rng(0xDEADBEEFCAFEF00D);
    for _ in 0..iters(20000) {
        let data = mutate(&mut r, &save);
        let res = catch_unwind(AssertUnwindSafe(|| {
            let mut e = Engine::new(Rc::new(Quiet));
            if e.load_game_world_from_data(&build(GAME, "x=x+1"), "f.qsp") {
                e.open_saved_game_from_data(&data, true);
                e.restart_game(true);
            }
        }));
        assert!(res.is_ok(), "Panic bei Spielstand {:?}", data);
    }
}

#[test]
fn random_code_does_not_panic() {
    const TOK: &[&str] = &[
        "x", "$s", "1", "-3", "'a'", "\"b\"", "{c}", "(", ")", "[", "]", "+", "-", "*", "/", "=", "<>", "<", "&", ":", ",", " ", "\n",
        "if ", "else", "act ", "end", "jump ", "gosub ", "gs ", "goto ", "xgoto ", "msg ", "set ", "let ", "killvar ", "copyarr ",
        "mid(", "len(", "arrsize(", "arrpos(", "instr(", "str(", "val(", "rand(", "iif(", "func(", "strcomp(", "strfind(", "strrepl(", "replace(",
        "dyneval(", "dynamic ", "addobj ", "delobj ", "killall", "cla", "*pl ", "*nl ", "<<", ">>", "$", "%", "!", "'", "\"", "{", "}",
        "mod", "and", "or", "no", "obj", "loc", "$user_text", "desc", ":lbl", "'lbl'",
    ];
    let mut r = Rng(0x1234567890ABCDEF);
    // große Stack-Reserve, damit tiefe Rekursion als Fehler (nicht als Absturz) auffällt
    let t = std::thread::Builder::new().stack_size(256 << 20).spawn(move || {
        for _ in 0..iters(30000) {
            let mut code = String::new();
            for _ in 0..1 + r.below(14) {
                code.push_str(TOK[r.below(TOK.len())]);
            }
            let res = catch_unwind(AssertUnwindSafe(|| {
                let mut e = Engine::new(Rc::new(Quiet));
                e.load_game_world_from_data(&build("x=1", "x=2"), "f.qsp");
                e.restart_game(true);
                e.exec_code_string(&code, true);
            }));
            assert!(res.is_ok(), "Panic bei Code {:?}", code);
        }
    });
    t.unwrap().join().unwrap();
}
