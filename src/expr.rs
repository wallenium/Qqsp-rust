//! Ausdrücke: Kompilierung (Shunting-Yard) und Auswertung – Port von mathops.c.
use crate::engine::*;
use crate::errors::*;
use crate::text::*;
use crate::value::*;
use std::sync::OnceLock;

pub const OP_UNKNOWN: u8 = 0;
pub const OP_START: u8 = 1;
pub const OP_END: u8 = 2;
pub const OP_VALUE: u8 = 3;
pub const OP_OPENBRACKET: u8 = 4;
pub const OP_MINUS: u8 = 5;
pub const OP_COMMA: u8 = 6;
pub const OP_CLOSEBRACKET: u8 = 7;
pub const OP_MUL: u8 = 8;
pub const OP_DIV: u8 = 9;
pub const OP_ADD: u8 = 10;
pub const OP_SUB: u8 = 11;
pub const OP_MOD: u8 = 12;
pub const OP_NE: u8 = 13;
pub const OP_LEQ: u8 = 14;
pub const OP_GEQ: u8 = 15;
pub const OP_EQ: u8 = 16;
pub const OP_LT: u8 = 17;
pub const OP_GT: u8 = 18;
pub const OP_AND: u8 = 19;
pub const OP_OR: u8 = 20;
pub const OP_APPEND: u8 = 21;
pub const OP_FIRST_FUNCTION: u8 = 22;
pub const OP_NOT: u8 = 22;
pub const OP_LOC: u8 = 23;
pub const OP_OBJ: u8 = 24;
pub const OP_MIN: u8 = 25;
pub const OP_MAX: u8 = 26;
pub const OP_RAND: u8 = 27;
pub const OP_IIF: u8 = 28;
pub const OP_RGB: u8 = 29;
pub const OP_LEN: u8 = 30;
pub const OP_ISNUM: u8 = 31;
pub const OP_LCASE: u8 = 32;
pub const OP_UCASE: u8 = 33;
pub const OP_INPUT: u8 = 34;
pub const OP_STR: u8 = 35;
pub const OP_VAL: u8 = 36;
pub const OP_ARRSIZE: u8 = 37;
pub const OP_ISPLAY: u8 = 38;
pub const OP_DESC: u8 = 39;
pub const OP_TRIM: u8 = 40;
pub const OP_GETOBJ: u8 = 41;
pub const OP_STRCOMP: u8 = 42;
pub const OP_STRFIND: u8 = 43;
pub const OP_STRPOS: u8 = 44;
pub const OP_MID: u8 = 45;
pub const OP_ARRPOS: u8 = 46;
pub const OP_ARRCOMP: u8 = 47;
pub const OP_INSTR: u8 = 48;
pub const OP_REPLACE: u8 = 49;
pub const OP_FUNC: u8 = 50;
pub const OP_DYNEVAL: u8 = 51;
pub const OP_RND: u8 = 52;
pub const OP_COUNTOBJ: u8 = 53;
pub const OP_MSECSCOUNT: u8 = 54;
pub const OP_QSPVER: u8 = 55;
pub const OP_USERTEXT: u8 = 56;
pub const OP_CURLOC: u8 = 57;
pub const OP_SELOBJ: u8 = 58;
pub const OP_SELACT: u8 = 59;
pub const OP_MAINTEXT: u8 = 60;
pub const OP_STATTEXT: u8 = 61;
pub const OP_CURACTS: u8 = 62;

const STACKSIZE: usize = 30;
const MAXITEMS: usize = 100;

