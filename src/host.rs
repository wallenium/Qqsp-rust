//! Rückruf-Schnittstelle zur Oberfläche (entspricht QSP_CALL_* in libqsp).
use crate::engine::Engine;

/// Fenster-Typen (QSP_WIN_*)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Win {
    Acts = 0,
    Objs = 1,
    Vars = 2,
    Input = 3,
}

/// Die Oberfläche implementiert dieses Trait. Alle Methoden bekommen `&mut Engine`,
/// um (wie in libqsp) während eines Callbacks API-Funktionen aufrufen zu können
/// (z. B. Laden eines Spielstands). Methoden mit Rückgabewert haben Standardwerte.
#[allow(unused_variables)]
pub trait Host {
    fn debug(&self, e: &mut Engine, text: &str) {}
    fn is_playing_file(&self, e: &mut Engine, file: &str) -> bool { false }
    fn play_file(&self, e: &mut Engine, file: &str, volume: i32) {}
    /// `None` = alle Dateien schließen
    fn close_file(&self, e: &mut Engine, file: Option<&str>) {}
    /// `None` = Bild ausblenden
    fn show_image(&self, e: &mut Engine, file: Option<&str>) {}
    fn show_window(&self, e: &mut Engine, win: Win, show: bool) {}
    fn delete_menu(&self, e: &mut Engine) {}
    fn add_menu_item(&self, e: &mut Engine, name: &str, image: Option<&str>) {}
    /// Gibt den gewählten Index (0-basiert) oder -1 zurück.
    fn show_menu(&self, e: &mut Engine) -> i32 { -1 }
    fn show_message(&self, e: &mut Engine, text: &str) {}
    fn refresh_int(&self, e: &mut Engine, redraw: bool) {}
    fn set_timer(&self, e: &mut Engine, msecs: i32) {}
    fn set_input_str_text(&self, e: &mut Engine, text: Option<&str>) {}
    fn system(&self, e: &mut Engine, cmd: &str) {}
    /// Spielstand laden (Datei oder `None` = Auswahldialog).
    fn open_game_status(&self, e: &mut Engine, file: Option<&str>) {}
    fn save_game_status(&self, e: &mut Engine, file: Option<&str>) {}
    fn sleep(&self, e: &mut Engine, msecs: i32) {}
    /// Millisekunden seit dem letzten Aufruf.
    fn ms_count(&self, e: &mut Engine) -> i32 { 0 }
    fn input_box(&self, e: &mut Engine, prompt: &str) -> String { String::new() }
}

/// Host ohne Reaktionen (für Tests / Hintergrundläufe).
pub struct NullHost;
impl Host for NullHost {}
