#!/usr/bin/env bash
set -euo pipefail

project_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
bundle_directory="$project_directory/src-tauri/target/release/bundle"
output_directory="$bundle_directory/dmg"
output_dmg="$output_directory/Darkstore-Concierge-v001-current.dmg"

clean_release_bundle() {
  local bundle_root="$1"
  rm -rf "$bundle_root/dmg" "$bundle_root/macos"
}

find_developer_id_identity() {
  security find-identity -v -p codesigning 2>/dev/null \
    | sed -n 's/.*"\(Developer ID Application:.*\)".*/\1/p' \
    | head -n 1
}

if [[ "${1:-}" == "--clean-only" ]]; then
  clean_release_bundle "${2:?Expected a bundle directory after --clean-only}"
  exit 0
fi

clean_release_bundle "$bundle_directory"
rm -rf "$project_directory/src-tauri/target/debug/bundle"
rm -f "$project_directory/src-tauri/target/debug/darkstore-concierge"
cargo clean --manifest-path "$project_directory/src-tauri/Cargo.toml" -p darkstore-concierge

signing_identity="${APPLE_SIGNING_IDENTITY:-}"
if [[ -z "$signing_identity" ]]; then
  signing_identity="$(find_developer_id_identity)"
fi

if [[ -z "$signing_identity" ]]; then
  printf 'No Developer ID Application signing identity found. Set APPLE_SIGNING_IDENTITY to a valid identity.\n' >&2
  exit 1
fi

(
  cd "$project_directory"
  LC_ALL=C LANG=C TAURI_BUNDLER_DMG_IGNORE_CI=1 APPLE_SIGNING_IDENTITY="$signing_identity" \
    pnpm tauri build --bundles dmg
)

dmgs_found="$(find "$output_directory" -maxdepth 1 -type f -name '*.dmg' -print | wc -l | tr -d ' ')"
if [[ "$dmgs_found" -ne 1 ]]; then
  printf 'Expected exactly one fresh DMG, found %s.\n' "$dmgs_found" >&2
  exit 1
fi

built_dmg="$(find "$output_directory" -maxdepth 1 -type f -name '*.dmg' -print -quit)"
mv "$built_dmg" "$output_dmg"
hdiutil verify "$output_dmg"
LC_ALL=C LANG=C shasum -a 256 "$output_dmg"
printf 'Fresh DMG: %s\n' "$output_dmg"
