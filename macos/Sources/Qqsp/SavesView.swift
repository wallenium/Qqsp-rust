import SwiftUI

/// Spielstand-Browser: Slots mit Datum und Ort, dazu Schnell- und Autospeicher.
/// Fenster folgt dem aktiven Spielfenster.
struct SavesView: View {
    var active = Active.shared
    var body: some View {
        if let m = active.model { SavesContent(model: m).id(ObjectIdentifier(m)) }
        else { ContentUnavailableView("Kein Spiel geöffnet", systemImage: "tray") }
    }
}

private struct SavesContent: View {
    var model: GameModel

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Text(model.gameLoaded ? model.title : "Spielstände").font(.headline)
                Spacer()
                Button("Neuer Slot", systemImage: "plus") { model.newSlot() }
                    .buttonStyle(.glassProminent)
                    .disabled(!model.gameLoaded || model.noSave)
            }
            .padding(14)
            Divider()
            if model.slots.isEmpty {
                ContentUnavailableView("Keine Spielstände", systemImage: "tray",
                                       description: Text("Mit „Neuer Slot“ oder F5 speichern, oder das Spiel beenden – dann entsteht ein Autospeicher."))
            } else {
                List(model.slots) { slot in
                    HStack(spacing: 12) {
                        VStack(alignment: .leading, spacing: 2) {
                            Text(slot.title)
                            Text("\(slot.kind) · \(slot.date.formatted(date: .abbreviated, time: .standard))")
                                .font(.caption).foregroundStyle(.secondary)
                            if !slot.preview.isEmpty {
                                Text(slot.preview).font(.caption2).foregroundStyle(.tertiary).lineLimit(2)
                            }
                        }
                        Spacer()
                        Button("Laden") { model.loadSlot(slot) }.buttonStyle(.glass)
                        Button("Löschen", systemImage: "trash", role: .destructive) { model.deleteSlot(slot) }
                            .labelStyle(.iconOnly).buttonStyle(.glass)
                    }
                    .padding(.vertical, 2)
                }
            }
        }
        .frame(minWidth: 480, minHeight: 340)
        .onAppear { model.refreshSlots() }
        .onChange(of: model.gameLoaded) { model.refreshSlots() }
    }
}
