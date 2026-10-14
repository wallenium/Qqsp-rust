import SwiftUI

/// Eintrag der Spielbibliothek (aus Verlauf + gespeicherten Metadaten).
struct LibEntry: Identifiable {
    let path: String
    var id: String { path }
    var title: String { URL(fileURLWithPath: path).deletingPathExtension().lastPathComponent }
    let cover: String?
    let lastPlayed: Date?
    let playtime: TimeInterval
    let resumeSave: String?

    let info: String?
    let author: String?

    init(path: String) {
        let d = UserDefaults.standard
        self.path = path
        lastPlayed = d.object(forKey: "lastPlayed." + path) as? Date
        playtime = d.double(forKey: "playtime." + path)
        let s = d.string(forKey: "resumeSave." + path)
        resumeSave = s.flatMap { FileManager.default.fileExists(atPath: $0) ? $0 : nil }
        let i = GameInfo.read(forFile: path)
        cover = i.cover; info = i.info; author = i.author
    }

    var playtimeText: String {
        let m = Int(playtime / 60)
        return m < 60 ? "\(m) min" : String(format: "%d h %02d min", m / 60, m % 60)
    }
}

/// Metadaten aus dem Spielordner: Titelbild, Beschreibung (info/readme/about/description .txt/.md) und „Autor:“-Zeile.
enum GameInfo {
    static func read(forFile path: String) -> (cover: String?, info: String?, author: String?) {
        let dir = (path as NSString).deletingLastPathComponent
        let files = (try? FileManager.default.contentsOfDirectory(atPath: dir)) ?? []
        func find(_ names: [String], _ exts: [String]) -> String? {
            files.first { f in
                let u = f as NSString
                return names.contains(u.deletingPathExtension.lowercased()) && exts.contains(u.pathExtension.lowercased())
            }.map { dir + "/" + $0 }
        }
        let cover = find(["cover", "title", "logo", "icon", "splash", "intro"], ["png", "jpg", "jpeg", "gif", "webp"])
        var info: String?
        var author: String?
        if let f = find(["info", "readme", "about", "description", "beschreibung", "autor", "author"], ["txt", "md"]),
           let t = try? String(contentsOfFile: f, encoding: .utf8) {
            info = String(t.trimmingCharacters(in: .whitespacesAndNewlines).prefix(600))
            if let r = t.range(of: #"(?im)^\s*(autor|author|von|by)\s*[:：]\s*(.+)$"#, options: .regularExpression) {
                let line = String(t[r])
                author = line.split(separator: ":", maxSplits: 1).last.map { $0.trimmingCharacters(in: .whitespaces) }
            }
        }
        return (cover, info, author)
    }
}

/// Startansicht ohne geladenes Spiel: Kacheln der zuletzt gespielten Spiele.
struct Library: View {
    var model: GameModel
    @State private var entries: [LibEntry] = []

    var body: some View {
        VStack(spacing: 18) {
            Image(systemName: "text.book.closed").font(.system(size: 40)).symbolRenderingMode(.hierarchical)
            Text("QSP-Spiel öffnen").font(.title2.bold())
            Text("Datei hierher ziehen oder auswählen (.qsp / .gam)").foregroundStyle(.secondary)
            Button("Spiel öffnen…") { model.openGameDialog() }.buttonStyle(.glassProminent).controlSize(.large)
            if !entries.isEmpty {
                Text("Zuletzt gespielt").font(.headline).padding(.top, 6)
                ScrollView {
                    LazyVGrid(columns: [GridItem(.adaptive(minimum: 170, maximum: 220), spacing: 12)], spacing: 12) {
                        ForEach(entries) { e in tile(e) }
                    }
                    .padding(.horizontal, 4)
                }
                .frame(maxHeight: 330)
            }
        }
        .padding(32)
        .frame(maxWidth: 760)
        .glassEffect(.regular, in: .rect(cornerRadius: 28))
        .onAppear { entries = model.existingRecent.map(LibEntry.init) }
    }

    private func tile(_ e: LibEntry) -> some View {
        VStack(spacing: 6) {
            ZStack {
                if let c = e.cover, let img = NSImage(contentsOfFile: c) {
                    Image(nsImage: img).resizable().scaledToFill()
                } else {
                    Image(systemName: "book.closed").font(.system(size: 30)).foregroundStyle(.secondary)
                }
            }
            .frame(height: 90).frame(maxWidth: .infinity).clipShape(.rect(cornerRadius: 12))
            Text(e.title).font(.callout.weight(.medium)).lineLimit(1)
            Text(sub(e)).font(.caption2).foregroundStyle(.secondary).lineLimit(1)
            HStack(spacing: 6) {
                if let s = e.resumeSave {
                    Button("Fortsetzen") { model.loadGame(url: URL(fileURLWithPath: e.path)); model.openSaved(path: s) }
                        .buttonStyle(.glassProminent)
                }
                Button("Neu") { model.loadGame(url: URL(fileURLWithPath: e.path)) }.buttonStyle(.glass)
            }
            .controlSize(.small)
        }
        .padding(10)
        .glassEffect(.regular, in: .rect(cornerRadius: 18))
        .help(e.info ?? e.title)
    }

    private func sub(_ e: LibEntry) -> String {
        var parts: [String] = []
        if let a = e.author { parts.append(a) }
        if let d = e.lastPlayed { parts.append(d.formatted(date: .abbreviated, time: .omitted)) }
        if e.playtime >= 60 { parts.append(e.playtimeText) }
        return parts.joined(separator: " · ")
    }
}
