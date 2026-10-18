import SwiftUI

@main
struct QqspApp: App {
    var body: some Scene {
        WindowGroup(id: "game") { GameWindow() }
            .commands { AppCommands() }

        Window("Spielstände", id: "saves") { SavesView() }
        Window("Variablen-Editor", id: "vars") { VarsEditorView() }
        Window("Watchlist", id: "watch") { WatchView() }
        Window("Zeitleiste", id: "steps") { StepsView() }
        Window("Debug", id: "debug") { DebugView() }
        Settings { OptionsView() }
    }
}

struct AppCommands: Commands {
    var active = Active.shared
    @Environment(\.openWindow) private var openWindow
    @AppStorage(Key.hotkeys) private var showHotkeys = false
    @AppStorage(Key.plainHTML) private var plainHTML = false
    @AppStorage(Key.autoscroll) private var autoScroll = true
    @AppStorage(Key.captions) private var captions = true
    @AppStorage(Key.zoom) private var zoom = 1.0

    /// Zoom gilt pro Spiel (Überschreibung), ohne Spiel global.
    private func setZoom(_ f: (Double) -> Double?) {
        let cur = m?.look.zoom ?? zoom
        if let m, m.gameLoaded { m.updateLook { $0.zoom = f(cur) } } else if let v = f(cur) { zoom = v }
    }
    private var m: GameModel? { active.model }
    private var loaded: Bool { m?.gameLoaded ?? false }
    private var canSave: Bool { loaded && !(m?.noSave ?? true) }
    private func show(_ i: Int) -> Binding<Bool> { Binding(get: { m?.show[i] ?? true }, set: { m?.show[i] = $0 }) }

    var body: some Commands {
        CommandGroup(replacing: .appInfo) {
            Button("Über Qqsp") { showAbout() }
        }
        CommandGroup(replacing: .newItem) {
            Button("Neues Fenster") { openWindow(id: "game") }.keyboardShortcut("n")
            Button("Neuer Tab") { Active.shared.newTab() }.keyboardShortcut("t")
            Divider()
            Button("Spiel öffnen…") { m?.openGameDialog() }.keyboardShortcut("o", modifiers: .option).disabled(m == nil)
            Menu("Zuletzt geöffnet") {
                ForEach(m?.existingRecent ?? [], id: \.self) { p in
                    Button(URL(fileURLWithPath: p).deletingPathExtension().lastPathComponent) { m?.openGame(url: URL(fileURLWithPath: p)) }
                }
                if !(m?.recent.isEmpty ?? true) {
                    Divider()
                    Button("Liste löschen") { m?.clearRecent() }
                }
            }
            .disabled(m?.existingRecent.isEmpty ?? true)
            Button("Spiel neu starten") { m?.restart() }.keyboardShortcut("n", modifiers: .option).disabled(!loaded)
        }
        CommandGroup(after: .newItem) {
            Divider()
            Button("Spielstand laden…") { m?.openSaved(path: nil) }.keyboardShortcut("o").disabled(!loaded)
            Button("Spielstand speichern…") { m?.saveGame(path: nil) }.keyboardShortcut("s").disabled(!canSave)
            Button("In neuem Slot speichern") { m?.newSlot() }.keyboardShortcut("s", modifiers: [.command, .shift]).disabled(!canSave)
            Button("Schnell laden") { m?.quickLoad() }.keyboardShortcut(KeyEquivalent("\u{F70C}"), modifiers: []).disabled(!loaded)
            Button("Schnell speichern") { m?.quickSaveNow() }.keyboardShortcut(KeyEquivalent("\u{F708}"), modifiers: []).disabled(!canSave)
            Button("Spielstände…") { openWindow(id: "saves") }.keyboardShortcut("l", modifiers: [.command, .shift])
        }
        CommandMenu("Spiel") {
            Button("Spielinfo…") { m?.showInfo() }.keyboardShortcut("i").disabled(!loaded)
            Button("Watchlist") { openWindow(id: "watch") }.keyboardShortcut("w", modifiers: [.command, .shift]).disabled(!loaded)
            Button("Variablen-Editor…") { openWindow(id: "vars") }.keyboardShortcut("e", modifiers: [.command, .shift]).disabled(!loaded)
            Divider()
            Button("Zeitleiste…") { openWindow(id: "steps") }.keyboardShortcut("t", modifiers: [.command, .option]).disabled(!loaded)
            Button("Zug zurück") { m?.undo() }.keyboardShortcut("[").disabled(!(m?.canUndo ?? false))
            Button("Zug wiederholen") { m?.redo() }.keyboardShortcut("]").disabled(!(m?.canRedo ?? false))
        }
        CommandMenu("Anzeige") {
            Toggle("Aktionen", isOn: show(0)).keyboardShortcut("1")
            Toggle("Objekte", isOn: show(1)).keyboardShortcut("2")
            Toggle("Status", isOn: show(2)).keyboardShortcut("3")
            Toggle("Eingabe", isOn: show(3)).keyboardShortcut("4")
            Divider()
            Toggle("Titelzeilen (Captions)", isOn: $captions).keyboardShortcut("6")
            Toggle("Tasten für Aktionen", isOn: $showHotkeys).keyboardShortcut("8")
            Toggle("HTML als Text anzeigen", isOn: $plainHTML).keyboardShortcut("d", modifiers: .option)
            Toggle("Automatisch scrollen", isOn: $autoScroll)
            Divider()
            Button("Vollbild") { NSApp.keyWindow?.toggleFullScreen(nil) }.keyboardShortcut(KeyEquivalent("\u{F70E}"), modifiers: [])
            Button("Text vergrößern") { setZoom { min($0 + 0.1, 3) } }.keyboardShortcut("+")
            Button("Text verkleinern") { setZoom { max($0 - 0.1, 0.5) } }.keyboardShortcut("-")
            Button("Originalgröße") { setZoom { _ in nil } }.keyboardShortcut("0")
            Divider()
            Button("Debug-Fenster") { openWindow(id: "debug") }.keyboardShortcut("d", modifiers: [.command, .shift])
        }
    }

