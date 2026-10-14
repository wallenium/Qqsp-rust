#!/bin/sh
# Baut die Rust-Engine als statische Bibliothek nach macos/lib/.
# Mit rustup: universell (arm64 + x86_64). Ohne rustup (z. B. Homebrew-Rust): nur für diesen Mac.
set -e
cd "$(dirname "$0")/.."
command -v cargo >/dev/null || { echo "cargo nicht gefunden (PATH prüfen, ggf. neues Terminal öffnen)"; exit 1; }
mkdir -p macos/lib
if command -v rustup >/dev/null; then
  rustup target add aarch64-apple-darwin x86_64-apple-darwin
  cargo build --release --target aarch64-apple-darwin
  cargo build --release --target x86_64-apple-darwin
  lipo -create target/aarch64-apple-darwin/release/libqsp_core.a target/x86_64-apple-darwin/release/libqsp_core.a -output macos/lib/libqsp_core.a
else
  cargo build --release
  cp target/release/libqsp_core.a macos/lib/libqsp_core.a
fi
echo "fertig: macos/lib/libqsp_core.a – jetzt: cd macos && swift run -c release"
