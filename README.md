# Qqsp-RS

A Rust port of the QSP text-quest engine (libqsp 5.7.0) with a native SwiftUI (Liquid Glass) player for macOS.

| Folder | Contents |
|---|---|
| `src/` | Engine (`qsp_core`); `src/ffi.rs` + `include/qsp_core.h` expose a C API |
| `tests/engine.rs` | Engine unit tests |
| `tests/diff/` | Differential tests against the original C engine (incl. random expressions) |
| `macos/` | SwiftUI app (macOS 26 / Xcode 26) |

## Features

- Engine ported 1:1 from libqsp, savegame-compatible (byte-identical saves)
- Multiple games at once: each window/tab has its own game state (⌘N window, ⌘T tab)
- Resume the last game with its newest save on launch; save slots, quick save (F5/F9), autosave on quit
- Debug window (variables, log, run code), keyboard navigation, persisted pane widths
- Undo/redo of moves, game library with covers/play time, save previews, per-game display settings
- UI in German and English (follows the system language)

## Build (macOS)

```sh
./macos/build-core.sh      # Rust engine → macos/lib/libqsp_core.a (universal with rustup)
./macos/make-app.sh        # → macos/Qqsp.app
./macos/make-dmg.sh        # → macos/Qqsp.dmg (optionally signed + notarized, see script)
```

## Tests

```sh
cargo test
(cd macos && swift test)    # Swift model tests (needs build-core.sh first)
QQSP_SRC=/path/to/Qqsp python3 tests/diff/run.py --fuzz 1000   # needs gcc
```

The GitHub Action (`.github/workflows/ci.yml`) runs both and builds the app.

## Performance

Compared with the C engine: on par or faster in all benchmarks (arrays, text keys, regex, string functions, 2M-iteration loop: 2.6 s vs 2.7 s). Loading a 5000-location game takes ~0.03 s.

## Known differences from the C engine

- `INT_MIN` is printed incorrectly in C (bug in `qspNumToStr`); correct here.
- `QSPSaveGameAsData` in the C library overwrites a pointer; not reproduced.

License: LGPL-2.1-or-later (derived from libqsp / Qqsp).
