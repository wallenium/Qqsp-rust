# Changelog

## Unreleased
- Variable editor window (⇧⌘E): view, edit, create and delete all variables of the active game; undoable via Undo Move
- Settings reorganized into tabs (General, Text, Colors, Sound, This Game) in macOS system-settings style
- List icons: adjustable size, larger preview popover on hover
- Tooltips on `exec:` links show the code they run
- Window size/position remembered per game; always-visible scrollbar (optional)
- Separate font size for the additional description (global and per game)
- "Classic look" option: tighter margins and Helvetica like Qt Qqsp
- Actions/Objects lists render bold, italic, underline and font colors from the game's HTML
- HTML mode now converts newlines to `<br>` like the original Qqsp (fixes broken table layouts)
- Automatic contrast fix: text colors of games below 4.5:1 contrast against the background are lightened/darkened (Settings ▸ Colors)
- Separate font sizes for the Actions and Objects lists (global and per game)
- Game info: author/description read from info/readme files in the game folder (Game ▸ Game Info…, library tooltips)
- `qqsp://` URL commands for Shortcuts/scripts: `open?path=`, `action?n=`, `quicksave`, `quickload`, `undo`, `redo`
- `.sav` files open in the active game; document icons for `.qsp`/`.gam`/`.sav`
- Optional sync folder for quick/auto/slot saves (e.g. iCloud Drive), device-independent file names
- Robustness: fuzz tests with corrupted game files, savegames and random code (no panics in 1.2M cases)
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
