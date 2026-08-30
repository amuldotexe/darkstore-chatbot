#!/usr/bin/env bash
set -euo pipefail

project_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fixture_directory="$(mktemp -d)"
trap 'rm -rf "$fixture_directory"' EXIT

if ! rg -q 'APPLE_SIGNING_IDENTITY' "$project_directory/scripts/build_clean_dmg_v001.sh"; then
  printf 'Expected the DMG build to select a Developer ID signing identity.\n' >&2
  exit 1
fi

if rg -q -- '--no-sign' "$project_directory/scripts/build_clean_dmg_v001.sh"; then
  printf 'DMG build must not opt out of signing.\n' >&2
  exit 1
fi

mkdir -p "$fixture_directory/dmg" "$fixture_directory/macos"
touch "$fixture_directory/dmg/old-release.dmg"
touch "$fixture_directory/macos/old-release.app"

"$project_directory/scripts/build_clean_dmg_v001.sh" --clean-only "$fixture_directory"

test ! -e "$fixture_directory/dmg"
test ! -e "$fixture_directory/macos"
