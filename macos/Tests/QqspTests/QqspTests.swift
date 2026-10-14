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
}