    private func showAbout() {
        let credits = NSMutableAttributedString(
            string: String(format: NSLocalizedString("Rust-Neuimplementierung der QSP-Engine, kompatibel zu libqsp %@.\nLizenz: LGPL-2.1 oder neuer.\nBasierend auf Qqsp (Sonnix1) und libqsp (Valeriy Argunov).\n", comment: ""), Core.versionString),
            attributes: [.font: NSFont.systemFont(ofSize: 11)])
        credits.append(NSAttributedString(string: "github.com/Sonnix1/Qqsp",
                                          attributes: [.link: URL(string: "https://github.com/Sonnix1/Qqsp")!, .font: NSFont.systemFont(ofSize: 11)]))
        NSApp.orderFrontStandardAboutPanel(options: [
            .applicationName: "Qqsp",
            .applicationVersion: "0.1.0",
            .version: "QSP \(Core.versionString)",
            .credits: credits,
        ])
    }
}

struct OptionsView: View {
    @AppStorage(Key.volume) private var volume = 100.0
    @AppStorage(Key.muted) private var muted = false
    @AppStorage(Key.useGameFont) private var useGameFont = true
    @AppStorage(Key.fontSize) private var fontSize = 16
    @AppStorage(Key.useFontName) private var useFontName = false
    @AppStorage(Key.fontName) private var fontName = "Helvetica Neue"
    @AppStorage(Key.zoom) private var zoom = 1.0
    @AppStorage(Key.darkGames) private var darkGames = false
    @AppStorage(Key.useBack) private var useBack = false
    @AppStorage(Key.back) private var backHex = "#FFFFFF"
    @AppStorage(Key.useFore) private var useFore = false
    @AppStorage(Key.fore) private var foreHex = "#000000"
    @AppStorage(Key.useLink) private var useLink = false
    @AppStorage(Key.link) private var linkHex = "#0A60FF"
    @AppStorage(Key.plainHTML) private var plainHTML = false
    @AppStorage(Key.autoscroll) private var autoScroll = true
    @AppStorage(Key.captions) private var captions = true
    @AppStorage(Key.hotkeys) private var showHotkeys = false
    @AppStorage(Key.autoContrast) private var autoContrast = true
    @AppStorage(Key.listIconSize) private var iconSize = 20
    @AppStorage(Key.statusFont) private var statusFont = 0
    @AppStorage(Key.classicLook) private var classicLook = false
    @AppStorage(Key.scrollbars) private var scrollbars = true
    @AppStorage(Key.actionsFont) private var actionsFont = 13
    @AppStorage(Key.objectsFont) private var objectsFont = 13

