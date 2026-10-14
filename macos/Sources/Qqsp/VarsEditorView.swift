import SwiftUI
import UniformTypeIdentifiers

/// Variablen-Editor („Savegame-Editor“): alle Variablen des aktiven Spiels ansehen und ändern.
/// Änderungen laufen als QSP-Code durch die Engine (Zeitreise macht sie rückgängig).
struct VarsEditorView: View {
    var active = Active.shared
    var body: some View {
        if let m = active.model, m.gameLoaded { VarsEditorContent(model: m).id(ObjectIdentifier(m)) }
        else { ContentUnavailableView("Kein Spiel geöffnet", systemImage: "slider.horizontal.3") }
    }
}

/// „5“ bzw. „5 / Text“ aus dem gespeicherten Vergleichswert.
private func shown(_ raw: String) -> String {
    if raw.isEmpty { return NSLocalizedString("neu", comment: "") }
    let p = raw.components(separatedBy: "\u{1}")
    let s = p.count > 1 ? p[1] : ""
    return s.isEmpty ? p[0] : "\(p[0]) / \(s)"
}

private struct VarsEditorContent: View {
    var model: GameModel
    @State private var filter = ""
    @State private var onlyPinned = false
    @State private var onlyChanged = false
    @State private var showAdd = false
    @State private var newName = ""
    @State private var newIndex = ""
    @State private var newValue = ""
    @State private var presetName = ""
    @State private var showPreset = false
    @State private var editingSave: String?

