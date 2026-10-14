import SwiftUI
import UniformTypeIdentifiers
import WebKit

/// Zeigt Beschreibung/Status als HTML. Verlinkungen `exec:` führen QSP-Code aus,
/// Bilder werden über das Schema `qspfile://` aus dem Spielverzeichnis geladen (ohne Groß-/Kleinschreibung).
/// Alle Darstellungs-Einstellungen sind hier direkt per @AppStorage eingebunden, damit die Ansicht sofort reagiert.
struct HTMLView: NSViewRepresentable {
    var html: String
    var model: GameModel
    var asHTML: Bool
    var scroll: Bool = true

    @Environment(\.colorScheme) private var scheme
    @AppStorage(Key.plainHTML) private var plainHTML = false
    @AppStorage(Key.autoscroll) private var autoScroll = true
    @AppStorage(Key.useGameFont) private var useGameFont = true
    @AppStorage(Key.fontSize) private var userFontSize = 16
    @AppStorage(Key.useFontName) private var useFontName = false
    @AppStorage(Key.fontName) private var userFontName = "Helvetica Neue"
    @AppStorage(Key.zoom) private var zoom = 1.0
    @AppStorage(Key.darkGames) private var darkGames = false
    @AppStorage(Key.useFore) private var useFore = false
    @AppStorage(Key.fore) private var foreHex = "#000000"
    @AppStorage(Key.useLink) private var useLink = false
    @AppStorage(Key.link) private var linkHex = "#0A60FF"

    func makeCoordinator() -> Coordinator { Coordinator(model) }
    func makeNSView(context: Context) -> WKWebView {
        let cfg = WKWebViewConfiguration()
        cfg.setURLSchemeHandler(context.coordinator, forURLScheme: "qspfile")
        let v = WKWebView(frame: .zero, configuration: cfg)
        v.navigationDelegate = context.coordinator
        v.setValue(false, forKey: "drawsBackground")
        return v
    }
    func updateNSView(_ v: WKWebView, context: Context) {
        let page = makePage()
        if context.coordinator.last != page {
            context.coordinator.last = page
            v.loadHTMLString(page, baseURL: URL(string: "qspfile://game/"))
        }
    }

    private func hex(_ c: Color?) -> String? { c.map { $0.hexString } }

    private func css() -> String {
        let m = model
        let base = m.look.fontSize ?? userFontSize
        let size = useGameFont && m.look.fontSize == nil ? (m.fontSize ?? userFontSize) : base
        let zoom = m.look.zoom ?? self.zoom
        var s = "html{color-scheme:light dark;zoom:\(zoom)} body{font-size:\(size)px;font-family:-apple-system,sans-serif;"
        if useFontName { s += "font-family:'\(userFontName)',-apple-system,sans-serif;" }
        else if useGameFont, let f = m.fontName { s += "font-family:'\(f)',-apple-system,sans-serif;" }
        if useFore { s += "color:\(foreHex);" } else if let c = hex(m.fcolor) { s += "color:\(c);" }
        s += "margin:12px;word-wrap:break-word} a{"
        if useLink { s += "color:\(linkHex);" } else if let c = hex(m.lcolor) { s += "color:\(c);" }
        s += "} img{max-width:100%}"
        // Dunkelmodus für helle Spielseiten: Seite invertieren, Bilder/Videos wieder zurück
        if (m.look.darkGames ?? darkGames) && scheme == .dark {
            s += "html{filter:invert(1) hue-rotate(180deg)} img,video,canvas{filter:invert(1) hue-rotate(180deg)}"
        }
        return s
    }
    private static func esc(_ s: String) -> String {
        s.replacingOccurrences(of: "&", with: "&amp;").replacingOccurrences(of: "<", with: "&lt;").replacingOccurrences(of: ">", with: "&gt;")
    }
    private func makePage() -> String {
        let body = (model.useHTML && !plainHTML && asHTML) ? html : "<div style='white-space:pre-wrap'>\(Self.esc(html))</div>"
        let auto = scroll && autoScroll && !model.disableScroll ? "<script>window.scrollTo(0,document.body.scrollHeight)</script>" : ""
        return "<!doctype html><meta charset=utf-8><style>\(css())</style><body>\(body)\(auto)"
    }

    final class Coordinator: NSObject, WKNavigationDelegate, WKURLSchemeHandler {
        let model: GameModel
        var last = ""
        init(_ m: GameModel) { model = m }

        func webView(_ w: WKWebView, decidePolicyFor a: WKNavigationAction, decisionHandler: @escaping (WKNavigationActionPolicy) -> Void) {
            guard let url = a.request.url, a.navigationType == .linkActivated else { return decisionHandler(.allow) }
            decisionHandler(.cancel)
            let s = url.absoluteString
            if s.lowercased().hasPrefix("exec:") {
                let code = String(s.dropFirst(5)).removingPercentEncoding ?? ""
                model.exec(code)
            } else if url.scheme == "http" || url.scheme == "https" || url.scheme == "mailto" {
                NSWorkspace.shared.open(url)
            }
        }
        func webView(_ w: WKWebView, start task: WKURLSchemeTask) {
            guard let url = task.request.url else { return task.didFailWithError(URLError(.badURL)) }
            let rel = url.path.removingPercentEncoding?.trimmingCharacters(in: CharacterSet(charactersIn: "/")) ?? ""
            let path = model.resolve(rel)
            guard let d = try? Data(contentsOf: URL(fileURLWithPath: path)) else { return task.didFailWithError(URLError(.fileDoesNotExist)) }
            let mime = UTType(filenameExtension: (path as NSString).pathExtension)?.preferredMIMEType ?? "application/octet-stream"
            task.didReceive(URLResponse(url: url, mimeType: mime, expectedContentLength: d.count, textEncodingName: nil))
            task.didReceive(d)
            task.didFinish()
        }
        func webView(_ w: WKWebView, stop task: WKURLSchemeTask) {}
    }
}
