#!/usr/bin/env bash
set -e

APP_NAME="yoinkctl"
VERSION="${VERSION:-1.0.0}"
ARCH=$(uname -m)
APP_DIR="AppDir"

echo "🚀 Building AppImage for $APP_NAME v$VERSION ($ARCH)..."

# Clean AppDir
rm -rf "$APP_DIR"
mkdir -p \
  "$APP_DIR/usr/bin" \
  "$APP_DIR/usr/share/applications" \
  "$APP_DIR/usr/share/icons/hicolor/256x256/apps"

echo "📦 Copying binary..."
cp "target/release/$APP_NAME" "$APP_DIR/usr/bin/"

echo "📄 Creating desktop file..."
cat > "$APP_DIR/usr/share/applications/$APP_NAME.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=yoinkctl
GenericName=Color Picker
Comment=Fast, beautiful color picker with global hotkey support
Exec=yoinkctl
Icon=yoinkctl
Terminal=false
Categories=Utility;
Keywords=color;picker;screenshot;eyedropper;
StartupNotify=true
Actions=Pick;Daemon;

[Desktop Action Pick]
Name=Pick Color
Exec=yoinkctl pick

[Desktop Action Daemon]
Name=Start Daemon
Exec=yoinkctl daemon
EOF

echo "🎨 Installing icon..."
if [ -f "assets/yoinkctl.png" ]; then
  cp "assets/yoinkctl.png" \
     "$APP_DIR/usr/share/icons/hicolor/256x256/apps/yoinkctl.png"
else
  echo "⚠️  No icon found, creating placeholder"
  printf '\x89PNG\r\n\x1a\n' > \
    "$APP_DIR/usr/share/icons/hicolor/256x256/apps/yoinkctl.png"
fi

echo "🔧 Downloading linuxdeploy..."
if [ ! -f "linuxdeploy-$ARCH.AppImage" ]; then
  wget -q \
    "https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-$ARCH.AppImage"
  chmod +x "linuxdeploy-$ARCH.AppImage"
fi

echo "🏗️  Building AppImage..."
export NO_STRIP=1

./linuxdeploy-$ARCH.AppImage \
  --appdir "$APP_DIR" \
  --desktop-file "$APP_DIR/usr/share/applications/$APP_NAME.desktop" \
  --icon-file "$APP_DIR/usr/share/icons/hicolor/256x256/apps/yoinkctl.png" \
  --executable "$APP_DIR/usr/bin/$APP_NAME" \
  --output appimage

OUTPUT_NAME="yoinkctl-$VERSION-$ARCH.AppImage"
mv yoinkctl-*.AppImage "$OUTPUT_NAME"

echo ""
echo "✅ AppImage built successfully!"
echo "📦 Output: $OUTPUT_NAME"
echo ""
echo "Test it:"
echo "  ./$OUTPUT_NAME"
echo "  ./$OUTPUT_NAME pick"
echo "  ./$OUTPUT_NAME daemon"
