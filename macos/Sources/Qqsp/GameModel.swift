import SwiftUI
import CQsp
import AppKit
import AVFoundation
import Observation
import UniformTypeIdentifiers

/// Spielzustand + Host-Callbacks. Wird ausschließlich auf dem Hauptthread benutzt.
@Observable
final class GameModel {
    static func from(_ p: UnsafeMutableRawPointer?) -> GameModel { Unmanaged<GameModel>.fromOpaque(p!).takeUnretainedValue() }

    // Anzeige
    var mainDesc = ""
    var varsDesc = ""
    var actions: [Item] = []
    var objects: [Item] = []
    var selAction = -1
    var selObject = -1
    var input = ""
    var imagePath: String?
    var show = [true, true, true, true]       // Aktionen, Objekte, Status, Eingabe
    var useHTML = false
    var noSave = false
    var disableScroll = false
    var backImage: String?
    var bcolor: Color?
    var fcolor: Color?
    var lcolor: Color?
    var fontName: String?
    var fontSize: Int?
    var title = "Qqsp"
    var revision = 0                          // erhöht sich bei jedem Hauptfenster-Update
    var gameLoaded = false

    // Einstellungen (persistiert)
    @ObservationIgnored @AppStorage("showHotkeys") var showHotkeys = false
    @ObservationIgnored @AppStorage("plainHTML") var plainHTML = false
    @ObservationIgnored @AppStorage("volume") var volume = 100.0
    @ObservationIgnored @AppStorage("muted") var muted = false
    @ObservationIgnored @AppStorage("useGameFont") var useGameFont = true
    @ObservationIgnored @AppStorage("fontSize") var userFontSize = 16
    @ObservationIgnored @AppStorage("autoscroll") var autoScroll = true

    var menuItems: [(String, String?)] = []
    @ObservationIgnored lazy var core = Core(model: self)
    @ObservationIgnored let audio = Audio()
    @ObservationIgnored var gamePath = ""         // Verzeichnis der .qsp-Datei
    @ObservationIgnored private var timer: Timer?
    @ObservationIgnored private var lastMs = Date()
    @ObservationIgnored private let defaults = UserDefaults.standard
    @ObservationIgnored private var gameKey: String { String(abs(core.fullPathHash)) }
    @ObservationIgnored private var supportDir: String {
        let d = (NSSearchPathForDirectoriesInDomains(.applicationSupportDirectory, .userDomainMask, true).first ?? NSTemporaryDirectory()) + "/Qqsp"
        try? FileManager.default.createDirectory(atPath: d, withIntermediateDirectories: true)
        return d
    }
    @ObservationIgnored private var quickSave: String { supportDir + "/quick-\(gameKey).sav" }
    @ObservationIgnored private var autoSave: String { supportDir + "/auto-\(gameKey).sav" }
    /// Zuletzt gespeicherter oder geladener Spielstand des aktuellen Spiels (neuester gewinnt).
    @ObservationIgnored private var lastSave: String? {
        get { defaults.string(forKey: "lastSave.\(gameKey)") }
        set { defaults.set(newValue, forKey: "lastSave.\(gameKey)") }
    }

    init() {
        audio.model = self
        // Beim Beenden bzw. Schließen des Fensters automatisch sichern, damit „Fortsetzen“ genau dort weitermacht.
        for n in [NSApplication.willTerminateNotification, NSWindow.willCloseNotification] {
            NotificationCenter.default.addObserver(forName: n, object: nil, queue: .main) { [weak self] note in
                // Dialoge/Panels (NSPanel) ignorieren – nur das Hauptfenster bzw. App-Ende löst die Sicherung aus
                if let w = note.object as? NSWindow, !w.canBecomeMain { return }
                self?.autoSaveNow()
            }
        }
    }

    func autoSaveNow() {
        guard gameLoaded, !noSave else { return }
        if core.save(path: autoSave) { lastSave = autoSave }
    }

    /// Beim Start: letztes Spiel samt neuestem Spielstand anbieten.
    func offerResume() {
        guard !gameLoaded, let p = defaults.string(forKey: "lastGamePath"), FileManager.default.fileExists(atPath: p) else { return }
        let url = URL(fileURLWithPath: p)
        let a = NSAlert()
        a.messageText = "Mit „\(url.deletingPathExtension().lastPathComponent)“ fortsetzen?"
        a.informativeText = "Das zuletzt gespielte Spiel wird geöffnet und der neueste Spielstand geladen."
        a.addButton(withTitle: "Fortsetzen")
        a.addButton(withTitle: "Neu beginnen")
        a.addButton(withTitle: "Anderes Spiel…")
        switch a.runModal() {
        case .alertFirstButtonReturn:
            openGame(url: url)
            if let s = lastSave, FileManager.default.fileExists(atPath: s) { openSaved(path: s) }
        case .alertSecondButtonReturn: openGame(url: url)
        default: openGameDialog()
        }
    }

