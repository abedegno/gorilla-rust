#!/usr/bin/env bash
# Succeed if the tap already holds a newer gorilla-rust than VERSION, so
# that re-running an old release's homebrew job, say after renewing an
# expired token, does not put the tap back to that release. The same
# version is not newer: a re-run of the latest release goes ahead, and
# changes nothing.
#
# Usage: tap-is-newer.sh VERSION TAP_DIR
set -euo pipefail
if [ $# -ne 2 ]; then
  echo "usage: tap-is-newer.sh VERSION TAP_DIR" >&2
  exit 2
fi
version="$1" cask="$2/Casks/gorillas.rb"

# Before the first release the tap has no cask, and nothing is newer.
[ -f "$cask" ] || exit 1
current="$(sed -n 's/^  version "\(.*\)"$/\1/p' "$cask")"
[ -n "$current" ] || exit 1
[ "$current" != "$version" ] || exit 1

# sort -V orders 1.10.0 after 1.9.0, which plain text sorting does not.
highest="$(printf '%s\n%s\n' "$current" "$version" | sort -V | tail -1)"
[ "$highest" = "$current" ]
