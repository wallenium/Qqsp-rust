import XCTest
@testable import Qqsp

/// Erzeugt eine minimale .qsp-Datei (UCS-2, Zeichen +5 kodiert) wie tests/diff/gen.py.
private func makeGame(_ locs: [(name: String, code: String, acts: [(desc: String, code: String)])]) -> Data {
    var u: [UInt16] = []
    func enc(_ s: String) -> [UInt16] { s.utf16.map { $0 == 5 ? 0xFFFB : $0 &- 5 } }
    func line(_ v: [UInt16]) { u += v; u += [13, 10] }
    line(Array("QSPGAME".utf16)); line(Array("5.7.0".utf16)); line(enc("")); line(enc(String(locs.count)))
    for l in locs {
        line(enc(l.name)); line(enc("")); line(enc(l.code.replacingOccurrences(of: "\n", with: "\r\n"))); line(enc(String(l.acts.count)))
        for a in l.acts { line(enc("")); line(enc(a.desc)); line(enc(a.code.replacingOccurrences(of: "\n", with: "\r\n"))) }
    }
    return u.withUnsafeBufferPointer { Data(buffer: $0) }
}

private func tempGame(name: String = "test") throws -> URL {
    let dir = FileManager.default.temporaryDirectory.appendingPathComponent("qqsp-test-\(UUID().uuidString)")
    try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
    let url = dir.appendingPathComponent(name + ".qsp")
    try makeGame([("start", "*clr\n*pl 'one'", [("next", "*clr\n*pl 'two'")])]).write(to: url)
    return url
}

@MainActor
final class QqspTests: XCTestCase {
    func testLoadAndRunAction() throws {
        let m = GameModel()
        m.loadGame(url: try tempGame())
        XCTAssertTrue(m.gameLoaded)
        XCTAssertTrue(m.mainDesc.contains("one"))
        XCTAssertEqual(m.actions.count, 1)
        m.run(action: 0)
        XCTAssertTrue(m.mainDesc.contains("two"))
    }

    func testUndoRedo() throws {
        let m = GameModel()
        m.loadGame(url: try tempGame())
        XCTAssertFalse(m.canUndo)
        m.run(action: 0)
        XCTAssertTrue(m.canUndo)
        m.undo()
        XCTAssertTrue(m.mainDesc.contains("one"))
        XCTAssertTrue(m.canRedo)
        m.redo()
        XCTAssertTrue(m.mainDesc.contains("two"))
    }

    func testRestartClearsHistory() throws {
        let m = GameModel()
        m.loadGame(url: try tempGame())
        m.run(action: 0)
        m.restart()
        XCTAssertFalse(m.canUndo)
        XCTAssertTrue(m.mainDesc.contains("one"))
    }

    func testLookIsPerGame() throws {
        let a = GameModel(), b = GameModel()
        let ua = try tempGame(name: "a"), ub = try tempGame(name: "b")
        a.loadGame(url: ua); b.loadGame(url: ub)
        a.updateLook { $0.zoom = 1.7 }
        XCTAssertEqual(a.look.zoom, 1.7)
        XCTAssertNil(b.look.zoom)
        let again = GameModel()
        again.loadGame(url: ua)
        XCTAssertEqual(again.look.zoom, 1.7)   // aus UserDefaults wiederhergestellt
        a.updateLook { $0 = Look() }
    }

    func testLibraryEntryFindsCover() throws {
        let url = try tempGame(name: "demo")
        let dir = url.deletingLastPathComponent()
        FileManager.default.createFile(atPath: dir.appendingPathComponent("Cover.PNG").path, contents: Data())
        let e = LibEntry(path: url.path)
        XCTAssertEqual(e.title, "demo")
        XCTAssertNotNil(e.cover)
    }

    func testURLSchemeAction() throws {
        let m = GameModel()
        m.loadGame(url: try tempGame())
        Active.shared.model = m
        Active.shared.handle(URL(string: "qqsp://action?n=1")!, in: m)
        XCTAssertTrue(m.mainDesc.contains("two"))
        Active.shared.handle(URL(string: "qqsp://undo")!, in: m)
        XCTAssertTrue(m.mainDesc.contains("one"))
    }

    func testGameInfoReadsAuthor() throws {
        let url = try tempGame()
        let dir = url.deletingLastPathComponent()
        try "Autor: Max Muster\nEin kleines Abenteuer.".write(to: dir.appendingPathComponent("info.txt"), atomically: true, encoding: .utf8)
        let i = GameInfo.read(forFile: url.path)
        XCTAssertEqual(i.author, "Max Muster")
        XCTAssertTrue(i.info?.contains("Abenteuer") == true)
    }

    func testHTMLBodyLikeQqsp() {
        XCTAssertEqual(HTMLView.htmlBody("a\r\nb\r\n"), "a<br>b")
        XCTAssertEqual(HTMLView.htmlBody("<table></table>\n<center>x</center>\ny"), "<table></table><center>x</center>y")
    }

    func testRichTextStripsTagsAndKeepsBold() {
        let a = RichText.make("Gehe <b>nach</b> &amp; <font color=\"red\">Hause</font><br>los", size: 13)
        XCTAssertEqual(String(a.characters), "Gehe nach & Hause los")
        XCTAssertGreaterThan(a.runs.count, 2)
    }

    func testEditVariableThroughExec() throws {
        let m = GameModel()
        m.loadGame(url: try tempGame())
        m.exec("score = 42")
        m.exec("$title = 'It''s'")
        m.reloadDebugVars()
        XCTAssertEqual(m.debugVars.first { $0.name == "SCORE" }?.num, 42)
        XCTAssertEqual(m.debugVars.first { $0.name == "TITLE" }?.str, "It's")
        m.exec("killvar 'score', 0")
        m.reloadDebugVars()
        XCTAssertNil(m.debugVars.first { $0.name == "SCORE" })
    }
}
