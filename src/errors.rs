//! Fehlercodes (identisch zu libqsp) und Beschreibungen.
pub const DIVBYZERO: i32 = 100;
pub const TYPEMISMATCH: i32 = 101;
pub const STACKOVERFLOW: i32 = 102;
pub const TOOMANYITEMS: i32 = 103;
pub const FILENOTFOUND: i32 = 104;
pub const CANTLOADFILE: i32 = 105;
pub const GAMENOTLOADED: i32 = 106;
pub const COLONNOTFOUND: i32 = 107;
pub const CANTINCFILE: i32 = 108;
pub const CANTADDACTION: i32 = 109;
pub const EQNOTFOUND: i32 = 110;
pub const LOCNOTFOUND: i32 = 111;
pub const ENDNOTFOUND: i32 = 112;
pub const LABELNOTFOUND: i32 = 113;
pub const NOTCORRECTNAME: i32 = 114;
pub const QUOTNOTFOUND: i32 = 115;
pub const BRACKNOTFOUND: i32 = 116;
pub const BRACKSNOTFOUND: i32 = 117;
pub const SYNTAX: i32 = 118;
pub const UNKNOWNACTION: i32 = 119;
pub const ARGSCOUNT: i32 = 120;
pub const CANTADDOBJECT: i32 = 121;
pub const CANTADDMENUITEM: i32 = 122;
pub const TOOMANYVARS: i32 = 123;
pub const INCORRECTREGEXP: i32 = 124;
pub const CODENOTFOUND: i32 = 125;
pub const TONOTFOUND: i32 = 126;

pub fn describe(n: i32) -> &'static str {
    match n {
        DIVBYZERO => "Division by zero!",
        TYPEMISMATCH => "Type mismatch!",
        STACKOVERFLOW => "Stack overflow!",
        TOOMANYITEMS => "Too many items in expression!",
        FILENOTFOUND => "File not found!",
        CANTLOADFILE => "Can't load file!",
        GAMENOTLOADED => "Game not loaded!",
        COLONNOTFOUND => "Sign [:] not found!",
        CANTINCFILE => "Can't add file!",
        CANTADDACTION => "Can't add action!",
        EQNOTFOUND => "Sign [=] not found!",
        LOCNOTFOUND => "Location not found!",
        ENDNOTFOUND => "[end] not found!",
        LABELNOTFOUND => "Label not found!",
        NOTCORRECTNAME => "Incorrect variable's name!",
        QUOTNOTFOUND => "Quote not found!",
        BRACKNOTFOUND => "Bracket not found!",
        BRACKSNOTFOUND => "Brackets not found!",
        SYNTAX => "Syntax error!",
        UNKNOWNACTION => "Unknown action!",
        ARGSCOUNT => "Incorrect arguments' count!",
        CANTADDOBJECT => "Can't add object!",
        CANTADDMENUITEM => "Can't add menu's item!",
        TOOMANYVARS => "Too many variables!",
        INCORRECTREGEXP => "Regular expression's error!",
        CODENOTFOUND => "Code not found!",
        TONOTFOUND => "[to] not found!",
        _ => "Unknown error!",
    }
}

/// Kontrollfluss-Abbruch. `Error`: Fehler ist in `Engine::err` vermerkt.
/// `Refresh`: ein GOTO/XGOTO hat die Location gewechselt, alle Aufrufer brechen ab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Abort {
    Error,
    Refresh,
}
pub type R<T> = Result<T, Abort>;