    // MARK: Spiel öffnen
    func openGame(url: URL) {
        let path = url.path
        guard core.load(path: path) else { report(); return }
        gamePath = url.deletingLastPathComponent().path + "/"
        audio.base = gamePath
        title = url.deletingPathExtension().lastPathComponent
        gameLoaded = true
        defaults.set(path, forKey: "lastGamePath")
        NSDocumentController.shared.noteNewRecentDocumentURL(url)
        if !core.restart() { report() }
    }
    func openGameDialog() {
        let p = NSOpenPanel()
        p.allowedContentTypes = ["qsp", "gam"].compactMap { UTType(filenameExtension: $0) }
        p.allowsOtherFileTypes = true
        if p.runModal() == .OK, let u = p.url { openGame(url: u) }
    }
    func restart() { if gameLoaded && !core.restart() { report() } }

    // MARK: Spielstände
    func openSaved(path: String?) {
        var file = path
        if file == nil {
            let p = NSOpenPanel()
            p.allowedContentTypes = [UTType(filenameExtension: "sav")].compactMap { $0 }
            p.allowsOtherFileTypes = true
            guard p.runModal() == .OK, let u = p.url else { return }
            file = u.path
        }
        if core.open(saved: file!) { lastSave = file } else { report() }
    }
    func saveGame(path: String?) {
        guard !noSave else { return }
        var file = path
        if file == nil {
            let p = NSSavePanel()
            p.allowedContentTypes = [UTType(filenameExtension: "sav")].compactMap { $0 }
            p.nameFieldStringValue = title + ".sav"
            guard p.runModal() == .OK, let u = p.url else { return }
            file = u.path
        }
        if core.save(path: file!) { lastSave = file } else { report() }
    }
    func quickLoad() { if FileManager.default.fileExists(atPath: quickSave) { openSaved(path: quickSave) } }
    func quickSaveNow() { saveGame(path: quickSave) }

    // MARK: Aktionen / Eingabe
    func choose(action i: Int) { core.selectAction(i) }
    func run(action i: Int) { core.selectAction(i); core.runAction() }
    func choose(object i: Int) { core.selectObject(i) }
    func submitInput() { core.setInput(input); core.userInput() }
    func exec(_ code: String) { _ = core.exec(code) }

    // MARK: Callbacks
    func refresh(redraw: Bool) {
        let (d, dc) = core.mainDesc()
        let (v, vc) = core.varsDesc()
        useHTML = (core.num("USEHTML") ?? 0) != 0
        noSave = (core.num("NOSAVE") ?? 0) != 0
        disableScroll = (core.num("DISABLESCROLL") ?? 0) != 0
        if core.actionsChanged { actions = core.actions() }
        if core.objectsChanged { objects = core.objects() }
        selAction = core.selAction
        selObject = core.selObject
        if dc { mainDesc = d; revision += 1 }
        if vc { varsDesc = v }
        backImage = core.str("BACKIMAGE").flatMap { $0.isEmpty ? nil : resolve($0) }
        bcolor = core.num("BCOLOR").flatMap { $0 == 0 ? nil : Self.color($0) }
        fcolor = core.num("FCOLOR").flatMap { $0 == 0 ? nil : Self.color($0) }
        lcolor = core.num("LCOLOR").flatMap { $0 == 0 ? nil : Self.color($0) }
        fontName = core.str("FNAME").flatMap { $0.isEmpty ? nil : $0 }
        fontSize = core.num("FSIZE").flatMap { $0 == 0 ? nil : $0 }
    }
    /// QSP-Farben sind 0xBBGGRR.
    static func color(_ v: Int) -> Color {
        Color(red: Double(v & 255) / 255, green: Double((v >> 8) & 255) / 255, blue: Double((v >> 16) & 255) / 255)
    }
    func showImage(_ p: String?) { imagePath = p.flatMap { $0.isEmpty ? nil : resolve($0) } }
    func setWindow(_ w: Int, _ on: Bool) { if (0..<4).contains(w) { show[w] = on } }
    func debug(_ s: String) { NSLog("QSP debug: %@", s) }
    func system(_ cmd: String) { NSLog("QSP system ignoriert: %@", cmd) }   // wie Qqsp: kein Shell-Zugriff
    func msSinceLast() -> Int { let n = Date(); defer { lastMs = n }; return Int(n.timeIntervalSince(lastMs) * 1000) }

