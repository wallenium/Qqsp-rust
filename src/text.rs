//! Textwerkzeuge. Alle Positionen sind Byte-Offsets in UTF-8; die Trennzeichen
//! der QSP-Syntax sind ASCII und kollidieren nie mit Multibyte-Folgen.
pub const DELIMS: &[u8] = b" \t&'\"()[]=!<>+-/*:,{}";
pub const SPACES: &[u8] = b" \t";
pub const QUOTS: &[u8] = b"'\"";

/// Längenerhaltende Großschreibung eines Zeichens (Byte-Offsets bleiben gültig).
pub fn up(c: char) -> char {
    if c.is_ascii() {
        return c.to_ascii_uppercase();
    }
    let mut it = c.to_uppercase();
    match (it.next(), it.next()) {
        (Some(u), None) if u.len_utf8() == c.len_utf8() => u,
        _ => c,
    }
}
pub fn low(c: char) -> char {
    if c.is_ascii() {
        return c.to_ascii_lowercase();
    }
    let mut it = c.to_lowercase();
    match (it.next(), it.next()) {
        (Some(u), None) if u.len_utf8() == c.len_utf8() => u,
        _ => c,
    }
}
pub fn upper(s: &str) -> String {
    s.chars().map(up).collect()
}
pub fn lower(s: &str) -> String {
    s.chars().map(low).collect()
}

#[inline]
pub fn at(t: &[u8], i: usize) -> u8 {
    t.get(i).copied().unwrap_or(0)
}
/// qspIsInList: NUL ist nie enthalten.
#[inline]
pub fn is_in_list(list: &[u8], ch: u8) -> bool {
    ch != 0 && has(list, ch)
}
/// qspIsInListEOL: Stringende (NUL) zählt als enthalten.
#[inline]
pub fn is_in_list_eol(list: &[u8], ch: u8) -> bool {
    ch == 0 || has(list, ch)
}
#[inline]
pub fn is_digit(ch: u8) -> bool {
    ch.is_ascii_digit()
}
pub fn skip_spaces(s: &str) -> usize {
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() && is_in_list(SPACES, b[i]) {
        i += 1;
    }
    i
}
pub fn del_spc(s: &str) -> &str {
    s.trim_matches(|c| c == ' ' || c == '\t')
}
pub fn is_any_string(s: &str) -> bool {
    skip_spaces(s) < s.len()
}