    private let families = NSFontManager.shared.availableFontFamilies

    var body: some View {
        TabView {
            tab { general }.tabItem { Label("Allgemein", systemImage: "gearshape") }
            tab { text }.tabItem { Label("Text", systemImage: "textformat") }
            tab { colors }.tabItem { Label("Farben", systemImage: "paintpalette") }
            tab { sound }.tabItem { Label("Ton", systemImage: "speaker.wave.2") }
            tab { thisGame }.tabItem { Label("Dieses Spiel", systemImage: "gamecontroller") }
        }
        .scenePadding()
        .frame(width: 520, height: 460)
    }

    /// Jede Seite: gruppiertes Formular wie in den macOS-Systemeinstellungen.
    private func tab<C: View>(@ViewBuilder _ content: () -> C) -> some View {
        Form { content() }.formStyle(.grouped)
    }

    @ViewBuilder private var general: some View {
        Section("Darstellung") {
            Toggle("Titelzeilen der Fenster anzeigen", isOn: $captions)
            Toggle("Klassische Darstellung (enger, wie Qt-Qqsp)", isOn: $classicLook)
            Toggle("Bildlaufleiste immer sichtbar", isOn: $scrollbars)
        }
        Section("Bedienung") {
            Toggle("Tasten 1–9 für Aktionen", isOn: $showHotkeys)
            Toggle("Automatisch nach unten scrollen", isOn: $autoScroll)
            Toggle("HTML als Text anzeigen", isOn: $plainHTML)
        }
        SaveFolderSection()
        Section("Über") { Text("Qqsp (Rust-Engine) · QSP \(Core.versionString)").foregroundStyle(.secondary) }
    }

    @ViewBuilder private var text: some View {
        Section("Haupttext") {
            Toggle("Schriftart des Spiels verwenden", isOn: $useGameFont)
            Toggle("Eigene Schriftart", isOn: $useFontName)
            if useFontName {
                Picker("Schriftart", selection: $fontName) { ForEach(families, id: \.self) { Text($0).tag($0) } }
            }
            Stepper("Schriftgröße: \(fontSize)", value: $fontSize, in: 8...48)
            Slider(value: $zoom, in: 0.5...3, step: 0.1) { Text("Textzoom: \(Int(zoom * 100)) %") }
            Stepper(statusFont == 0 ? NSLocalizedString("Schrift Zusatzbeschreibung: wie Haupttext", comment: "") : String(format: NSLocalizedString("Schrift Zusatzbeschreibung: %lld", comment: ""), statusFont),
                    value: $statusFont, in: 0...48)
        }
        Section("Listen") {
            Stepper("Schrift Aktionen: \(actionsFont)", value: $actionsFont, in: 9...32)
            Stepper("Schrift Objekte: \(objectsFont)", value: $objectsFont, in: 9...32)
            Stepper("Symbolgröße in Listen: \(iconSize)", value: $iconSize, in: 12...64, step: 4)
        }
    }

    @ViewBuilder private var colors: some View {
        Section("Eigene Farben (überschreiben die Vorgaben des Spiels)") {
            colorRow("Hintergrund", on: $useBack, hex: $backHex)
            colorRow("Text", on: $useFore, hex: $foreHex)
            colorRow("Links", on: $useLink, hex: $linkHex)
        }
        Section("Lesbarkeit") {
            Toggle("Schwer lesbare Textfarben automatisch anpassen", isOn: $autoContrast)
            Toggle("Helle Spiele im Dunkelmodus abdunkeln", isOn: $darkGames)
        }
    }

    @ViewBuilder private var sound: some View {
        Section("Ton") {
            Slider(value: $volume, in: 0...100) { Text("Lautstärke") }.onChange(of: volume) { Active.shared.model?.audio.applyMute() }
            Toggle("Stumm", isOn: $muted).onChange(of: muted) { Active.shared.model?.audio.applyMute() }
        }
    }

    @ViewBuilder private var thisGame: some View {
        if let g = Active.shared.model, g.gameLoaded {
            GameLookSection(model: g)
        } else {
            ContentUnavailableView("Kein Spiel geöffnet", systemImage: "gamecontroller",
                                   description: Text("Öffne ein Spiel, um Einstellungen nur für dieses Spiel festzulegen."))
        }
    }

