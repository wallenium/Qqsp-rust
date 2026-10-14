import SwiftUI

/// Anordnung wie im Original-Qqsp: links Zusatzbeschreibung + Aktionen, Mitte Hauptbeschreibung + Eingabe, rechts Objekte.
struct ContentView: View {
    @Bindable var model: GameModel
    @AppStorage(Key.hotkeys) private var showHotkeys = false
    @FocusState private var inputFocused: Bool

    // Breiten/Höhe der Bereiche – bleiben über Neustarts erhalten
    @AppStorage("layout.leftW") private var leftW = 300.0
    @AppStorage("layout.rightW") private var rightW = 200.0
    @AppStorage("layout.statusH") private var statusH = 380.0

    var body: some View {
        GeometryReader { geo in
            let showLeft = model.show[0] || model.show[2]
            let leftMax = max(200, geo.size.width * 0.5)
            let rightMax = max(140, geo.size.width * 0.4)
            let lw = min(max(leftW, 200), leftMax)
            let rw = min(max(rightW, 140), rightMax)
            GlassEffectContainer(spacing: 10) {
                HStack(spacing: 0) {
                    if showLeft {
                        leftColumn(height: geo.size.height).frame(width: lw)
                        SplitHandle(value: $leftW, axis: .horizontal, range: 200...leftMax, standard: 300)
                    }
                    centerColumn.frame(maxWidth: .infinity)
                    if model.show[1] {
                        SplitHandle(value: $rightW, axis: .horizontal, inverted: true, range: 140...rightMax, standard: 200)
                        Panel(title: "Objekte") { ObjectsCard(model: model) }.frame(width: rw)
                    }
                }
                .padding(4)
            }
        }
        .background { Backdrop(model: model) }
        .frame(minWidth: 820, minHeight: 520)
        .navigationTitle(model.title)
        .toolbar {
            ToolbarItemGroup {
                Button("Spiel öffnen", systemImage: "folder") { model.openGameDialog() }
                Button("Neu starten", systemImage: "arrow.counterclockwise") { model.restart() }.disabled(!model.gameLoaded)
                Button("Zug zurück", systemImage: "arrow.uturn.backward") { model.undo() }.disabled(!model.canUndo)
                Button("Zug wiederholen", systemImage: "arrow.uturn.forward") { model.redo() }.disabled(!model.canRedo)
            }
            ToolbarSpacer(.fixed)
            ToolbarItemGroup {
                Button("Spielstand laden", systemImage: "tray.and.arrow.down") { model.openSaved(path: nil) }.disabled(!model.gameLoaded)
                Button("Spielstand speichern", systemImage: "tray.and.arrow.up") { model.saveGame(path: nil) }.disabled(!model.gameLoaded || model.noSave)
            }
        }
        // Tastenkürzel für Aktionen (1–9), nur wenn der Eingabefokus nicht im Textfeld liegt
        .onKeyPress(characters: .decimalDigits, phases: .down) { p in
            guard showHotkeys, !inputFocused, let n = Int(p.characters), n >= 1, n <= model.actions.count else { return .ignored }
            model.run(action: n - 1)
            return .handled
        }
        .dropDestination(for: URL.self) { urls, _ in
            guard let u = urls.first else { return false }
            model.openGame(url: u); return true
        }
        .overlay { if !model.gameLoaded { Library(model: model) } }
    }
    @ViewBuilder private func leftColumn(height: CGFloat) -> some View {
        let both = model.show[2] && model.show[0]
        let maxH = max(80, height - 160)
        let sh = min(max(statusH, 80), maxH)
        VStack(spacing: 0) {
            if model.show[2] {
                Panel(title: "Zusatzbeschreibung") { StatusCard(model: model) }
                    .frame(height: both ? sh : nil)
            }
            if both { SplitHandle(value: $statusH, axis: .vertical, range: 80...maxH, standard: 380) }
            if model.show[0] {
                Panel(title: "Aktionen") { ActionsCard(model: model, hotkeys: showHotkeys) }
            }
        }
    }

