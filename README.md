# Darkstore fashion concierge — v002

A local Tauri desktop demonstration of a narrow dark-store shopping journey:

1. The shopper enters an OpenAI key for the current memory-only session.
2. GPT-4o receives only the available dress candidates and chooses up to three offered SKU identifiers.
3. Rust validates the SKU set and falls back to local propensity ranking when GPT’s response is unavailable or invalid.
4. The shopper chooses a dress, receives fixture-grounded styling guidance, selects a size, and adds it to a local cart.

The v002 inventory contains eight dresses from [data/darkstore-dresses-v001.json](data/darkstore-dresses-v001.json), an embedded projection of the source-compatible [SQL seed](data/darkstore-dresses-v001.sql). `fixture_*` fields are deliberately synthetic demo data. There is no checkout, payment, persistent cart, or account.

## Local setup

Build one self-contained macOS DMG with stale package outputs removed first:

```bash
pnpm install
pnpm build:dmg:clean
```

The build removes old bundle outputs, targets both Apple Silicon and Intel Macs, selects a locally installed `Developer ID Application` identity (or a specific `APPLE_SIGNING_IDENTITY`), and produces exactly one signed artifact: `src-tauri/target/universal-apple-darwin/release/bundle/dmg/Darkstore-Concierge-v002-universal.dmg`. Mount it and drag out the app. It contains the eight-product demo catalogue and needs no Turso URL, token, dotenv file, or launch-shell configuration. The first screen asks the shopper to enter their own OpenAI key for the current memory-only session; never put that key in `.env`, source code, or git.

Developer ID signing and Apple notarization are separate. The build verifies the signature and reports the Gatekeeper assessment, but a publicly shareable no-warning release requires successful notarization with Apple credentials.

## Verification

```bash
pnpm test
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets --all-features
cargo build --manifest-path src-tauri/Cargo.toml --all-targets --all-features
```

The current executable improvement specification is [docs/spec-improvement-02.md](docs/spec-improvement-02.md). The source-of-truth journey and runtime architecture are in [docs/diagrams](docs/diagrams).

# User Journey

- After entering an OpenAI key, the shopper sees 3 cards and a "More" button.

![alt text](image.png)