struct OpInfo {
    pri: i32,
    res: u8,
    min: i32,
    max: i32,
    types: [u8; 10],
}
fn op_info(op: u8, ext: bool) -> OpInfo {
    let o = |pri, res, min, max, t: &[u8]| {
        let mut types = [0u8; 10];
        types[..t.len()].copy_from_slice(t);
        OpInfo { pri, res, min, max, types }
    };
    let ten0: &[u8] = &[0; 10];
    if ext {
        match op {
            crate::ext59::OP_ARRITEM => return o(30, 0, 1, 2, &[1, 0]),
            OP_COUNTOBJ => return o(30, 2, 0, 1, &[1]),
            OP_REPLACE => return o(30, 1, 2, 4, &[1, 1, 1, 2]),
            _ => {}
        }
    }
    match op {
        OP_START | OP_OPENBRACKET => o(127, 0, 0, 0, &[]),
        OP_MINUS => o(18, 2, 1, 1, &[2]),
        OP_ADD => o(14, 0, 2, 2, &[0, 0]),
        OP_SUB => o(14, 2, 2, 2, &[2, 2]),
        OP_MUL | OP_DIV => o(17, 2, 2, 2, &[2, 2]),
        OP_MOD => o(16, 2, 2, 2, &[2, 2]),
        OP_NE | OP_LEQ | OP_GEQ | OP_EQ | OP_LT | OP_GT => o(10, 2, 2, 2, &[0, 0]),
        OP_APPEND => o(4, 1, 2, 2, &[1, 1]),
        OP_COMMA => o(0, 1, 2, 2, &[1, 1]),
        OP_AND => o(7, 2, 2, 2, &[2, 2]),
        OP_OR => o(6, 2, 2, 2, &[2, 2]),
        OP_LOC | OP_OBJ => o(8, 2, 1, 1, &[1]),
        OP_NOT => o(8, 2, 1, 1, &[2]),
        OP_MIN | OP_MAX => o(30, 0, 1, 10, ten0),
        OP_RAND => o(30, 2, 1, 2, &[2, 2]),
        OP_IIF => o(30, 0, 3, 3, &[2, 0, 0]),
        OP_RGB => o(30, 2, 3, 4, &[2, 2, 2, 2]),
        OP_LEN => o(30, 2, 1, 1, &[1]),
        OP_ISNUM => o(30, 2, 1, 1, &[0]),
        OP_LCASE | OP_UCASE | OP_INPUT | OP_STR => o(30, 1, 1, 1, &[1]),
        OP_VAL => o(30, 2, 1, 1, &[0]),
        OP_ARRSIZE | OP_ISPLAY => o(30, 2, 1, 1, &[1]),
        OP_DESC | OP_TRIM => o(30, 1, 1, 1, &[1]),
        OP_GETOBJ => o(30, 1, 1, 1, &[2]),
        OP_STRCOMP => o(30, 2, 2, 2, &[1, 1]),
        OP_STRFIND => o(30, 1, 2, 3, &[1, 1, 2]),
        OP_STRPOS => o(30, 2, 2, 3, &[1, 1, 2]),
        OP_MID => o(30, 1, 2, 3, &[1, 2, 2]),
        OP_ARRPOS | OP_ARRCOMP => o(30, 2, 2, 3, &[0, 0, 0]),
        OP_INSTR => o(30, 2, 2, 3, &[0, 1, 1]),
        OP_REPLACE => o(30, 1, 2, 3, &[1, 1, 1]),
        OP_FUNC | OP_DYNEVAL => o(30, 0, 1, 10, &[1, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        OP_RND | OP_COUNTOBJ | OP_MSECSCOUNT => o(30, 2, 0, 0, &[]),
        OP_QSPVER | OP_USERTEXT | OP_CURLOC | OP_SELOBJ | OP_SELACT | OP_MAINTEXT | OP_STATTEXT
        | OP_CURACTS => o(30, 1, 0, 0, &[]),
        _ => o(0, 0, 0, 0, &[]),
    }
}

struct OpNames {
    levels: [Vec<(String, u8)>; 2],
    max_len: usize,
}
fn op_names() -> &'static OpNames {
    static N: OnceLock<OpNames> = OnceLock::new();
    N.get_or_init(|| {
        let l0: Vec<(&str, u8)> = vec![("<>", OP_NE), ("<=", OP_LEQ), ("=<", OP_LEQ), (">=", OP_GEQ), ("=>", OP_GEQ)];
        let mut l1: Vec<(String, u8)> = vec![
            (")".into(), OP_CLOSEBRACKET), ("+".into(), OP_ADD), ("-".into(), OP_SUB), ("*".into(), OP_MUL),
            ("/".into(), OP_DIV), ("MOD".into(), OP_MOD), ("!".into(), OP_NE), ("=".into(), OP_EQ),
            ("<".into(), OP_LT), (">".into(), OP_GT), ("&".into(), OP_APPEND), (",".into(), OP_COMMA),
            ("AND".into(), OP_AND), ("OR".into(), OP_OR), ("LOC".into(), OP_LOC), ("OBJ".into(), OP_OBJ),
            ("NO".into(), OP_NOT), ("RAND".into(), OP_RAND), ("RGB".into(), OP_RGB), ("LEN".into(), OP_LEN),
            ("ISNUM".into(), OP_ISNUM), ("VAL".into(), OP_VAL), ("ARRSIZE".into(), OP_ARRSIZE),
            ("ISPLAY".into(), OP_ISPLAY), ("STRCOMP".into(), OP_STRCOMP), ("STRPOS".into(), OP_STRPOS),
            ("ARRPOS".into(), OP_ARRPOS), ("ARRCOMP".into(), OP_ARRCOMP), ("INSTR".into(), OP_INSTR),
            ("RND".into(), OP_RND), ("COUNTOBJ".into(), OP_COUNTOBJ), ("MSECSCOUNT".into(), OP_MSECSCOUNT),
        ];
        // Namen, die auch mit `$` vorkommen
        for (n, c) in [
            ("MIN", OP_MIN), ("MAX", OP_MAX), ("IIF", OP_IIF), ("LCASE", OP_LCASE), ("UCASE", OP_UCASE),
            ("INPUT", OP_INPUT), ("STR", OP_STR), ("DESC", OP_DESC), ("TRIM", OP_TRIM), ("GETOBJ", OP_GETOBJ),
            ("STRFIND", OP_STRFIND), ("MID", OP_MID), ("REPLACE", OP_REPLACE), ("FUNC", OP_FUNC),
            ("DYNEVAL", OP_DYNEVAL), ("QSPVER", OP_QSPVER), ("USER_TEXT", OP_USERTEXT), ("USRTXT", OP_USERTEXT),
            ("CURLOC", OP_CURLOC), ("SELOBJ", OP_SELOBJ), ("SELACT", OP_SELACT), ("MAINTXT", OP_MAINTEXT),
            ("STATTXT", OP_STATTEXT), ("CURACTS", OP_CURACTS),
        ] {
            l1.push((n.into(), c));
            l1.push((format!("${}", n), c));
        }
        let mut l0v: Vec<(String, u8)> = l0.into_iter().map(|(n, c)| (n.to_string(), c)).collect();
        l0v.sort_by(|a, b| a.0.cmp(&b.0));
        l1.sort_by(|a, b| a.0.cmp(&b.0));
        let max_len = l0v.iter().chain(l1.iter()).map(|(n, _)| n.len()).max().unwrap_or(0);
        OpNames { levels: [l0v, l1], max_len }
    })
}

fn function_op_code(name: &str, ext: bool) -> u8 {
    if ext && crate::ext59::is_arritem(name) {
        return crate::ext59::OP_ARRITEM;
    }
    let u = upper(name);
    let l1 = &op_names().levels[1];
    match bsearch(l1, |e| cmp_str(&u, &e.0)) {
        Some(i) => l1[i].1,
        None => OP_UNKNOWN,
    }
}
/// qspOperatorOpCode: liefert Code und verbrauchte Länge.
fn operator_op_code(s: &str) -> (u8, usize) {
    if s.is_empty() {
        return (OP_END, 0);
    }
    let on = op_names();
    // Schlüssel ohne Heap-Allokation: höchstens max_len Zeichen, groß geschrieben
    let mut buf = [0u8; 64];
    let mut n = 0;
    for c in s.chars().take(on.max_len) {
        let mut t = [0u8; 4];
        let e = up(c).encode_utf8(&mut t).as_bytes();
        if n + e.len() > buf.len() {
            break;
        }
        buf[n..n + e.len()].copy_from_slice(e);
        n += e.len();
    }
    let key = &buf[..n];
    for level in &on.levels {
        if let Some(i) = bsearch(level, |e| ncmp(key, e.0.as_bytes())) {
            return (level[i].1, level[i].0.len());
        }
    }
    (OP_UNKNOWN, 0)
}

struct Item {
    op: u8,
    val: Val,
    argc: usize,
}

// ---------- reguläre Ausdrücke ----------
pub fn compile_anchored(pat: &str) -> Option<fancy_regex::Regex> {
    fancy_regex::Regex::new(&format!(r"\A(?:{})", pat)).ok()
}
pub fn full_match(re: &fancy_regex::Regex, s: &str) -> bool {
    matches!(re.find(s), Ok(Some(m)) if m.start() == 0 && m.end() == s.len())
}

fn num_of(v: &Val) -> i32 {
    match v {
        Val::Num(n) => *n,
        Val::Str(s) => str_to_num_raw(s),
    }
}

fn get_number(s: &str) -> (i32, usize) {
    let b = s.as_bytes();
    let mut i = 0;
    while is_digit(at(b, i)) {
        i += 1;
    }
    if i > 10 {
        return (i32::MAX, i);
    }
    (str_to_num_raw(&s[..i]), i)
}

impl Engine {
    fn aborted(&self, old: i32) -> R<()> {
        if self.err.num != 0 {
            Err(Abort::Error)
        } else if self.refresh_count != old {
            Err(Abort::Refresh)
        } else {
            Ok(())
        }
    }

    /// qspGetName
    fn get_name<'a>(&mut self, s: &'a str) -> R<(&'a str, usize)> {
        let b = s.as_bytes();
        let mut pos = 0usize;
        loop {
            pos += 1;
            if at(b, pos) == b'[' {
                match str_pos(&s[pos..], "]", false) {
                    Some(p) => pos += p + 1,
                    None => return self.fail(BRACKNOTFOUND),
                }
                break;
            }
            if is_in_list_eol(DELIMS, at(b, pos)) {
                break;
            }
        }
        Ok((&s[..pos], pos + skip_spaces(&s[pos..])))
    }

    /// qspGetString: Text in Anführungszeichen, danach `<<..>>`-Ersetzung.
    fn get_string(&mut self, s: &str) -> R<(String, usize)> {
        let b = s.as_bytes();
        let quot = b[0];
        let mut buf: Vec<u8> = Vec::new();
        let mut pos = 0usize;
        loop {
            pos += 1;
            if pos >= b.len() {
                return self.fail(QUOTNOTFOUND);
            }
            if b[pos] == quot {
                pos += 1;
                if at(b, pos) != quot {
                    break;
                }
            }
            buf.push(b[pos]);
        }
        let text = String::from_utf8_lossy(&buf).into_owned();
        let r = self.format_text(&text)?;
        Ok((r, pos))
    }
    fn get_qstring(&mut self, s: &str) -> R<(String, usize)> {
        match str_pos(s, "}", false) {
            Some(p) => Ok((s[1..p].to_string(), p + 1)),
            None => self.fail(QUOTNOTFOUND),
        }
    }

    /// qspFormatText: `<<ausdruck>>` ersetzen.
    pub(crate) fn format_text(&mut self, txt: &str) -> R<String> {
        if self.get_var_num("DISABLESUBEX") != 0 {
            return Ok(txt.to_string());
        }
        let mut l = match txt.find("<<") {
            Some(l) => l,
            None => return Ok(txt.to_string()),
        };
        let mut rest = txt;
        let mut out = String::new();
        let old = self.refresh_count;
        loop {
            out.push_str(&rest[..l]);
            rest = &rest[l + 2..];
            let r = match str_pos(rest, ">>", false) {
                Some(r) => r,
                None => return self.fail(BRACKNOTFOUND),
            };
            let mut v = self.expr_value(&rest[..r])?;
            if self.refresh_count != old {
                return Err(Abort::Refresh);
            }
            v.conv_to(true);
            out.push_str(v.s());
            rest = &rest[r + 2..];
            match rest.find("<<") {
                Some(n) => l = n,
                None => break,
            }
        }
        out.push_str(rest);
        Ok(out)
    }

    fn push_op(&mut self, ops: &mut [u8; STACKSIZE], opsp: &mut isize, args: &mut [i32; STACKSIZE], argsp: &mut isize, op: u8) -> R<()> {
        if *opsp == STACKSIZE as isize - 1 || *argsp == STACKSIZE as isize - 1 {
            return self.fail(STACKOVERFLOW);
        }
        *opsp += 1;
        ops[*opsp as usize] = op;
        *argsp += 1;
        args[*argsp as usize] = if op < OP_FIRST_FUNCTION { op_info(op, self.ext59).min } else { 0 };
        Ok(())
    }
    fn append(&mut self, items: &mut Vec<Item>, op: u8, argc: i32, val: Val) -> R<()> {
        if items.len() == MAXITEMS {
            return self.fail(TOOMANYITEMS);
        }
        items.push(Item { op, val, argc: argc.max(0) as usize });
        Ok(())
    }

    /// qspCompileExpression
    fn compile_expr(&mut self, mut s: &str) -> R<Vec<Item>> {
        let mut items: Vec<Item> = Vec::new();
        let mut wait_op = false;
        let mut ops = [0u8; STACKSIZE];
        let mut args = [0i32; STACKSIZE];
        let (mut opsp, mut argsp) = (-1isize, -1isize);
        self.push_op(&mut ops, &mut opsp, &mut args, &mut argsp, OP_START)?;
        let old = self.refresh_count;
        loop {
            s = &s[skip_spaces(s)..];
            if wait_op {
                let (mut op, used) = operator_op_code(s);
                if op == OP_UNKNOWN || op >= OP_FIRST_FUNCTION {
                    return self.fail(UNKNOWNACTION);
                }
                s = &s[used..];
                if (op == OP_AND || op == OP_OR || op == OP_MOD) && !is_in_list(b" \t'\"{(", at(s.as_bytes(), 0)) {
                    return self.fail(SYNTAX);
                }
                while op_info(op, self.ext59).pri <= op_info(ops[opsp as usize], self.ext59).pri && op_info(ops[opsp as usize], self.ext59).pri != 127 {
                    if ops[opsp as usize] >= OP_FIRST_FUNCTION {
                        args[argsp as usize] += 1;
                    }
                    self.append(&mut items, ops[opsp as usize], args[argsp as usize], Val::Num(0))?;
                    opsp -= 1;
                    argsp -= 1;
                    if opsp < 0 || argsp < 0 {
                        return self.fail(SYNTAX);
                    }
                }
                match op {
                    OP_END => {
                        if opsp != 0 {
                            return self.fail(BRACKNOTFOUND);
                        }
                        return Ok(items);
                    }
                    OP_CLOSEBRACKET => {
                        if ops[opsp as usize] != OP_OPENBRACKET {
                            return self.fail(BRACKNOTFOUND);
                        }
                        opsp -= 1;
                        op = ops[opsp as usize];
                        if op >= OP_FIRST_FUNCTION {
                            let i = op_info(op, self.ext59);
                            let n = args[argsp as usize] + 1;
                            if n < i.min || n > i.max {
                                return self.fail(ARGSCOUNT);
                            }
                        } else {
                            argsp -= 1;
                        }
                    }
                    OP_COMMA => {
                        if opsp > 0 && ops[opsp as usize - 1] >= OP_FIRST_FUNCTION {
                            args[argsp as usize] += 1;
                            if args[argsp as usize] > op_info(ops[opsp as usize - 1], self.ext59).max {
                                return self.fail(ARGSCOUNT);
                            }
                        } else {
                            self.push_op(&mut ops, &mut opsp, &mut args, &mut argsp, OP_COMMA)?;
                        }
                        wait_op = false;
                    }
                    _ => {
                        self.push_op(&mut ops, &mut opsp, &mut args, &mut argsp, op)?;
                        wait_op = false;
                    }
                }
            } else {
                let b = s.as_bytes();
                let c = at(b, 0);
                if is_digit(c) {
                    let (mut n, used) = get_number(s);
                    s = &s[used..];
                    if ops[opsp as usize] == OP_MINUS {
                        n = n.wrapping_neg();
                        opsp -= 1;
                        argsp -= 1;
                    }
                    self.append(&mut items, OP_VALUE, 0, Val::Num(n))?;
                    wait_op = true;
                } else if is_in_list(QUOTS, c) {
                    let (t, used) = self.get_string(s)?;
                    s = &s[used..];
                    self.append(&mut items, OP_VALUE, 0, Val::Str(t))?;
                    wait_op = true;
                } else if c == b'{' {
                    let (t, used) = self.get_qstring(s)?;
                    s = &s[used..];
                    self.append(&mut items, OP_VALUE, 0, Val::Str(t))?;
                    wait_op = true;
                } else if c == b'+' {
                    s = &s[1..];
                } else if c == b'-' {
                    self.push_op(&mut ops, &mut opsp, &mut args, &mut argsp, OP_MINUS)?;
                    s = &s[1..];
                } else if c == b'(' {
                    self.push_op(&mut ops, &mut opsp, &mut args, &mut argsp, OP_OPENBRACKET)?;
                    s = &s[1..];
                } else if c == b')' {
                    let op = ops[opsp as usize];
                    if op != OP_OPENBRACKET {
                        return self.fail(if op >= OP_FIRST_FUNCTION { ARGSCOUNT } else { SYNTAX });
                    }
                    opsp -= 1;
                    let op = ops[opsp as usize];
                    if op < OP_FIRST_FUNCTION {
                        return self.fail(SYNTAX);
                    }
                    if op_info(op, self.ext59).min != 0 {
                        return self.fail(ARGSCOUNT);
                    }
                    s = &s[1..];
                    self.append(&mut items, op, 0, Val::Num(0))?;
                    opsp -= 1;
                    argsp -= 1;
                    wait_op = true;
                } else if !is_in_list_eol(DELIMS, c) {
                    let (name, used) = self.get_name(s)?;
                    let op = function_op_code(name, self.ext59);
                    if op >= OP_FIRST_FUNCTION {
                        s = &s[used..];
                        if at(s.as_bytes(), 0) == b'(' {
                            self.push_op(&mut ops, &mut opsp, &mut args, &mut argsp, op)?;
                            self.push_op(&mut ops, &mut opsp, &mut args, &mut argsp, OP_OPENBRACKET)?;
                            s = &s[1..];
                            argsp -= 1;
                        } else if op_info(op, self.ext59).min < 2 {
                            if op_info(op, self.ext59).min != 0 {
                                self.push_op(&mut ops, &mut opsp, &mut args, &mut argsp, op)?;
                            } else {
                                self.append(&mut items, op, 0, Val::Num(0))?;
                                wait_op = true;
                            }
                        } else {
                            return self.fail(BRACKSNOTFOUND);
                        }
                    } else {
                        let v = self.get_var(name)?;
                        s = &s[used..];
                        if self.refresh_count != old {
                            return Err(Abort::Refresh);
                        }
                        self.append(&mut items, OP_VALUE, 0, v)?;
                        wait_op = true;
                    }
                } else {
                    let op = ops[opsp as usize];
                    return self.fail(if op >= OP_FIRST_FUNCTION { ARGSCOUNT } else { SYNTAX });
                }
            }
        }
    }

    /// qspExprValue
    pub(crate) fn expr_value(&mut self, expr: &str) -> R<Val> {
        let items = self.compile_expr(expr)?;
        if items.is_empty() {
            return Ok(Val::Num(0));
        }
        self.eval_items(items)
    }

    fn eval_items(&mut self, items: Vec<Item>) -> R<Val> {
        let old = self.refresh_count;
        let mut stack: Vec<Val> = Vec::with_capacity(STACKSIZE);
        let mut tos = Val::Num(0);
        for it in items {
            let info = op_info(it.op, self.ext59);
            let mut args: Vec<Val> = Vec::new();
            if it.argc > 0 {
                if stack.len() + 1 < it.argc {
                    return self.fail(SYNTAX);
                }
                let base = stack.len() - (it.argc - 1);
                args = stack.drain(base..).collect();
                args.push(std::mem::replace(&mut tos, Val::Num(0)));
                for i in 0..it.argc {
                    let t = info.types[i];
                    if t != 0 && args[i].conv_to(t == 1) {
                        return self.fail(TYPEMISMATCH);
                    }
                }
            } else {
                if stack.len() == STACKSIZE {
                    return self.fail(STACKOVERFLOW);
                }
                stack.push(std::mem::replace(&mut tos, Val::Num(0)));
            }
            tos = self.apply_op(it.op, it.val, args)?;
            self.aborted(old)?;
        }
        Ok(tos)
    }

    fn apply_op(&mut self, op: u8, value: Val, mut a: Vec<Val>) -> R<Val> {
        let cmp_res = |c: bool| Val::Num(-(c as i32));
        Ok(match op {
            OP_VALUE => value,
            OP_MUL => Val::Num(a[0].num().wrapping_mul(a[1].num())),
            OP_DIV => {
                if a[1].num() == 0 {
                    return self.fail(DIVBYZERO);
                }
                Val::Num(a[0].num().wrapping_div(a[1].num()))
            }
            OP_ADD => {
                if a[0].is_str() && a[1].is_str() {
                    Val::Str(format!("{}{}", a[0].s(), a[1].s()))
                } else if a[0].can_num() && a[1].can_num() {
                    a[0].conv_to(false);
                    a[1].conv_to(false);
                    Val::Num(a[0].num().wrapping_add(a[1].num()))
                } else {
                    a[0].conv_to(true);
                    a[1].conv_to(true);
                    Val::Str(format!("{}{}", a[0].s(), a[1].s()))
                }
            }
            OP_SUB => Val::Num(a[0].num().wrapping_sub(a[1].num())),
            OP_MOD => {
                if a[1].num() == 0 {
                    return self.fail(DIVBYZERO);
                }
                Val::Num(a[0].num().wrapping_rem(a[1].num()))
            }
            OP_APPEND => Val::Str(format!("{}{}", a[0].s(), a[1].s())),
            OP_COMMA => Val::Str(format!("{}|{}", a[0].s(), a[1].s())),
            OP_EQ | OP_LT | OP_GT | OP_LEQ | OP_GEQ | OP_NE => {
                let (x, y) = a.split_at_mut(1);
                let c = auto_compare(&mut x[0], &mut y[0]);
                cmp_res(match op {
                    OP_EQ => c == 0,
                    OP_LT => c < 0,
                    OP_GT => c > 0,
                    OP_LEQ => c <= 0,
                    OP_GEQ => c >= 0,
                    _ => c != 0,
                })
            }
            OP_MINUS => Val::Num(a[0].num().wrapping_neg()),
            OP_LOC => cmp_res(self.loc_index(a[0].s()) >= 0),
            OP_OBJ => cmp_res(self.obj_index(a[0].s()) >= 0),
            OP_NOT => Val::Num(!a[0].num()),
            OP_AND => Val::Num(a[0].num() & a[1].num()),
            OP_OR => Val::Num(a[0].num() | a[1].num()),
            OP_IIF => {
                if a[0].num() != 0 { a.swap_remove(1) } else { a.swap_remove(2) }
            }
            OP_LEN => Val::Num(ulen(a[0].s()) as i32),
            OP_ISNUM => cmp_res(a[0].can_num()),
            OP_LCASE => Val::Str(lower(a[0].s())),
            OP_UCASE => Val::Str(upper(a[0].s())),
            OP_STR => Val::Str(a[0].s().to_string()),
            OP_VAL => {
                if a[0].conv_to(false) { Val::Num(0) } else { Val::Num(a[0].num()) }
            }
            OP_ARRSIZE => Val::Num(self.array_size(a[0].s())),
            OP_TRIM => Val::Str(del_spc(a[0].s()).to_string()),
            OP_INPUT => Val::Str(self.call_input_box(a[0].s())),
            OP_RND => Val::Num(self.rand.next() % 1000 + 1),
            OP_COUNTOBJ => Val::Num(if a.is_empty() { self.cur_objects.len() as i32 } else { self.count_objects_by_name(a[0].s()) }),
            crate::ext59::OP_ARRITEM => self.func_arr_item(&a)?,
            OP_MSECSCOUNT => Val::Num(self.get_time()),
            OP_QSPVER => Val::Str(QSP_VER.to_string()),
            OP_USERTEXT => Val::Str(self.cur_input.clone()),
            OP_CURLOC => Val::Str(if self.cur_loc >= 0 { self.locs[self.cur_loc as usize].name.clone() } else { String::new() }),
            OP_SELOBJ => Val::Str(if self.cur_sel_object >= 0 { self.cur_objects[self.cur_sel_object as usize].desc.clone() } else { String::new() }),
            OP_SELACT => Val::Str(if self.cur_sel_action >= 0 { self.cur_actions[self.cur_sel_action as usize].desc.clone() } else { String::new() }),
            OP_MAINTEXT => Val::Str(self.cur_desc.clone()),
            OP_STATTEXT => Val::Str(self.cur_vars.clone()),
            OP_CURACTS => Val::Str(self.get_all_actions_as_code()),
            // ---- Funktionen mit eigener Logik ----
            OP_STRCOMP => {
                match compile_anchored(a[1].s()) {
                    None => return self.fail(INCORRECTREGEXP),
                    Some(re) => cmp_res(full_match(&re, a[0].s())),
                }
            }
            OP_STRFIND | OP_STRPOS => {
                let re = match fancy_regex::Regex::new(a[1].s()) {
                    Ok(r) => r,
                    Err(_) => return self.fail(INCORRECTREGEXP),
                };
                let pos = if a.len() == 3 && a[2].num() >= 0 { a[2].num() as usize } else { 0 };
                let text = a[0].s().to_string();
                let m = re.captures(&text).ok().flatten().and_then(|c| c.get(pos));
                if op == OP_STRFIND {
                    Val::Str(m.map(|m| m.as_str().to_string()).unwrap_or_default())
                } else {
                    Val::Num(m.map(|m| byte_to_u16(&text, m.start()) as i32 + 1).unwrap_or(0))
                }
            }
            OP_RGB => {
                let c = |v: i32| v.clamp(0, 255);
                let (r, g, b) = (c(a[0].num()), c(a[1].num()), c(a[2].num()));
                let al = if a.len() == 4 { c(a[3].num()) } else { 255 };
                Val::Num((al << 24) | (b << 16) | (g << 8) | r)
            }
            OP_MID => {
                let mut beg = a[1].num().wrapping_sub(1);
                if beg < 0 {
                    beg = 0;
                }
                let total = ulen(a[0].s()) as i32;
                if beg < total {
                    let mut len = total - beg;
                    if a.len() == 3 {
                        let sub = a[2].num();
                        if sub < 0 {
                            len = 0;
                        } else if sub < len {
                            len = sub;
                        }
                    }
                    Val::Str(usub(a[0].s(), beg as usize, len as usize))
                } else {
                    Val::Str(String::new())
                }
            }
            OP_RAND => {
                let mut min = a[0].num();
                let mut max = if a.len() == 2 { a[1].num() } else { 0 };
                if min > max {
                    std::mem::swap(&mut min, &mut max);
                }
                let m = max.wrapping_sub(min).wrapping_add(1);
                let r = self.rand.next();
                Val::Num(if m == 0 { 0 } else { r.wrapping_rem(m).wrapping_add(min) })
            }
            OP_DESC => {
                let i = self.loc_index(a[0].s());
                if i < 0 {
                    return self.fail(LOCNOTFOUND);
                }
                let d = self.locs[i as usize].desc.clone();
                Val::Str(self.format_text(&d)?)
            }
            OP_GETOBJ => {
                let i = a[0].num().wrapping_sub(1);
                if i >= 0 && (i as usize) < self.cur_objects.len() {
                    Val::Str(self.cur_objects[i as usize].desc.clone())
                } else {
                    Val::Str(String::new())
                }
            }
            OP_ISPLAY => {
                if is_any_string(a[0].s()) {
                    let f = a[0].s().to_string();
                    cmp_res(self.call_is_playing(&f))
                } else {
                    Val::Num(0)
                }
            }
            OP_INSTR => {
                let count = a.len();
                if a[0].conv_to(count == 2) {
                    return self.fail(TYPEMISMATCH);
                }
                let (txt, pat, beg) = if count == 2 {
                    (a[0].s().to_string(), a[1].s().to_string(), 0i32)
                } else {
                    (a[1].s().to_string(), a[2].s().to_string(), (a[0].num().wrapping_sub(1)).max(0))
                };
                let t: Vec<u16> = txt.encode_utf16().collect();
                let p: Vec<u16> = pat.encode_utf16().collect();
                let beg = beg as usize;
                if beg < t.len() {
                    let found = if p.is_empty() {
                        Some(beg)
                    } else {
                        t[beg..].windows(p.len()).position(|w| w == &p[..]).map(|x| x + beg)
                    };
                    Val::Num(found.map(|x| x as i32 + 1).unwrap_or(0))
                } else {
                    Val::Num(0)
                }
            }
            OP_ARRPOS | OP_ARRCOMP => {
                let rx = op == OP_ARRCOMP;
                if a.len() == 2 {
                    let name = a[0].s().to_string();
                    let (_, rest) = a.split_at_mut(1);
                    Val::Num(self.array_pos(&name, &mut rest[0], 0, rx))
                } else {
                    let ind = num_of(&a[0]);
                    let name = a[1].s().to_string();
                    let (_, rest) = a.split_at_mut(2);
                    Val::Num(self.array_pos(&name, &mut rest[0], ind, rx))
                }
            }
            OP_REPLACE => {
                let search = a[1].s().to_string();
                if search.is_empty() {
                    Val::Str(a[0].s().to_string())
                } else if a.len() == 2 {
                    Val::Str(a[0].s().replace(&search, ""))
                } else if a.len() == 4 {
                    Val::Str(if a[3].num() > 0 { a[0].s().replacen(&search, a[2].s(), a[3].num() as usize) } else { a[0].s().to_string() })
                } else {
                    Val::Str(a[0].s().replace(&search, a[2].s()))
                }
            }
            OP_MIN | OP_MAX => {
                if a.len() == 1 {
                    a[0].conv_to(true);
                    let name = a[0].s().to_string();
                    self.array_min_max(&name, op == OP_MIN)
                } else {
                    let mut best = 0usize;
                    for i in 1..a.len() {
                        let (lo, hi) = a.split_at_mut(i);
                        let c = auto_compare(&mut hi[0], &mut lo[best]);
                        if (op == OP_MIN && c < 0) || (op == OP_MAX && c > 0) {
                            best = i;
                        }
                    }
                    a.swap_remove(best)
                }
            }
            OP_FUNC => {
                let name = a[0].s().to_string();
                self.func_call(&name, &a[1..], false)?
            }
            OP_DYNEVAL => {
                let code = a[0].s().to_string();
                self.func_call(&code, &a[1..], true)?
            }
            _ => Val::Num(0),
        })
    }

    /// FUNC / DYNEVAL: führt aus und liefert RESULT zurück (RESULT des Aufrufers bleibt erhalten).
    fn func_call(&mut self, target: &str, args: &[Val], is_code: bool) -> R<Val> {
        let vi = self.var_ref("RESULT", true)?;
        let saved = self.take_var(vi);
        let old = self.refresh_count;
        let r = if is_code {
            self.exec_string_as_code_with_args(target, args)
        } else {
            self.exec_loc_by_name_with_args(target, args)
        };
        r?;
        if self.refresh_count != old {
            return Err(Abort::Refresh);
        }
        let vi = self.var_ref("RESULT", true)?;
        let out = match self.vars[vi].data.values.first() {
            Some(v) => match &v.str {
                Some(t) => Val::Str(t.clone()),
                None => Val::Num(v.num),
            },
            None => Val::Str(String::new()),
        };
        self.empty_var(vi);
        self.put_var(vi, saved);
        Ok(out)
    }
}
