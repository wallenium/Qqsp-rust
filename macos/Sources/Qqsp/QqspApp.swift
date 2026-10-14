import SwiftUI

@main
struct QqspApp: App {
    @State private var model = GameModel()
    @AppStorage("showHotkeys") private var showHotkeys = false
    @AppStorage("plainHTML") private var plainHTML = false
    @AppStorage("autoscroll") private var autoScroll = true

    var body: some Scene {
        WindowGroup {
            ContentView(model: model)
                .onOpenURL { model.openGame(url: $0) }
                .onAppear { DispatchQueue.main.asyncAfter(deadline: .now() + 0.4) { model.offerResume() } }
        }
        .commands {
            CommandGroup(replacing: .newItem) {
                Button("Spiel öffnen…") { model.openGameDialog() }.keyboardShortcut("o", modifiers: .option)
                Button("Spiel neu starten") { model.restart() }.keyboardShortcut("n", modifiers: .option)
            }
            CommandGroup(after: .newItem) {
                Divider()
                Button("Spielstand laden…") { model.openSaved(path: nil) }.keyboardShortcut("o")
                Button("Spielstand speichern…") { model.saveGame(path: nil) }.keyboardShortcut("s").disabled(model.noSave)
                Button("Schnell laden") { model.quickLoad() }.keyboardShortcut(KeyEquivalent("\u{F70C}"), modifiers: [])
                Button("Schnell speichern") { model.quickSaveNow() }.keyboardShortcut(KeyEquivalent("\u{F708}"), modifiers: [])
            }
            CommandMenu("Anzeige") {
                Toggle("Aktionen", isOn: $model.show[0]).keyboardShortcut("1")
                Toggle("Objekte", isOn: $model.show[1]).keyboardShortcut("2")
                Toggle("Status", isOn: $model.show[2]).keyboardShortcut("3")
                Toggle("Eingabe", isOn: $model.show[3]).keyboardShortcut("4")
                Divider()
                Toggle("Tasten für Aktionen", isOn: $showHotkeys).keyboardShortcut("8")
                Toggle("HTML als Text anzeigen", isOn: $plainHTML).keyboardShortcut("d", modifiers: .option)
                Toggle("Automatisch scrollen", isOn: $autoScroll)
            }
        }
        Settings { OptionsView(model: model) }
    }
}

struct OptionsView: View {
    var model: GameModel
    @AppStorage("volume") private var volume = 100.0
    @AppStorage("muted") private var muted = false
    @AppStorage("useGameFont") private var useGameFont = true
    @AppStorage("fontSize") private var fontSize = 16

    var body: some View {
        Form {
            Section("Ton") {
                Slider(value: $volume, in: 0...100) { Text("Lautstärke") }.onChange(of: volume) { model.audio.applyMute() }
                Toggle("Stumm", isOn: $muted).onChange(of: muted) { model.audio.applyMute() }
            }
            Section("Text") {
                Toggle("Schrift des Spiels verwenden", isOn: $useGameFont)
                Stepper("Schriftgröße: \(fontSize)", value: $fontSize, in: 8...48)
            }
            Section("Über") { Text("Qqsp (Rust-Engine) · QSP \(model.core.version)").foregroundStyle(.secondary) }
        }
        .formStyle(.grouped).frame(width: 420).padding()
    }
}
