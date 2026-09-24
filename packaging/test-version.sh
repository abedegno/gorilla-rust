#!/usr/bin/env bash
# Check packaging/version.sh: it prints Cargo.toml's version, and on a tag
# push it refuses a tag that names another version, before anything is
# built from it.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
expected="$(sed -n 's/^version = "\(.*\)"$/\1/p' "$here/../Cargo.toml" | head -1)"

got="$(env -u GITHUB_EVENT_NAME "$here/version.sh")"
[ "$got" = "$expected" ] || { echo "FAIL: printed '$got', expected '$expected'"; exit 1; }

got="$(GITHUB_EVENT_NAME=push GITHUB_REF_TYPE=tag GITHUB_REF_NAME="v$expected" "$here/version.sh")"
[ "$got" = "$expected" ] || { echo "FAIL: a matching tag printed '$got'"; exit 1; }

if GITHUB_EVENT_NAME=push GITHUB_REF_TYPE=tag GITHUB_REF_NAME=v0.0.0-not-this "$here/version.sh" >/dev/null 2>&1; then
  echo "FAIL: a tag naming another version was accepted"
  exit 1
fi

# A manual run from a branch has no tag to compare, so anything goes.
GITHUB_EVENT_NAME=workflow_dispatch GITHUB_REF_TYPE=branch GITHUB_REF_NAME=main "$here/version.sh" >/dev/null

echo "version.sh: ok"
