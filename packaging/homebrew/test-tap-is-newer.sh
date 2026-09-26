#!/usr/bin/env bash
# Check tap-is-newer.sh, which stops a re-run of an old release's homebrew
# job from putting the tap back to that release: it answers yes only when
# the tap already holds a strictly newer version, compared as versions,
# not as text.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
tap="$(mktemp -d)"
trap 'rm -rf "$tap"' EXIT

newer() { "$here/tap-is-newer.sh" "$1" "$tap"; }
holds() {
  mkdir -p "$tap/Casks"
  printf 'cask "gorillas" do\n  version "%s"\nend\n' "$1" > "$tap/Casks/gorillas.rb"
}
expect_yes() { newer "$1" || { echo "FAIL: tap with $2 should count as newer than $1"; exit 1; }; }
expect_no() { if newer "$1"; then echo "FAIL: tap with $2 should not count as newer than $1"; exit 1; fi; }

# An empty tap, before the first release.
expect_no 1.4.0 "nothing"

holds 1.4.0
expect_no 1.4.0 1.4.0 # a re-run of the same release goes ahead
expect_no 1.5.0 1.4.0
holds 1.4.1
expect_yes 1.4.0 1.4.1
holds 1.10.0
expect_yes 1.9.0 1.10.0
holds 1.9.0
expect_no 1.10.0 1.9.0

echo "tap-is-newer.sh: ok"
