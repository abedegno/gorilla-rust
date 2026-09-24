#!/usr/bin/env bash
# Build Gorillas.app around a gorilla-rust binary: Info.plist from its
# template, and Gorillas.icns from a 1024 pixel PNG. Signing is left to the
# caller, since a release signs it and a local build need not.
#
# Usage: make-app.sh VERSION BINARY ICON_PNG OUT_DIR
set -euo pipefail
if [ $# -ne 4 ]; then
  echo "usage: make-app.sh VERSION BINARY ICON_PNG OUT_DIR" >&2
  exit 2
fi
version="$1" binary="$2" icon_png="$3" out="$4"
here="$(cd "$(dirname "$0")" && pwd)"
app="$out/Gorillas.app"

rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"

sed "s/@VERSION@/$version/g" "$here/Info.plist" > "$app/Contents/Info.plist"
if grep -n '@[A-Z_]*@' "$app/Contents/Info.plist" >&2; then
  echo "make-app.sh: Info.plist still has a placeholder" >&2
  exit 1
fi
plutil -lint "$app/Contents/Info.plist" >/dev/null

cp "$binary" "$app/Contents/MacOS/gorilla-rust"
chmod 755 "$app/Contents/MacOS/gorilla-rust"

# An iconset holds each size at 1x and 2x. The icon is 32 blocks across,
# so every size from 32 up keeps the blocks whole.
iconset="$(mktemp -d)/Gorillas.iconset"
mkdir "$iconset"
for size in 16 32 128 256 512; do
  double=$((size * 2))
  sips -z "$size" "$size" "$icon_png" --out "$iconset/icon_${size}x${size}.png" >/dev/null
  sips -z "$double" "$double" "$icon_png" --out "$iconset/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$iconset" -o "$app/Contents/Resources/Gorillas.icns"
rm -rf "$(dirname "$iconset")"

echo "$app"
