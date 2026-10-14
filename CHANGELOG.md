# Changelog

## Unreleased
- Undo/redo of moves (⌘[ / ⌘]) via in-memory snapshots, up to 100 steps
- Game library on the start screen: recent games with cover, last played, play time, one-click resume
- Save previews: the last text of the game is shown next to each save
- Per-game display settings (zoom, font size, dark mode) remembered per game
- Engine: faster expression evaluation (no allocations for operator/variable lookup); now on par with or faster than the C engine in all benchmarks
- Swift unit tests (`swift test`), run in CI
- Multiple games: one window/tab per game with separate state (⌘N, ⌘T); opening a game in a loaded window opens a new tab
- English localization (German remains the source language)
- Saves and Debug windows follow the active game window
- Autosave only for the closing window's game

## 0.1.0
- Rust port of the QSP engine (libqsp 5.7.0), verified by unit and differential tests
- C API and SwiftUI Liquid Glass player for macOS
- Original-style layout with persisted splitter widths, HTML rendering, audio, themes/colors/fonts
- Save slots, quick save/load, autosave, resume last game
- Debug window, keyboard navigation, hover highlight, app icon
