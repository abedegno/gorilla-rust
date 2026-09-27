#!/usr/bin/env bash
# Check what the published crate would contain. Cargo's include patterns
# match at any depth unless they start with "/", so "README.md" would also
# take every README under web/node_modules. Decoys planted there, where git
# does not look, prove the patterns are anchored whatever else the checkout
# holds, and a publish can never be a surprise.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
decoy="$root/web/node_modules/crate-decoy"
mkdir -p "$decoy"
trap 'rm -rf "$decoy"' EXIT
for f in README.md LICENSE CHANGELOG.md; do
  echo "decoy" > "$decoy/$f"
done

list="$(cd "$root" && cargo package --list --allow-dirty 2>/dev/null)"

allowed='^(\.cargo_vcs_info\.json|Cargo\.toml|Cargo\.toml\.orig|Cargo\.lock|README\.md|LICENSE|CHANGELOG\.md|docs/screenshot\.png|assets/.+|src/.+)$'
stray="$(printf '%s\n' "$list" | grep -vE "$allowed" || true)"
if [ -n "$stray" ]; then
  echo "FAIL: the crate would ship files it should not:"
  echo "$stray"
  exit 1
fi

# And it must still ship what it needs to build and to show on crates.io.
for needed in src/main.rs src/lib.rs assets/fonts/ega8x14.bin README.md LICENSE; do
  printf '%s\n' "$list" | grep -qx "$needed" || { echo "FAIL: the crate is missing $needed"; exit 1; }
done

echo "crate contents: ok"
