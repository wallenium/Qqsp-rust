import SwiftUI

/// Minimaler HTML-Renderer für Listeneinträge (Aktionen/Objekte): <b>, <strong>, <i>, <em>, <u>, <font color>, <br>.
/// Alle anderen Tags werden verworfen, Entitäten aufgelöst.
enum RichText {
    static func make(_ s: String, size: CGFloat) -> AttributedString {
        guard s.contains("<") || s.contains("&") else {
            var plain = AttributedString(s)
            plain.font = .system(size: size)
            return plain
        }
        var out = AttributedString()
        var bold = 0, italic = 0, underline = 0
        var colors: [Color] = []
        var text = ""

        func flush() {
            guard !text.isEmpty else { return }
            var run = AttributedString(decode(text))
            var f = Font.system(size: size)
            if bold > 0 { f = f.bold() }
            if italic > 0 { f = f.italic() }
            run.font = f
            if underline > 0 { run.underlineStyle = .single }
            if let c = colors.last { run.foregroundColor = c }
            out += run
            text = ""
        }

        var i = s.startIndex
        while i < s.endIndex {
            if s[i] == "<", let end = s[i...].firstIndex(of: ">") {
                let tag = String(s[s.index(after: i)..<end])
                flush()
                let closing = tag.hasPrefix("/")
                let name = tag.drop(while: { $0 == "/" }).prefix(while: { !$0.isWhitespace && $0 != "/" }).lowercased()
                switch name {
                case "b", "strong": bold += closing ? -1 : 1; bold = max(bold, 0)
                case "i", "em": italic += closing ? -1 : 1; italic = max(italic, 0)
                case "u": underline += closing ? -1 : 1; underline = max(underline, 0)
                case "br": out += AttributedString(" ")
                case "font":
                    if closing { _ = colors.popLast() }
                    else if let c = color(in: tag) { colors.append(c) } else { colors.append(colors.last ?? .primary) }
                default: break
                }
                i = s.index(after: end)
            } else {
                text.append(s[i]); i = s.index(after: i)
            }
        }
        flush()
        return out
    }

    /// Text ohne Tags (für Beschriftungen).
    static func plainText(_ s: String) -> String { String(make(s, size: 13).characters) }

    private static func color(in tag: String) -> Color? {
        guard let r = tag.range(of: #"color\s*=\s*["']?([^"' >]+)"#, options: [.regularExpression, .caseInsensitive]) else { return nil }
        let v = String(tag[r]).split(separator: "=", maxSplits: 1).last!.trimmingCharacters(in: CharacterSet(charactersIn: "\"' ")).lowercased()
        let names: [String: String] = ["red": "#FF0000", "green": "#008000", "blue": "#0000FF", "black": "#000000", "white": "#FFFFFF",
                                       "gray": "#808080", "grey": "#808080", "orange": "#FFA500", "yellow": "#FFD700", "purple": "#800080", "brown": "#8B4513"]
        if let h = names[v] { return Color(hex: h) }
        return v.hasPrefix("#") && v.count == 7 ? Color(hex: v) : nil
    }

    private static func decode(_ s: String) -> String {
        guard s.contains("&") else { return s }
        var r = s
        for (a, b) in [("&nbsp;", " "), ("&lt;", "<"), ("&gt;", ">"), ("&quot;", "\""), ("&apos;", "'"), ("&amp;", "&")] { r = r.replacingOccurrences(of: a, with: b) }
        return r
    }
}
