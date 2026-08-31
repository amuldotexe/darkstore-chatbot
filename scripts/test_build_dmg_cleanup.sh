#!/usr/bin/env bash
set -euo pipefail

project_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fixture_directory="$(mktemp -d)"
trap 'rm -rf "$fixture_directory"' EXIT

if ! rg -q 'APPLE_SIGNING_IDENTITY' "$project_directory/scripts/build_clean_dmg_v001.sh"; then
  printf 'Expected the DMG build to select a Developer ID signing identity.\n' >&2
  exit 1
fi

if ! rg -q 'universal-apple-darwin' "$project_directory/scripts/build_clean_dmg_v001.sh"; then
  printf 'Expected the DMG build to target a universal Apple binary.\n' >&2
  exit 1
fi

if ! rg -q 'aarch64-apple-darwin/release' "$project_directory/scripts/build_clean_dmg_v001.sh" \
  || ! rg -q 'x86_64-apple-darwin/release' "$project_directory/scripts/build_clean_dmg_v001.sh"; then
  printf 'Expected both architecture-specific release directories to be cleared before packaging.\n' >&2
  exit 1
fi

if ! rg -q 'codesign --verify --deep --strict' "$project_directory/scripts/build_clean_dmg_v001.sh"; then
  printf 'Expected the DMG build to verify the mounted application signature.\n' >&2
  exit 1
fi

if ! rg -q 'APPLE_NOTARY_KEYCHAIN_PROFILE' "$project_directory/scripts/build_clean_dmg_v001.sh" \
  || ! rg -q 'notarytool submit' "$project_directory/scripts/build_clean_dmg_v001.sh" \
  || ! rg -q 'stapler staple' "$project_directory/scripts/build_clean_dmg_v001.sh"; then
  printf 'Expected the shareable DMG build to require Keychain-backed Apple notarization and staple its ticket.\n' >&2
  exit 1
fi

if ! rg -q 'ALLOW_UNNOTARIZED_LOCAL_BUILD' "$project_directory/scripts/build_clean_dmg_v001.sh"; then
  printf 'Expected local-only builds to require an explicit unnotarized opt-out.\n' >&2
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
