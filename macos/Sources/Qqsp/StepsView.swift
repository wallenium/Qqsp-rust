import SwiftUI

/// Zeitleiste: alle Züge der Sitzung; direkter Sprung zu jedem Zustand, Lesezeichen als benannte Spielstände.
struct StepsView: View {
    var active = Active.shared
    var body: some View {
        if let m = active.model, m.gameLoaded { StepsContent(model: m).id(ObjectIdentifier(m)) }
        else { ContentUnavailableView("Kein Spiel geöffnet", systemImage: "clock.arrow.circlepath") }
    }
}

private struct StepsContent: View {
    var model: GameModel
    @State private var marking: Step?
    @State private var markName = ""
    @State private var showMark = false

    private var currentID: Int { model.timeline.first { $0.kind == .current }?.id ?? 0 }

    var body: some View {
        ScrollViewReader { proxy in
            List(model.timeline) { row in
                rowView(row).id(row.id)
                    .listRowBackground(row.kind == .current ? Color.accentColor.opacity(0.18) : Color.clear)
            }
            .listStyle(.plain)
            .onAppear { proxy.scrollTo(currentID, anchor: .center) }
            .onChange(of: model.timeline.count) { proxy.scrollTo(currentID, anchor: .center) }
        }
        .frame(minWidth: 520, minHeight: 360)
        .safeAreaInset(edge: .bottom) {
            Text("Lesezeichen erscheinen im Spielstände-Fenster. Die Zeitleiste gilt nur für diese Sitzung (bis 100 Züge).")
                .font(.caption).foregroundStyle(.secondary).padding(8)
        }
        .alert("Lesezeichen", isPresented: $showMark) {
            TextField("Name", text: $markName)
            Button("Speichern") { if let s = marking { model.bookmark(step: s, name: markName) }; markName = ""; marking = nil }
            Button("Abbrechen", role: .cancel) { markName = ""; marking = nil }
        }
    }

    @ViewBuilder private func rowView(_ row: TimelineRow) -> some View {
        HStack(alignment: .top, spacing: 10) {
            Image(systemName: row.kind == .current ? "location.fill" : (row.kind == .past ? "arrow.uturn.backward.circle" : "arrow.uturn.forward.circle"))
                .foregroundStyle(row.kind == .current ? Color.accentColor : .secondary).frame(width: 20)
            VStack(alignment: .leading, spacing: 2) {
                if row.kind == .current {
                    Text("Jetzt").bold()
                    Text(row.step.loc).font(.caption).foregroundStyle(.secondary)
                } else {
                    Text("\(row.step.loc) · \(row.step.date.formatted(date: .omitted, time: .standard))").font(.callout)
                    if !row.step.label.isEmpty { Text("→ " + row.step.label).font(.caption).bold().foregroundStyle(.tint) }
                    if !row.step.preview.isEmpty { Text(row.step.preview).font(.caption2).foregroundStyle(.secondary).lineLimit(2) }
                }
            }
            Spacer()
            if row.kind != .current {
                Button("Springen") {
                    if row.kind == .past { model.jump(toStep: row.id) } else { model.jump(toFuture: row.id - currentID - 1) }
                }.buttonStyle(.glass)
                Button("Lesezeichen", systemImage: "bookmark") { marking = row.step; showMark = true }
                    .labelStyle(.iconOnly).buttonStyle(.glass)
            }
        }
        .padding(.vertical, 2)
    }
}
