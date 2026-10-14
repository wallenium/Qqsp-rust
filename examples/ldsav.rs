use qsp_core::*;
use std::rc::Rc;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut e = Engine::new(Rc::new(NullHost));
    assert!(e.load_game_world_from_data(&std::fs::read(&a[1]).unwrap(), "t.qsp"));
    assert!(e.restart_game(true));
    e.exec_code_string("x=99 & killall", true);
    assert!(e.open_saved_game(&a[2], true), "{}", e.error_description());
    println!("x={} s={:?} a2={} acts={} objs={}", e.var_num("x"),
        e.get_expr_value("$s").map(|v| v.s().to_string()), e.var_num("a[2]"),
        e.actions().0.len(), e.objects().0.len());
    let d = e.save_game_as_data(false).unwrap();
    std::fs::write(&a[3], d).unwrap();
}
