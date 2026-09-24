#!/usr/bin/env bash
# Check release.yml for mistakes a dry run cannot show, because the steps
# involved only run on a tag push, on a re-run, or with the signing
# secrets set.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
exec ruby "$here/test-workflows.rb" "$here/../.github/workflows/release.yml"
