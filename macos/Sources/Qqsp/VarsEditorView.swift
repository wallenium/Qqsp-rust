import SwiftUI

/// Variablen-Editor („Savegame-Editor“): alle Variablen des aktiven Spiels ansehen und ändern.
/// Änderungen laufen als QSP-Code durch die Engine (Zeitreise macht sie rückgängig).
struct VarsEditorView: View {
    var active = Active.shared
    var body: some View {
        if let m = active.model, m.gameLoaded { VarsEditorContent(model: m).id(ObjectIdentifier(m)) }
        else { ContentUnavailableView("Kein Spiel geöffnet", systemImage: "slider.horizontal.3") }
    }
}

private func q(_ s: String) -> String { "'" + s.replacingOccurrences(of: "'", with: "''") + "'" }

private struct VarsEditorContent: View {
    var model: GameModel
    @State private var filter = ""
    @State private var showAdd = false
    @State private var newName = ""
    @State private var newIndex = ""
    @State private var newValue = ""

    private var rows: [DebugVar] {
        guard !filter.isEmpty else { return model.debugVars }
        return model.debugVars.filter {
            $0.name.localizedCaseInsensitiveContains(filter) || $0.index.localizedCaseInsensitiveContains(filter) || ($0.str ?? "").localizedCaseInsensitiveContains(filter)
        }
    }

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 10) {
                TextField("Filtern…", text: $filter).textFieldStyle(.roundedBorder)
                Button("Neue Variable", systemImage: "plus") { showAdd = true }.buttonStyle(.glass)
                Button("Aktualisieren", systemImage: "arrow.clockwise") { model.reloadDebugVars() }.buttonStyle(.glass)
            }
            .padding(12)
            Divider()
            HStack {
                Text("Name").frame(width: 190, alignment: .leading)
                Text("Zahl").frame(width: 110, alignment: .leading)
                Text("Text").frame(maxWidth: .infinity, alignment: .leading)
                Spacer().frame(width: 28)
            }
            .font(.caption.weight(.semibold)).foregroundStyle(.secondary)
            .padding(.horizontal, 16).padding(.vertical, 6)
            List(rows) { v in
                VarRow(model: model, v: v)
            }
            .listStyle(.plain)
            Divider()
            Text("\(rows.count) Werte · Änderungen wirken sofort und lassen sich mit „Zug zurück“ widerrufen.")
                .font(.caption).foregroundStyle(.secondary).padding(8)
        }
        .frame(minWidth: 640, minHeight: 420)
        .onAppear { model.reloadDebugVars() }
        .onChange(of: model.revision) { model.reloadDebugVars() }
        .sheet(isPresented: $showAdd) { addSheet }
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
        let value = name.hasPrefix("$") ? q(newValue) : String(Int(newValue) ?? 0)
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
    private func target(text: Bool) -> String { "\(text ? "$" : "")\(v.name)[\(v.isKey ? q(v.index) : v.index)]" }

    private func commitNum() {
        guard let n = Int(num.trimmingCharacters(in: .whitespaces)) else { num = String(v.num); return }
        guard n != v.num else { return }
        model.exec("\(target(text: false)) = \(n)")
        model.reloadDebugVars()
    }
    private func commitStr() {
        guard str != (v.str ?? "") else { return }
        model.exec("\(target(text: true)) = \(q(str))")
        model.reloadDebugVars()
    }
    private func remove() {
        if v.isKey {   // killvar kennt nur numerische Indizes (wie in der C-Engine): Textschlüssel werden geleert
            model.exec("\(target(text: false)) = 0")
            model.exec("\(target(text: true)) = ''")
        } else {
            model.exec("killvar \(q(v.name)), \(v.index)")
        }
        model.reloadDebugVars()
    }

    var body: some View {
        HStack {
            Text("\(v.name)[\(v.index)]").font(.system(.body, design: .monospaced)).bold()
                .frame(width: 190, alignment: .leading).lineLimit(1).truncationMode(.middle)
            TextField("", text: $num).frame(width: 110).textFieldStyle(.roundedBorder).onSubmit(commitNum)
            TextField("", text: $str).frame(maxWidth: .infinity).textFieldStyle(.roundedBorder).onSubmit(commitStr)
            Button("Löschen", systemImage: "trash", role: .destructive, action: remove).labelStyle(.iconOnly).buttonStyle(.borderless)
                .frame(width: 28)
        }
    }
}
