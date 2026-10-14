import SwiftUI

/// Debug-Fenster: Variablen ansehen, Protokoll der DEBUG-Ausgaben, QSP-Code ausführen.
/// Fenster folgt dem aktiven Spielfenster.
struct DebugView: View {
    var active = Active.shared
    var body: some View {
        if let m = active.model { DebugContent(model: m).id(ObjectIdentifier(m)) }
        else { ContentUnavailableView("Kein Spiel geöffnet", systemImage: "tray") }
    }
}

private struct DebugContent: View {
    var model: GameModel
    private enum Tab: String, CaseIterable, Identifiable { case vars = "Variablen", log = "Protokoll"; var id: String { rawValue } }
    @State private var tab: Tab = .vars
    @State private var filter = ""
    @State private var code = ""

    private var filtered: [DebugVar] {
        guard !filter.isEmpty else { return model.debugVars }
        return model.debugVars.filter { $0.name.localizedCaseInsensitiveContains(filter) || ($0.str ?? "").localizedCaseInsensitiveContains(filter) }
    }

    var body: some View {
        VStack(spacing: 10) {
            Picker("", selection: $tab) { ForEach(Tab.allCases) { Text(LocalizedStringKey($0.rawValue)).tag($0) } }
                .pickerStyle(.segmented).labelsHidden()
            switch tab {
            case .vars:
                HStack {
                    TextField("Filtern…", text: $filter).textFieldStyle(.roundedBorder)
                    Button("Aktualisieren", systemImage: "arrow.clockwise") { model.reloadDebugVars() }.buttonStyle(.glass)
                }
                List(filtered) { v in
                    HStack(alignment: .firstTextBaseline) {
                        Text("\(v.name)[\(v.index)]").font(.system(.body, design: .monospaced)).bold()
                        Spacer(minLength: 12)
                        Text(valueText(v)).font(.system(.body, design: .monospaced)).foregroundStyle(.secondary)
                            .textSelection(.enabled).lineLimit(3)
                    }
                }
                Text("\(filtered.count) Werte").font(.caption).foregroundStyle(.secondary).frame(maxWidth: .infinity, alignment: .trailing)
            case .log:
                List(Array(model.debugLog.enumerated()), id: \.offset) { item in
                    Text(item.element).font(.system(.body, design: .monospaced)).textSelection(.enabled)
                }
                Button("Protokoll leeren") { model.debugLog.removeAll() }.buttonStyle(.glass)
            }
            HStack {
                TextField("QSP-Code ausführen, z. B. gold = 100", text: $code)
                    .textFieldStyle(.roundedBorder).font(.system(.body, design: .monospaced))
                    .onSubmit(run)
                Button("Ausführen", action: run).buttonStyle(.glassProminent).disabled(code.isEmpty || !model.gameLoaded)
            }
        }
        .padding(14)
        .frame(minWidth: 520, minHeight: 420)
        .onAppear { model.core.enableDebug(true); model.reloadDebugVars() }
        .onDisappear { model.core.enableDebug(false) }
        .onChange(of: model.revision) { model.reloadDebugVars() }
    }

    private func valueText(_ v: DebugVar) -> String {
        switch (v.num, v.str) {
        case (let n, let s?) where !s.isEmpty && n != 0: return "\(n)  ·  \"\(s)\""
        case (_, let s?) where !s.isEmpty: return "\"\(s)\""
        default: return String(v.num)
        }
    }
    private func run() {
        guard !code.isEmpty, model.gameLoaded else { return }
        model.exec(code)
        model.reloadDebugVars()
    }
}
