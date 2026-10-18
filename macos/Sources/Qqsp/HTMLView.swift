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
    @AppStorage(Key.autoContrast) private var autoContrast = true
    @AppStorage(Key.statusFont) private var statusFont = 0
    @AppStorage(Key.classicLook) private var classicLook = false
    @AppStorage(Key.scrollbars) private var scrollbars = true
    @AppStorage(Key.useBack) private var useBack = false
    @AppStorage(Key.back) private var backHex = "#FFFFFF"
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
        var size = useGameFont && m.look.fontSize == nil ? (m.fontSize ?? userFontSize) : base
        if !scroll, let sf = m.look.statusFont ?? (statusFont > 0 ? statusFont : nil) { size = sf }   // Zusatzbeschreibung
        let zoom = m.look.zoom ?? self.zoom
        var s = "html{color-scheme:light dark;zoom:\(zoom)} body{font-size:\(size)px;font-family:-apple-system,sans-serif;"
        if useFontName { s += "font-family:'\(userFontName)',-apple-system,sans-serif;" }
        else if useGameFont, let f = m.fontName { s += "font-family:'\(f)',-apple-system,sans-serif;" }
        if useFore { s += "color:\(foreHex);" } else if let c = hex(m.fcolor) { s += "color:\(c);" }
        s += (classicLook ? "margin:4px;line-height:1.15;font-family:'Helvetica Neue',Helvetica,sans-serif;" : "margin:12px;") + "word-wrap:break-word} a{"
        if useLink { s += "color:\(linkHex);" } else if let c = hex(m.lcolor) { s += "color:\(c);" }
        s += "} img{max-width:100%}"
        // Bildlaufleiste dauerhaft sichtbar (macOS blendet sie sonst aus)
        if scrollbars { s += "::-webkit-scrollbar{width:12px;height:12px} ::-webkit-scrollbar-thumb{background:rgba(128,128,128,.55);border-radius:6px;border:2px solid transparent;background-clip:content-box} ::-webkit-scrollbar-track{background:transparent}" }
        // Dunkelmodus für helle Spielseiten: Seite invertieren, Bilder/Videos wieder zurück
        if (m.look.darkGames ?? darkGames) && scheme == .dark {
            s += "html{filter:invert(1) hue-rotate(180deg)} img,video,canvas{filter:invert(1) hue-rotate(180deg)}"
        }
        return s
    }
    private static func esc(_ s: String) -> String {
        s.replacingOccurrences(of: "&", with: "&amp;").replacingOccurrences(of: "<", with: "&lt;").replacingOccurrences(of: ">", with: "&gt;")
    }
    /// Wie Qqsp im HTML-Modus: Zeilenumbrüche des Spieltexts sind `<br>` (abschließender Umbruch entfällt),
    /// direkt nach `</center>` und `</table>` aber nicht.
    static func htmlBody(_ text: String) -> String {
        // „\r\n“ ist in Swift ein einzelnes Zeichen: erst \r entfernen, dann den abschließenden Umbruch (wie Qqsp)
        var t = text.replacingOccurrences(of: "\r", with: "")
        if t.hasSuffix("\n") { t.removeLast() }
        t = t.replacingOccurrences(of: "\n", with: "<br>")
        t = t.replacingOccurrences(of: "</center><br>", with: "</center>", options: .caseInsensitive)
        return t.replacingOccurrences(of: "</table><br>", with: "</table>", options: .caseInsensitive)
    }
    private func makePage() -> String {
        let body = (model.useHTML && !plainHTML && asHTML) ? Self.htmlBody(html) : "<div style='white-space:pre-wrap'>\(Self.esc(html))</div>"
        let auto = scroll && autoScroll && !model.disableScroll ? "<script>window.scrollTo(0,document.body.scrollHeight)</script>" : ""
        return "<!doctype html><meta charset=utf-8><style>\(css())</style><body>\(body)\(Self.tooltipScript)\(contrastScript())\(auto)"
    }

    /// `exec:`-Links bekommen den Code als Tooltip (Debuggen / „was macht dieser Link?“).
    static let tooltipScript = """
    <script>document.querySelectorAll('a[href^="exec:" i]').forEach(function(a){if(!a.title){var c=a.getAttribute('href').slice(5);try{c=decodeURIComponent(c)}catch(e){}a.title=c.length>200?c.slice(0,200)+'…':c}});</script>
    """
    /// Hintergrundfarbe unter dem Text (Override, Spielfarbe oder Fensterfarbe) als [r,g,b].
    private func backgroundRGB() -> [Int] {
        let h = useBack ? backHex : (hex(model.bcolor) ?? (scheme == .dark ? "#1E1E1E" : "#ECECEC"))
        let v = Int(h.trimmingCharacters(in: CharacterSet(charactersIn: "#")), radix: 16) ?? 0xECECEC
        return [(v >> 16) & 255, (v >> 8) & 255, v & 255]
    }
    /// Hebt Textfarben des Spiels an/ab, deren Kontrast zum Hintergrund unter 4,5:1 liegt (WCAG AA).
    private func contrastScript() -> String {
        guard autoContrast, !(model.look.darkGames ?? darkGames && scheme == .dark) else { return "" }
        let bg = backgroundRGB().map(String.init).joined(separator: ",")
        return """
        <script>(function(){var def=[\(bg)];
        function L(c){c=c.map(function(v){v/=255;return v<=.03928?v/12.92:Math.pow((v+.055)/1.055,2.4)});return .2126*c[0]+.7152*c[1]+.0722*c[2]}
        function R(a,b){var x=L(a),y=L(b);return (Math.max(x,y)+.05)/(Math.min(x,y)+.05)}
        function rgb(s){var m=s.match(/\\d+(\\.\\d+)?/g);return m?m.map(Number):null}
        function bgOf(el){for(;el&&el.nodeType==1;el=el.parentElement){var m=rgb(getComputedStyle(el).backgroundColor);if(m&&(m.length<4||m[3]>.5))return m.slice(0,3)}return def}
        var MID=[120,120,120];
        document.querySelectorAll('a').forEach(function(el){
          if(el.querySelector('img'))return;var c=rgb(getComputedStyle(el).color);if(!c)return;c=c.slice(0,3);
          // Hintergrund unter dem Glas ist unbekannt: Link muss auf Fensterfarbe UND auf Mittelgrau lesbar sein
          if(Math.min(R(c,def),R(c,MID))>=4.5)return;
          el.style.setProperty('color','#fff','important');
          el.style.setProperty('text-shadow','0 0 1px #000,0 0 2px #000,0 0 3px rgba(0,0,0,.7)','important');
          el.dataset.fixed='1'});
        document.querySelectorAll('body, body *').forEach(function(el){
          var c=rgb(getComputedStyle(el).color);if(!c)return;c=c.slice(0,3);
          if(el.dataset.fixed)return;var bg=bgOf(el);if(R(c,bg)>=4.5)return;
          var t=R([255,255,255],bg)>=R([0,0,0],bg)?255:0;
          for(var k=1;k<=20;k++){var n=c.map(function(v){return Math.round(v+(t-v)*k/20)});
            if(R(n,bg)>=4.5||k==20){el.style.setProperty('color','rgb('+n.join(',')+')','important');return}}
        });})();</script>
        """
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
