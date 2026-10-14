use crate::text::*;

/// QSP-Variant: Zahl oder Text.
#[derive(Clone, Debug, PartialEq)]
pub enum Val {
    Num(i32),
    Str(String),
}
impl Val {
    pub fn empty(is_str: bool) -> Val {
        if is_str { Val::Str(String::new()) } else { Val::Num(0) }
    }
    pub fn is_str(&self) -> bool {
        matches!(self, Val::Str(_))
    }
    pub fn num(&self) -> i32 {
        match self {
            Val::Num(n) => *n,
            Val::Str(_) => 0,
        }
    }
    pub fn s(&self) -> &str {
        match self {
            Val::Str(s) => s,
            Val::Num(_) => "",
        }
    }
    /// Konvertierung wie qspConvertVariantTo; `true` = fehlgeschlagen.
    pub fn conv_to(&mut self, to_str: bool) -> bool {
        match self {
            Val::Str(s) if !to_str => match str_to_num(s) {
                Some(n) => {
                    *self = Val::Num(n);
                    false
                }
                None => true,
            },
            Val::Num(n) if to_str => {
                *self = Val::Str(n.to_string());
                false
            }
            _ => false,
        }
    }
    pub fn can_num(&self) -> bool {
        match self {
            Val::Str(s) => str_to_num(s).is_some(),
            Val::Num(_) => true,
        }
    }
}

/// qspAutoConvertCompare (konvertiert die Operanden wie das Original in place).
pub fn auto_compare(v1: &mut Val, v2: &mut Val) -> i32 {
    if v1.is_str() != v2.is_str() {
        if v2.is_str() {
            if v2.can_num() {
                v2.conv_to(false);
            } else {
                v1.conv_to(true);
            }
        } else if v1.can_num() {
            v1.conv_to(false);
        } else {
            v2.conv_to(true);
        }
    }
    match (&*v1, &*v2) {
        (Val::Str(a), Val::Str(b)) => cmp_str(a, b),
        (Val::Num(a), Val::Num(b)) => (a > b) as i32 - (a < b) as i32,
        _ => 0,
    }
}