    @ViewBuilder private var centerColumn: some View {
        VStack(spacing: 0) {
            Panel(title: "Hauptbeschreibung") {
                VStack(spacing: 0) {
                    if let img = model.imagePath { ImageCard(path: img) }
                    DescriptionCard(model: model)
                }
            }
            if model.show[3] { InputBar(model: model, focus: $inputFocused) }
        }
    }
}

/// Ziehbarer Trenner; Wert wird im Binding (AppStorage) gespeichert, Doppelklick = Standardwert.
struct SplitHandle: View {
    enum Axis { case horizontal, vertical }
    @Binding var value: Double
    let axis: Axis
    var inverted = false
    let range: ClosedRange<Double>
    let standard: Double
    @State private var start: Double?
    @State private var hover = false

    var body: some View {
        let h = axis == .horizontal
        Capsule()
            .fill(.secondary.opacity(hover ? 0.5 : 0))
            .frame(width: h ? 3 : nil, height: h ? nil : 3)
            .padding(h ? .vertical : .horizontal, 40)
            .frame(width: h ? 4 : nil, height: h ? nil : 4)
            .contentShape(Rectangle().inset(by: -5))
            .onHover { hover = $0 }
            .pointerStyle(h ? .columnResize : .rowResize)
            .gesture(DragGesture(minimumDistance: 1, coordinateSpace: .global)
                .onChanged { g in
                    if start == nil { start = value }
                    let d = h ? g.translation.width : g.translation.height
                    value = min(max((start ?? value) + (inverted ? -d : d), range.lowerBound), range.upperBound)
                }
                .onEnded { _ in start = nil })
            .onTapGesture(count: 2) { value = standard }
    }
}

/// Fenster mit Titelzeile (wie die Dock-Widgets im Original) auf einer Glasfläche.
struct Panel<Content: View>: View {
    let title: LocalizedStringKey
    @ViewBuilder var content: Content
    @AppStorage(Key.captions) private var captions = true
    var body: some View {
        VStack(spacing: 0) {
            if captions {
                Text(title).font(.caption.weight(.medium)).foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity).padding(.vertical, 6)
                Divider().opacity(0.5)
            }
            content.frame(maxWidth: .infinity, maxHeight: .infinity)
        }
        .clipShape(.rect(cornerRadius: 18))
        .glassEffect(.regular, in: .rect(cornerRadius: 18))
        .padding(4)   // Abstand zwischen den Fenstern (zusammen mit dem Trenner-Griff von 4 pt)
    }
}

struct Backdrop: View {
    var model: GameModel
    @AppStorage(Key.useBack) private var useBack = false
    @AppStorage(Key.back) private var backHex = "#FFFFFF"
    var body: some View {
        ZStack {
            (useBack ? Color(hex: backHex) : (model.bcolor ?? Color(nsColor: .windowBackgroundColor)))
            if let p = model.backImage, let img = NSImage(contentsOfFile: p) {
                Image(nsImage: img).resizable().scaledToFill().opacity(0.9)
            }
        }
        .ignoresSafeArea()
    }
}

struct DescriptionCard: View {
    var model: GameModel
    var body: some View {
        HTMLView(html: model.mainDesc, model: model, asHTML: true)
    }
}

struct ImageCard: View {
    let path: String
    var body: some View {
        if let img = NSImage(contentsOfFile: path) {
            Image(nsImage: img).resizable().scaledToFit().frame(maxHeight: 260).padding(8)
        }
    }
}

struct InputBar: View {
    @Bindable var model: GameModel
    var focus: FocusState<Bool>.Binding
    var body: some View {
        HStack {
            TextField("Befehl eingeben…", text: $model.input)
                .textFieldStyle(.plain)
                .focused(focus)
                .onSubmit { model.submitInput(); focus.wrappedValue = true }
            Button("OK", systemImage: "return") { model.submitInput() }
                .buttonStyle(.glassProminent).labelStyle(.iconOnly)
        }
        .padding(.horizontal, 16).padding(.vertical, 8)
        .glassEffect(.regular.interactive(), in: .capsule)
        .padding(.horizontal, 6).padding(.bottom, 6)
    }
}

