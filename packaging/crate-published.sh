#!/usr/bin/env bash
# Succeed if crates.io already has this version of the crate, so that a
# re-run of a release does not try to publish it again, which crates.io
# refuses. Exits 0 if it is published, 1 if it is not, and 2 if crates.io
# could not say, so an outage is never mistaken for an answer.
#
# Usage: crate-published.sh CRATE VERSION
set -euo pipefail
if [ $# -ne 2 ]; then
  echo "usage: crate-published.sh CRATE VERSION" >&2
  exit 2
fi
crate="$1" version="$2"

# crates.io asks every client of its API to say who it is.
agent="gorilla-rust release (https://github.com/abedegno/gorilla-rust)"
code="$(curl -s -o /dev/null -w '%{http_code}' -A "$agent" \
  "https://crates.io/api/v1/crates/$crate/$version" || true)"
case "$code" in
  200) exit 0 ;;
  404) exit 1 ;;
  *)
    echo "crate-published.sh: crates.io answered '$code' for $crate $version" >&2
    exit 2
    ;;
esac