    func setTimer(_ ms: Int) {
        timer?.invalidate()
        guard ms > 0 else { return }
        timer = Timer.scheduledTimer(withTimeInterval: Double(ms) / 1000, repeats: true) { [weak self] _ in self?.core.counter() }
    }

    func message(_ s: String) {
        let a = NSAlert()
        a.messageText = Self.plain(s)
        a.addButton(withTitle: "OK")
        a.runModal()
    }
    func inputBox(_ prompt: String) -> String? {
        let a = NSAlert()
        a.messageText = Self.plain(prompt)
        let f = NSTextField(frame: NSRect(x: 0, y: 0, width: 280, height: 24))
        a.accessoryView = f
        a.addButton(withTitle: "OK")
        a.addButton(withTitle: NSLocalizedString("Cancel", comment: ""))
        a.window.initialFirstResponder = f
        return a.runModal() == .alertFirstButtonReturn ? f.stringValue : ""
    }
    func runMenu() -> Int {
        let m = NSMenu()
        var chosen = -1
        let target = MenuTarget { chosen = $0 }
        for (i, it) in menuItems.enumerated() {
            let mi = NSMenuItem(title: Self.plain(it.0), action: #selector(MenuTarget.pick(_:)), keyEquivalent: "")
            mi.target = target; mi.tag = i
            if let img = it.1, let im = NSImage(contentsOfFile: resolve(img)) { im.size = NSSize(width: 16, height: 16); mi.image = im }
            m.addItem(mi)
        }
        let loc = NSApp.keyWindow.map { $0.mouseLocationOutsideOfEventStream } ?? .zero
        m.popUp(positioning: nil, at: loc, in: NSApp.keyWindow?.contentView)   // blockiert bis zur Auswahl
        return chosen
    }
    static func plain(_ s: String) -> String {
        guard s.contains("<") else { return s }
        return s.replacingOccurrences(of: "<[^>]+>", with: "", options: .regularExpression)
    }

    // MARK: Pfade (Groß-/Kleinschreibung ignorieren wie Qqsp)
    func resolve(_ rel: String) -> String {
        let r = rel.replacingOccurrences(of: "\\", with: "/")
        if r.hasPrefix("/") { return r }
        var cur = gamePath
        let fm = FileManager.default
        for comp in r.split(separator: "/") where comp != "." {
            let c = String(comp)
            if let names = try? fm.contentsOfDirectory(atPath: cur), let hit = names.first(where: { $0.caseInsensitiveCompare(c) == .orderedSame }) {
                cur += (cur.hasSuffix("/") ? "" : "/") + hit
            } else { cur += (cur.hasSuffix("/") ? "" : "/") + c }
        }
        return cur
    }

    func report() { message(core.errorText) }
}

final class MenuTarget: NSObject {
    let f: (Int) -> Void
    init(_ f: @escaping (Int) -> Void) { self.f = f }
    @objc func pick(_ s: NSMenuItem) { f(s.tag) }
}

extension Core {
    var fullPathHash: Int { var h = 0; for u in take(qsp_qst_full_path(e)).unicodeScalars { h = h &* 31 &+ Int(u.value) }; return h }
}

/// Audio: mehrere Dateien parallel, Lautstärke 0–100, Stumm-Schalter.
final class Audio {
    var base = ""
    weak var model: GameModel?
    private var players: [String: AVAudioPlayer] = [:]
    func key(_ f: String) -> String { f.lowercased() }
    func isPlaying(_ f: String) -> Bool { players[key(f)]?.isPlaying ?? false }
    func play(_ f: String, volume: Int) {
        guard let m = model else { return }
        let url = URL(fileURLWithPath: m.resolve(f))
        guard let p = try? AVAudioPlayer(contentsOf: url) else { return }
        p.volume = m.muted ? 0 : Float(volume) / 100 * Float(m.volume / 100)
        p.play()
        players[key(f)] = p
    }
    func close(_ f: String?) {
        if let f { players[key(f)]?.stop(); players[key(f)] = nil }
        else { players.values.forEach { $0.stop() }; players.removeAll() }
    }
    func applyMute() { players.values.forEach { $0.volume = (model?.muted ?? false) ? 0 : Float((model?.volume ?? 100) / 100) } }
}
