#!/bin/sh
# Baut Qqsp.dmg (Qqsp.app + Verknüpfung auf /Programme). Baut die App vorher neu.
#
# Optional signieren + notarisieren (für Weitergabe an andere Macs):
#   SIGN_ID="Developer ID Application: Dein Name (TEAMID)" \
#   NOTARY_PROFILE=mein-profil ./make-dmg.sh
# Das Notary-Profil legst du einmalig an mit:
#   xcrun notarytool store-credentials mein-profil --apple-id <ID> --team-id <TEAMID> --password <app-spezifisches-Passwort>
set -e
cd "$(dirname "$0")"
./make-app.sh
APP=Qqsp.app

if [ -n "$SIGN_ID" ]; then
  codesign --force --options runtime --timestamp --sign "$SIGN_ID" "$APP"
fi

STAGE="$(mktemp -d)"
cp -R "$APP" "$STAGE/"
ln -s /Applications "$STAGE/Applications"
rm -f Qqsp.dmg
hdiutil create -volname "Qqsp" -srcfolder "$STAGE" -ov -format UDZO Qqsp.dmg >/dev/null
rm -rf "$STAGE"

if [ -n "$SIGN_ID" ]; then
  codesign --force --timestamp --sign "$SIGN_ID" Qqsp.dmg
  if [ -n "$NOTARY_PROFILE" ]; then
    xcrun notarytool submit Qqsp.dmg --keychain-profile "$NOTARY_PROFILE" --wait
    xcrun stapler staple Qqsp.dmg
  fi
fi
echo "fertig: $(pwd)/Qqsp.dmg"
