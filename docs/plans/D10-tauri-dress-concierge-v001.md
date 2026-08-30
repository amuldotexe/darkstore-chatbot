# Tauri Dress Concierge V001 Implementation Plan

> **For Codex:** Execute test-first against the canonical [v001 specification](../specs-v001.md).

**Goal:** Deliver one Tauri desktop app that accepts a runtime OpenAI key, interprets a dress brief, deterministically presents a three-card Turso catalogue set, customizes a selected dress in chat, and adds a validated size to an in-memory cart.

**Architecture:** The React/TypeScript WebView contains only ephemeral UI state and typed Tauri invokes. Rust owns a `CatalogRepository` trait, a strict libSQL/Turso adapter, session mutation, validation, and serializable `AppError` values. Tests inject an in-memory repository and deterministic model double; no supplied secret is used by tests or stored in the repository.

**Tech Stack:** Tauri 2, Rust, libSQL/Turso, reqwest, serde, thiserror, Vite, TypeScript, Vitest.

---

### Task 1: Establish the test-first desktop shell

**Files:**

- Create: `package.json`, `vite.config.ts`, `tsconfig.json`, `index.html`
- Create: `src-tauri/Cargo.toml`, `src-tauri/build.rs`, `src-tauri/tauri.conf.json`
- Create: `src-tauri/capabilities/default.json`
- Test: `src-tauri/tests/catalog_contracts.rs`, `src/app.test.ts`

1. Add isolated Rust and frontend test runners without application behavior.
2. Write tests for key validation, category validation, deterministic trio selection, absent inventory, unavailable inventory, and complete-page exhaustion.
3. Run them and record the expected missing-module failures.

### Task 2: Build the Rust domain and adapter seams

**Files:**

- Create: `src-tauri/src/domain.rs`, `src-tauri/src/error.rs`, `src-tauri/src/catalog.rs`, `src-tauri/src/model.rs`
- Modify: `src-tauri/tests/catalog_contracts.rs`

1. Create four-word domain functions and tagged response DTOs.
2. Implement pure category validation and deterministic three-card page selection.
3. Implement `CatalogRepository` and `CategoryModel` traits with deterministic test doubles.
4. Add a libSQL/Turso adapter that reads only backend environment configuration.

### Task 3: Add session, commands, and least-privilege configuration

**Files:**

- Create: `src-tauri/src/session.rs`, `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/main.rs`
- Modify: `src-tauri/capabilities/default.json`, `src-tauri/tauri.conf.json`
- Test: `src-tauri/tests/command_contracts.rs`

1. Implement in-memory key, brief, selection, request-ID, and cart state without persistence.
2. Register a centralized set of async Tauri commands returning serializable errors.
3. Keep the WebView CSP at `connect-src 'self'`; grant no filesystem, shell, plugin, updater, or sidecar capability.
4. Exercise the full command sequence with injected doubles.

### Task 4: Implement the guided desktop UI

**Files:**

- Create: `src/main.ts`, `src/styles.css`, `src/app.test.ts`

1. Render a key gate before concierge actions.
2. Render exactly three product cards plus the fourth alternative action.
3. Support brief entry, not-in-inventory and inventory-unavailable recovery, product-anchored chat, variant selection, local cart, and product-detail hyperlinks.
4. Test UI state transitions with mocked typed invokes.

### Task 5: Verify, launch, review, and publish

**Files:**

- Modify: `README.md`, `docs/journals/turso-dress-data-v001.md`

1. Run Rust format, clippy, tests, and build; run frontend typecheck, tests, and production build.
2. Run a local end-to-end desktop session with deterministic doubles and visually inspect the screen.
3. Run static secret, capability, SQL, spec-traceability, and diff checks.
4. Commit the verified app and documentation, then push `main` to `origin`.
