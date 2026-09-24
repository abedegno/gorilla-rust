#!/usr/bin/env bash
# Print the version from Cargo.toml. On a tag push, fail unless the tag
# names the same version: the tag decides the download URLs, and
# Cargo.toml decides what `gorilla-rust --version` prints, which the
# Homebrew formula's test checks.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
version="$(sed -n 's/^version = "\(.*\)"$/\1/p' "$root/Cargo.toml" | head -1)"
if [ -z "$version" ]; then
  echo "version.sh: no version in Cargo.toml" >&2
  exit 1
fi
if [ "${GITHUB_EVENT_NAME:-}" = push ] && [ "${GITHUB_REF_TYPE:-}" = tag ] \
  && [ "${GITHUB_REF_NAME#v}" != "$version" ]; then
  echo "version.sh: the tag is $GITHUB_REF_NAME but Cargo.toml says $version" >&2
  exit 1
fi
echo "$version"
