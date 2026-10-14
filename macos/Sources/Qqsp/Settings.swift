import SwiftUI

/// Schlüssel der persistierten Einstellungen (UserDefaults / @AppStorage).
enum Key {
    static let plainHTML = "plainHTML"
    static let autoscroll = "autoscroll"
    static let hotkeys = "showHotkeys"
    static let captions = "showCaptions"
    static let useGameFont = "useGameFont"
    static let fontSize = "fontSize"
    static let useFontName = "useFontName"
    static let fontName = "userFontName"
    static let zoom = "textZoom"
    static let darkGames = "darkGames"
    static let useBack = "useBackColor", back = "backColor"
    static let useFore = "useForeColor", fore = "foreColor"
    static let useLink = "useLinkColor", link = "linkColor"
    static let volume = "volume", muted = "muted"
}

extension Color {
    /// "#RRGGBB"
    init(hex: String) {
        var h = hex.trimmingCharacters(in: CharacterSet(charactersIn: "# "))
        if h.count != 6 { h = "808080" }
        let v = UInt32(h, radix: 16) ?? 0x808080
        self.init(red: Double((v >> 16) & 255) / 255, green: Double((v >> 8) & 255) / 255, blue: Double(v & 255) / 255)
    }
    var hexString: String {
        guard let n = NSColor(self).usingColorSpace(.sRGB) else { return "#808080" }
        return String(format: "#%02X%02X%02X", Int((n.redComponent * 255).rounded()), Int((n.greenComponent * 255).rounded()), Int((n.blueComponent * 255).rounded()))
    }
}

/// Bindet einen Hex-String aus den Einstellungen an einen ColorPicker.
func colorBinding(_ hex: Binding<String>) -> Binding<Color> {
    Binding(get: { Color(hex: hex.wrappedValue) }, set: { hex.wrappedValue = $0.hexString })
}