    private var rows: [DebugVar] {
        var r = model.debugVars
        if onlyPinned { r = r.filter { model.pins.contains($0.key) } }
        if onlyChanged { r = r.filter { model.changedVars.contains($0.key) } }
        if !filter.isEmpty {
            r = r.filter { $0.name.localizedCaseInsensitiveContains(filter) || $0.index.localizedCaseInsensitiveContains(filter) || ($0.str ?? "").localizedCaseInsensitiveContains(filter) }
        }
        return r.sorted { a, b in   // angepinnte zuerst
            let pa = model.pins.contains(a.key), pb = model.pins.contains(b.key)
            return pa != pb ? pa : false
        }
    }

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 10) {
                TextField("Filtern…", text: $filter).textFieldStyle(.roundedBorder)
                Toggle("Angepinnt", systemImage: "pin", isOn: $onlyPinned).toggleStyle(.button)
                Toggle("Geändert", systemImage: "arrow.triangle.2.circlepath", isOn: $onlyChanged).toggleStyle(.button)
                presetsMenu
                fileMenu
                Button("Neue Variable", systemImage: "plus") { showAdd = true }.buttonStyle(.glass)
                Button("Aktualisieren", systemImage: "arrow.clockwise") { model.reloadDebugVars() }.buttonStyle(.glass)
            }
            .padding(12)
            if let f = editingSave {
                HStack {
                    Image(systemName: "doc.badge.gearshape")
                    Text(String(format: NSLocalizedString("Bearbeite Spielstand-Datei: %@", comment: ""), (f as NSString).lastPathComponent))
                    Spacer()
                    Button("In Datei zurückschreiben") { writeBack(f) }.buttonStyle(.glassProminent)
                    Button("Beenden") { editingSave = nil }.buttonStyle(.glass)
                }
                .font(.callout).padding(.horizontal, 12).padding(.bottom, 8)
            }
            Divider()
            HStack {
                Spacer().frame(width: 24)
                Text("Name").frame(width: 190, alignment: .leading)
                Text("Zahl").frame(width: 110, alignment: .leading)
                Text("Text").frame(maxWidth: .infinity, alignment: .leading)
                Spacer().frame(width: 28)
            }
            .font(.caption.weight(.semibold)).foregroundStyle(.secondary)
            .padding(.horizontal, 16).padding(.vertical, 6)
            List(rows) { v in
                VarRow(model: model, v: v)
                    .listRowBackground(model.changedVars.contains(v.key) ? Color.yellow.opacity(0.18) : Color.clear)
            }
            .listStyle(.plain)
            Divider()
            Text("\(rows.count) Werte · gelb = seit dem letzten Zug geändert · Änderungen lassen sich mit „Zug zurück“ widerrufen.")
                .font(.caption).foregroundStyle(.secondary).padding(8)
        }
        .frame(minWidth: 760, minHeight: 440)
        .onAppear { model.reloadDebugVars() }
        .onChange(of: model.revision) { model.reloadDebugVars() }
        .sheet(isPresented: $showAdd) { addSheet }
        .alert("Preset speichern", isPresented: $showPreset) {
            TextField("Name", text: $presetName)
            Button("Speichern") { model.savePreset(name: presetName); presetName = "" }
            Button("Abbrechen", role: .cancel) { presetName = "" }
        } message: { Text("Speichert die aktuellen Werte aller angepinnten Variablen.") }
    }

    private var presetsMenu: some View {
        Menu("Presets", systemImage: "wand.and.stars") {
            ForEach(model.presets) { p in
                Button(p.name) { model.applyPreset(p) }
            }
            if !model.presets.isEmpty {
                Divider()
                Menu("Löschen") { ForEach(model.presets) { p in Button(p.name, role: .destructive) { model.deletePreset(p) } } }
            }
            Button("Aus angepinnten Variablen speichern…") { showPreset = true }.disabled(model.pins.isEmpty)
        }
        .menuStyle(.button).fixedSize()
    }

    private var fileMenu: some View {
        Menu("Datei", systemImage: "doc") {
            Button("Spielstand-Datei öffnen und bearbeiten…") { openSaveFile() }
        }
        .menuStyle(.button).fixedSize()
    }

    /// Lädt eine .sav in das aktive Spiel; mit „In Datei zurückschreiben“ landen die Änderungen wieder in genau dieser Datei.
    private func openSaveFile() {
        let p = NSOpenPanel()
        p.allowedContentTypes = [UTType(filenameExtension: "sav")].compactMap { $0 }
        p.allowsOtherFileTypes = true
        guard p.runModal() == .OK, let u = p.url else { return }
        if model.loadSavedFile(u.path) { editingSave = u.path; model.reloadDebugVars() }
    }
    private func writeBack(_ path: String) {
        let a = NSAlert()
        a.messageText = NSLocalizedString("Spielstand-Datei überschreiben?", comment: "")
        a.informativeText = (path as NSString).lastPathComponent
        a.addButton(withTitle: NSLocalizedString("Überschreiben", comment: ""))
        a.addButton(withTitle: NSLocalizedString("Abbrechen", comment: ""))
        if a.runModal() == .alertFirstButtonReturn { model.saveGame(path: path) }
    }

    private var addSheet: some View {
        Form {
            TextField("Name (z. B. gold oder $name)", text: $newName)
            TextField("Index (leer = nächster Platz; Text in Anführungszeichen)", text: $newIndex)
            TextField("Wert", text: $newValue)
            HStack {
                Spacer()
                Button("Abbrechen", role: .cancel) { showAdd = false }
                Button("Anlegen") { add(); showAdd = false }.buttonStyle(.glassProminent)
                    .disabled(newName.trimmingCharacters(in: .whitespaces).isEmpty)
            }
        }
        .formStyle(.grouped).padding().frame(width: 460)
    }

    /// `$name` setzt Text, sonst Zahl; ein Wert ohne gültige Zahl wird zu 0.
    private func add() {
        let name = newName.trimmingCharacters(in: .whitespaces)
        let idx = newIndex.trimmingCharacters(in: .whitespaces)
        let target = idx.isEmpty ? "\(name)[]" : "\(name)[\(idx)]"
        let value = name.hasPrefix("$") ? qspQuote(newValue) : String(Int(newValue) ?? 0)
        model.exec("\(target) = \(value)")
        model.reloadDebugVars()
        newName = ""; newIndex = ""; newValue = ""
    }
}

private struct VarRow: View {
    var model: GameModel
    let v: DebugVar
    @State private var num: String
    @State private var str: String

    init(model: GameModel, v: DebugVar) {
        self.model = model; self.v = v
        _num = State(initialValue: String(v.num))
        _str = State(initialValue: v.str ?? "")
    }

