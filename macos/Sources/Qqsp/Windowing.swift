import SwiftUI
import AppKit

/// Aktives Spielfenster (zuletzt in den Vordergrund gekommenes). Menüs, Spielstände- und Debug-Fenster beziehen sich darauf.
@Observable
final class Active {
    static let shared = Active()
    var model: GameModel?
    /// Spiel, das ein neu geöffnetes Fenster/Tab beim Erscheinen laden soll.
    @ObservationIgnored var pending: URL?
    @ObservationIgnored var launched = false
    @ObservationIgnored private var lastURL: (URL, Date)?

    /// Datei aus Finder/„Öffnen mit“ oder `qqsp://`-Befehl. Doppelte Events (je Fenster) werden verworfen.
    /// Befehle (bewusst ohne Code-Ausführung): open?path=…, action?n=…, quicksave, quickload, undo, redo.
    func handle(_ url: URL, in model: GameModel) {
        if let l = lastURL, l.0 == url, Date().timeIntervalSince(l.1) < 1 { return }
        lastURL = (url, Date())
        let m = self.model ?? model
        if url.scheme == "qqsp" {
            let q = URLComponents(url: url, resolvingAgainstBaseURL: false)?.queryItems ?? []
            func arg(_ k: String) -> String? { q.first { $0.name == k }?.value }
            switch url.host {
            case "open": if let p = arg("path") { m.openGame(url: URL(fileURLWithPath: p)) }
            case "action": if let n = arg("n").flatMap({ Int($0) }), n >= 1, n <= m.actions.count { m.run(action: n - 1) }
            case "quicksave": m.quickSaveNow()
            case "quickload": m.quickLoad()
            case "undo": m.undo()
            case "redo": m.redo()
            default: break
            }
        } else if url.pathExtension.lowercased() == "sav" {
            if m.gameLoaded { m.openSaved(path: url.path) }
        } else {
            model.openGame(url: url)
        }
    }
    /// Neuer Tab im aktuellen Fenster (optional gleich mit Spiel).
    func newTab(loading url: URL? = nil) {
        pending = url
        if NSApp.keyWindow != nil { NSApp.sendAction(#selector(NSResponder.newWindowForTab(_:)), to: nil, from: nil) }
    }
}

/// Ein Spielfenster mit eigenem Zustand (eigene Engine, Audio, Timer).
struct GameWindow: View {
    @State private var model = GameModel()
    var body: some View {
        ContentView(model: model)
            .background(WindowAccessor(model: model))
            .onOpenURL { Active.shared.handle($0, in: model) }
            .onAppear {
                DispatchQueue.main.async {
                    if let u = Active.shared.pending { Active.shared.pending = nil; model.loadGame(url: u) }
                    else if !Active.shared.launched {
                        Active.shared.launched = true
                        DispatchQueue.main.asyncAfter(deadline: .now() + 0.4) { model.offerResume() }
                    }
                    if Active.shared.model == nil { Active.shared.model = model }
                }
            }
    }
}

/// Verbindet GameModel mit seinem NSWindow (Tabs aktivieren, aktives Fenster melden, Autosave beim Schließen).
struct WindowAccessor: NSViewRepresentable {
    let model: GameModel
    func makeNSView(context: Context) -> NSView {
        let v = NSView()
        DispatchQueue.main.async {
            guard let w = v.window else { return }
            model.window = w
            w.tabbingMode = .preferred
            w.tabbingIdentifier = "qqsp-game"
            if w.isKeyWindow { Active.shared.model = model }
            NotificationCenter.default.addObserver(forName: NSWindow.didBecomeKeyNotification, object: w, queue: .main) { _ in
                Active.shared.model = model
                model.focusGained()
            }
            NotificationCenter.default.addObserver(forName: NSWindow.didResignKeyNotification, object: w, queue: .main) { _ in
                model.focusLost()
            }
        }
        return v
    }
    func updateNSView(_ v: NSView, context: Context) {}
}
