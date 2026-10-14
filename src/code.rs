//! Zeilen-Vorverarbeitung (Statement-Cache) – Port von codetools.c / qspInitLineOfCode.
use crate::text::*;
use std::sync::OnceLock;

pub mod st {
    pub const UNKNOWN: u8 = 0;
    pub const LABEL: u8 = 1;
    pub const COMMENT: u8 = 2;
    pub const ACT: u8 = 3;
    pub const FOR: u8 = 4;
    pub const LOCAL: u8 = 5;
    pub const IF: u8 = 6;
    pub const ELSEIF: u8 = 7;
    pub const ELSE: u8 = 8;
    pub const END: u8 = 9;
    pub const ADDOBJ: u8 = 10;
    pub const CLA: u8 = 11;
    pub const CLEAR: u8 = 12;
    pub const CLOSEALL: u8 = 13;
    pub const CLOSE: u8 = 14;
    pub const CLS: u8 = 15;
    pub const CMDCLEAR: u8 = 16;
    pub const COPYARR: u8 = 17;
    pub const DELACT: u8 = 18;
    pub const DELOBJ: u8 = 19;
    pub const DYNAMIC: u8 = 20;
    pub const EXEC: u8 = 21;
    pub const EXIT: u8 = 22;
    pub const FREELIB: u8 = 23;
    pub const GOSUB: u8 = 24;
    pub const GOTO: u8 = 25;
    pub const INCLIB: u8 = 26;
    pub const JUMP: u8 = 27;
    pub const KILLALL: u8 = 28;
    pub const KILLOBJ: u8 = 29;
    pub const KILLVAR: u8 = 30;
    pub const MCLEAR: u8 = 31;
    pub const MENU: u8 = 32;
    pub const MNL: u8 = 33;
    pub const MPL: u8 = 34;
    pub const MP: u8 = 35;
    pub const MSG: u8 = 36;
    pub const NL: u8 = 37;
    pub const OPENGAME: u8 = 38;
    pub const OPENQST: u8 = 39;
    pub const PLAY: u8 = 40;
    pub const PL: u8 = 41;
    pub const P: u8 = 42;
    pub const REFINT: u8 = 43;
    pub const SAVEGAME: u8 = 44;
    pub const SETTIMER: u8 = 45;
    pub const SET: u8 = 46;
    pub const SHOWACTS: u8 = 47;
    pub const SHOWINPUT: u8 = 48;
    pub const SHOWOBJS: u8 = 49;
    pub const SHOWVARS: u8 = 50;
    pub const UNSELECT: u8 = 51;
    pub const VIEW: u8 = 52;
    pub const WAIT: u8 = 53;
    pub const XGOTO: u8 = 54;
}
use st::*;

#[derive(Clone, Copy, Debug)]
pub struct Stat {
    pub kind: u8,
    pub end: usize,
    pub param: usize,
}
#[derive(Clone, Debug)]
pub struct Line {
    pub text: String,
    pub line_num: i32,
    pub is_multiline: bool,
    pub label: Option<String>,
    pub stats: Vec<Stat>,
}

