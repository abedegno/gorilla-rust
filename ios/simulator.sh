#!/usr/bin/env bash
# Print the UDID of an available iPhone Simulator, preferring a booted one.
# Simulator names change with each Xcode, so nothing names one directly.
set -euo pipefail
xcrun simctl list devices available -j | ruby -rjson -e '
  devices = JSON.parse(STDIN.read)["devices"]
    .select { |runtime, _| runtime.include?("iOS") }
    .values.flatten
    .select { |d| d["name"].start_with?("iPhone") }
  pick = devices.find { |d| d["state"] == "Booted" } || devices.last
  abort "no iPhone Simulator is available" unless pick
  puts pick["udid"]'
