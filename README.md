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

## QSP 5.9 functions (optional, per game)

The default behaviour is bit-exact libqsp 5.7.0. Per game (Settings → This Game) an extra mode adds
additive features from libqsp 5.9.6: `SORTARR`, `SCANSTR`, `SETVAR`, `ARRITEM`, `COUNTOBJ('name')` and the
optional count argument of `REPLACE`. `$QSPVER` still reports 5.7.0, and 5.8+ semantics changes
(typed values, 1/0 booleans, new argument orders, tuples, loops …) are **not** part of this mode.
Variables keep the 5.7 model; results match 5.9.6 for uniformly typed arrays without holes
(differential tests in `tests/diff59`, `tests/diff59/README.md`).

## License

Mixed: LGPL-2.1-or-later (derived from libqsp 5.7 / Qqsp, see `LICENSE`) for everything except
`src/ext59.rs`, which implements behaviour of libqsp 5.9 and is licensed under MPL-2.0 (see `LICENSE_MPL2`).
