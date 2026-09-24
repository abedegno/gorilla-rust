#!/usr/bin/env bash
# Submit one file (a zip, a DMG or a package) to Apple's notary service and
# wait for the verdict. On anything but Accepted, print Apple's log, which
# says what to fix, and fail. Stapling is left to the caller, since a bare
# binary cannot be stapled.
#
# Usage: notarize.sh FILE
# Needs APPLE_API_KEY_PATH (the .p8 file), APPLE_API_KEY_ID and
# APPLE_API_ISSUER_ID.
set -euo pipefail
if [ $# -ne 1 ]; then
  echo "usage: notarize.sh FILE" >&2
  exit 2
fi
file="$1"
: "${APPLE_API_KEY_PATH:?}" "${APPLE_API_KEY_ID:?}" "${APPLE_API_ISSUER_ID:?}"

result="$(mktemp)"
# notarytool can exit non-zero for a rejected submission. The verdict is
# read from its JSON either way, so that a rejection prints Apple's log.
xcrun notarytool submit "$file" \
  --key "$APPLE_API_KEY_PATH" --key-id "$APPLE_API_KEY_ID" --issuer "$APPLE_API_ISSUER_ID" \
  --wait --timeout 20m --output-format json > "$result" || true
cat "$result"

status="$(plutil -extract status raw "$result" 2>/dev/null || echo "no verdict")"
if [ "$status" != "Accepted" ]; then
  if id="$(plutil -extract id raw "$result" 2>/dev/null)"; then
    xcrun notarytool log "$id" \
      --key "$APPLE_API_KEY_PATH" --key-id "$APPLE_API_KEY_ID" --issuer "$APPLE_API_ISSUER_ID" || true
  fi
  echo "::error::Notarizing $(basename "$file") finished with status: $status."
  exit 1
fi
rm -f "$result"
