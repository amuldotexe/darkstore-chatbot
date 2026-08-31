#!/usr/bin/env bash
set -euo pipefail

project_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
target_triple="universal-apple-darwin"
target_release_directory="$project_directory/src-tauri/target/$target_triple/release"
bundle_directory="$target_release_directory/bundle"
output_directory="$bundle_directory/dmg"
output_dmg="$output_directory/Darkstore-Concierge-v002-universal.dmg"

clean_release_bundle() {
  local bundle_root="$1"
  rm -rf "$bundle_root/dmg" "$bundle_root/macos"
}

clear_prior_release_artifacts() {
  rm -rf \
    "$target_release_directory" \
    "$project_directory/src-tauri/target/aarch64-apple-darwin/release" \
    "$project_directory/src-tauri/target/x86_64-apple-darwin/release" \
    "$project_directory/src-tauri/target/release/bundle"
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

clear_prior_release_artifacts
cargo clean --manifest-path "$project_directory/src-tauri/Cargo.toml" \
  --package darkstore-concierge

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
    pnpm tauri build --bundles dmg --target "$target_triple"
)

dmgs_found="$(find "$output_directory" -maxdepth 1 -type f -name '*.dmg' -print | wc -l | tr -d ' ')"
if [[ "$dmgs_found" -ne 1 ]]; then
  printf 'Expected exactly one fresh DMG, found %s.\n' "$dmgs_found" >&2
  exit 1
fi

built_dmg="$(find "$output_directory" -maxdepth 1 -type f -name '*.dmg' -print -quit)"
mv "$built_dmg" "$output_dmg"
hdiutil verify "$output_dmg"
mount_directory="$(mktemp -d)"
cleanup_mounted_dmg() {
  hdiutil detach "$mount_directory" -quiet 2>/dev/null || true
  rmdir "$mount_directory" 2>/dev/null || true
}
trap cleanup_mounted_dmg EXIT
hdiutil attach "$output_dmg" -nobrowse -readonly -mountpoint "$mount_directory" -quiet
mounted_app="$(find "$mount_directory" -maxdepth 1 -type d -name '*.app' -print -quit)"
if [[ -z "$mounted_app" ]]; then
  printf 'Expected a single application bundle in the fresh DMG.\n' >&2
  exit 1
fi
codesign --verify --deep --strict --verbose=2 "$mounted_app"
binary_path="$mounted_app/Contents/MacOS/darkstore-concierge"
binary_architectures="$(lipo -archs "$binary_path")"
if [[ "$binary_architectures" != *"arm64"* || "$binary_architectures" != *"x86_64"* ]]; then
  printf 'Expected universal arm64 and x86_64 application binary, found: %s\n' "$binary_architectures" >&2
  exit 1
fi
if ! spctl --assess --type execute --verbose=2 "$mounted_app"; then
  printf 'Note: Developer ID signature is present, but Gatekeeper approval requires successful Apple notarization.\n' >&2
fi
LC_ALL=C LANG=C shasum -a 256 "$output_dmg"
printf 'Fresh DMG: %s\n' "$output_dmg"
