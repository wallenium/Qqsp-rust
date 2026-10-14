import SwiftUI
import WebKit

/// Zeigt Beschreibung/Status/Listen als HTML. Verlinkungen `exec:` führen QSP-Code aus,
/// Bilder werden über das Schema `qspfile://` aus dem Spielverzeichnis geladen (ohne Groß-/Kleinschreibung).
struct HTMLView: NSViewRepresentable {
    var html: String
    var model: GameModel
    var asHTML: Bool
    var scroll: Bool = true

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
        let page = Self.page(html, model: model, asHTML: asHTML, scroll: scroll)
        if context.coordinator.last != page {
            context.coordinator.last = page
            v.loadHTMLString(page, baseURL: URL(string: "qspfile://game/"))
        }
    }

    static func css(_ m: GameModel) -> String {
        func hex(_ c: Color?) -> String? {
            guard let c, let n = NSColor(c).usingColorSpace(.sRGB) else { return nil }
            return String(format: "#%02X%02X%02X", Int(n.redComponent * 255), Int(n.greenComponent * 255), Int(n.blueComponent * 255))
        }
        var s = "html{color-scheme:light dark} body{font:\(m.useGameFont ? (m.fontSize ?? m.userFontSize) : m.userFontSize)px -apple-system,sans-serif;"
        if m.useGameFont, let f = m.fontName { s += "font-family:'\(f)',-apple-system,sans-serif;" }
        if let c = hex(m.fcolor) { s += "color:\(c);" }
        s += "margin:12px;word-wrap:break-word} a{"
        if let c = hex(m.lcolor) { s += "color:\(c);" }
        s += "} img{max-width:100%}"
        return s
    }
    static func esc(_ s: String) -> String {
        s.replacingOccurrences(of: "&", with: "&amp;").replacingOccurrences(of: "<", with: "&lt;").replacingOccurrences(of: ">", with: "&gt;")
    }
    static func page(_ text: String, model m: GameModel, asHTML: Bool, scroll: Bool) -> String {
        let body = (m.useHTML && !m.plainHTML && asHTML) ? text : "<div style='white-space:pre-wrap'>\(esc(text))</div>"
        let auto = scroll && m.autoScroll && !m.disableScroll ? "<script>window.scrollTo(0,document.body.scrollHeight)</script>" : ""
        return "<!doctype html><meta charset=utf-8><style>\(css(m))</style><body>\(body)\(auto)"
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
import UniformTypeIdentifiers
