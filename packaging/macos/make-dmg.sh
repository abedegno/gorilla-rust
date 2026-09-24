#!/usr/bin/env bash
# Put an app in a compressed disk image beside a link to /Applications, so
# installing it is one drag. Signing is left to the caller.
#
# Usage: make-dmg.sh APP OUT_DMG
set -euo pipefail
if [ $# -ne 2 ]; then
  echo "usage: make-dmg.sh APP OUT_DMG" >&2
  exit 2
fi
app="$1" dmg="$2"

stage="$(mktemp -d)"
# ditto keeps what cp -R can drop: extended attributes, and the stapled
# ticket and signature exactly as they are.
ditto "$app" "$stage/$(basename "$app")"
ln -s /Applications "$stage/Applications"

rm -f "$dmg"
# hdiutil fails now and then on CI runners with "Resource busy", which a
# retry a few seconds later gets past.
for attempt in 1 2 3; do
  if hdiutil create -volname Gorillas -srcfolder "$stage" -format UDZO -ov "$dmg"; then
    rm -rf "$stage"
    echo "$dmg"
    exit 0
  fi
  echo "make-dmg.sh: hdiutil failed, attempt $attempt of 3" >&2
  sleep 5
done
rm -rf "$stage"
exit 1