struct ListCard: View {
    let items: [Item]
    let selected: Int
    var hotkeys = false
    var centered = false
    var model: GameModel
    var onSelect: (Int) -> Void
    var onActivate: ((Int) -> Void)? = nil
    var onMove: ((Int) -> Void)? = nil
    var onReturn: (() -> Void)? = nil
    @FocusState private var focused: Bool

    var body: some View {
        ScrollViewReader { proxy in
            ScrollView {
                VStack(spacing: 2) {
                    ForEach(items) { it in
                        ListRow(item: it, selected: it.id == selected, hotkeys: hotkeys, centered: centered, model: model) {
                            focused = true
                            onSelect(it.id); onActivate?(it.id)
                        }
                        .id(it.id)
                    }
                }
                .padding(6)
            }
            // Pfeiltasten wählen, Return führt aus (nach Klick in die Liste oder per Tab)
            .focusable()
            .focusEffectDisabled()
            .focused($focused)
            .onKeyPress(.upArrow) { onMove?(-1); return onMove == nil ? .ignored : .handled }
            .onKeyPress(.downArrow) { onMove?(1); return onMove == nil ? .ignored : .handled }
            .onKeyPress(.return) { onReturn?(); return onReturn == nil ? .ignored : .handled }
            .onChange(of: selected) { _, new in if new >= 0 { withAnimation { proxy.scrollTo(new) } } }
        }
    }
}

/// Zeile einer Liste mit Hover-Hervorhebung.
struct ListRow: View {
    let item: Item
    let selected: Bool
    let hotkeys: Bool
    let centered: Bool
    var model: GameModel
    let action: () -> Void
    @State private var hover = false

    var body: some View {
        Button(action: action) {
            HStack(spacing: 8) {
                if let p = item.image, let img = NSImage(contentsOfFile: model.resolve(p)) {
                    Image(nsImage: img).resizable().scaledToFit().frame(width: 20, height: 20)
                }
                if hotkeys && item.id < 9 { Text("\(item.id + 1)").font(.caption.monospacedDigit()).foregroundStyle(.secondary) }
                Text(GameModel.plain(item.text))
                    .multilineTextAlignment(centered ? .center : .leading)
                    .frame(maxWidth: .infinity, alignment: centered ? .center : .leading)
            }
            .padding(.horizontal, 10).padding(.vertical, 6)
            .background(fill, in: .rect(cornerRadius: 8))
            .contentShape(.rect)
        }
        .buttonStyle(.plain)
        .onHover { hover = $0 }
        .animation(.easeOut(duration: 0.12), value: hover)
    }

    private var fill: AnyShapeStyle {
        if selected { return AnyShapeStyle(.tint.opacity(hover ? 0.4 : 0.3)) }
        return hover ? AnyShapeStyle(.primary.opacity(0.1)) : AnyShapeStyle(.clear)
    }
}

struct ActionsCard: View {
    var model: GameModel
    var hotkeys: Bool
    var body: some View {
        // Einfachklick wählt, Doppelklick führt aus – wie in Qqsp
        ListCard(items: model.actions, selected: model.selAction, hotkeys: hotkeys, model: model,
                 onSelect: { model.choose(action: $0) }, onActivate: { _ in model.runSelected() },
                 onMove: { d in
                     let n = model.actions.count
                     if n > 0 { model.choose(action: min(max(model.selAction + d, 0), n - 1)) }
                 },
                 onReturn: { model.runSelected() })
    }
}
struct ObjectsCard: View {
    var model: GameModel
    var body: some View {
        ListCard(items: model.objects, selected: model.selObject, centered: true, model: model,
                 onSelect: { model.choose(object: $0) },
                 onMove: { d in
                     let n = model.objects.count
                     if n > 0 { model.choose(object: min(max(model.selObject + d, 0), n - 1)) }
                 })
    }
}
struct StatusCard: View {
    var model: GameModel
    var body: some View {
        HTMLView(html: model.varsDesc, model: model, asHTML: true, scroll: false)
    }
}
