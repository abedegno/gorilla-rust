#!/usr/bin/env bash
# Check render.sh: each checksum lands beside its own download, nothing is
# left unfilled, both files parse as Ruby, and a missing checksum is an
# error rather than a blank.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

version=9.8.7
mkdir "$work/artifacts"
# A different, recognisable checksum for each asset: 000…01, 000…02 and
# so on, so a checksum in the wrong place is caught.
n=0
for asset in gorilla-rust-universal-apple-darwin.tar.gz \
  gorilla-rust-x86_64-unknown-linux-gnu.tar.gz \
  gorilla-rust-aarch64-unknown-linux-gnu.tar.gz \
  "Gorillas-$version.dmg"; do
  n=$((n + 1))
  printf '%064d  %s\n' "$n" "$asset" > "$work/artifacts/$asset.sha256"
done

"$here/render.sh" "$version" "$work/artifacts" "$work/out"
formula="$work/out/Formula/gorilla-rust.rb"
cask="$work/out/Casks/gorillas.rb"

for f in "$formula" "$cask"; do
  if grep -n '@[A-Z0-9_]*@' "$f"; then echo "FAIL: a placeholder is left in $f"; exit 1; fi
  ruby -c "$f" >/dev/null
done

# The line after each url must carry that asset's checksum.
expect() { # file, part of the url, checksum number
  local got
  got="$(grep -A1 "$2" "$1" | grep -o 'sha256 "[0-9a-f]*"' || true)"
  [ "$got" = "sha256 \"$(printf '%064d' "$3")\"" ] || {
    echo "FAIL: $2 in $(basename "$1") has '$got'"
    exit 1
  }
}
expect "$formula" universal-apple-darwin 1
expect "$formula" x86_64-unknown-linux-gnu 2
expect "$formula" aarch64-unknown-linux-gnu 3
grep -q "download/v$version/" "$formula" || { echo "FAIL: the formula's URLs miss v$version"; exit 1; }
grep -q "version \"$version\"" "$cask" || { echo "FAIL: the cask's version is not $version"; exit 1; }
grep -q "sha256 \"$(printf '%064d' 4)\"" "$cask" || { echo "FAIL: the cask has the wrong checksum"; exit 1; }

rm "$work/artifacts/Gorillas-$version.dmg.sha256"
if "$here/render.sh" "$version" "$work/artifacts" "$work/again" 2>/dev/null; then
  echo "FAIL: rendered without the disk image's checksum"
  exit 1
fi

echo "render.sh: ok"