/// Argument-Spezifikation eines Statements: (min, max, Typen, extArg)
/// Typen: 0 = beliebig, 1 = Text, 2 = Zahl
pub struct StatSpec {
    pub min: usize,
    pub max: usize,
    pub types: [u8; 10],
    pub ext: i32,
}
pub fn stat_spec(code: u8) -> StatSpec {
    let s = |min, max, t: &[u8], ext| {
        let mut types = [0u8; 10];
        types[..t.len()].copy_from_slice(t);
        StatSpec { min, max, types, ext }
    };
    let any10: &[u8] = &[1, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    match code {
        ELSEIF | IF => s(1, 1, &[2], 0),
        ACT => s(1, 2, &[1, 1], 0),
        ADDOBJ => s(1, 3, &[1, 1, 2], 0),
        CLA => s(0, 0, &[], 3),
        CLOSEALL => s(0, 0, &[], 1),
        CLOSE => s(0, 1, &[1], 0),
        CLS => s(0, 0, &[], 4),
        CMDCLEAR => s(0, 0, &[], 2),
        COPYARR => s(2, 4, &[1, 1, 2, 2], 0),
        DELACT => s(1, 1, &[1], 0),
        DELOBJ => s(1, 1, &[1], 0),
        DYNAMIC => s(1, 10, any10, 0),
        EXEC => s(1, 1, &[1], 0),
        FREELIB => s(0, 0, &[], 6),
        GOSUB => s(1, 10, any10, 0),
        GOTO => s(1, 10, any10, 1),
        INCLIB => s(1, 1, &[1], 1),
        JUMP => s(1, 1, &[1], 0),
        KILLALL => s(0, 0, &[], 5),
        KILLOBJ => s(0, 1, &[2], 1),
        KILLVAR => s(0, 2, &[1, 2], 0),
        MENU => s(1, 3, &[1, 2, 2], 0),
        MCLEAR => s(0, 0, &[], 1),
        MNL => s(0, 1, &[1], 5),
        MPL => s(0, 1, &[1], 3),
        MP => s(1, 1, &[1], 1),
        CLEAR => s(0, 0, &[], 0),
        NL => s(0, 1, &[1], 4),
        PL => s(0, 1, &[1], 2),
        P => s(1, 1, &[1], 0),
        MSG => s(1, 1, &[1], 0),
        OPENGAME => s(0, 1, &[1], 0),
        OPENQST => s(1, 1, &[1], 0),
        PLAY => s(1, 2, &[1, 2], 0),
        SAVEGAME => s(0, 1, &[1], 0),
        SETTIMER => s(1, 1, &[2], 0),
        SHOWACTS => s(1, 1, &[2], 0),
        SHOWINPUT => s(1, 1, &[2], 3),
        SHOWOBJS => s(1, 1, &[2], 1),
        SHOWVARS => s(1, 1, &[2], 2),
        VIEW => s(0, 1, &[1], 0),
        WAIT => s(1, 1, &[2], 0),
        XGOTO => s(1, 10, any10, 0),
        _ => s(0, 0, &[], 0),
    }
}

struct StatNames {
    levels: [Vec<(String, u8)>; 3],
    max_len: usize,
}
fn stat_names() -> &'static StatNames {
    static N: OnceLock<StatNames> = OnceLock::new();
    N.get_or_init(|| {
        let l2: &[(&str, u8)] = &[
            ("ELSE", ELSE), ("END", END), ("LOCAL", LOCAL), ("SET", SET), ("LET", SET), ("IF", IF),
            ("ACT", ACT), ("FOR", FOR), ("ADDOBJ", ADDOBJ), ("ADD OBJ", ADDOBJ), ("CLA", CLA),
            ("CLOSE", CLOSE), ("CLS", CLS), ("CMDCLEAR", CMDCLEAR), ("CMDCLR", CMDCLEAR),
            ("COPYARR", COPYARR), ("DELACT", DELACT), ("DEL ACT", DELACT), ("DELOBJ", DELOBJ),
            ("DEL OBJ", DELOBJ), ("DYNAMIC", DYNAMIC), ("EXEC", EXEC), ("EXIT", EXIT),
            ("FREELIB", FREELIB), ("DELLIB", FREELIB), ("KILLQST", FREELIB), ("GOSUB", GOSUB),
            ("GS", GOSUB), ("GOTO", GOTO), ("GT", GOTO), ("INCLIB", INCLIB), ("ADDLIB", INCLIB),
            ("ADDQST", INCLIB), ("JUMP", JUMP), ("KILLALL", KILLALL), ("KILLOBJ", KILLOBJ),
            ("KILLVAR", KILLVAR), ("MENU", MENU), ("*CLEAR", MCLEAR), ("*CLR", MCLEAR),
            ("*NL", MNL), ("*P", MP), ("CLEAR", CLEAR), ("CLR", CLEAR), ("NL", NL), ("P", P),
            ("MSG", MSG), ("OPENGAME", OPENGAME), ("OPENQST", OPENQST), ("REFINT", REFINT),
            ("SAVEGAME", SAVEGAME), ("SHOWACTS", SHOWACTS), ("SHOWINPUT", SHOWINPUT),
            ("SHOWOBJS", SHOWOBJS), ("SHOWSTAT", SHOWVARS), ("UNSEL", UNSELECT), ("VIEW", VIEW),
            ("WAIT", WAIT), ("XGOTO", XGOTO), ("XGT", XGOTO),
        ];
        let l1: &[(&str, u8)] = &[
            ("ELSEIF", ELSEIF), ("CLOSE ALL", CLOSEALL), ("*PL", MPL), ("PL", PL),
            ("SETTIMER", SETTIMER), ("UNSELECT", UNSELECT),
        ];
        let l0: &[(&str, u8)] = &[("PLAY", PLAY)];
        let mk = |l: &[(&str, u8)]| {
            let mut v: Vec<(String, u8)> = l.iter().map(|(n, c)| (n.to_string(), *c)).collect();
            v.sort_by(|a, b| a.0.cmp(&b.0));
            v
        };
        let levels = [mk(l0), mk(l1), mk(l2)];
        let max_len = levels.iter().flatten().map(|(n, _)| n.len()).max().unwrap_or(0);
        StatNames { levels, max_len }
    })
}

