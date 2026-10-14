import AppKit
import CQsp

/// Dünner Swift-Wrapper um die C-API. Alle Callbacks laufen synchron auf dem Hauptthread.
final class Core {
    private(set) var e: OpaquePointer!
    unowned let model: GameModel

    init(model: GameModel) {
        self.model = model
        var h = QspHost()
        h.user = Unmanaged.passUnretained(model).toOpaque()
        h.debug = { u, _, s in GameModel.from(u).debug(cstr(s)) }
        h.is_playing_file = { u, _, s in GameModel.from(u).audio.isPlaying(cstr(s)) ? 1 : 0 }
        h.play_file = { u, _, s, v in GameModel.from(u).audio.play(cstr(s), volume: Int(v)) }
        h.close_file = { u, _, s in GameModel.from(u).audio.close(s == nil ? nil : cstr(s)) }
        h.show_image = { u, _, s in GameModel.from(u).showImage(s == nil ? nil : cstr(s)) }
        h.show_window = { u, _, w, on in GameModel.from(u).setWindow(Int(w), on != 0) }
        h.delete_menu = { u, _ in GameModel.from(u).menuItems.removeAll() }
        h.add_menu_item = { u, _, n, i in GameModel.from(u).menuItems.append((cstr(n), i == nil ? nil : cstr(i))) }
        h.show_menu = { u, _ in Int32(GameModel.from(u).runMenu()) }
        h.show_message = { u, _, s in GameModel.from(u).message(cstr(s)) }
        h.refresh_int = { u, _, r in GameModel.from(u).refresh(redraw: r != 0) }
        h.set_timer = { u, _, ms in GameModel.from(u).setTimer(Int(ms)) }
        h.set_input_str_text = { u, _, s in GameModel.from(u).input = s == nil ? "" : cstr(s) }
        h.system = { u, _, s in GameModel.from(u).system(cstr(s)) }
        h.open_game_status = { u, _, s in GameModel.from(u).openSaved(path: s == nil ? nil : cstr(s)) }
        h.save_game_status = { u, _, s in GameModel.from(u).saveGame(path: s == nil ? nil : cstr(s)) }
        h.sleep = { _, _, ms in Thread.sleep(forTimeInterval: Double(ms) / 1000) }
        h.ms_count = { u, _ in Int32(GameModel.from(u).msSinceLast()) }
        h.input_box = { u, _, s in GameModel.from(u).inputBox(cstr(s)).map { strdup($0) } ?? nil }
        e = qsp_new(&h)
    }
    deinit { qsp_free(e) }

    // MARK: Aufrufe
    func load(path: String) -> Bool { qsp_load_game(e, path) != 0 }
    func restart() -> Bool { qsp_restart(e, 1) != 0 }
    func exec(_ code: String) -> Bool { qsp_exec_string(e, code, 1) != 0 }
    func counter() { _ = qsp_exec_counter(e, 1) }
    func userInput() { _ = qsp_exec_user_input(e, 1) }
    func selectAction(_ i: Int) { _ = qsp_set_sel_action(e, Int32(i), 1) }
    func runAction() { _ = qsp_exec_sel_action(e, 1) }
    func selectObject(_ i: Int) { _ = qsp_set_sel_object(e, Int32(i), 1) }
    func save(path: String) -> Bool { qsp_save_game(e, path, 0) != 0 }
    func open(saved path: String) -> Bool { qsp_open_saved_game(e, path, 1) != 0 }
    func setInput(_ s: String) { qsp_set_input_text(e, s) }
    var errorText: String { take(qsp_error_description(e)) }
    var version: String { take(qsp_version()) }
    var fullRefreshCount: Int { Int(qsp_full_refresh_count(e)) }
    func visible(_ w: Int) -> Bool { qsp_window_visible(e, Int32(w)) != 0 }

    func mainDesc() -> (String, Bool) { var c: Int32 = 0; let s = take(qsp_main_desc(e, &c)); return (s, c != 0) }
    func varsDesc() -> (String, Bool) { var c: Int32 = 0; let s = take(qsp_vars_desc(e, &c)); return (s, c != 0) }
    var actionsChanged: Bool { qsp_actions_changed(e) != 0 }
    var objectsChanged: Bool { qsp_objects_changed(e) != 0 }
    var selAction: Int { Int(qsp_sel_action(e)) }
    var selObject: Int { Int(qsp_sel_object(e)) }
    func actions() -> [Item] { items(Int(qsp_actions_count(e))) { qsp_action_data(e, Int32($0), &$1, &$2) } }
    func objects() -> [Item] { items(Int(qsp_objects_count(e))) { qsp_object_data(e, Int32($0), &$1, &$2) } }
    private func items(_ n: Int, _ f: (Int, inout UnsafeMutablePointer<CChar>?, inout UnsafeMutablePointer<CChar>?) -> Void) -> [Item] {
        (0..<n).map { i in
            var img: UnsafeMutablePointer<CChar>? = nil, d: UnsafeMutablePointer<CChar>? = nil
            f(i, &img, &d)
            let im = img.map { take($0) }
            return Item(id: i, image: im, text: take(d))
        }
    }

    /// Variable `name[0]` lesen (ohne Codeausführung, daher auch im Refresh-Callback erlaubt).
    private func slot(_ name: String) -> (Int, String?)? {
        var n: Int32 = 0; var p: UnsafeMutablePointer<CChar>? = nil
        guard qsp_var_values(e, name, 0, &n, &p) != 0 else { return nil }
        return (Int(n), p.map { take($0) })
    }
    func num(_ name: String) -> Int? { slot(name)?.0 }
    func str(_ name: String) -> String? { slot(name)?.1 }
}

struct Item: Identifiable, Equatable { let id: Int; let image: String?; let text: String }

func cstr(_ p: UnsafePointer<CChar>?) -> String { p.map { String(cString: $0) } ?? "" }
func take(_ p: UnsafeMutablePointer<CChar>?) -> String {
    guard let p else { return "" }
    defer { qsp_string_free(p) }
    return String(cString: p)
}
