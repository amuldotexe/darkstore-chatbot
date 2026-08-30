# Darkstore fashion concierge — v001

A local Tauri desktop demonstration of a narrow dark-store shopping journey:

1. The shopper enters an OpenAI key for the current memory-only session.
2. GPT-4o classifies an intent against the bundled `dresses` taxonomy; it never chooses products.
3. Rust validates that category, ranks the three highest-propensity available dresses, and returns only rehydrated product facts.
4. The shopper chooses a dress, receives fixture-grounded styling guidance, selects a size, and adds it to a local cart.

The v001 inventory contains eight dresses from [data/darkstore-dresses-v001.json](data/darkstore-dresses-v001.json), an embedded projection of the source-compatible [SQL seed](data/darkstore-dresses-v001.sql). `fixture_*` fields are deliberately synthetic demo data. There is no checkout, payment, persistent cart, account, or automatic category substitution.

## Local setup

Build one self-contained macOS DMG:

```bash
pnpm install
pnpm tauri build --bundles dmg --no-sign
```

The result is `src-tauri/target/release/bundle/dmg/Darkstore Concierge_0.1.0_aarch64.dmg` on this Apple Silicon machine. Mount it and drag out the app. It contains the eight-product demo catalogue and needs no Turso URL, token, dotenv file, or launch-shell configuration. The first screen asks the shopper to enter their own OpenAI key for the current in-memory session; never put that key in `.env`, source code, or git.

## Verification

```bash
pnpm test
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets --all-features
cargo build --manifest-path src-tauri/Cargo.toml --all-targets --all-features
```

The canonical executable specification is [docs/specs-v001.md](docs/specs-v001.md). The source-of-truth journey and runtime architecture are in [docs/diagrams](docs/diagrams).