/// qspGetStatCode: liefert (Code, Parameter-Offset relativ zu `s`).
pub fn get_stat_code(s: &str) -> (u8, Option<usize>) {
    let b = s.as_bytes();
    if b.is_empty() {
        return (UNKNOWN, None);
    }
    if b[0] == b':' {
        return (LABEL, None);
    }
    if b[0] == b'!' {
        return (COMMENT, None);
    }
    let sn = stat_names();
    let key: String = upper(&s.chars().take(sn.max_len).collect::<String>());
    for level in &sn.levels {
        if let Some(i) = bsearch(level, |e| ncmp(key.as_bytes(), e.0.as_bytes())) {
            let len = level[i].0.len();
            if is_in_list_eol(DELIMS, at(b, len)) {
                return (level[i].1, Some(len));
            }
        }
    }
    (UNKNOWN, None)
}

pub fn get_line_label(s: &str) -> Option<String> {
    let s = &s[skip_spaces(s)..];
    if s.starts_with(':') {
        let part = match s.find('&') {
            Some(d) => &s[1..d],
            None => &s[1..],
        };
        return Some(upper(del_spc(part)));
    }
    None
}

fn find_delim(str: &str, from: usize, pat: &str) -> Option<usize> {
    str_pos(&str[from..], pat, false).map(|p| p + from)
}

/// Port von qspInitLineOfCode.
pub fn init_line(text: String, line_num: i32) -> Line {
    let str = text.as_str();
    let len = str.len();
    let mut stats: Vec<Stat> = Vec::new();
    let mut buf = skip_spaces(str);
    let (mut stat, rel) = get_stat_code(&str[buf..]);
    let mut param: Option<usize> = rel.map(|r| r + buf);
    let mut delim: Option<usize> = None;
    let mut next = 0usize;
    let mut in_loop = true;
    let mut search_else = true;
    let mut else_pos: Option<usize> = None;
    let u = upper(str);

    // Hilfsfunktion: nächste Statement-Grenze bestimmen (gemeinsam für erste Zeile und Schleife)
    // `first` unterscheidet die kleinen Unterschiede der beiden Stellen im Original.
    macro_rules! analyse {
        ($first:expr) => {
            match stat {
                ACT | FOR | IF | ELSEIF => {
                    delim = find_delim(str, buf, ":");
                    if let Some(d) = delim {
                        next = d + 1;
                        if next >= len {
                            in_loop = false;
                        }
                    }
                }
                ELSE => {
                    let p = param.unwrap_or(buf);
                    next = p + skip_spaces(&str[p..]);
                    if at(str.as_bytes(), next) == b':' {
                        next += 1;
                    }
                    delim = if next < len { Some(next) } else { None };
                }
                _ => {
                    delim = find_delim(str, buf, "&");
                    if let Some(d) = delim {
                        next = d + 1;
                    }
                    if !$first {
                        if let Some(e) = else_pos {
                            if buf >= e {
                                else_pos = None;
                            }
                        }
                    }
                    if $first || (else_pos.is_none() && search_else) {
                        else_pos = str_pos(&u[buf..], "ELSE", true).map(|p| p + buf);
                        if else_pos.is_none() {
                            search_else = false;
                        }
                    }
                    if let Some(e) = else_pos {
                        if delim.map_or(true, |d| e < d) {
                            next = e;
                            delim = Some(e);
                            else_pos = None;
                        }
                    }
                    if stat == UNKNOWN && Some(buf) != delim {
                        let end = delim.unwrap_or(len);
                        let temp = str_pos(&str[buf..end], "=", false);
                        stat = if temp.is_some() { SET } else { MPL };
                    }
                }
            }
        };
    }

    if buf < len && stat != COMMENT {
        analyse!(true);
        while delim.is_some() && in_loop {
            stats.push(Stat {
                kind: stat,
                end: delim.unwrap(),
                param: match param {
                    Some(p) => p + skip_spaces(&str[p..]),
                    None => buf,
                },
            });
            buf = next + skip_spaces(&str[next.min(len)..]);
            let (s2, r2) = get_stat_code(&str[buf.min(len)..]);
            stat = s2;
            param = r2.map(|r| r + buf);
            if buf < len && stat != COMMENT {
                analyse!(false);
            } else {
                delim = None;
            }
        }
    }
    // ELSE IF
    let count;
    if stats.len() == 1
        && delim.is_some()
        && stats[0].kind == ELSE
        && stat == IF
        && at(str.as_bytes(), stats[0].param) != b':'
    {
        count = 0;
        stat = ELSEIF;
    } else {
        count = stats.len();
        stats.push(Stat { kind: 0, end: 0, param: 0 });
    }
    stats[count] = Stat {
        kind: stat,
        end: delim.unwrap_or(len),
        param: match param {
            Some(p) => p + skip_spaces(&str[p.min(len)..]),
            None => buf,
        },
    };
    let is_multiline = match stats[0].kind {
        ACT | FOR | IF | ELSEIF => stats.len() == 1 && at(str.as_bytes(), stats[0].end) == b':',
        _ => false,
    };
    let label = get_line_label(str);
    Line { text, line_num, is_multiline, label, stats }
}

