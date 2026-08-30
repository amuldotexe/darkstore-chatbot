# Embedded Catalog DMG Delivery Implementation Plan

> **For Codex:** REQUIRED SKILLS: `test-driven-development`, `tauri-coder-01`, and `verification-before-completion`.

**Goal:** Deliver one macOS DMG whose installed application can show the eight approved v001 dress fixtures without a Turso URL or token in the launch environment.

**Architecture:** Keep the existing `CatalogRepository` port and workflow unchanged. Add an embedded, read-only catalogue adapter backed by a checked-in JSON projection of the approved SQL seed, and make it the Tauri composition root’s default. GPT-4o remains a runtime-key-gated category interpreter; no OpenAI key or remote database credential is bundled.

**Tech Stack:** Rust 2024, Tauri 2, `serde_json`, Vitest, Cargo tests, macOS `hdiutil`.

---

### Task 1: Specify the self-contained runtime contract

**Files:**
- Modify: `docs/specs-v001.md`
- Modify: `README.md`
- Test: `src-tauri/tests/catalog_contracts.rs`

**Step 1: Write the failing test**

Assert that an `EmbeddedCatalogRepository` loads one `dresses` taxonomy value and the eight seed products without reading `TURSO_DATABASE_URL` or `TURSO_AUTH_TOKEN`.

**Step 2: Run the test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test catalog_contracts embedded_catalog -- --exact`

Expected: compilation failure because the embedded repository does not exist.

**Step 3: Update the specification**

Add `REQ-TAURI-017.0`: the release DMG shall bundle a fixed eight-product local projection and must not require Turso environment configuration. Keep the user-supplied OpenAI key in memory only.

### Task 2: Add a read-only embedded catalogue adapter

**Files:**
- Create: `data/darkstore-dresses-v001.json`
- Modify: `src-tauri/src/catalog.rs`
- Test: `src-tauri/tests/catalog_contracts.rs`

**Step 1: Implement the minimal adapter**

Create `EmbeddedCatalogRepository` with `create_embedded_catalog_repository()`. It deserializes the included JSON projection once into `CatalogProduct` values and implements the existing `CatalogRepository` trait. Its taxonomy is derived from the loaded products.

**Step 2: Run the focused test**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test catalog_contracts embedded_catalog -- --exact`

Expected: PASS.

### Task 3: Compose the DMG runtime with embedded data

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Test: `src-tauri/tests/command_contracts.rs`

**Step 1: Write the failing composition test**

Validate that `create_runtime_app_services()` can configure a syntactically valid session key and does not depend on Turso configuration before the model boundary.

**Step 2: Implement the one-line composition change**

Construct `EmbeddedCatalogRepository` instead of `TursoCatalogRepository` in `create_runtime_app_services()`. Keep remote Turso code unavailable to the release default, not reachable via a WebView capability.

**Step 3: Run Rust tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --all-targets --all-features`

Expected: PASS.

### Task 4: Produce and verify the DMG

**Files:**
- Modify: `README.md`

**Step 1: Update distribution instructions**

Document the one DMG path and clarify that the first screen asks the shopper to enter their own OpenAI key at runtime.

**Step 2: Execute verification gates**

Run:

```bash
pnpm test
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets --all-features
pnpm tauri build --bundles dmg --no-sign
hdiutil imageinfo "src-tauri/target/release/bundle/dmg/Darkstore Concierge_0.1.0_aarch64.dmg"
```

**Step 3: Mount and launch-test**

Mount the DMG at a temporary mount point, use `open` on the copied `Darkstore Concierge.app`, verify its process runs, and manually inspect the key gate. Do not enter or persist an OpenAI key.
