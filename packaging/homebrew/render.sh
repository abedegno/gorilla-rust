#!/usr/bin/env bash
# Fill in the Homebrew formula and cask for one release, from the .sha256
# files the release build writes beside each download.
#
# Usage: render.sh VERSION ARTIFACT_DIR OUT_DIR
# Writes OUT_DIR/Formula/gorilla-rust.rb and OUT_DIR/Casks/gorillas.rb.
set -euo pipefail
if [ $# -ne 3 ]; then
  echo "usage: render.sh VERSION ARTIFACT_DIR OUT_DIR" >&2
  exit 2
fi
version="$1" artifacts="$2" out="$3"
here="$(cd "$(dirname "$0")" && pwd)"

# The checksum of one download: the first field of its .sha256 file.
sha() {
  local file="$artifacts/$1.sha256" hash
  if [ ! -f "$file" ]; then
    echo "render.sh: no checksum for $1 in $artifacts" >&2
    exit 1
  fi
  hash="$(awk '{ print $1; exit }' "$file")"
  if ! printf '%s' "$hash" | grep -Eq '^[0-9a-f]{64}$'; then
    echo "render.sh: $file does not start with a SHA-256" >&2
    exit 1
  fi
  echo "$hash"
}

macos="$(sha gorilla-rust-universal-apple-darwin.tar.gz)"
linux_x86_64="$(sha gorilla-rust-x86_64-unknown-linux-gnu.tar.gz)"
linux_aarch64="$(sha gorilla-rust-aarch64-unknown-linux-gnu.tar.gz)"
dmg="$(sha "Gorillas-$version.dmg")"

fill() {
  sed -e "s/@VERSION@/$version/g" \
    -e "s/@SHA256_MACOS@/$macos/g" \
    -e "s/@SHA256_LINUX_X86_64@/$linux_x86_64/g" \
    -e "s/@SHA256_LINUX_AARCH64@/$linux_aarch64/g" \
    -e "s/@SHA256_DMG@/$dmg/g" \
    "$1" > "$2"
  if grep -n '@[A-Z0-9_]*@' "$2" >&2; then
    echo "render.sh: $2 still has a placeholder" >&2
    exit 1
  fi
}

mkdir -p "$out/Formula" "$out/Casks"
fill "$here/gorilla-rust.rb.in" "$out/Formula/gorilla-rust.rb"
fill "$here/gorillas.rb.in" "$out/Casks/gorillas.rb"