/// qspProcessPreformattedStrings
fn preformatted(data: &str) -> Vec<(String, i32)> {
    let d = data.as_bytes();
    let mut out: Vec<(String, i32)> = Vec::new();
    let mut cur: Vec<u8> = Vec::new();
    let (mut line_num, mut last_line_num) = (0i32, 0i32);
    let mut quots = 0i32;
    let mut quot = 0u8;
    let mut i = 0usize;
    let finish = |cur: &mut Vec<u8>, ln: i32, out: &mut Vec<(String, i32)>| {
        let s = String::from_utf8_lossy(cur).into_owned();
        out.push((del_spc(&s).to_string(), ln));
        cur.clear();
    };
    while i < d.len() {
        let is_nl = d[i] == b'\r' && at(d, i + 1) == b'\n';
        if is_nl {
            line_num += 1;
        }
        if quots > 0 || quot != 0 || !is_nl {
            cur.push(d[i]);
            if quot != 0 {
                if d[i] == quot {
                    if at(d, i + 1) == quot {
                        cur.push(d[i]);
                        i += 1;
                    } else {
                        quot = 0;
                    }
                }
            } else if d[i] == b'{' {
                quots += 1;
            } else if d[i] == b'}' {
                if quots > 0 {
                    quots -= 1;
                }
            } else if is_in_list(QUOTS, d[i]) {
                quot = d[i];
            }
            i += 1;
        } else {
            finish(&mut cur, last_line_num, &mut out);
            last_line_num = line_num;
            i += 2;
        }
    }
    finish(&mut cur, last_line_num, &mut out);
    out
}

/// qspPreprocessData: Text -> Zeilen mit Statement-Cache.
pub fn preprocess(data: &str) -> Vec<Line> {
    let pre = preformatted(data);
    let count = pre.len();
    let mut out = Vec::new();
    let mut i = 0usize;
    let mut last_num = 0i32;
    while i < count {
        let mut s = pre[i].0.clone();
        if s.len() >= 2 {
            while s.ends_with(" _") {
                i += 1;
                if i == count {
                    break;
                }
                s.truncate(s.len() - 1);
                s.push_str(&pre[i].0);
            }
        }
        out.push(init_line(s, last_num));
        i += 1;
        last_num = pre.get(i).map(|p| p.1).unwrap_or(0);
    }
    out
}

/// Zeilen wieder zu Text verbinden (für CURACTS).
pub fn join_lines(lines: &[std::rc::Rc<Line>], delim: &str) -> String {
    lines.iter().map(|l| l.text.as_str()).collect::<Vec<_>>().join(delim)
}
