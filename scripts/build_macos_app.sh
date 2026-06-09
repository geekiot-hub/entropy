#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_NAME="${APP_NAME:-Entropy}"
BUNDLE_ID="${BUNDLE_ID:-com.ergohaven.entropy}"
MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-10.15}"
export MACOSX_DEPLOYMENT_TARGET

VERSION="$(
  awk -F '"' '/^version = / { print $2; exit }' "$ROOT/Cargo.toml"
)"

TARGET="${TARGET:-}"
if [[ -n "$TARGET" ]]; then
  BUILD_ARGS=(--release --target "$TARGET")
  BIN="$ROOT/target/$TARGET/release/entropy"
  ARCH="${TARGET%%-*}"
else
  BUILD_ARGS=(--release)
  BIN="$ROOT/target/release/entropy"
  ARCH="$(uname -m)"
fi

DIST_DIR="$ROOT/dist/macos"
APP_PATH="$DIST_DIR/$APP_NAME.app"
CONTENTS_DIR="$APP_PATH/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"
ZIP_PATH="$DIST_DIR/entropy-v$VERSION-macos-$ARCH.app.zip"
DMG_PATH="$DIST_DIR/entropy-v$VERSION-macos-$ARCH.dmg"

cd "$ROOT"
cargo build "${BUILD_ARGS[@]}"

rm -rf "$APP_PATH" "$ZIP_PATH" "$DMG_PATH"
mkdir -p "$MACOS_DIR" "$RESOURCES_DIR"

cp "$BIN" "$MACOS_DIR/entropy"
chmod 755 "$MACOS_DIR/entropy"

ICON_PLIST=""
if [[ -f "$ROOT/assets/entropy.icns" ]]; then
  cp "$ROOT/assets/entropy.icns" "$RESOURCES_DIR/entropy.icns"
  ICON_PLIST='
    <key>CFBundleIconFile</key>
    <string>entropy</string>'
fi

cat > "$CONTENTS_DIR/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleDisplayName</key>
    <string>$APP_NAME</string>
    <key>CFBundleExecutable</key>
    <string>entropy</string>
    <key>CFBundleIdentifier</key>
    <string>$BUNDLE_ID</string>$ICON_PLIST
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>$APP_NAME</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>LSMinimumSystemVersion</key>
    <string>$MACOSX_DEPLOYMENT_TARGET</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
PLIST

if command -v ditto >/dev/null 2>&1; then
  ditto -c -k --sequesterRsrc --keepParent "$APP_PATH" "$ZIP_PATH"
else
  (cd "$DIST_DIR" && zip -qry "$(basename "$ZIP_PATH")" "$APP_NAME.app")
fi

if command -v hdiutil >/dev/null 2>&1; then
  hdiutil create \
    -volname "$APP_NAME" \
    -srcfolder "$APP_PATH" \
    -ov \
    -format UDZO \
    "$DMG_PATH" >/dev/null
  echo "Built $DMG_PATH"
else
  echo "hdiutil not found; skipped DMG build"
fi

echo "Built $APP_PATH"
echo "Built $ZIP_PATH"