/// Ordnung nach Codepunkten (entspricht qspStrsComp).
pub fn cmp_str(a: &str, b: &str) -> i32 {
    match a.cmp(b) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

/// Kurze Zeichenlisten: einfache Schleife ist schneller als memchr-Aufruf.
#[inline(always)]
fn has(list: &[u8], ch: u8) -> bool {
    let mut i = 0;
    while i < list.len() {
        if list[i] == ch {
            return true;
        }
        i += 1;
    }
    false
}

pub fn str_p_brk(s: &str, set: &[u8]) -> Option<usize> {
    s.as_bytes().iter().position(|&b| has(set, b))
}

pub fn in_str_r_chars(s: &str, chars: &[u8], end: Option<usize>) -> Option<usize> {
    let b = s.as_bytes();
    let mut pos = None;
    for (i, &c) in b.iter().enumerate() {
        if Some(i) == end {
            break;
        }
        if is_in_list(chars, c) {
            pos = Some(i);
        }
    }
    pos
}

/// qspStrToNum ohne Endprüfung (endChar == NULL).
pub fn str_to_num_raw(s: &str) -> i32 {
    parse_num(s).0
}
/// qspStrToNum mit Endprüfung: `None`, wenn nach der Zahl noch Text folgt.
pub fn str_to_num(s: &str) -> Option<i32> {
    let (n, rest) = parse_num(s);
    if rest { None } else { Some(n) }
}
fn parse_num(s: &str) -> (i32, bool) {
    let b = s.as_bytes();
    let mut i = skip_spaces(s);
    let mut neg = false;
    if at(b, i) == b'-' {
        neg = true;
        i += 1;
    } else if at(b, i) == b'+' {
        i += 1;
    }
    let mut num: i32 = 0;
    while is_digit(at(b, i)) {
        num = num.wrapping_mul(10).wrapping_add((b[i] - b'0') as i32);
        i += 1;
    }
    let rest_start = i + skip_spaces(&s[i..]);
    let rest = rest_start < s.len();
    (if neg { num.wrapping_neg() } else { num }, rest)
}

/// Port von qspStrPos (Suche außerhalb von Anführungszeichen/Klammern).
pub fn str_pos(txt: &str, pat: &str, isolated: bool) -> Option<usize> {
    let t = txt.as_bytes();
    let p = pat.as_bytes();
    let first = txt.find(pat)?;
    if !(isolated || t.iter().any(|b| b"'\"{([".contains(b))) {
        return Some(first);
    }
    let n = p.len();
    let lim = t.len() + 1 - n.min(t.len() + 1);
    let (mut c1, mut c2, mut c3) = (0i32, 0i32, 0i32);
    let mut last_delim = true;
    let mut i = 0usize;
    while i < lim {
        if is_in_list(QUOTS, t[i]) {
            let quot = t[i];
            loop {
                i += 1;
                if i >= lim {
                    break;
                }
                if t[i] == quot {
                    i += 1;
                    if at(t, i) != quot {
                        break;
                    }
                }
            }
            if i >= lim {
                return None;
            }
            last_delim = true;
        }
        match at(t, i) {
            b'(' => c1 += 1,
            b')' => {
                if c1 > 0 {
                    c1 -= 1
                }
            }
            b'[' => c2 += 1,
            b']' => {
                if c2 > 0 {
                    c2 -= 1
                }
            }
            b'{' => c3 += 1,
            b'}' => {
                if c3 > 0 {
                    c3 -= 1
                }
            }
            _ => {}
        }
        if c1 == 0 && c2 == 0 && c3 == 0 {
            if isolated {
                if is_in_list(DELIMS, at(t, i)) {
                    last_delim = true;
                } else if last_delim {
                    if is_in_list_eol(DELIMS, at(t, i + n)) && t.get(i..i + n) == Some(p) {
                        return Some(i);
                    }
                    last_delim = false;
                }
            } else if t.get(i..i + n) == Some(p) {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

// ---- UTF-16-Einheiten (QSP rechnet in 16-Bit-Zeichen) ----
pub fn ulen(s: &str) -> usize {
    if s.is_ascii() { s.len() } else { s.encode_utf16().count() }
}
/// Teilstring nach UTF-16-Einheiten.
pub fn usub(s: &str, start: usize, len: usize) -> String {
    if s.is_ascii() {
        let e = (start + len).min(s.len());
        return s[start.min(e)..e].to_string();
    }
    let v: Vec<u16> = s.encode_utf16().collect();
    let e = (start + len).min(v.len());
    String::from_utf16_lossy(&v[start.min(e)..e])
}
/// Byte-Offset -> UTF-16-Einheiten-Offset.
pub fn byte_to_u16(s: &str, byte: usize) -> usize {
    ulen(&s[..byte])
}

/// glibc-kompatible Binärsuche; `cmp(elem)` liefert den Vergleich Schlüssel↔Element.
pub fn bsearch<T>(arr: &[T], mut cmp: impl FnMut(&T) -> i32) -> Option<usize> {
    let (mut l, mut u) = (0usize, arr.len());
    while l < u {
        let idx = (l + u) / 2;
        let c = cmp(&arr[idx]);
        if c < 0 {
            u = idx;
        } else if c > 0 {
            l = idx + 1;
        } else {
            return Some(idx);
        }
    }
    None
}
/// Präfix-Vergleich wie qspStrsNComp(key, name, name.len()).
pub fn ncmp(key: &[u8], name: &[u8]) -> i32 {
    for i in 0..name.len() {
        let a = at(key, i) as i32;
        let b = name[i] as i32;
        if a != b {
            return a - b;
        }
        if a == 0 {
            return 0;
        }
    }
    0
}