    private func colorRow(_ title: String, on: Binding<Bool>, hex: Binding<String>) -> some View {
        HStack {
            Toggle(title, isOn: on)
            Spacer()
            ColorPicker("", selection: colorBinding(hex), supportsOpacity: false).labelsHidden().disabled(!on.wrappedValue)
        }
    }
}

/// Einstellungen nur für das aktive Spiel (überschreiben die globalen).
struct GameLookSection: View {
    @Bindable var model: GameModel
    @AppStorage(Key.zoom) private var zoom = 1.0
    @AppStorage(Key.fontSize) private var fontSize = 16
    @AppStorage(Key.darkGames) private var darkGames = false
    @AppStorage(Key.actionsFont) private var actionsFont = 13
    @AppStorage(Key.objectsFont) private var objectsFont = 13

    var body: some View {
        Section(String(format: NSLocalizedString("Nur für „%@“", comment: ""), model.title)) {
            Slider(value: Binding(get: { model.look.zoom ?? zoom }, set: { v in model.updateLook { $0.zoom = v } }), in: 0.5...3, step: 0.1) {
                Text("Textzoom: \(Int((model.look.zoom ?? zoom) * 100)) %")
            }
            Stepper("Schriftgröße: \(model.look.fontSize ?? fontSize)",
                    value: Binding(get: { model.look.fontSize ?? fontSize }, set: { v in model.updateLook { $0.fontSize = v } }), in: 8...48)
            Stepper("Schrift Aktionen: \(model.look.actionsFont ?? actionsFont)",
                    value: Binding(get: { model.look.actionsFont ?? actionsFont }, set: { v in model.updateLook { $0.actionsFont = v } }), in: 9...32)
            Stepper(model.look.statusFont.map { String(format: NSLocalizedString("Schrift Zusatzbeschreibung: %lld", comment: ""), $0) } ?? NSLocalizedString("Schrift Zusatzbeschreibung: wie Haupttext", comment: ""),
                    value: Binding(get: { model.look.statusFont ?? 0 }, set: { v in model.updateLook { $0.statusFont = v == 0 ? nil : v } }), in: 0...48)
            Stepper("Schrift Objekte: \(model.look.objectsFont ?? objectsFont)",
                    value: Binding(get: { model.look.objectsFont ?? objectsFont }, set: { v in model.updateLook { $0.objectsFont = v } }), in: 9...32)
            Toggle("Helle Spiele im Dunkelmodus abdunkeln",
                   isOn: Binding(get: { model.look.darkGames ?? darkGames }, set: { v in model.updateLook { $0.darkGames = v } }))
            Button("Auf globale Einstellungen zurücksetzen") { model.updateLook { $0 = Look() } }
                .disabled(model.look == Look())
            Toggle("QSP-5.9-Funktionen (SORTARR, SCANSTR, SETVAR, ARRITEM …)",
                   isOn: Binding(get: { model.ext59 }, set: { model.setExt59($0) }))
            Text("Zusätzliche Befehle aus QSP 5.9. Das Spiel wird dabei neu gestartet; alles andere verhält sich wie 5.7.")
                .font(.caption).foregroundStyle(.secondary)
        }
    }
}

/// Ordner für Schnell-/Auto-/Slot-Spielstände, z. B. in iCloud Drive, damit sie zwischen Macs geteilt werden.
struct SaveFolderSection: View {
    @AppStorage(Key.saveFolder) private var folder = ""
    private var icloud: String { NSHomeDirectory() + "/Library/Mobile Documents/com~apple~CloudDocs" }

    var body: some View {
        Section("Spielstände synchronisieren") {
            Text(folder.isEmpty ? NSLocalizedString("Standardordner (nur dieser Mac)", comment: "") : folder)
                .font(.caption).foregroundStyle(.secondary).lineLimit(2).truncationMode(.middle)
            HStack {
                Button("Ordner wählen…") {
                    let p = NSOpenPanel()
                    p.canChooseDirectories = true; p.canChooseFiles = false; p.canCreateDirectories = true
                    if p.runModal() == .OK, let u = p.url { folder = u.path }
                }
                Button("iCloud Drive") { folder = icloud }.disabled(!FileManager.default.fileExists(atPath: icloud))
                Button("Zurücksetzen") { folder = "" }.disabled(folder.isEmpty)
            }
        }
    }
}
