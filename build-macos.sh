#!/bin/bash

export MACOSX_DEPLOYMENT_TARGET="11.0"
cargo build --release --target=x86_64-apple-darwin
cargo build --release --target=aarch64-apple-darwin

TARGET="IQViewer"
ASSETS_DIR="assets"
RELEASE_DIR="target/release"
APP_NAME="IQViewer.app"
APP_TEMPLATE="$ASSETS_DIR/macos/$APP_NAME"
APP_TEMPLATE_PLIST="$APP_TEMPLATE/Contents/Info.plist"
APP_DIR="$RELEASE_DIR/macos-iqviewer"
APP_BINARY="$RELEASE_DIR/$TARGET"
APP_BINARY_DIR="$APP_DIR/$APP_NAME/Contents/MacOS"
APP_EXTRAS_DIR="$APP_DIR/$APP_NAME/Contents/Resources"

DMG_NAME="IQViewer.dmg"
DMG_DIR="$RELEASE_DIR/macos-iqviewer"

VERSION=$(cargo metadata --format-version=1 --no-deps | jq -r '.packages[0].version')
BUILD=$(git describe --always --dirty --exclude='*')

# update version and build
sed -i '' -e "s/{{ VERSION }}/$VERSION/g" "$APP_TEMPLATE_PLIST"
sed -i '' -e "s/{{ BUILD }}/$BUILD/g" "$APP_TEMPLATE_PLIST"

# build binary
lipo "target/x86_64-apple-darwin/release/$TARGET" "target/aarch64-apple-darwin/release/$TARGET" -create -output "$APP_BINARY"

# build app
mkdir -p "$APP_BINARY_DIR"
mkdir -p "$APP_EXTRAS_DIR"
cp -fRp "$APP_TEMPLATE" "$APP_DIR"
cp -fp "$APP_BINARY" "$APP_BINARY_DIR"
touch -r "$APP_BINARY" "$APP_DIR/$APP_NAME"
echo "Created '$APP_NAME' in '$APP_DIR'"

# package dmg
echo "Packing disk image..."
ln -sf /Applications "$DMG_DIR/Applications"
hdiutil create "$DMG_DIR/$DMG_NAME" -volname $TARGET -fs HFS+ -srcfolder "$APP_DIR" -ov -format UDZO
echo "Packed '$APP_NAME' in '$APP_DIR'"