    /// `NAME[index]` bzw. `$NAME[index]` als QSP-Ausdruck.
    private func target(text: Bool) -> String { "\(text ? "$" : "")\(v.name)[\(v.isKey ? qspQuote(v.index) : v.index)]" }

    private func commitNum() {
        guard let n = Int(num.trimmingCharacters(in: .whitespaces)) else { num = String(v.num); return }
        guard n != v.num else { return }
        model.exec("\(target(text: false)) = \(n)")
        model.reloadDebugVars()
    }
    private func commitStr() {
        guard str != (v.str ?? "") else { return }
        model.exec("\(target(text: true)) = \(qspQuote(str))")
        model.reloadDebugVars()
    }
    private func remove() {
        if v.isKey {   // killvar kennt nur numerische Indizes (wie in der C-Engine): Textschlüssel werden geleert
            model.execBatch(["\(target(text: false)) = 0", "\(target(text: true)) = ''"])
        } else {
            model.exec("killvar \(qspQuote(v.name)), \(v.index)")
        }
        model.reloadDebugVars()
    }

    var body: some View {
        HStack {
            Button { model.togglePin(v.key) } label: {
                Image(systemName: model.pins.contains(v.key) ? "pin.fill" : "pin").foregroundStyle(model.pins.contains(v.key) ? Color.accentColor : .secondary)
            }
            .buttonStyle(.borderless).frame(width: 24).help("Anpinnen (Watchlist)")
            VStack(alignment: .leading, spacing: 1) {
                Text("\(v.name)[\(v.index)]").font(.system(.body, design: .monospaced)).bold().lineLimit(1).truncationMode(.middle)
                if let old = model.previousValues[v.key] {
                    Text(String(format: NSLocalizedString("vorher: %@", comment: ""), shown(old))).font(.caption2).foregroundStyle(.orange).lineLimit(1)
                }
            }
            .frame(width: 190, alignment: .leading)
            TextField("", text: $num).frame(width: 110).textFieldStyle(.roundedBorder).onSubmit(commitNum)
            TextField("", text: $str).frame(maxWidth: .infinity).textFieldStyle(.roundedBorder).onSubmit(commitStr)
            Button("Löschen", systemImage: "trash", role: .destructive, action: remove).labelStyle(.iconOnly).buttonStyle(.borderless)
                .frame(width: 28)
        }
    }
}

/// Kleines Beobachtungsfenster: nur die angepinnten Variablen, live, mit Hervorhebung geänderter Werte.
struct WatchView: View {
    var active = Active.shared
    var body: some View {
        if let m = active.model, m.gameLoaded { WatchContent(model: m).id(ObjectIdentifier(m)) }
        else { ContentUnavailableView("Kein Spiel geöffnet", systemImage: "eye") }
    }
}

private struct WatchContent: View {
    var model: GameModel
    private var rows: [DebugVar] { model.debugVars.filter { model.pins.contains($0.key) } }

    var body: some View {
        Group {
            if model.pins.isEmpty {
                ContentUnavailableView("Nichts angepinnt", systemImage: "pin",
                                       description: Text("Im Variablen-Editor Variablen mit der Pin-Schaltfläche zur Beobachtung hinzufügen."))
            } else {
                List(rows) { v in
                    HStack {
                        Text("\(v.name)[\(v.index)]").font(.system(.callout, design: .monospaced))
                        Spacer()
                        VStack(alignment: .trailing, spacing: 0) {
                            Text(shown("\(v.num)\u{1}\(v.str ?? "")")).font(.system(.callout, design: .monospaced)).bold()
                            if let old = model.previousValues[v.key] {
                                Text(shown(old)).font(.caption2).foregroundStyle(.orange).strikethrough()
                            }
                        }
                    }
                    .listRowBackground(model.changedVars.contains(v.key) ? Color.yellow.opacity(0.18) : Color.clear)
                }
                .listStyle(.plain)
            }
        }
        .frame(minWidth: 260, minHeight: 160)
        .onAppear { model.reloadDebugVars() }
        .onChange(of: model.revision) { model.reloadDebugVars() }
    }
}
