# Qqsp-RS

Rust-Port der QSP-Engine (libqsp 5.7.0) + SwiftUI-Shell mit Liquid Glass für macOS.

- `src/` – Engine (`qsp_core`), `src/ffi.rs` + `include/qsp_core.h` – C-Schnittstelle
- `tests/`, `examples/` – Unit-Tests und Treiber für den Differenztest gegen die C-Engine
- `macos/` – SwiftUI-App (macOS 26 / Xcode 26): `./macos/build-core.sh && cd macos && swift run -c release`

Lizenz: LGPL-2.1-or-later (abgeleitet von libqsp / Qqsp).
