#!/bin/sh
# Baut Qqsp.app (Release) nach macos/Qqsp.app. Vorher einmal ./build-core.sh ausführen.
set -e
cd "$(dirname "$0")"
swift build -c release
BIN="$(swift build -c release --show-bin-path)/Qqsp"
APP=Qqsp.app
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BIN" "$APP/Contents/MacOS/Qqsp"
cp Resources/AppIcon.icns "$APP/Contents/Resources/AppIcon.icns"
cp -R Resources/en.lproj Resources/de.lproj "$APP/Contents/Resources/"
cat > "$APP/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleName</key><string>Qqsp</string>
  <key>CFBundleDisplayName</key><string>Qqsp</string>
  <key>CFBundleIdentifier</key><string>de.kompsoft.qqsp-rs</string>
  <key>CFBundleExecutable</key><string>Qqsp</string>
  <key>CFBundleDevelopmentRegion</key><string>de</string>
  <key>CFBundleLocalizations</key><array><string>de</string><string>en</string></array>
  <key>CFBundleIconFile</key><string>AppIcon</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>0.1.0</string>
  <key>CFBundleVersion</key><string>1</string>
  <key>LSMinimumSystemVersion</key><string>26.0</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>CFBundleURLTypes</key><array><dict>
    <key>CFBundleURLName</key><string>de.kompsoft.qqsp-rs</string>
    <key>CFBundleURLSchemes</key><array><string>qqsp</string></array>
  </dict></array>
  <key>CFBundleDocumentTypes</key><array><dict>
    <key>CFBundleTypeName</key><string>QSP-Spielstand</string>
    <key>CFBundleTypeRole</key><string>Viewer</string>
    <key>CFBundleTypeIconFile</key><string>AppIcon</string>
    <key>CFBundleTypeExtensions</key><array><string>sav</string></array>
  </dict><dict>
    <key>CFBundleTypeName</key><string>QSP-Spiel</string>
    <key>CFBundleTypeRole</key><string>Viewer</string>
    <key>CFBundleTypeIconFile</key><string>AppIcon</string>
    <key>LSHandlerRank</key><string>Default</string>
    <key>CFBundleTypeExtensions</key><array><string>qsp</string><string>gam</string></array>
  </dict></array>
</dict></plist>
PLIST
codesign --force --sign - "$APP" >/dev/null 2>&1 || true
echo "fertig: $(pwd)/$APP  (zum Installieren nach /Programme ziehen)"
