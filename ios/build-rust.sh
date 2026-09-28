#!/usr/bin/env bash
# Build the game core for iOS as ios/GorillaRust.xcframework: a static
# library for devices, and one for the Simulator on Apple Silicon and
# Intel Macs combined. Also draws the App Store icon into the asset
# catalogue. Xcode links the framework into the app.
#
# Usage: ios/build-rust.sh [release|debug]   (release by default)
set -euo pipefail
profile="${1:-release}"
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

flags=""
if [ "$profile" = release ]; then flags="--release"; fi

for target in aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios; do
  # The crate is an rlib everywhere else; only here is it a static library.
  cargo rustc --lib $flags --target "$target" --crate-type staticlib
done

sim="target/ios-sim/$profile"
mkdir -p "$sim"
lipo -create \
  "target/aarch64-apple-ios-sim/$profile/libgorillas.a" \
  "target/x86_64-apple-ios/$profile/libgorillas.a" \
  -output "$sim/libgorillas.a"

rm -rf ios/GorillaRust.xcframework
xcodebuild -create-xcframework \
  -library "target/aarch64-apple-ios/$profile/libgorillas.a" \
  -library "$sim/libgorillas.a" \
  -output ios/GorillaRust.xcframework

cargo run -q --example icon -- --opaque ios/GorillaRust/Assets.xcassets/AppIcon.appiconset/icon-1024.png
