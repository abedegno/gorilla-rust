#!/usr/bin/env bash
# Take the App Store screenshots in a 6.9-inch iPhone and a 13-inch iPad
# Simulator, into ios/build/screenshots.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
out="$root/ios/build/screenshots"
mkdir -p "$out"
"$root/ios/build-rust.sh" debug

device_for() { # device type name pattern
  local type
  type="$(xcrun simctl list devicetypes | grep -E "$1" | head -1 | sed -E 's/.*\((com\.apple[^)]*)\).*/\1/')"
  [ -n "$type" ] || { echo "no device type matches '$1'" >&2; exit 1; }
  xcrun simctl create "screenshots-$RANDOM" "$type"
}

for pattern in "iPhone [0-9]+ Pro Max" "iPad Pro 13"; do
  udid="$(device_for "$pattern")"
  TEST_RUNNER_SCREENSHOTS_DIR="$out" xcodebuild test \
    -project "$root/ios/GorillaRust.xcodeproj" -scheme GorillaRust \
    -destination "id=$udid" CODE_SIGNING_ALLOWED=NO \
    -only-testing:GorillaRustUITests/ScreenshotUITests
  xcrun simctl delete "$udid"
done
ls -1 "$out"
