#!/usr/bin/env bash
# Check crate-published.sh against crates.io itself: a version that has
# been published for years, and one that never will be. It needs the
# network, as the release job that uses it does.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"

"$here/crate-published.sh" serde 1.0.0 || {
  echo "FAIL: serde 1.0.0 should count as published (exit $?)"
  exit 1
}

status=0
"$here/crate-published.sh" gorilla-rust 0.0.0-never || status=$?
[ "$status" -eq 1 ] || { echo "FAIL: an unpublished version should exit 1, not $status"; exit 1; }

status=0
"$here/crate-published.sh" 2>/dev/null || status=$?
[ "$status" -eq 2 ] || { echo "FAIL: no arguments should exit 2, not $status"; exit 1; }

echo "crate-published.sh: ok"
