//! .qsp-Spieldateien und Spielstände: Kodierung (coding.c) und CRC.
use crate::cp1251::CP1251;
use crate::crc::CRC_TABLE;

const CODREMOV: u16 = 5;

pub fn crc(data: &[u8]) -> i32 {
    let mut c: i32 = 0;
    for &b in data {
        c = (CRC_TABLE[(((c & 0xFF) ^ b as i32) & 0xFF) as usize] as i32 ^ (c >> 8)) ^ (0xD202EF8Du32 as i32);
    }
    c
}

/// Zerlegt die (mit Nullen abgeschlossenen) Spieldaten in Zeilen (16-Bit-Einheiten).
pub fn split_game(data: &[u8], ucs2: bool) -> Vec<Vec<u16>> {
    let units: Vec<u16> = if ucs2 {
        data.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).take_while(|&u| u != 0).collect()
    } else {
        data.iter().take_while(|&&b| b != 0).map(|&b| b as u16).collect()
    };
    split_units(&units)
}
/// Trennt nach "\r\n".
pub fn split_units(units: &[u16]) -> Vec<Vec<u16>> {
    let mut out = Vec::new();
    let mut cur = Vec::new();
    let mut i = 0;
    while i < units.len() {
        if units[i] == 13 && i + 1 < units.len() && units[i + 1] == 10 {
            out.push(std::mem::take(&mut cur));
            i += 2;
        } else {
            cur.push(units[i]);
            i += 1;
        }
    }
    out.push(cur);
    out
}

/// qspGameToQSPString
pub fn game_to_string(u: &[u16], ucs2: bool, coded: bool) -> String {
    let mut v: Vec<u16> = Vec::with_capacity(u.len());
    for &x in u {
        if ucs2 {
            let c = if coded { if x == 0xFFFB { CODREMOV } else { x.wrapping_add(CODREMOV) } } else { x };
            v.push(c);
        } else {
            let mut b = x as u8;
            if coded {
                b = if b == 0xFB { CODREMOV as u8 } else { b.wrapping_add(CODREMOV as u8) };
            }
            v.push(if b >= 0x80 { CP1251[(b - 0x80) as usize] } else { b as u16 });
        }
    }
    String::from_utf16_lossy(&v)
}

/// Spielstand: Zeichen +5 (qspCodeReCode, decodieren)
pub fn decode_units(u: &[u16]) -> Vec<u16> {
    u.iter().map(|&x| if x == 0xFFFB { CODREMOV } else { x.wrapping_add(CODREMOV) }).collect()
}
pub fn encode_units(u: &[u16]) -> Vec<u16> {
    u.iter().map(|&x| if x == CODREMOV { 0xFFFB } else { x.wrapping_sub(CODREMOV) }).collect()
}

/// Schreibt einen Spielstand zeilenweise (UTF-16).
#[derive(Default)]
pub struct SaveWriter {
    pub buf: Vec<u16>,
}
impl SaveWriter {
    pub fn val(&mut self, v: Option<&str>, coded: bool) {
        if let Some(s) = v {
            let u: Vec<u16> = s.encode_utf16().collect();
            if coded {
                self.buf.extend(encode_units(&u));
            } else {
                self.buf.extend(u);
            }
        }
        self.buf.extend([13u16, 10]);
    }
    pub fn int(&mut self, n: i32, coded: bool) {
        self.val(Some(&n.to_string()), coded);
    }
    pub fn into_bytes(self) -> Vec<u8> {
        self.buf.iter().flat_map(|u| u.to_le_bytes()).collect()
    }
}
