# Darkstore fashion concierge — v001

A local Tauri desktop demonstration of a narrow dark-store shopping journey:

1. The shopper enters an OpenAI key for the current memory-only session.
2. GPT-4o classifies an intent against the live Turso taxonomy; it never chooses products.
3. Rust validates that category, ranks the three highest-propensity available dresses, and returns only rehydrated product facts.
4. The shopper chooses a dress, receives fixture-grounded styling guidance, selects a size, and adds it to a local cart.

The v001 inventory contains eight dresses seeded by [data/darkstore-dresses-v001.sql](data/darkstore-dresses-v001.sql). `fixture_*` fields are deliberately synthetic demo data. There is no checkout, payment, persistent cart, account, or automatic category substitution.

## Local setup

Set user-owned Turso values in the shell that launches the desktop app (the Rust backend reads process environment rather than a dotenv file):

```bash
export TURSO_DATABASE_URL='libsql://your-database.turso.io'
export TURSO_AUTH_TOKEN='your-scoped-token'
```

Create the named Turso database yourself and import the checked-in SQL seed. The app asks for the OpenAI key in its own first screen; do not put it in `.env`, source code, or git.

```bash
pnpm install
pnpm tauri dev
```

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
